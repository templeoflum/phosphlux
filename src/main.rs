//! Phosphlux - Real-time GPU Analog Video Synthesizer
//!
//! *phosphor + flux — flowing light substance*
//!
//! GPU-accelerated emulation of classic analog video synthesis hardware.
//! Architecture mirrors actual hardware signal flow:
//! - Sync generator provides master timing (NTSC/PAL)
//! - Modules process signals in parallel on GPU (like analog circuits)
//! - Feedback loops with minimal latency
//! - CRT emulation as final output stage

mod core;
mod gpu;
mod modules;
mod ui;

use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};
use std::sync::Arc;

use crate::core::timing::{SyncGenerator, VideoStandard};
use crate::gpu::renderer::{Renderer, SynthUniforms};
use crate::ui::ControlPanel;
use crate::ui::controls::{SynthParams, NoiseType, ColorizerType, ShapeType};
use crate::ui::patch::SignalPatch;
use crate::ui::node_editor::{NodeEditor, AspectRatio, render_node_canvas, render_node_palette, render_inspector};

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    sync_gen: SyncGenerator,
    frame_count: u64,

    // egui integration
    egui_ctx: egui::Context,
    egui_state: Option<egui_winit::State>,
    egui_renderer: Option<egui_wgpu::Renderer>,
    control_panel: ControlPanel,

    // Node graph
    patch: SignalPatch,
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
            frame_count: 0,
            egui_ctx: egui::Context::default(),
            egui_state: None,
            egui_renderer: None,
            control_panel: ControlPanel::new(),
            patch: SignalPatch::default_patch(),
            node_editor: NodeEditor::new(),
            last_preview_size: (320, 240),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            // Create maximized window
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

            // Register output texture with egui for paned display
            renderer.register_output_texture(&mut egui_renderer);

            self.egui_state = Some(egui_state);
            self.egui_renderer = Some(egui_renderer);
            self.renderer = Some(renderer);
            self.window = Some(window);

            log::info!("Window created with node editor layout");
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
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
                // Advance sync generator
                self.sync_gen.tick();
                self.frame_count += 1;

                // Sync UI params to renderer
                if let Some(renderer) = &mut self.renderer {
                    renderer.feedback = self.control_panel.params.feedback;
                    sync_synth_params(&self.control_panel.params, &mut renderer.synth);
                }

                // Render frame
                self.render_frame(event_loop);

                // Request next frame
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
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(key_code),
                    state: ElementState::Pressed,
                    ..
                },
                ..
            } => {
                match key_code {
                    KeyCode::Escape => {
                        event_loop.exit();
                    }
                    KeyCode::Space => {
                        self.control_panel.params = ui::controls::SynthParams::default();
                        log::info!("Reset to defaults");
                    }
                    KeyCode::Digit1 => {
                        self.control_panel.params.feedback = crate::gpu::renderer::FeedbackUniforms {
                            mix: 0.85, zoom: 0.98, rotation: 0.015, hue_shift: 0.003,
                            offset_x: 0.0, offset_y: 0.0, saturation: 1.0, decay: 0.99,
                        };
                    }
                    KeyCode::Digit2 => {
                        self.control_panel.params.feedback = crate::gpu::renderer::FeedbackUniforms {
                            mix: 0.9, zoom: 0.95, rotation: 0.0, hue_shift: 0.005,
                            offset_x: 0.0, offset_y: 0.0, saturation: 1.2, decay: 0.98,
                        };
                    }
                    KeyCode::Digit3 => {
                        self.control_panel.params.feedback = crate::gpu::renderer::FeedbackUniforms {
                            mix: 0.92, zoom: 0.97, rotation: 0.02, hue_shift: 0.01,
                            offset_x: 0.003, offset_y: 0.002, saturation: 1.3, decay: 0.995,
                        };
                    }
                    KeyCode::Digit4 => {
                        self.control_panel.params.feedback = crate::gpu::renderer::FeedbackUniforms {
                            mix: 0.8, zoom: 0.995, rotation: 0.002, hue_shift: 0.001,
                            offset_x: 0.001, offset_y: 0.0, saturation: 0.9, decay: 0.97,
                        };
                    }
                    KeyCode::Digit5 => {
                        self.control_panel.params.feedback = crate::gpu::renderer::FeedbackUniforms {
                            mix: 0.88, zoom: 0.99, rotation: 0.025, hue_shift: 0.008,
                            offset_x: 0.0, offset_y: 0.0, saturation: 1.5, decay: 0.99,
                        };
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

impl App {
    /// Main render frame logic - extracted to avoid borrow conflicts
    fn render_frame(&mut self, event_loop: &ActiveEventLoop) {
        let timing = self.sync_gen.get_timing();

        // Render synthesizer to texture
        if let Some(renderer) = &mut self.renderer {
            renderer.render_to_texture(timing);
        }

        // Collect data we need from self before entering egui closure
        let window = match &self.window {
            Some(w) => w.clone(),
            None => return,
        };

        let egui_state = match &mut self.egui_state {
            Some(s) => s,
            None => return,
        };

        let raw_input = egui_state.take_egui_input(window.as_ref());

        // Build UI - need to extract mutable state to pass to closure
        let full_output = {
            let control_panel_params = &mut self.control_panel.params;
            let patch = &mut self.patch;
            let node_editor = &mut self.node_editor;
            let last_preview_size = &mut self.last_preview_size;
            let renderer = self.renderer.as_mut().unwrap();
            let egui_renderer = self.egui_renderer.as_mut().unwrap();

            self.egui_ctx.run(raw_input, |ctx| {
                render_node_editor_layout(
                    ctx,
                    control_panel_params,
                    patch,
                    node_editor,
                    last_preview_size,
                    renderer,
                    egui_renderer,
                );
            })
        };

        // Handle platform output
        if let Some(egui_state) = &mut self.egui_state {
            egui_state.handle_platform_output(window.as_ref(), full_output.platform_output);
        }

        // Render egui to screen
        if let (Some(renderer), Some(egui_renderer)) = (&mut self.renderer, &mut self.egui_renderer) {
            match renderer.render_with_egui(
                timing,
                &self.egui_ctx,
                full_output.textures_delta,
                egui_renderer,
                full_output.shapes,
                window.scale_factor() as f32,
            ) {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size),
                Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                Err(e) => log::error!("Render error: {:?}", e),
            }
        }
    }
}

/// Node editor layout:
/// ┌─────────────────────────────────────────────────────────────────┐
/// │ Menu Bar                                                [FPS]   │
/// ├──────────┬────────────────────────────────────────┬─────────────┤
/// │          │                                        │  PREVIEW    │
/// │  NODE    │                                        │             │
/// │ PALETTE  │       NODE GRAPH EDITOR                ├─────────────┤
/// │          │                                        │  INSPECTOR  │
/// │          │                                        │             │
/// └──────────┴────────────────────────────────────────┴─────────────┘
fn render_node_editor_layout(
    ctx: &egui::Context,
    control_panel_params: &mut SynthParams,
    patch: &mut SignalPatch,
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
                    *patch = SignalPatch::new();
                    ui.close_menu();
                }
                if ui.button("Default Patch").clicked() {
                    *patch = SignalPatch::default_patch();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Exit").clicked() {
                    std::process::exit(0);
                }
            });
            ui.menu_button("Edit", |ui| {
                if ui.button("Reset Parameters").clicked() {
                    *control_panel_params = SynthParams::default();
                    ui.close_menu();
                }
            });
            ui.menu_button("Presets", |ui| {
                if ui.button("Spiral").clicked() {
                    control_panel_params.feedback = crate::gpu::renderer::FeedbackUniforms {
                        mix: 0.85, zoom: 0.98, rotation: 0.015, hue_shift: 0.003,
                        offset_x: 0.0, offset_y: 0.0, saturation: 1.0, decay: 0.99,
                    };
                    ui.close_menu();
                }
                if ui.button("Tunnel").clicked() {
                    control_panel_params.feedback = crate::gpu::renderer::FeedbackUniforms {
                        mix: 0.9, zoom: 0.95, rotation: 0.0, hue_shift: 0.005,
                        offset_x: 0.0, offset_y: 0.0, saturation: 1.2, decay: 0.98,
                    };
                    ui.close_menu();
                }
                if ui.button("Chaos").clicked() {
                    control_panel_params.feedback = crate::gpu::renderer::FeedbackUniforms {
                        mix: 0.92, zoom: 0.97, rotation: 0.02, hue_shift: 0.01,
                        offset_x: 0.003, offset_y: 0.002, saturation: 1.3, decay: 0.995,
                    };
                    ui.close_menu();
                }
                if ui.button("Drift").clicked() {
                    control_panel_params.feedback = crate::gpu::renderer::FeedbackUniforms {
                        mix: 0.8, zoom: 0.995, rotation: 0.002, hue_shift: 0.001,
                        offset_x: 0.001, offset_y: 0.0, saturation: 0.9, decay: 0.97,
                    };
                    ui.close_menu();
                }
                if ui.button("Kaleidoscope").clicked() {
                    control_panel_params.feedback = crate::gpu::renderer::FeedbackUniforms {
                        mix: 0.88, zoom: 0.99, rotation: 0.025, hue_shift: 0.008,
                        offset_x: 0.0, offset_y: 0.0, saturation: 1.5, decay: 0.99,
                    };
                    ui.close_menu();
                }
            });
            ui.menu_button("Help", |ui| {
                ui.label("Space - Reset params");
                ui.label("1-5 - Quick presets");
                ui.label("Esc - Exit");
                ui.separator();
                ui.label("Node Editor:");
                ui.label("  Drag nodes to move");
                ui.label("  Drag from output to input to connect");
                ui.label("  Right-click node to delete");
                ui.label("  Middle-drag or Alt+drag to pan");
                ui.label("  Scroll to zoom");
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new("60 FPS").color(egui::Color32::GREEN));
            });
        });
    });

    // Left panel - Node Palette
    egui::SidePanel::left("node_palette")
        .resizable(true)
        .default_width(150.0)
        .width_range(120.0..=250.0)
        .show(ctx, |ui| {
            render_node_palette(ui, patch);
        });

    // Right panel - Preview + Inspector (stacked vertically)
    egui::SidePanel::right("right_panel")
        .resizable(true)
        .default_width(280.0)
        .width_range(200.0..=400.0)
        .show(ctx, |ui| {
            // Top section: Preview
            let available_height = ui.available_height();
            let preview_height = (available_height * 0.45).max(150.0);

            egui::Frame::dark_canvas(&ui.style())
                .show(ui, |ui| {
                    ui.set_min_height(preview_height);
                    ui.set_max_height(preview_height);

                    // Preview header with aspect ratio selector
                    ui.horizontal(|ui| {
                        ui.heading("Preview");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            egui::ComboBox::from_id_salt("aspect_ratio")
                                .selected_text(node_editor.aspect_ratio.display_name())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut node_editor.aspect_ratio, AspectRatio::Free, AspectRatio::Free.display_name());
                                    ui.selectable_value(&mut node_editor.aspect_ratio, AspectRatio::Square, AspectRatio::Square.display_name());
                                    ui.selectable_value(&mut node_editor.aspect_ratio, AspectRatio::Standard, AspectRatio::Standard.display_name());
                                    ui.selectable_value(&mut node_editor.aspect_ratio, AspectRatio::Wide, AspectRatio::Wide.display_name());
                                    ui.selectable_value(&mut node_editor.aspect_ratio, AspectRatio::Portrait43, AspectRatio::Portrait43.display_name());
                                    ui.selectable_value(&mut node_editor.aspect_ratio, AspectRatio::Portrait169, AspectRatio::Portrait169.display_name());
                                    ui.selectable_value(&mut node_editor.aspect_ratio, AspectRatio::Cinemascope, AspectRatio::Cinemascope.display_name());
                                });
                        });
                    });

                    // Get available size for preview
                    let available = ui.available_size();
                    let content_area = egui::Vec2::new(available.x.max(64.0), (available.y - 8.0).max(64.0));

                    // Calculate size with aspect ratio lock
                    let preview_size = node_editor.aspect_ratio.fit_size(content_area);
                    let preview_width = preview_size.x as u32;
                    let preview_h = preview_size.y as u32;

                    // Resize output texture if needed
                    if (preview_width, preview_h) != *last_preview_size {
                        renderer.resize_output(preview_width, preview_h, egui_renderer);
                        *last_preview_size = (preview_width, preview_h);
                    }

                    // Center the preview in the available space
                    ui.vertical_centered(|ui| {
                        // Display the synthesizer output
                        if let Some(texture_id) = renderer.output_texture_id {
                            ui.add(egui::Image::new(egui::load::SizedTexture::new(texture_id, preview_size)));
                        } else {
                            ui.centered_and_justified(|ui| {
                                ui.label("Initializing...");
                            });
                        }
                    });
                });

            ui.separator();

            // Bottom section: Inspector
            egui::ScrollArea::vertical().show(ui, |ui| {
                render_inspector(ui, node_editor, patch);

                ui.separator();

                // Also show global parameters here
                ui.collapsing("Global Parameters", |ui| {
                    render_parameters_compact(ui, control_panel_params);
                });
            });
        });

    // Central panel - Node Graph Editor
    egui::CentralPanel::default()
        .frame(egui::Frame::dark_canvas(&ctx.style()))
        .show(ctx, |ui| {
            render_node_canvas(ui, node_editor, patch);
        });
}

