//! GPU Renderer
//!
//! Manages the wgpu device, surface, and render pipeline.
//! Implements ping-pong feedback buffers for video feedback effects.

use std::sync::Arc;
use bytemuck::Zeroable;
use wgpu::util::DeviceExt;
use winit::window::Window;
use winit::dpi::PhysicalSize;

use crate::core::timing::TimingUniforms;
use crate::gpu::graph_compiler::{CompiledGraph, CompiledGraphHeader, GraphOp, MAX_OPS};
use crate::ui::patch::SignalPatch;

/// Vertex for fullscreen quad
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}

const FULLSCREEN_QUAD: &[Vertex] = &[
    Vertex { position: [-1.0, -1.0], uv: [0.0, 1.0] },
    Vertex { position: [ 1.0, -1.0], uv: [1.0, 1.0] },
    Vertex { position: [ 1.0,  1.0], uv: [1.0, 0.0] },
    Vertex { position: [-1.0, -1.0], uv: [0.0, 1.0] },
    Vertex { position: [ 1.0,  1.0], uv: [1.0, 0.0] },
    Vertex { position: [-1.0,  1.0], uv: [0.0, 0.0] },
];

/// Feedback parameters passed to GPU
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FeedbackUniforms {
    /// Feedback mix amount (0 = no feedback, 1 = full feedback)
    pub mix: f32,
    /// Zoom factor per frame (1.0 = no zoom, <1 = zoom in, >1 = zoom out)
    pub zoom: f32,
    /// Rotation per frame in radians
    pub rotation: f32,
    /// Hue shift per frame (0-1)
    pub hue_shift: f32,
    /// X offset per frame
    pub offset_x: f32,
    /// Y offset per frame
    pub offset_y: f32,
    /// Saturation adjustment (1.0 = normal)
    pub saturation: f32,
    /// Brightness decay (1.0 = no decay)
    pub decay: f32,
}

impl Default for FeedbackUniforms {
    fn default() -> Self {
        Self {
            mix: 0.85,
            zoom: 0.98,
            rotation: 0.01,
            hue_shift: 0.002,
            offset_x: 0.0,
            offset_y: 0.0,
            saturation: 1.0,
            decay: 0.99,
        }
    }
}

/// Synthesis parameters passed to GPU (must be 64 bytes for alignment)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SynthUniforms {
    // Oscillators (8 floats = 32 bytes)
    pub osc1_freq: f32,
    pub osc1_phase: f32,
    pub osc2_freq: f32,
    pub osc2_phase: f32,
    pub noise_mix: f32,
    pub noise_type: f32,  // 0=white, 1=pink, 2=brown
    pub _pad0: f32,
    pub _pad1: f32,

    // Keyer (8 floats = 32 bytes)
    pub key_enabled: f32,  // 0 or 1
    pub key_clip_black: f32,
    pub key_clip_white: f32,
    pub key_invert: f32,  // 0 or 1
    pub rutt_etra_enabled: f32,  // 0 or 1
    pub rutt_etra_amount: f32,
    pub colorizer_type: f32,  // 0=spectrum, 1=threshold, 2=grayscale
    pub colorizer_levels: f32,

    // CRT + Shape (8 floats = 32 bytes)
    pub scanline_intensity: f32,
    pub curvature: f32,
    pub vignette_amount: f32,
    pub shape_type: f32,  // 0=none, 1=circle, 2=rect, 3=diamond, etc.
    pub shape_size: f32,
    pub shape_softness: f32,
    pub _pad2: f32,
    pub _pad3: f32,
}

impl Default for SynthUniforms {
    fn default() -> Self {
        Self {
            osc1_freq: 4.0,
            osc1_phase: 0.0,
            osc2_freq: 3.0,
            osc2_phase: 0.0,
            noise_mix: 0.1,
            noise_type: 1.0,  // Pink
            _pad0: 0.0,
            _pad1: 0.0,
            key_enabled: 0.0,
            key_clip_black: 0.2,
            key_clip_white: 0.8,
            key_invert: 0.0,
            rutt_etra_enabled: 1.0,
            rutt_etra_amount: 0.15,
            colorizer_type: 0.0,  // Spectrum
            colorizer_levels: 8.0,
            scanline_intensity: 0.12,
            curvature: 0.1,
            vignette_amount: 0.4,
            shape_type: 1.0,  // Circle
            shape_size: 0.3,
            shape_softness: 0.05,
            _pad2: 0.0,
            _pad3: 0.0,
        }
    }
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,

    // Pipeline
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,

