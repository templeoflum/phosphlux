//! Off-screen rendering of a patch, for tests (T-0374).
//!
//! Composes the patch the way the app does (`ShaderComposer::compose_plan`), renders its single
//! segment into an Rgba32Float target with no window, and reads the floats back: the device's
//! output before any clamp and before sRGB, as FAITHFUL.md §2 asks for.

use wgpu::util::DeviceExt;

use crate::gpu::{DevicePatch, SegmentKind, SegmentShader, ShaderComposer};

/// Render `patch` at `width`×`height`; returns row-major RGBA floats (top row first).
/// Single-pass patches only. Errors are returned as text so a test can report them.
pub fn render_patch_f32(patch: &DevicePatch, width: u32, height: u32) -> Result<Vec<[f32; 4]>, String> {
    let plan = ShaderComposer::new().compose_plan(patch).map_err(|e| e.to_string())?;
    if plan.segments.len() != 1 {
        return Err(format!("expected one segment, got {}", plan.segments.len()));
    }
    render_segment_f32(patch, &plan.segments[0], width, height)
}

/// Name, backend and driver of the adapter `render_segment_f32` picks, for a report to record.
pub fn adapter_summary() -> Option<String> {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))?;
    let i = adapter.get_info();
    Some(format!("{} ({:?}, {:?}, driver {:?})", i.name, i.backend, i.device_type, i.driver_info))
}

/// Render one composed segment of a plan on its own (no input textures), e.g. the pass that
/// feeds a scan modulator, to read back what that pass hands on.
pub fn render_segment_f32(
    patch: &DevicePatch,
    segment: &SegmentShader,
    width: u32,
    height: u32,
) -> Result<Vec<[f32; 4]>, String> {
    if segment.kind != SegmentKind::Composed || segment.input_texture_count != 0 {
        return Err("only a composed segment with no input textures can render alone".into());
    }

    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
        .ok_or("no GPU adapter")?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
        .map_err(|e| e.to_string())?;

    let uniform_layout = |label| {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(label),
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
        })
    };
    let timing_layout = uniform_layout("Offscreen Timing BGL");
    let params_layout = uniform_layout("Offscreen Params BGL");
    let empty_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Offscreen Empty BGL"),
        entries: &[],
    });

    let timing_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Offscreen Timing"),
        contents: &[0u8; 64],
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let mut param_values = patch.collect_param_values(&segment.param_layout);
    while param_values.len() < 4 || param_values.len() % 4 != 0 {
        param_values.push(0.0);
    }
    let params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Offscreen Params"),
        contents: bytemuck::cast_slice(&param_values),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let bind = |layout: &wgpu::BindGroupLayout, buf: &wgpu::Buffer| {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: buf.as_entire_binding() }],
        })
    };
    let timing_bg = bind(&timing_layout, &timing_buf);
    let params_bg = bind(&params_layout, &params_buf);
    let empty_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &empty_layout,
        entries: &[],
    });

    let format = wgpu::TextureFormat::Rgba32Float;
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Offscreen Segment"),
        source: wgpu::ShaderSource::Wgsl(segment.wgsl_source.as_str().into()),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&timing_layout, &empty_layout, &params_layout],
        push_constant_ranges: &[],
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Offscreen Pipeline"),
        layout: Some(&layout),
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
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Offscreen Target"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let row_bytes = width * 16;
    let padded = row_bytes.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Offscreen Readback"),
        size: (padded * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Offscreen Pass"),
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
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &timing_bg, &[]);
        pass.set_bind_group(1, &empty_bg, &[]);
        pass.set_bind_group(2, &params_bg, &[]);
        pass.draw(0..3, 0..1);
    }
    encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            texture: &target,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::ImageCopyBuffer {
            buffer: &readback,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
    );
    queue.submit(Some(encoder.finish()));

    let slice = readback.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::Maintain::Wait);
    let data = slice.get_mapped_range();
    let mut out = Vec::with_capacity((width * height) as usize);
    for row in 0..height as usize {
        let start = row * padded as usize;
        let floats: &[f32] = bytemuck::cast_slice(&data[start..start + row_bytes as usize]);
        out.extend(floats.chunks_exact(4).map(|c| [c[0], c[1], c[2], c[3]]));
    }
    Ok(out)
}

