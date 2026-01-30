//! Phosphlux Lite - Simple Fixed-Chain Video Synthesizer
//!
//! A unified instrument capturing the aesthetic of vintage analog video synthesis.
//! Inspired by Paik/Abe, Sandin IP, Rutt/Etra, Jones Colorizer, and more.

mod app;
mod presets;
mod renderer;
mod synth;
mod ui;

use app::App;
use renderer::Renderer;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

const WINDOW_WIDTH: u32 = 1024;
const WINDOW_HEIGHT: u32 = 768;
const SYNTH_WIDTH: u32 = 640;
const SYNTH_HEIGHT: u32 = 480;

struct AppState {
    window: Arc<Window>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    synth_renderer: Renderer,
    app: App,
    last_frame_time: instant::Instant,
    egui_texture_id: egui::TextureId,
}

struct PhosphluxLite {
    state: Option<AppState>,
}

impl PhosphluxLite {
    fn new() -> Self {
        Self { state: None }
    }
}

impl ApplicationHandler for PhosphluxLite {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        // Create window
        let window_attrs = Window::default_attributes()
            .with_title("Phosphlux Lite")
            .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));

        let window = Arc::new(event_loop.create_window(window_attrs).unwrap());

        // Initialize wgpu
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("Failed to find adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
            },
            None,
        ))
        .expect("Failed to create device");

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        // Configure surface
        let size = window.inner_size();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        // Initialize egui
        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx,
            egui::ViewportId::ROOT,
            event_loop,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        let mut egui_renderer = egui_wgpu::Renderer::new(&device, surface_format, None, 1, false);

        // Create synth renderer
        let synth_renderer =
            Renderer::new(device.clone(), queue.clone(), SYNTH_WIDTH, SYNTH_HEIGHT);

        // Register synth output texture with egui
        let egui_texture_id = egui_renderer.register_native_texture(
            &device,
            synth_renderer.output_view(),
            wgpu::FilterMode::Linear,
        );

        self.state = Some(AppState {
            window,
            device,
            queue,
            surface,
            surface_config,
            egui_state,
            egui_renderer,
            synth_renderer,
            app: App::new(),
            last_frame_time: instant::Instant::now(),
            egui_texture_id,
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(s) => s,
            None => return,
        };

        // Pass events to egui
        let _ = state.egui_state.on_window_event(&state.window, &event);

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if new_size.width > 0 && new_size.height > 0 {
                    state.surface_config.width = new_size.width;
                    state.surface_config.height = new_size.height;
                    state.surface.configure(&state.device, &state.surface_config);
                }
            }
            WindowEvent::RedrawRequested => {
                // Calculate delta time
                let now = instant::Instant::now();
                let dt = now.duration_since(state.last_frame_time).as_secs_f32();
                state.last_frame_time = now;

                // Update app
                state.app.update(dt);

                // Render synth
                state.synth_renderer.render(
                    &state.app.synth,
                    state.app.time,
                    state.app.frame,
                );

                // Update egui texture
                state.egui_renderer.update_egui_texture_from_wgpu_texture(
                    &state.device,
                    state.synth_renderer.output_view(),
                    wgpu::FilterMode::Linear,
                    state.egui_texture_id,
                );

                // Get surface texture
                let output = match state.surface.get_current_texture() {
                    Ok(t) => t,
                    Err(wgpu::SurfaceError::Lost) => {
                        state.surface.configure(&state.device, &state.surface_config);
                        return;
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        event_loop.exit();
                        return;
                    }
                    Err(_) => return,
                };

                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                // Run egui
                let raw_input = state.egui_state.take_egui_input(&state.window);
                let egui_ctx = state.egui_state.egui_ctx().clone();

                let full_output = egui_ctx.run(raw_input, |ctx| {
                    // Draw UI
                    ui::draw_ui(ctx, &mut state.app);

                    // Draw video preview in center
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.centered_and_justified(|ui| {
                            let available = ui.available_size();
                            let aspect = SYNTH_WIDTH as f32 / SYNTH_HEIGHT as f32;

                            // Calculate size maintaining aspect ratio
                            let (w, h) = if available.x / available.y > aspect {
                                (available.y * aspect, available.y)
                            } else {
                                (available.x, available.x / aspect)
                            };

                            ui.image(egui::ImageSource::Texture(egui::load::SizedTexture::new(
                                state.egui_texture_id,
                                [w, h],
                            )));
                        });
                    });
                });

                // Handle platform output
                state
                    .egui_state
                    .handle_platform_output(&state.window, full_output.platform_output);

                // Prepare egui render
                let tris = egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

                for (id, delta) in &full_output.textures_delta.set {
                    state
                        .egui_renderer
                        .update_texture(&state.device, &state.queue, *id, delta);
                }

                let screen_descriptor = egui_wgpu::ScreenDescriptor {
                    size_in_pixels: [state.surface_config.width, state.surface_config.height],
                    pixels_per_point: full_output.pixels_per_point,
                };

                let mut encoder =
                    state
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Egui Encoder"),
                        });

                state.egui_renderer.update_buffers(
                    &state.device,
                    &state.queue,
                    &mut encoder,
                    &tris,
                    &screen_descriptor,
                );

                {
                    let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Egui Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1,
                                    g: 0.1,
                                    b: 0.1,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });

                    let mut render_pass = render_pass.forget_lifetime();
                    state
                        .egui_renderer
                        .render(&mut render_pass, &tris, &screen_descriptor);
                }

                state.queue.submit(std::iter::once(encoder.finish()));
                output.present();

                // Free textures
                for id in &full_output.textures_delta.free {
                    state.egui_renderer.free_texture(id);
                }

                // Request another frame
                state.window.request_redraw();
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = PhosphluxLite::new();
    event_loop.run_app(&mut app).unwrap();
}
