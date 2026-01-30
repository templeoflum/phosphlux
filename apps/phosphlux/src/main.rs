//! Phosphlux v2 - Analog Video Synthesizer
//!
//! A real-time GPU-accelerated emulator of historical video synthesis hardware.
//!
//! ## Architecture
//!
//! v2 uses a **device-centric** architecture:
//! - Each historical device (Paik/Abe, Sandin, Jones, etc.) is a module
//! - Devices expose authentic circuit behavior through WGSL shaders
//! - The shader composer builds specialized shaders per patch topology
//! - No runtime dispatch - maximum GPU efficiency

mod core;
mod devices;
mod gpu;
mod ui;

use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::core::timing::{SyncGenerator, VideoStandard};
use crate::gpu::renderer::Renderer;
use crate::gpu::{DevicePatch, ShaderComposer};
use crate::ui::node_editor::{
    render_inspector, render_node_canvas, render_node_palette, AspectRatio, NodeEditor,
};
use crate::ui::theme::apply_theme;

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    sync_gen: SyncGenerator,
    composer: ShaderComposer,

    // egui integration
    egui_ctx: egui::Context,
    egui_state: Option<egui_winit::State>,
    egui_renderer: Option<egui_wgpu::Renderer>,

    // Node graph
    patch: DevicePatch,
    node_editor: NodeEditor,

    // Track preview size for texture resize
    last_preview_size: (u32, u32),
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            sync_gen: SyncGenerator::new(VideoStandard::NTSC),
            composer: ShaderComposer::new(),
            egui_ctx: egui::Context::default(),
            egui_state: None,
            egui_renderer: None,
            patch: Self::default_patch(),
            node_editor: NodeEditor::new(),
            last_preview_size: (320, 240),
        }
    }

    /// Create the default patch: TestSignal -> PaikAbeColorizer (top-down flow)
    fn default_patch() -> DevicePatch {
        let mut patch = DevicePatch::new();

        // Top-down layout: signal generator at top, colorizer below
        let signal_id = patch.add_device(
            Box::new(devices::TestSignalGenerator::new()),
            100.0,
            50.0,
        );
        let colorizer_id = patch.add_device(
            Box::new(devices::PaikAbeColorizer::new()),
            100.0,
            180.0,
        );

        patch.connect(signal_id, "video_out", colorizer_id, "video_in");
        patch.output_device = colorizer_id;

        patch
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Phosphlux")
                .with_maximized(true);

            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

            // Initialize GPU renderer
            let mut renderer = pollster::block_on(Renderer::new(Arc::clone(&window)));

            // Initialize egui
            let egui_state = egui_winit::State::new(
                self.egui_ctx.clone(),
                egui::ViewportId::ROOT,
                &window,
                Some(window.scale_factor() as f32),
                None,
                None,
            );

            let mut egui_renderer = egui_wgpu::Renderer::new(
                renderer.device(),
                renderer.surface_format(),
                None,
                1,
                false,
            );

            // Register output texture with egui
            renderer.register_output_texture(&mut egui_renderer);

            // Apply vintage oscilloscope theme
            apply_theme(&self.egui_ctx);

            // Initial compile of the patch
            renderer.recompile(&self.patch, &mut self.composer);

            self.egui_state = Some(egui_state);
            self.egui_renderer = Some(egui_renderer);
            self.renderer = Some(renderer);
            self.window = Some(window);

            log::info!("Window created with node editor layout");
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) {
        // Pass events to egui first
        if let Some(state) = &mut self.egui_state {
            if let Some(window) = &self.window {
                let response = state.on_window_event(window.as_ref(), &event);
                if response.consumed {
                    return;
                }
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                log::info!("Close requested, exiting");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.sync_gen.tick();
                self.render_frame(event_loop);

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(physical_size);
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                if key_code == KeyCode::Escape {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }
}

impl App {
    fn render_frame(&mut self, event_loop: &ActiveEventLoop) {
        let timing = self.sync_gen.get_timing();

        // Apply LFO automation before rendering
        let lfo_time = self.node_editor.lfo_time();
        if self.node_editor.apply_lfos(&mut self.patch, lfo_time) {
            if let Some(renderer) = &mut self.renderer {
                renderer.update_params(&self.patch);
            }
        }

        // Render synth to texture (before egui build, so preview is up to date)
        if let Some(renderer) = &mut self.renderer {
            renderer.render_to_texture(timing);
        }

        // Collect what we need for egui
        let window = match &self.window {
            Some(w) => w.clone(),
            None => return,
        };

        let egui_state = match &mut self.egui_state {
            Some(s) => s,
            None => return,
        };

        let raw_input = egui_state.take_egui_input(window.as_ref());

        // Build UI
        let full_output = {
            let patch = &mut self.patch;
            let node_editor = &mut self.node_editor;
            let last_preview_size = &mut self.last_preview_size;
            let renderer = self.renderer.as_mut().unwrap();
            let egui_renderer = self.egui_renderer.as_mut().unwrap();

            self.egui_ctx.run(raw_input, |ctx| {
                render_node_editor_layout(
                    ctx,
                    patch,
                    node_editor,
                    last_preview_size,
                    renderer,
                    egui_renderer,
                );
            })
        };

        // Check if graph structure changed → recompile
        if self.node_editor.graph_modified {
            if let Some(renderer) = &mut self.renderer {
                renderer.recompile(&self.patch, &mut self.composer);
            }
            self.node_editor.graph_modified = false;
            self.node_editor.params_modified = false;
        }
        // Check if only params changed → lightweight update
        else if self.node_editor.params_modified {
            if let Some(renderer) = &mut self.renderer {
                renderer.update_params(&self.patch);
            }
            self.node_editor.params_modified = false;
        }

        // Handle platform output
        if let Some(egui_state) = &mut self.egui_state {
            egui_state.handle_platform_output(window.as_ref(), full_output.platform_output);
        }

        // Present frame
        if let (Some(renderer), Some(egui_renderer)) =
            (&mut self.renderer, &mut self.egui_renderer)
        {
            match renderer.render_with_egui(
                timing,
                &self.egui_ctx,
                full_output.textures_delta,
                egui_renderer,
                full_output.shapes,
                window.scale_factor() as f32,
            ) {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                    renderer.resize(renderer.size);
                }
                Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                Err(e) => log::error!("Render error: {:?}", e),
            }
        }
    }
}

/// Node editor layout with 3 panels
fn render_node_editor_layout(
    ctx: &egui::Context,
    patch: &mut DevicePatch,
    node_editor: &mut NodeEditor,
    last_preview_size: &mut (u32, u32),
    renderer: &mut Renderer,
    egui_renderer: &mut egui_wgpu::Renderer,
) {
    // Top menu bar
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New Patch").clicked() {
                    *patch = DevicePatch::new();
                    node_editor.graph_modified = true;
                    ui.close_menu();
                }
                if ui.button("Default Patch").clicked() {
                    *patch = App::default_patch();
                    node_editor.graph_modified = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Exit").clicked() {
                    std::process::exit(0);
                }
            });
            ui.menu_button("Help", |ui| {
                ui.label("Keyboard:");
                ui.label("  Esc - Exit");
                ui.separator();
                ui.label("Node Editor:");
                ui.label("  Drag output \u{2192} input to connect");
                ui.label("  Drag input to unplug");
                ui.label("  Right-click to delete");
                ui.label("  Middle-drag to pan");
                ui.label("  Scroll to zoom");
                ui.label("  Del - Delete selected");
                ui.label("  F - Frame selected");
                ui.label("  Home - Reset view");
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(ref err) = renderer.shader_error {
                    ui.label(
                        egui::RichText::new(format!("Shader Error: {}", err))
                            .color(egui::Color32::RED),
                    );
                } else {
                    ui.label(egui::RichText::new("OK").color(egui::Color32::GREEN));
                }
            });
        });
    });

    // Left panel - Device Palette + Inspector
    egui::SidePanel::left("device_palette")
        .resizable(true)
        .default_width(220.0)
        .width_range(150.0..=400.0)
        .show(ctx, |ui| {
            render_node_palette(ui, node_editor, patch);
            ui.separator();
            egui::ScrollArea::vertical()
                .id_salt("inspector_scroll")
                .show(ui, |ui| {
                    render_inspector(ui, node_editor, patch);
                });
        });

    // Right panel - Preview Monitor
    egui::SidePanel::right("right_panel")
        .resizable(true)
        .default_width(500.0)
        .width_range(200.0..=1200.0)
        .show(ctx, |ui| {
            // Preview header with aspect ratio selector
            ui.horizontal(|ui| {
                ui.heading("Preview");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::ComboBox::from_id_salt("aspect_ratio")
                        .selected_text(node_editor.aspect_ratio.display_name())
                        .show_ui(ui, |ui| {
                            for ar in [
                                AspectRatio::Free,
                                AspectRatio::Square,
                                AspectRatio::Standard,
                                AspectRatio::Wide,
                                AspectRatio::Portrait43,
                                AspectRatio::Portrait169,
                                AspectRatio::Cinemascope,
                            ] {
                                ui.selectable_value(
                                    &mut node_editor.aspect_ratio,
                                    ar,
                                    ar.display_name(),
                                );
                            }
                        });
                });
            });

            // Preview display — full available space
            let panel_width = ui.available_width();
            let available_height = ui.available_height();
            let content_area = egui::Vec2::new(panel_width.max(64.0), available_height.max(64.0));
            let preview_size = node_editor.aspect_ratio.fit_size(content_area);
            let preview_w = preview_size.x.max(64.0) as u32;
            let preview_h = preview_size.y.max(64.0) as u32;

            if (preview_w, preview_h) != *last_preview_size {
                renderer.resize_output(preview_w, preview_h, egui_renderer);
                *last_preview_size = (preview_w, preview_h);
            }

            // Center the preview image
            let extra_width = (panel_width - preview_size.x).max(0.0);
            let extra_height = (available_height - preview_size.y).max(0.0);
            if extra_height > 0.0 {
                ui.add_space(extra_height / 2.0);
            }
            ui.horizontal(|ui| {
                if extra_width > 0.0 {
                    ui.add_space(extra_width / 2.0);
                }
                if let Some(texture_id) = renderer.output_texture_id {
                    ui.add(egui::Image::new(egui::load::SizedTexture::new(
                        texture_id,
                        preview_size,
                    )));
                } else {
                    ui.label("Initializing...");
                }
            });
        });

    // Central panel - Node Graph Editor
    egui::CentralPanel::default()
        .frame(egui::Frame::dark_canvas(&ctx.style()))
        .show(ctx, |ui| {
            render_node_canvas(ui, node_editor, patch);
        });
}

fn main() {
    env_logger::init();
    log::info!("Starting Phosphlux v2");

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