/// Render all upstream passes at a chosen time, using the window renderer's sRGB
/// 8-bit textures and linear clamp-to-edge sampling. Returned bytes are display RGB.
#[cfg(feature = "startup-checks")]
pub fn render_patch_rgba8_at(
    patch: &DevicePatch,
    width: u32,
    height: u32,
    time: f32,
) -> Result<Vec<[u8; 4]>, String> {
    let plan = ShaderComposer::new()
        .compose_plan(patch)
        .map_err(|e| e.to_string())?;
    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .ok_or("no GPU adapter")?;
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
            .map_err(|e| e.to_string())?;

    let uniform_layout = |label| {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(label),
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
        })
    };
    let timing_layout = uniform_layout("Offscreen Timing BGL");
    let params_layout = uniform_layout("Offscreen Params BGL");
    let timing_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Offscreen Timing"),
        contents: bytemuck::bytes_of(&crate::core::timing::TimingUniforms {
            time,
            lines: 480.0,
            field_rate: 59.94,
            ..bytemuck::Zeroable::zeroed()
        }),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let bind = |layout: &wgpu::BindGroupLayout, buf: &wgpu::Buffer| {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buf.as_entire_binding(),
            }],
        })
    };
    let timing_bg = bind(&timing_layout, &timing_buf);
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let mut targets = Vec::new();
    let mut views = Vec::new();
    for _ in &plan.segments {
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Offscreen Target"),
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
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = target.create_view(&wgpu::TextureViewDescriptor::default());
        targets.push(target);
        views.push(view);
    }
    let row_bytes = width * 4;
    let padded =
        row_bytes.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Offscreen Readback"),
        size: (padded * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    for (index, segment) in plan.segments.iter().enumerate() {
        if segment.input_source_segments.len() != segment.input_texture_count as usize
            || segment
                .input_source_segments
                .iter()
                .any(|&source| source >= index)
        {
            return Err(format!(
                "segment {index}: missing or non-upstream texture source"
            ));
        }
        let mut layout_entries = Vec::new();
        let mut input_entries = Vec::new();
        for (slot, &source) in segment.input_source_segments.iter().enumerate() {
            let binding = (slot * 2) as u32;
            layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            });
            layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: binding + 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
            input_entries.push(wgpu::BindGroupEntry {
                binding,
                resource: wgpu::BindingResource::TextureView(&views[source]),
            });
            input_entries.push(wgpu::BindGroupEntry {
                binding: binding + 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            });
        }
        let input_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Offscreen Input BGL"),
            entries: &layout_entries,
        });
        let input_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Offscreen Input BG"),
            layout: &input_layout,
            entries: &input_entries,
        });
        let mut param_values = patch.collect_param_values(&segment.param_layout);
        while param_values.len() < 4 || param_values.len() % 4 != 0 {
            param_values.push(0.0);
        }
        let params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Offscreen Params"),
            contents: bytemuck::cast_slice(&param_values),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let params_bg = bind(&params_layout, &params_buf);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Offscreen Segment"),
            source: wgpu::ShaderSource::Wgsl(segment.wgsl_source.as_str().into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&timing_layout, &input_layout, &params_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Offscreen Pipeline"),
            layout: Some(&layout),
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
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Offscreen Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &views[index],
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
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &timing_bg, &[]);
            pass.set_bind_group(1, &input_bg, &[]);
            pass.set_bind_group(2, &params_bg, &[]);
            pass.draw(0..3, 0..1);
        }
    }
    encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            texture: targets.last().ok_or("empty render plan")?,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::ImageCopyBuffer {
            buffer: &readback,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(Some(encoder.finish()));

    let slice = readback.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::Maintain::Wait);
    let data = slice.get_mapped_range();
    let mut out = Vec::with_capacity((width * height) as usize);
    for row in 0..height as usize {
        let start = row * padded as usize;
        out.extend(
            data[start..start + row_bytes as usize]
                .chunks_exact(4)
                .map(|c| [c[0], c[1], c[2], c[3]]),
        );
    }
    Ok(out)
}

/// Minimal 8-bit greyscale PNG (stored, uncompressed deflate), so tests need no image crate.
pub fn grey_png(width: u32, height: u32, pixels: &[u8]) -> Vec<u8> {
    encode_png(width, height, pixels, 1, 0)
}

#[cfg(feature = "startup-checks")]
pub fn rgb_png(width: u32, height: u32, pixels: &[[u8; 4]]) -> Vec<u8> {
    let rgb: Vec<u8> = pixels.iter().flat_map(|p| p[..3].iter().copied()).collect();
    encode_png(width, height, &rgb, 3, 2)
}

fn encode_png(width: u32, height: u32, pixels: &[u8], channels: u32, color_type: u8) -> Vec<u8> {
    fn crc32(bytes: &[u8]) -> u32 {
        let mut c = 0xFFFF_FFFFu32;
        for &b in bytes {
            c ^= b as u32;
            for _ in 0..8 {
                c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
            }
        }
        !c
    }
    fn chunk(png: &mut Vec<u8>, kind: &[u8], data: &[u8]) {
        png.extend((data.len() as u32).to_be_bytes());
        let mut body = kind.to_vec();
        body.extend(data);
        png.extend(&body);
        png.extend(crc32(&body).to_be_bytes());
    }
    let mut raw = Vec::with_capacity(((width * channels + 1) * height) as usize);
    for row in pixels.chunks_exact((width * channels) as usize) {
        raw.push(0); // filter: none
        raw.extend(row);
    }
    let mut z = vec![0x78, 0x01];
    let blocks: Vec<&[u8]> = raw.chunks(65_535).collect();
    for (i, block) in blocks.iter().enumerate() {
        z.push((i + 1 == blocks.len()) as u8);
        let len = block.len() as u16;
        z.extend(len.to_le_bytes());
        z.extend((!len).to_le_bytes());
        z.extend(*block);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for &x in &raw {
        a = (a + x as u32) % 65_521;
        b = (b + a) % 65_521;
    }
    z.extend(((b << 16) | a).to_be_bytes());

    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut ihdr = Vec::new();
    ihdr.extend(width.to_be_bytes());
    ihdr.extend(height.to_be_bytes());
    ihdr.extend([8, color_type, 0, 0, 0]);
    chunk(&mut png, b"IHDR", &ihdr);
    chunk(&mut png, b"IDAT", &z);
    chunk(&mut png, b"IEND", &[]);
    png
}
