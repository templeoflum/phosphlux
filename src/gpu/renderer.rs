//! GPU Renderer for Phosphlux v2
//!
//! Manages the wgpu device, surface, and render pipeline.
//! Unlike v1's fixed pipeline, v2 recompiles the pipeline when
//! the patch topology changes (compose-based architecture).
//!
//! v2.1: Multi-pass rendering with segment pipelines.
//! When a patch contains ScanModulators, the renderer breaks
//! the signal chain into multiple render passes with intermediate textures.

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

// Dummy group 1 (empty) and group 2 (params placeholder)
struct DeviceParams { _pad0: f32, _pad1: f32, _pad2: f32, _pad3: f32, }
@group(2) @binding(0) var<uniform> params: DeviceParams;

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

// ─── Segment Pipeline ────────────────────────────────────────

/// A compiled render pipeline for one segment of the render plan.
struct SegmentPipeline {
    pipeline: wgpu::RenderPipeline,
    params_buffer: wgpu::Buffer,
    params_bind_group: wgpu::BindGroup,
    /// Group 1 bind group: empty (no input texture) or texture+sampler
    input_bind_group: wgpu::BindGroup,
    param_layout: ParamLayout,
}

// ─── Renderer ────────────────────────────────────────────────

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,

    // Fallback pipeline (used before first recompile)
    fallback_pipeline: wgpu::RenderPipeline,
    fallback_params_buffer: wgpu::Buffer,
    fallback_params_bind_group: wgpu::BindGroup,

    // Multi-pass segment pipelines (populated after recompile)
    segment_pipelines: Vec<SegmentPipeline>,

    // Bind group layouts
    timing_bg_layout: wgpu::BindGroupLayout,
    empty_bg_layout: wgpu::BindGroupLayout,
    texture_input_bg_layout: wgpu::BindGroupLayout,
    params_bg_layout: wgpu::BindGroupLayout,

    // Global bind groups + buffers
    timing_bind_group: wgpu::BindGroup,
    timing_buffer: wgpu::Buffer,
    empty_bind_group: wgpu::BindGroup,

    // Shared sampler for intermediate textures
    sampler: wgpu::Sampler,

    // Intermediate textures for multi-pass rendering
    intermediate_textures: Vec<(wgpu::Texture, wgpu::TextureView)>,

    // Default 1x1 black texture (used when scan modulator has no upstream)
    black_texture_view: wgpu::TextureView,
    black_bind_group: wgpu::BindGroup,

    // Output texture for egui preview display
    output_texture: wgpu::Texture,
    output_view: wgpu::TextureView,
    pub output_texture_id: Option<egui::TextureId>,
    output_size: (u32, u32),

    // State
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
        let timing_bg_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        // Group 1 (empty): Placeholder for segments without input textures
        let empty_bg_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Empty BG Layout"),
                entries: &[],
            });

        // Group 1 (texture): For segments that read from a previous segment's output
        let texture_input_bg_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Input BG Layout"),
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

        // Group 2: Params uniform
        let params_bg_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        let fallback_params_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Fallback Params Buffer"),
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

        let fallback_params_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Fallback Params BG"),
            layout: &params_bg_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: fallback_params_buffer.as_entire_binding(),
            }],
        });

        // === Sampler for intermediate textures ===
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Segment Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // === Default 1x1 black texture (for scan modulators with no upstream) ===
        let black_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Black 1x1"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: surface_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        // Write black pixel
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &black_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[0u8, 0, 0, 255], // BGRA black
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        let black_texture_view =
            black_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let black_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Black Texture BG"),
            layout: &texture_input_bg_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&black_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // === Fallback pipeline ===
        let fallback_pipeline = Self::create_pipeline(
            &device,
            FALLBACK_SHADER,
            surface_format,
            &timing_bg_layout,
            &empty_bg_layout,
            &params_bg_layout,
        );

        // === Output texture for egui ===
        let output_size = (320, 240);
        let (output_texture, output_view) =
            Self::create_output_texture(&device, output_size.0, output_size.1, surface_format);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            fallback_pipeline,
            fallback_params_buffer,
            fallback_params_bind_group,
            segment_pipelines: Vec::new(),
            timing_bg_layout,
            empty_bg_layout,
            texture_input_bg_layout,
            params_bg_layout,
            timing_bind_group,
            timing_buffer,
            empty_bind_group,
            sampler,
            intermediate_textures: Vec::new(),
            black_texture_view,
            black_bind_group,
            output_texture,
            output_view,
            output_texture_id: None,
            output_size,
            shader_error: None,
        }
    }

    /// Create a render pipeline from WGSL source with specified group layouts.
    fn create_pipeline(
        device: &wgpu::Device,
        wgsl: &str,
        format: wgpu::TextureFormat,
        timing_layout: &wgpu::BindGroupLayout,
        group1_layout: &wgpu::BindGroupLayout,
        params_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Segment Shader"),
            source: wgpu::ShaderSource::Wgsl(wgsl.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[timing_layout, group1_layout, params_layout],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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

    /// Create an intermediate texture for multi-pass rendering.
    fn create_intermediate_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        index: usize,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("Intermediate Texture {}", index)),
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
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    // ─── Recompile ───────────────────────────────────────────

    /// Recompile the pipeline from a device patch.
    /// Uses compose_plan() for multi-pass rendering when scan modulators are present.
    pub fn recompile(&mut self, patch: &DevicePatch, composer: &mut ShaderComposer) {
        match composer.compose_plan(patch) {
            Ok(plan) => {
                self.build_segment_pipelines(&plan, patch);
                self.shader_error = None;
                log::info!(
                    "Recompiled: {} segment(s), {} intermediate texture(s)",
                    self.segment_pipelines.len(),
                    self.intermediate_textures.len()
                );
            }
            Err(e) => {
                self.shader_error = Some(format!("{}", e));
                log::error!("Shader compose error: {}", e);
            }
        }
    }

    /// Build per-segment pipelines and intermediate textures from a render plan.
    fn build_segment_pipelines(
        &mut self,
        plan: &crate::gpu::shader_composer::RenderPlan,
        patch: &DevicePatch,
    ) {
        self.segment_pipelines.clear();
        self.intermediate_textures.clear();

        let num_segments = plan.segments.len();

        // Create intermediate textures (one per segment except the last)
        let num_intermediates = if num_segments > 1 {
            num_segments - 1
        } else {
            0
        };
        for i in 0..num_intermediates {
            let (tex, view) = Self::create_intermediate_texture(
                &self.device,
                self.output_size.0,
                self.output_size.1,
                self.config.format,
                i,
            );
            self.intermediate_textures.push((tex, view));
        }

        // Build a pipeline for each segment
        for (i, segment) in plan.segments.iter().enumerate() {
            // Choose group 1 layout based on whether this segment needs input texture
            let group1_layout = if segment.needs_input_texture {
                &self.texture_input_bg_layout
            } else {
                &self.empty_bg_layout
            };

            // Create pipeline
            let pipeline = Self::create_pipeline(
                &self.device,
                &segment.wgsl_source,
                self.config.format,
                &self.timing_bg_layout,
                group1_layout,
                &self.params_bg_layout,
            );

            // Create params buffer
            let param_values = patch.collect_param_values(&segment.param_layout);
            let param_bytes = bytemuck::cast_slice(&param_values);
            let params_buffer =
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(&format!("Segment {} Params Buffer", i)),
                        contents: param_bytes,
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    });

            let params_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("Segment {} Params BG", i)),
                layout: &self.params_bg_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                }],
            });

            // Create input bind group
            let input_bind_group = if segment.needs_input_texture {
                if i > 0 {
                    // Bind previous segment's output texture
                    let prev_view = &self.intermediate_textures[i - 1].1;
                    self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some(&format!("Segment {} Input BG", i)),
                        layout: &self.texture_input_bg_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(prev_view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(&self.sampler),
                            },
                        ],
                    })
                } else {
                    // First segment but needs input: use black texture
                    self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Segment 0 Black Input BG"),
                        layout: &self.texture_input_bg_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(
                                    &self.black_texture_view,
                                ),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(&self.sampler),
                            },
                        ],
                    })
                }
            } else {
                // No input texture: use empty bind group
                self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(&format!("Segment {} Empty Input BG", i)),
                    layout: &self.empty_bg_layout,
                    entries: &[],
                })
            };

            self.segment_pipelines.push(SegmentPipeline {
                pipeline,
                params_buffer,
                params_bind_group,
                input_bind_group,
                param_layout: segment.param_layout.clone(),
            });
        }
    }

    // ─── Parameter Updates ───────────────────────────────────

    /// Update parameter values without recompiling.
    /// Updates each segment's params buffer independently.
    pub fn update_params(&mut self, patch: &DevicePatch) {
        for segment in &self.segment_pipelines {
            let values = patch.collect_param_values(&segment.param_layout);
            let bytes = bytemuck::cast_slice(&values);
            self.queue
                .write_buffer(&segment.params_buffer, 0, bytes);
        }
    }

    // ─── Rendering ───────────────────────────────────────────

    /// Render composed shader to output texture.
    /// Executes all segments in order, writing to intermediate textures
    /// and finally to the output texture.
    pub fn render_to_texture(&mut self, timing: TimingUniforms) {
        // Update timing uniform
        self.queue.write_buffer(
            &self.timing_buffer,
            0,
            bytemuck::cast_slice(&[timing]),
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render To Texture"),
            });

        if self.segment_pipelines.is_empty() {
            // Fallback: use fallback pipeline
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Fallback Synth Pass"),
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

                pass.set_pipeline(&self.fallback_pipeline);
                pass.set_bind_group(0, &self.timing_bind_group, &[]);
                pass.set_bind_group(1, &self.empty_bind_group, &[]);
                pass.set_bind_group(2, &self.fallback_params_bind_group, &[]);
                pass.draw(0..3, 0..1);
            }
        } else {
            // Multi-pass rendering: execute segments in order
            let num_segments = self.segment_pipelines.len();
            for i in 0..num_segments {
                let target_view = if i < num_segments - 1 {
                    &self.intermediate_textures[i].1
                } else {
                    &self.output_view
                };

                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some(&format!("Segment {} Pass", i)),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: target_view,
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

                    let segment = &self.segment_pipelines[i];
                    pass.set_pipeline(&segment.pipeline);
                    pass.set_bind_group(0, &self.timing_bind_group, &[]);
                    pass.set_bind_group(1, &segment.input_bind_group, &[]);
                    pass.set_bind_group(2, &segment.params_bind_group, &[]);
                    pass.draw(0..3, 0..1);
                }
            }
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
    pub fn resize_output(
        &mut self,
        width: u32,
        height: u32,
        egui_renderer: &mut egui_wgpu::Renderer,
    ) {
        if width == 0 || height == 0 {
            return;
        }
        if self.output_size == (width, height) {
            return;
        }

        self.output_size = (width, height);
        let (texture, view) =
            Self::create_output_texture(&self.device, width, height, self.config.format);
        self.output_texture = texture;
        self.output_view = view;
        self.register_output_texture(egui_renderer);

        // Resize intermediate textures to match
        self.recreate_intermediate_textures();
    }

    /// Recreate intermediate textures at the current output size.
    /// Called when output is resized or after recompile.
    fn recreate_intermediate_textures(&mut self) {
        let count = self.intermediate_textures.len();
        if count == 0 {
            return;
        }

        self.intermediate_textures.clear();
        for i in 0..count {
            let (tex, view) = Self::create_intermediate_texture(
                &self.device,
                self.output_size.0,
                self.output_size.1,
                self.config.format,
                i,
            );
            self.intermediate_textures.push((tex, view));
        }

        // Rebuild input bind groups for segments that reference intermediate textures
        for i in 0..self.segment_pipelines.len() {
            if i > 0 && i - 1 < self.intermediate_textures.len() {
                // Check if this segment needs input texture by looking at bind group layout
                // We reconstruct the bind group with the new texture view
                let prev_view = &self.intermediate_textures[i - 1].1;
                // Only rebuild if this segment had a texture input
                // We can check by trying to create with texture layout
                let new_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(&format!("Segment {} Input BG (resized)", i)),
                    layout: &self.texture_input_bg_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(prev_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                    ],
                });
                self.segment_pipelines[i].input_bind_group = new_bg;
            }
        }
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
        let surface_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Frame Encoder"),
            });

        // Pass 1: Render synth to output texture (multi-pass segments)
        if self.segment_pipelines.is_empty() {
            // Fallback
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Fallback Synth Pass"),
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
            pass.set_pipeline(&self.fallback_pipeline);
            pass.set_bind_group(0, &self.timing_bind_group, &[]);
            pass.set_bind_group(1, &self.empty_bind_group, &[]);
            pass.set_bind_group(2, &self.fallback_params_bind_group, &[]);
            pass.draw(0..3, 0..1);
        } else {
            let num_segments = self.segment_pipelines.len();
            for i in 0..num_segments {
                let target_view = if i < num_segments - 1 {
                    &self.intermediate_textures[i].1
                } else {
                    &self.output_view
                };

                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some(&format!("Segment {} Pass", i)),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: target_view,
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

                    let segment = &self.segment_pipelines[i];
                    pass.set_pipeline(&segment.pipeline);
                    pass.set_bind_group(0, &self.timing_bind_group, &[]);
                    pass.set_bind_group(1, &segment.input_bind_group, &[]);
                    pass.set_bind_group(2, &segment.params_bind_group, &[]);
                    pass.draw(0..3, 0..1);
                }
            }
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
            // Pass drops immediately -- just clearing
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
                        load: wgpu::LoadOp::Load,
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
