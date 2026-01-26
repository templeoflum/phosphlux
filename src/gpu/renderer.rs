//! GPU Renderer for Phosphlux v2
//!
//! Manages the wgpu device, surface, and render pipeline.
//! Unlike v1's fixed pipeline, v2 recompiles the pipeline when
//! the patch topology changes (compose-based architecture).

use std::sync::Arc;
use bytemuck::Zeroable;
use wgpu::util::DeviceExt;
use winit::window::Window;
use winit::dpi::PhysicalSize;

use crate::core::timing::TimingUniforms;
use crate::gpu::shader_composer::{DevicePatch, ParamLayout, ShaderComposer};

/// A fallback shader used when no patch is loaded or on compile error
const FALLBACK_SHADER: &str = r#"
struct Timing {
    field: u32, frame: u32, frame_phase: f32, time: f32,
    h_phase: f32, v_phase: f32, lines: f32, field_rate: f32,
    subcarrier_phase: f32, _p0: f32, _p1: f32, _p2: f32,
    _p3: f32, _p4: f32, _p5: f32, _p6: f32,
}
@group(0) @binding(0) var<uniform> timing: Timing;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var out: VertexOutput;
    let u = f32((vi << 1u) & 2u);
    let v = f32(vi & 2u);
    out.clip_position = vec4<f32>(u * 2.0 - 1.0, v * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(u, 1.0 - v);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Test pattern: color bars
    let bar = u32(in.uv.x * 8.0);
    var c = vec3<f32>(0.0);
    if bar == 0u { c = vec3<f32>(1.0, 1.0, 1.0); }
    else if bar == 1u { c = vec3<f32>(1.0, 1.0, 0.0); }
    else if bar == 2u { c = vec3<f32>(0.0, 1.0, 1.0); }
    else if bar == 3u { c = vec3<f32>(0.0, 1.0, 0.0); }
    else if bar == 4u { c = vec3<f32>(1.0, 0.0, 1.0); }
    else if bar == 5u { c = vec3<f32>(1.0, 0.0, 0.0); }
    else if bar == 6u { c = vec3<f32>(0.0, 0.0, 1.0); }
    return vec4<f32>(c, 1.0);
}
"#;

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,

    // Pipeline — recreated on topology change
    pipeline: wgpu::RenderPipeline,

    // Bind group layouts
    timing_bg_layout: wgpu::BindGroupLayout,
    empty_bg_layout: wgpu::BindGroupLayout,  // Placeholder for group 1 (feedback textures)
    params_bg_layout: wgpu::BindGroupLayout,

    // Bind groups + Buffers
    timing_bind_group: wgpu::BindGroup,
    timing_buffer: wgpu::Buffer,
    empty_bind_group: wgpu::BindGroup,
    params_bind_group: wgpu::BindGroup,
    params_buffer: wgpu::Buffer,

    // Output texture for egui preview display
    output_texture: wgpu::Texture,
    output_view: wgpu::TextureView,
    pub output_texture_id: Option<egui::TextureId>,
    output_size: (u32, u32),

    // Composition state
    current_param_layout: Option<ParamLayout>,
    pub shader_error: Option<String>,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        log::info!("GPU: {}", adapter.get_info().name);

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Phosphlux Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .unwrap();

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

        // === Bind group layouts ===

        // Group 0: Timing uniform
        let timing_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Timing BG Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // Group 1: Empty placeholder (for feedback textures later)
        let empty_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Empty BG Layout"),
            entries: &[],
        });

        // Group 2: Params uniform
        let params_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Params BG Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // === Buffers ===
        let timing_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Timing Buffer"),
            contents: bytemuck::cast_slice(&[TimingUniforms::zeroed()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Default params buffer (16 bytes minimum for uniform alignment)
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Params Buffer"),
            contents: &[0u8; 16],
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // === Bind groups ===
        let timing_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Timing BG"),
            layout: &timing_bg_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: timing_buffer.as_entire_binding(),
            }],
        });

        let empty_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Empty BG"),
            layout: &empty_bg_layout,
            entries: &[],
        });

        let params_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Params BG"),
            layout: &params_bg_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buffer.as_entire_binding(),
            }],
        });

        // === Initial pipeline (fallback) ===
        let pipeline = Self::create_pipeline_from_wgsl(
            &device,
            FALLBACK_SHADER,
            &config,
            &timing_bg_layout,
            &empty_bg_layout,
            // Fallback shader doesn't use group 2, use timing layout as dummy
            &timing_bg_layout,
            true, // fallback mode: only groups 0
        );

        // === Output texture for egui ===
        let output_size = (320, 240);
        let (output_texture, output_view) = Self::create_output_texture(
            &device,
            output_size.0,
            output_size.1,
            surface_format,
        );

        Self {
            surface,
            device,
            queue,
            config,
            size,
            pipeline,
            timing_bg_layout,
            empty_bg_layout,
            params_bg_layout,
            timing_bind_group,
            timing_buffer,
            empty_bind_group,
            params_bind_group,
            params_buffer,
            output_texture,
            output_view,
            output_texture_id: None,
            output_size,
            current_param_layout: None,
            shader_error: None,
        }
    }

    /// Create a render pipeline from WGSL source
    fn create_pipeline_from_wgsl(
        device: &wgpu::Device,
        wgsl: &str,
        config: &wgpu::SurfaceConfiguration,
        timing_layout: &wgpu::BindGroupLayout,
        empty_layout: &wgpu::BindGroupLayout,
        params_layout: &wgpu::BindGroupLayout,
        fallback: bool,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Composed Shader"),
            source: wgpu::ShaderSource::Wgsl(wgsl.into()),
        });

        let bind_group_layouts: Vec<&wgpu::BindGroupLayout> = if fallback {
            // Fallback only uses group 0
            vec![timing_layout]
        } else {
            // Full: groups 0, 1, 2
            vec![timing_layout, empty_layout, params_layout]
        };

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &bind_group_layouts,
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[], // No vertex buffer — fullscreen triangle via vertex_index
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
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
        })
    }

    fn create_output_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Output Texture"),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    /// Recompile the pipeline from a device patch
    pub fn recompile(&mut self, patch: &DevicePatch, composer: &mut ShaderComposer) {
        match composer.compose(patch) {
            Ok(composed) => {
                // Create new params buffer sized to the layout
                let param_size = composed.param_layout.size.max(16);
                let param_values = patch.collect_param_values(&composed.param_layout);
                let param_bytes = bytemuck::cast_slice(&param_values);

                self.params_buffer = self.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Params Buffer"),
                        contents: param_bytes,
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    },
                );

                // Recreate params bind group with new buffer
                self.params_bind_group = self.device.create_bind_group(
                    &wgpu::BindGroupDescriptor {
                        label: Some("Params BG"),
                        layout: &self.params_bg_layout,
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: self.params_buffer.as_entire_binding(),
                        }],
                    },
                );

                // Create new pipeline
                self.pipeline = Self::create_pipeline_from_wgsl(
                    &self.device,
                    &composed.wgsl_source,
                    &self.config,
                    &self.timing_bg_layout,
                    &self.empty_bg_layout,
                    &self.params_bg_layout,
                    false,
                );

                self.current_param_layout = Some(composed.param_layout.clone());
                self.shader_error = None;

                log::info!("Shader recompiled: {} bytes WGSL, {} bytes params",
                    composed.wgsl_source.len(), param_size);
            }
            Err(e) => {
                self.shader_error = Some(format!("{}", e));
                log::error!("Shader compose error: {}", e);
            }
        }
    }

    /// Update parameter values without recompiling
    pub fn update_params(&mut self, patch: &DevicePatch) {
        if let Some(layout) = &self.current_param_layout {
            let values = patch.collect_param_values(layout);
            let bytes = bytemuck::cast_slice(&values);
            self.queue.write_buffer(&self.params_buffer, 0, bytes);
        }
    }

    /// Render composed shader to output texture
    pub fn render_to_texture(&mut self, timing: TimingUniforms) {
        // Update timing uniform
        self.queue.write_buffer(
            &self.timing_buffer,
            0,
            bytemuck::cast_slice(&[timing]),
        );

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Render To Texture"),
            },
        );

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Synth Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.output_view,
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

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.timing_bind_group, &[]);

            // Only set groups 1 and 2 if we have a compiled shader (not fallback)
            if self.current_param_layout.is_some() {
                pass.set_bind_group(1, &self.empty_bind_group, &[]);
                pass.set_bind_group(2, &self.params_bind_group, &[]);
            }

            pass.draw(0..3, 0..1); // Fullscreen triangle
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    /// Register output texture with egui for display
    pub fn register_output_texture(&mut self, egui_renderer: &mut egui_wgpu::Renderer) {
        if let Some(old_id) = self.output_texture_id.take() {
            egui_renderer.free_texture(&old_id);
        }

        let id = egui_renderer.register_native_texture(
            &self.device,
            &self.output_view,
            wgpu::FilterMode::Linear,
        );
        self.output_texture_id = Some(id);
    }

    /// Resize the output texture for the preview pane
    pub fn resize_output(&mut self, width: u32, height: u32, egui_renderer: &mut egui_wgpu::Renderer) {
        if width == 0 || height == 0 {
            return;
        }
        if self.output_size == (width, height) {
            return;
        }

        self.output_size = (width, height);
        let (texture, view) = Self::create_output_texture(
            &self.device,
            width,
            height,
            self.config.format,
        );
        self.output_texture = texture;
        self.output_view = view;
        self.register_output_texture(egui_renderer);
    }

    /// Resize the surface when the window resizes
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Get reference to wgpu device (for egui init)
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Get surface format (for egui init)
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    /// Full frame: render synth to output texture, then clear surface + paint egui on surface
    pub fn render_with_egui(
        &mut self,
        timing: TimingUniforms,
        egui_ctx: &egui::Context,
        textures_delta: egui::TexturesDelta,
        egui_renderer: &mut egui_wgpu::Renderer,
        shapes: Vec<egui::epaint::ClippedShape>,
        scale_factor: f32,
    ) -> Result<(), wgpu::SurfaceError> {
        // Update timing
        self.queue.write_buffer(
            &self.timing_buffer,
            0,
            bytemuck::cast_slice(&[timing]),
        );

        // Get surface output
        let output = self.surface.get_current_texture()?;
        let surface_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Frame Encoder"),
            },
        );

        // Pass 1: Render synth to output texture
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Synth Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.output_view,
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

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.timing_bind_group, &[]);
            if self.current_param_layout.is_some() {
                pass.set_bind_group(1, &self.empty_bind_group, &[]);
                pass.set_bind_group(2, &self.params_bind_group, &[]);
            }
            pass.draw(0..3, 0..1);
        }

        // Pass 2: Clear surface (egui will paint on it)
        {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Surface"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.08,
                            g: 0.08,
                            b: 0.09,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            // Pass drops immediately — just clearing
        }

        // Handle egui texture updates
        for (id, image_delta) in &textures_delta.set {
            egui_renderer.update_texture(&self.device, &self.queue, *id, image_delta);
        }

        // Tessellate egui shapes
        let paint_jobs = egui_ctx.tessellate(shapes, scale_factor);

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.size.width, self.size.height],
            pixels_per_point: scale_factor,
        };

        egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );

        // Pass 3: Render egui on surface
        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Don't clear — draw over background
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let mut render_pass = render_pass.forget_lifetime();
            egui_renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
        }

        // Free old textures
        for id in &textures_delta.free {
            egui_renderer.free_texture(id);
        }

        // Submit all passes as one batch
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
