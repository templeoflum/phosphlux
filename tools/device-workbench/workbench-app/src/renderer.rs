//! GPU renderer using wgpu

use anyhow::{anyhow, Result};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Instant;
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::state::{AppState, NUM_INPUT_CHANNELS};

/// Main renderer struct
pub struct Renderer {
    // Keep window alive for surface lifetime
    _window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    // Pipeline state
    pipeline: Option<wgpu::RenderPipeline>,
    bind_group: Option<wgpu::BindGroup>,
    bind_group_layout: wgpu::BindGroupLayout,

    // Resources - 7 input textures for matrix mixer support
    input_textures: [wgpu::Texture; NUM_INPUT_CHANNELS],
    input_views: [wgpu::TextureView; NUM_INPUT_CHANNELS],
    sampler: wgpu::Sampler,
    params_buffer: wgpu::Buffer,

    // Track which inputs need updating
    input_dirty: [bool; NUM_INPUT_CHANNELS],

    // Readback for screenshots
    output_texture: wgpu::Texture,
    output_buffer: wgpu::Buffer,

    // Shared state
    state: Arc<RwLock<AppState>>,

    // Current shader for change detection
    current_shader_code: String,

    // Metrics
    last_frame_time: Instant,
    frame_count: u32,
    fps_update_time: Instant,
}

impl Renderer {
    pub async fn new(window: Arc<Window>, state: Arc<RwLock<AppState>>) -> Result<Self> {
        let size = window.inner_size();

        // Create instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface - window Arc keeps it alive for 'static
        let surface = instance.create_surface(window.clone())?;

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow!("Failed to find GPU adapter"))?;

        // Create device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Workbench Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await?;

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create bind group layout with 7 input textures
        // Layout: bindings 0-6 = input textures, 7 = sampler, 8 = params
        let mut layout_entries = Vec::with_capacity(NUM_INPUT_CHANNELS + 2);