    // Uniforms
    timing_buffer: wgpu::Buffer,
    feedback_buffer: wgpu::Buffer,
    synth_buffer: wgpu::Buffer,

    // Ping-pong feedback textures
    feedback_textures: [wgpu::Texture; 2],
    feedback_views: [wgpu::TextureView; 2],
    feedback_bind_groups: [wgpu::BindGroup; 2],
    feedback_sampler: wgpu::Sampler,
    current_feedback_index: usize,

    // Bind group layouts (needed for recreation on resize)
    timing_bind_group_layout: wgpu::BindGroupLayout,
    feedback_bind_group_layout: wgpu::BindGroupLayout,
    timing_bind_group: wgpu::BindGroup,

    // Output texture for egui display (paned layout)
    output_texture: wgpu::Texture,
    output_view: wgpu::TextureView,
    pub output_texture_id: Option<egui::TextureId>,
    output_size: (u32, u32),

    pub feedback: FeedbackUniforms,
    pub synth: SynthUniforms,

    // Node graph compilation
    graph_header_buffer: wgpu::Buffer,
    graph_ops_buffer: wgpu::Buffer,
    graph_bind_group_layout: wgpu::BindGroupLayout,
    graph_bind_group: wgpu::BindGroup,
    compiled_graph: CompiledGraph,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        // Create instance with best available backend
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface
        let surface = instance.create_surface(window).unwrap();

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        log::info!("GPU: {}", adapter.get_info().name);

        // Request device
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Video Synth Device"),
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
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create timing uniform buffer
        let timing_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Timing Uniforms"),
            contents: bytemuck::cast_slice(&[TimingUniforms::zeroed()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create feedback uniform buffer
        let feedback = FeedbackUniforms::default();
        let feedback_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Feedback Uniforms"),
            contents: bytemuck::cast_slice(&[feedback]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create synth uniform buffer
        let synth = SynthUniforms::default();
        let synth_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Synth Uniforms"),
            contents: bytemuck::cast_slice(&[synth]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group layout for timing (group 0)
        let timing_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Timing Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let timing_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Timing Bind Group"),
            layout: &timing_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: timing_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: feedback_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: synth_buffer.as_entire_binding(),
                },
            ],
        });

        // Create feedback texture bind group layout (group 1)
        let feedback_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Feedback Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        // Create feedback sampler
        let feedback_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Feedback Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create ping-pong feedback textures (must match surface format)
        let (feedback_textures, feedback_views, feedback_bind_groups) =
            Self::create_feedback_textures(&device, &feedback_bind_group_layout, &feedback_sampler, size.width.max(1), size.height.max(1), surface_format);