/// Compact parameter view for the inspector panel
fn render_parameters_compact(ui: &mut egui::Ui, params: &mut SynthParams) {
    ui.collapsing("Feedback", |ui| {
        let fb = &mut params.feedback;
        ui.add(egui::Slider::new(&mut fb.mix, 0.0..=0.99).text("Mix"));
        ui.add(egui::Slider::new(&mut fb.zoom, 0.9..=1.1).text("Zoom"));
        ui.add(egui::Slider::new(&mut fb.rotation, -0.1..=0.1).text("Rotation"));
        ui.add(egui::Slider::new(&mut fb.hue_shift, -0.05..=0.05).text("Hue"));
        ui.add(egui::Slider::new(&mut fb.decay, 0.9..=1.0).text("Decay"));
    });

    ui.collapsing("Luma Key", |ui| {
        ui.checkbox(&mut params.key_enabled, "Enable");
        if params.key_enabled {
            ui.add(egui::Slider::new(&mut params.key_clip_black, 0.0..=1.0).text("Black"));
            ui.add(egui::Slider::new(&mut params.key_clip_white, 0.0..=1.0).text("White"));
        }
    });

    ui.collapsing("Shapes", |ui| {
        egui::ComboBox::from_label("Type")
            .selected_text(format!("{:?}", params.shape_type))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut params.shape_type, ShapeType::None, "None");
                ui.selectable_value(&mut params.shape_type, ShapeType::Circle, "Circle");
                ui.selectable_value(&mut params.shape_type, ShapeType::Rectangle, "Rect");
                ui.selectable_value(&mut params.shape_type, ShapeType::Diamond, "Diamond");
                ui.selectable_value(&mut params.shape_type, ShapeType::Star, "Star");
            });
        if params.shape_type != ShapeType::None {
            ui.add(egui::Slider::new(&mut params.shape_size, 0.05..=0.8).text("Size"));
        }
    });

    ui.collapsing("Colorizer", |ui| {
        egui::ComboBox::from_label("Mode")
            .selected_text(format!("{:?}", params.colorizer_type))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut params.colorizer_type, ColorizerType::Spectrum, "Spectrum");
                ui.selectable_value(&mut params.colorizer_type, ColorizerType::Threshold, "Threshold");
                ui.selectable_value(&mut params.colorizer_type, ColorizerType::Grayscale, "Gray");
            });
    });
}