        // Input textures (bindings 0-6)
        for i in 0..NUM_INPUT_CHANNELS {
            layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: i as u32,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            });
        }

        // Sampler (binding 7)
        layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: NUM_INPUT_CHANNELS as u32,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        });

        // Params uniform (binding 8)
        layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: (NUM_INPUT_CHANNELS + 1) as u32,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Workbench Bind Group Layout (7-input)"),
            entries: &layout_entries,
        });

        // Create 7 input textures (512x512 gray default)
        let tex_size = wgpu::Extent3d {
            width: 512,
            height: 512,
            depth_or_array_layers: 1,
        };

        let gray_data: Vec<u8> = vec![128, 128, 128, 255].repeat(512 * 512);

        let input_textures: [wgpu::Texture; NUM_INPUT_CHANNELS] = std::array::from_fn(|i| {
            let tex = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(&format!("Input Texture {}", i)),
                size: tex_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            // Initialize with gray
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &gray_data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(512 * 4),
                    rows_per_image: Some(512),
                },
                tex_size,
            );

            tex
        });

        let input_views: [wgpu::TextureView; NUM_INPUT_CHANNELS] = std::array::from_fn(|i| {
            input_textures[i].create_view(&wgpu::TextureViewDescriptor::default())
        });

        // Create sampler
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create params buffer
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Params Buffer"),
            contents: bytemuck::cast_slice(&[0.0f32; 16]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create output texture for readback (matches window size)
        let output_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Output Texture"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: surface_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        // Create output buffer for CPU readback (with 256-byte row alignment)
        let padded_bytes_per_row = ((config.width * 4) + 255) & !255;
        let buffer_size = (padded_bytes_per_row * config.height) as u64;
        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let now = Instant::now();

        let mut renderer = Self {
            _window: window,
            surface,
            device,
            queue,
            config,
            pipeline: None,
            bind_group: None,
            bind_group_layout,
            input_textures,
            input_views,
            sampler,
            params_buffer,
            input_dirty: [false; NUM_INPUT_CHANNELS],
            output_texture,
            output_buffer,
            state,
            current_shader_code: String::new(),
            last_frame_time: now,
            frame_count: 0,
            fps_update_time: now,
        };

        // Compile initial shader
        renderer.maybe_recompile_shader()?;

        Ok(renderer)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);

            // Recreate output texture to match new size
            self.output_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Output Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });

            let padded_bytes_per_row = ((width * 4) + 255) & !255;
            let buffer_size = (padded_bytes_per_row * height) as u64;
            self.output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Output Buffer"),
                size: buffer_size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
        }
    }

    fn maybe_recompile_shader(&mut self) -> Result<()> {
        let (shader_code, shader_dirty) = {
            let state = self.state.read();
            (state.shader_code.clone(), state.shader_dirty)
        };

        if !shader_dirty && shader_code == self.current_shader_code {
            return Ok(());
        }

        let start = Instant::now();

        // Pre-validate WGSL before sending to GPU
        // This catches syntax errors before they cause a panic
        let validation_result = {
            use naga::front::wgsl;
            use naga::valid::{Capabilities, ValidationFlags, Validator};

            match wgsl::parse_str(&shader_code) {
                Ok(module) => {
                    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
                    match validator.validate(&module) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(format!("Validation error: {:?}", e)),
                    }
                }
                Err(e) => Err(format!("Parse error: {}", e.emit_to_string(&shader_code))),
            }
        };

        if let Err(error_msg) = validation_result {
            let mut state = self.state.write();
            state.shader_error = Some(error_msg);
            state.status = "error".to_string();
            state.shader_dirty = false;
            return Ok(());
        }

        // Try to compile shader (now safe after validation)
        let shader_module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Workbench Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.clone().into()),
        });

        // Create pipeline layout
        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Workbench Pipeline Layout"),
                bind_group_layouts: &[&self.bind_group_layout],
                push_constant_ranges: &[],
            });

        // Create render pipeline
        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Workbench Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: self.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        // Create bind group with all 7 input textures
        let bind_group = self.create_bind_group();

        let compile_time = start.elapsed().as_secs_f32() * 1000.0;

        self.pipeline = Some(pipeline);
        self.bind_group = Some(bind_group);
        self.current_shader_code = shader_code;

        // Update state
        {
            let mut state = self.state.write();
            state.shader_dirty = false;
            state.shader_error = None;
            state.shader_compile_ms = compile_time;
            state.status = "ok".to_string();
        }

        tracing::info!("Shader compiled in {:.1}ms", compile_time);

        Ok(())
    }

    /// Create bind group with all 7 input texture views
    fn create_bind_group(&self) -> wgpu::BindGroup {
        let mut entries: Vec<wgpu::BindGroupEntry> = Vec::with_capacity(NUM_INPUT_CHANNELS + 2);

        // Input textures (bindings 0-6)
        for (i, view) in self.input_views.iter().enumerate() {
            entries.push(wgpu::BindGroupEntry {
                binding: i as u32,
                resource: wgpu::BindingResource::TextureView(view),
            });
        }

        // Sampler (binding 7)
        entries.push(wgpu::BindGroupEntry {
            binding: NUM_INPUT_CHANNELS as u32,
            resource: wgpu::BindingResource::Sampler(&self.sampler),
        });

        // Params (binding 8)
        entries.push(wgpu::BindGroupEntry {
            binding: (NUM_INPUT_CHANNELS + 1) as u32,
            resource: self.params_buffer.as_entire_binding(),
        });

        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Workbench Bind Group (7-input)"),
            layout: &self.bind_group_layout,
            entries: &entries,
        })
    }

    fn update_input_textures(&mut self) {
        // Collect input data from state
        let input_data: [Option<Vec<u8>>; NUM_INPUT_CHANNELS] = {
            let mut state = self.state.write();
            let data = state.input_images.clone();
            // Clear to avoid re-uploading
            for img in state.input_images.iter_mut() {
                *img = None;
            }
            data
        };

        let mut needs_rebind = false;

        for (i, data) in input_data.into_iter().enumerate() {
            if let Some(bytes) = data {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    let rgba = img.to_rgba8();
                    let (width, height) = rgba.dimensions();

                    // Check if texture size changed
                    let current_width = self.input_textures[i].width();
                    let current_height = self.input_textures[i].height();

                    if width != current_width || height != current_height {
                        // Recreate texture with new size
                        self.input_textures[i] = self.device.create_texture(&wgpu::TextureDescriptor {
                            label: Some(&format!("Input Texture {}", i)),
                            size: wgpu::Extent3d {
                                width,
                                height,
                                depth_or_array_layers: 1,
                            },
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: wgpu::TextureDimension::D2,
                            format: wgpu::TextureFormat::Rgba8UnormSrgb,
                            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                            view_formats: &[],
                        });
                        self.input_views[i] = self.input_textures[i]
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        needs_rebind = true;
                    }

                    // Upload texture data
                    self.queue.write_texture(
                        wgpu::ImageCopyTexture {
                            texture: &self.input_textures[i],
                            mip_level: 0,
                            origin: wgpu::Origin3d::ZERO,
                            aspect: wgpu::TextureAspect::All,
                        },
                        &rgba,
                        wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(width * 4),
                            rows_per_image: Some(height),
                        },
                        wgpu::Extent3d {
                            width,
                            height,
                            depth_or_array_layers: 1,
                        },
                    );
                }
            }
        }

        // Rebuild bind group if any texture was resized
        if needs_rebind && self.pipeline.is_some() {
            self.bind_group = Some(self.create_bind_group());
        }
    }

    fn capture_output(&mut self) {
        let width = self.config.width;
        let height = self.config.height;
        let bytes_per_row = width * 4;
        let padded_bytes_per_row = (bytes_per_row + 255) & !255;
        let is_bgra = matches!(self.config.format,
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb);

        // Map buffer for reading with proper error handling
        let buffer_slice = self.output_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        self.device.poll(wgpu::Maintain::Wait);

        // Check if mapping succeeded
        match rx.recv() {
            Ok(Ok(())) => {
                // Mapping succeeded, read the data
                let data = buffer_slice.get_mapped_range();

                // Remove padding and convert BGRA to RGBA if needed
                let mut pixels: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);
                for row in 0..height {
                    let start = (row * padded_bytes_per_row) as usize;
                    for col in 0..width as usize {
                        let idx = start + col * 4;
                        if idx + 4 <= data.len() {
                            if is_bgra {
                                // BGRA -> RGBA
                                pixels.push(data[idx + 2]); // R
                                pixels.push(data[idx + 1]); // G
                                pixels.push(data[idx]);     // B
                                pixels.push(data[idx + 3]); // A
                            } else {
                                pixels.extend_from_slice(&data[idx..idx + 4]);
                            }
                        }
                    }
                }
                drop(data); // Release the mapped range before unmapping

                // Create PNG
                match image::RgbaImage::from_raw(width, height, pixels) {
                    Some(img) => {
                        let mut png_bytes = Vec::new();
                        let mut cursor = std::io::Cursor::new(&mut png_bytes);
                        match img.write_to(&mut cursor, image::ImageFormat::Png) {
                            Ok(_) => {
                                let mut state = self.state.write();
                                state.output_image = Some(png_bytes);
                                tracing::debug!("Screenshot captured: {}x{}", width, height);
                            }
                            Err(e) => {
                                tracing::error!("Failed to encode PNG: {}", e);
                            }
                        }
                    }
                    None => {
                        tracing::error!("Failed to create image from pixels ({}x{}, {} bytes)",
                            width, height, width * height * 4);
                    }
                }
            }
            Ok(Err(e)) => {
                tracing::error!("Buffer mapping failed: {:?}", e);
            }
            Err(e) => {
                tracing::error!("Failed to receive mapping result: {}", e);
            }
        }

        self.output_buffer.unmap();
    }

    pub fn render(&mut self) -> Result<()> {
        let frame_start = Instant::now();

        // Check for shader changes
        if let Err(e) = self.maybe_recompile_shader() {
            let mut state = self.state.write();
            state.shader_error = Some(format!("{}", e));
            state.status = "error".to_string();
        }

        // Update input textures if changed
        self.update_input_textures();

        // Update params buffer
        {
            let state = self.state.read();
            let params = state.get_params_for_gpu();
            self.queue
                .write_buffer(&self.params_buffer, 0, bytemuck::cast_slice(&params));
        }

        // Get surface texture
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Render pass
        if let (Some(pipeline), Some(bind_group)) = (&self.pipeline, &self.bind_group) {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        // Copy to output texture for screenshots
        let output_view = self.output_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create a second render pass to the output texture
        {
            let mut copy_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Copy Encoder"),
            });

            if let (Some(pipeline), Some(bind_group)) = (&self.pipeline, &self.bind_group) {
                let mut render_pass = copy_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Output Capture Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &output_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                render_pass.set_pipeline(pipeline);
                render_pass.set_bind_group(0, bind_group, &[]);
                render_pass.draw(0..3, 0..1);
            }

            // Copy output texture to buffer
            let bytes_per_row = self.config.width * 4;
            let padded_bytes_per_row = (bytes_per_row + 255) & !255; // Align to 256

            copy_encoder.copy_texture_to_buffer(
                wgpu::ImageCopyTexture {
                    texture: &self.output_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::ImageCopyBuffer {
                    buffer: &self.output_buffer,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(padded_bytes_per_row),
                        rows_per_image: Some(self.config.height),
                    },
                },
                wgpu::Extent3d {
                    width: self.config.width,
                    height: self.config.height,
                    depth_or_array_layers: 1,
                },
            );

            self.queue.submit(std::iter::once(copy_encoder.finish()));
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        // Read back output for screenshot (every 10 frames to avoid overhead)
        if self.frame_count % 10 == 0 {
            self.capture_output();
        }

        // Update metrics
        let frame_time = frame_start.elapsed().as_secs_f32() * 1000.0;
        self.frame_count += 1;

        let elapsed = self.fps_update_time.elapsed().as_secs_f32();
        if elapsed >= 1.0 {
            let fps = self.frame_count as f32 / elapsed;
            self.frame_count = 0;
            self.fps_update_time = Instant::now();

            let mut state = self.state.write();
            state.fps = fps;
            state.frame_time_ms = frame_time;
        }

        Ok(())
    }
}