        // Create graph buffers for node-based synthesis
        let compiled_graph = CompiledGraph::default_fallback();
        let graph_header_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Graph Header Buffer"),
            contents: bytemuck::cast_slice(&[compiled_graph.header]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let graph_ops_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Graph Ops Buffer"),
            contents: bytemuck::cast_slice(&compiled_graph.ops),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        // Graph bind group layout (group 2)
        let graph_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Graph Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let graph_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Graph Bind Group"),
            layout: &graph_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: graph_header_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: graph_ops_buffer.as_entire_binding(),
                },
            ],
        });

        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Video Synth Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/main.wgsl").into()),
        });

        // Pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&timing_bind_group_layout, &feedback_bind_group_layout, &graph_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                    ],
                }],
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
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        // Create vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(FULLSCREEN_QUAD),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create output texture for egui display
        let output_size = (size.width.max(1), size.height.max(1));
        let (output_texture, output_view) = Self::create_output_texture(&device, output_size.0, output_size.1, surface_format);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            timing_buffer,
            feedback_buffer,
            synth_buffer,
            feedback_textures,
            feedback_views,
            feedback_bind_groups,
            feedback_sampler,
            current_feedback_index: 0,
            timing_bind_group_layout,
            feedback_bind_group_layout,
            timing_bind_group,
            output_texture,
            output_view,
            output_texture_id: None,
            output_size,
            feedback,
            synth,
            graph_header_buffer,
            graph_ops_buffer,
            graph_bind_group_layout,
            graph_bind_group,
            compiled_graph,
        }
    }

    fn create_output_texture(device: &wgpu::Device, width: u32, height: u32, format: wgpu::TextureFormat) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Output Texture"),
            size: wgpu::Extent3d {
                width,
                height,
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

    fn create_feedback_textures(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> ([wgpu::Texture; 2], [wgpu::TextureView; 2], [wgpu::BindGroup; 2]) {
        let texture_desc = wgpu::TextureDescriptor {
            label: Some("Feedback Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format, // Use the same format as the surface
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        };

        let tex_a = device.create_texture(&texture_desc);
        let tex_b = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Feedback Texture B"),
            ..texture_desc
        });

        let view_a = tex_a.create_view(&wgpu::TextureViewDescriptor::default());
        let view_b = tex_b.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Feedback Bind Group A"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view_a),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });

        let bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Feedback Bind Group B"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view_b),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });

        ([tex_a, tex_b], [view_a, view_b], [bind_group_a, bind_group_b])
    }

    /// Get reference to wgpu device (for egui)
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Get surface format (for egui)
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    /// Get reference to queue (for egui)
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Compile and upload a new node graph to the GPU
    pub fn update_graph(&mut self, patch: &SignalPatch) {
        use crate::gpu::graph_compiler::GraphCompiler;

        self.compiled_graph = GraphCompiler::compile(patch);

        // Upload header
        self.queue.write_buffer(
            &self.graph_header_buffer,
            0,
            bytemuck::cast_slice(&[self.compiled_graph.header]),
        );

        // Upload operations
        self.queue.write_buffer(
            &self.graph_ops_buffer,
            0,
            bytemuck::cast_slice(&self.compiled_graph.ops),
        );
    }

    /// Update only node parameters without recompiling the graph structure
    /// This is much faster than update_graph() when only parameters changed
    pub fn update_graph_params(&mut self, patch: &SignalPatch) {
        // Update parameters in the compiled ops
        if self.compiled_graph.update_params(patch) {
            // Only upload if something changed
            self.queue.write_buffer(
                &self.graph_ops_buffer,
                0,
                bytemuck::cast_slice(&self.compiled_graph.ops),
            );
        }
    }

    /// Register output texture with egui for display in UI
    pub fn register_output_texture(&mut self, egui_renderer: &mut egui_wgpu::Renderer) {
        // Remove old texture if exists
        if let Some(old_id) = self.output_texture_id.take() {
            egui_renderer.free_texture(&old_id);
        }

        // Register new texture
        let id = egui_renderer.register_native_texture(
            &self.device,
            &self.output_view,
            wgpu::FilterMode::Linear,
        );
        self.output_texture_id = Some(id);
    }

    /// Get the output texture size
    pub fn output_size(&self) -> (u32, u32) {
        self.output_size
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
        let (texture, view) = Self::create_output_texture(&self.device, width, height, self.config.format);
        self.output_texture = texture;
        self.output_view = view;

        // Re-register with egui
        self.register_output_texture(egui_renderer);
    }

    /// Render synthesizer to output texture (for paned layout)
    pub fn render_to_texture(&mut self, timing: TimingUniforms) {
        // Update uniforms
        self.queue.write_buffer(&self.timing_buffer, 0, bytemuck::cast_slice(&[timing]));
        self.queue.write_buffer(&self.feedback_buffer, 0, bytemuck::cast_slice(&[self.feedback]));
        self.queue.write_buffer(&self.synth_buffer, 0, bytemuck::cast_slice(&[self.synth]));

        // Determine which feedback buffer to read from and write to
        let read_index = self.current_feedback_index;
        let write_index = 1 - self.current_feedback_index;

        // Create command encoder
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render To Texture Encoder"),
        });

        // Pass 1: Render to feedback texture (write_index)
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Feedback Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.feedback_views[write_index],
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

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.timing_bind_group, &[]);
            render_pass.set_bind_group(1, &self.feedback_bind_groups[read_index], &[]);
            render_pass.set_bind_group(2, &self.graph_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..6, 0..1);
        }

        // Pass 2: Copy feedback texture to output texture
        // Use GPU texture copy when sizes match (fast path), otherwise re-render (slow path)
        let feedback_size = (self.size.width, self.size.height);
        if feedback_size == self.output_size {
            // Fast path: direct GPU copy (no shader invocation)
            encoder.copy_texture_to_texture(
                wgpu::ImageCopyTexture {
                    texture: &self.feedback_textures[write_index],
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::ImageCopyTexture {
                    texture: &self.output_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: self.output_size.0,
                    height: self.output_size.1,
                    depth_or_array_layers: 1,
                },
            );
        } else {
            // Slow path: re-render when sizes differ (needed for aspect ratio scaling)
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Output Texture Pass"),
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

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.timing_bind_group, &[]);
            render_pass.set_bind_group(1, &self.feedback_bind_groups[write_index], &[]);
            render_pass.set_bind_group(2, &self.graph_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..6, 0..1);
        }

        // Swap buffers
        self.current_feedback_index = write_index;

        // Submit
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            // Recreate feedback textures at new size
            let (textures, views, bind_groups) = Self::create_feedback_textures(
                &self.device,
                &self.feedback_bind_group_layout,
                &self.feedback_sampler,
                new_size.width,
                new_size.height,
                self.config.format,
            );
            self.feedback_textures = textures;
            self.feedback_views = views;
            self.feedback_bind_groups = bind_groups;
        }
    }

    pub fn render(&mut self, timing: TimingUniforms) -> Result<(), wgpu::SurfaceError> {
        // Update uniforms
        self.queue.write_buffer(&self.timing_buffer, 0, bytemuck::cast_slice(&[timing]));
        self.queue.write_buffer(&self.feedback_buffer, 0, bytemuck::cast_slice(&[self.feedback]));
        self.queue.write_buffer(&self.synth_buffer, 0, bytemuck::cast_slice(&[self.synth]));

        // Get output texture
        let output = self.surface.get_current_texture()?;
        let output_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Determine which feedback buffer to read from and write to
        let read_index = self.current_feedback_index;
        let write_index = 1 - self.current_feedback_index;

        // Create command encoder
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Pass 1: Render to feedback texture (write_index)
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Feedback Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.feedback_views[write_index],
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

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.timing_bind_group, &[]);
            render_pass.set_bind_group(1, &self.feedback_bind_groups[read_index], &[]);
            render_pass.set_bind_group(2, &self.graph_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..6, 0..1);
        }

        // Pass 2: Copy feedback texture to screen
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Screen Render Pass"),
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

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.timing_bind_group, &[]);
            render_pass.set_bind_group(1, &self.feedback_bind_groups[write_index], &[]);
            render_pass.set_bind_group(2, &self.graph_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..6, 0..1);
        }

        // Swap buffers
        self.current_feedback_index = write_index;

        // Submit
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Render with egui overlay
    pub fn render_with_egui(
        &mut self,
        timing: TimingUniforms,
        egui_ctx: &egui::Context,
        textures_delta: egui::TexturesDelta,
        egui_renderer: &mut egui_wgpu::Renderer,
        shapes: Vec<egui::epaint::ClippedShape>,
        scale_factor: f32,
    ) -> Result<(), wgpu::SurfaceError> {
        // Update uniforms
        self.queue.write_buffer(&self.timing_buffer, 0, bytemuck::cast_slice(&[timing]));
        self.queue.write_buffer(&self.feedback_buffer, 0, bytemuck::cast_slice(&[self.feedback]));
        self.queue.write_buffer(&self.synth_buffer, 0, bytemuck::cast_slice(&[self.synth]));

        // Get output texture
        let output = self.surface.get_current_texture()?;
        let output_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Determine which feedback buffer to read from and write to
        let read_index = self.current_feedback_index;
        let write_index = 1 - self.current_feedback_index;

        // Create command encoder
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Pass 1: Render to feedback texture (write_index)
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Feedback Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.feedback_views[write_index],
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

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.timing_bind_group, &[]);
            render_pass.set_bind_group(1, &self.feedback_bind_groups[read_index], &[]);
            render_pass.set_bind_group(2, &self.graph_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..6, 0..1);
        }

        // Pass 2: Copy feedback texture to screen
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Screen Render Pass"),
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

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.timing_bind_group, &[]);
            render_pass.set_bind_group(1, &self.feedback_bind_groups[write_index], &[]);
            render_pass.set_bind_group(2, &self.graph_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..6, 0..1);
        }

        // Swap buffers
        self.current_feedback_index = write_index;

        // Handle egui texture updates
        for (id, image_delta) in &textures_delta.set {
            egui_renderer.update_texture(&self.device, &self.queue, *id, image_delta);
        }

        // Tessellate shapes into primitives
        let paint_jobs = egui_ctx.tessellate(shapes, scale_factor);

        // Prepare egui render
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.size.width, self.size.height],
            pixels_per_point: scale_factor,
        };

        // Prepare egui rendering (uses same encoder to batch submissions)
        egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );

        // Render egui pass (in same encoder as synth passes)
        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Don't clear - we want to draw over the synth
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Convert to 'static lifetime as required by egui_wgpu
            let mut render_pass = render_pass.forget_lifetime();
            egui_renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
        }

        // Free textures
        for id in &textures_delta.free {
            egui_renderer.free_texture(id);
        }

        // Single batched submission (synth + egui in one command buffer)
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