/// Sync UI SynthParams to GPU SynthUniforms
fn sync_synth_params(params: &SynthParams, synth: &mut SynthUniforms) {
    // Oscillators
    synth.osc1_freq = params.osc1_freq;
    synth.osc1_phase = params.osc1_phase;
    synth.osc2_freq = params.osc2_freq;
    synth.osc2_phase = params.osc2_phase;

    // Noise
    synth.noise_mix = params.noise_mix;
    synth.noise_type = match params.noise_type {
        NoiseType::White => 0.0,
        NoiseType::Pink => 1.0,
        NoiseType::Brown => 2.0,
    };

    // Keyer
    synth.key_enabled = if params.key_enabled { 1.0 } else { 0.0 };
    synth.key_clip_black = params.key_clip_black;
    synth.key_clip_white = params.key_clip_white;
    synth.key_invert = if params.key_invert { 1.0 } else { 0.0 };

    // Luminance Displacement (renamed from Rutt/Etra)
    synth.rutt_etra_enabled = if params.rutt_etra_enabled { 1.0 } else { 0.0 };
    synth.rutt_etra_amount = params.rutt_etra_amount;

    // Colorizer
    synth.colorizer_type = match params.colorizer_type {
        ColorizerType::Spectrum => 0.0,
        ColorizerType::Threshold => 1.0,
        ColorizerType::Grayscale => 2.0,
    };
    synth.colorizer_levels = params.colorizer_levels;

    // CRT
    synth.scanline_intensity = params.scanline_intensity;
    synth.curvature = params.curvature;
    synth.vignette_amount = params.vignette_amount;

    // Shape
    synth.shape_type = match params.shape_type {
        ShapeType::None => 0.0,
        ShapeType::Circle => 1.0,
        ShapeType::Rectangle => 2.0,
        ShapeType::Diamond => 3.0,
        ShapeType::Star => 4.0,
        ShapeType::Bars => 5.0,
        ShapeType::Checker => 6.0,
    };
    synth.shape_size = params.shape_size;
    synth.shape_softness = params.shape_softness;
}

fn main() {
    env_logger::init();
    log::info!("Starting Phosphlux");

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll); // Continuous rendering

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
