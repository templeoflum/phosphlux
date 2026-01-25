//! Phosphlux - Real-time GPU Analog Video Synthesizer
//!
//! *phosphor + flux — flowing light substance*
//!
//! GPU-accelerated emulation of classic analog video synthesis hardware.
//! Architecture mirrors actual hardware signal flow:
//! - Sync generator provides master timing (NTSC/PAL)
//! - Modules process signals in parallel on GPU (like analog circuits)
//! - Feedback loops emerge from node topology (Frame Buffer nodes)
//! - CRT emulation via CRT Display nodes

mod core;
mod gpu;
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
use crate::gpu::renderer::Renderer;
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

    // Node graph
    patch: SignalPatch,
    node_editor: NodeEditor,
    graph_needs_update: bool,

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
            patch: SignalPatch::default_patch(),
            node_editor: NodeEditor::new(),
            graph_needs_update: true,  // Compile initial graph
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

        // Update graph if patch has changed
        if self.graph_needs_update {
            if let Some(renderer) = &mut self.renderer {
                renderer.update_graph(&self.patch);
            }
            self.graph_needs_update = false;
        }

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
            let patch = &mut self.patch;
            let node_editor = &mut self.node_editor;
            let last_preview_size = &mut self.last_preview_size;
            let renderer = self.renderer.as_mut().unwrap();
            let egui_renderer = self.egui_renderer.as_mut().unwrap();

            self.egui_ctx.run(raw_input, |ctx| {
                let graph_changed = render_node_editor_layout(
                    ctx,
                    patch,
                    node_editor,
                    last_preview_size,
                    renderer,
                    egui_renderer,
                );
                // Store the graph_changed flag for later access
                if graph_changed {
                    node_editor.graph_modified = true;
                }
            })
        };

        // Check if graph structure changed (requires full recompile)
        if self.node_editor.graph_modified {
            if let Some(renderer) = &mut self.renderer {
                renderer.update_graph(&self.patch);
            }
            self.node_editor.graph_modified = false;
            self.node_editor.params_modified = false;  // Clear params too since we just recompiled
            self.graph_needs_update = false;
        }
        // Check if only parameters changed (lightweight update - no recompile)
        else if self.node_editor.params_modified {
            if let Some(renderer) = &mut self.renderer {
                renderer.update_graph_params(&self.patch);
            }
            self.node_editor.params_modified = false;
        }

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
    patch: &mut SignalPatch,
    node_editor: &mut NodeEditor,
    last_preview_size: &mut (u32, u32),
    renderer: &mut Renderer,
    egui_renderer: &mut egui_wgpu::Renderer,
) -> bool {
    let mut graph_changed = false;

    // Top menu bar
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New Patch").clicked() {
                    *patch = SignalPatch::new();
                    graph_changed = true;
                    ui.close_menu();
                }
                if ui.button("Default Patch").clicked() {
                    *patch = SignalPatch::default_patch();
                    graph_changed = true;
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
                ui.label("  Drag nodes to move");
                ui.label("  Drag output → input to connect");
                ui.label("  Drag input to unplug");
                ui.label("  Right-click to delete");
                ui.label("  Middle-drag to pan");
                ui.label("  Scroll to zoom");
                ui.label("  Del - Delete selected");
                ui.label("  F - Frame selected");
                ui.label("  Home - Reset view");
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
            render_node_palette(ui, node_editor, patch);
        });

    // Right panel - Preview + Inspector (stacked vertically)
    egui::SidePanel::right("right_panel")
        .resizable(true)
        .default_width(280.0)
        .width_range(200.0..=400.0)
        .show(ctx, |ui| {
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

            // Calculate preview size based on panel width and aspect ratio
            let panel_width = ui.available_width();
            let max_preview_height = 300.0_f32; // Cap preview height
            let content_area = egui::Vec2::new(panel_width.max(64.0), max_preview_height);
            let preview_size = node_editor.aspect_ratio.fit_size(content_area);
            let preview_width = preview_size.x.max(64.0) as u32;
            let preview_h = preview_size.y.max(64.0) as u32;

            // Resize output texture if needed
            if (preview_width, preview_h) != *last_preview_size {
                renderer.resize_output(preview_width, preview_h, egui_renderer);
                *last_preview_size = (preview_width, preview_h);
            }

            // Display the synthesizer output - sized exactly to content
            ui.horizontal(|ui| {
                // Center horizontally
                let extra_width = (panel_width - preview_size.x).max(0.0);
                if extra_width > 0.0 {
                    ui.add_space(extra_width / 2.0);
                }
                if let Some(texture_id) = renderer.output_texture_id {
                    ui.add(egui::Image::new(egui::load::SizedTexture::new(texture_id, preview_size)));
                } else {
                    ui.label("Initializing...");
                }
            });

            ui.separator();

            // Bottom section: Inspector (shows selected node parameters)
            egui::ScrollArea::vertical().show(ui, |ui| {
                render_inspector(ui, node_editor, patch);
            });
        });

    // Central panel - Node Graph Editor
    egui::CentralPanel::default()
        .frame(egui::Frame::dark_canvas(&ctx.style()))
        .show(ctx, |ui| {
            render_node_canvas(ui, node_editor, patch);
        });

    // Return whether graph was modified (combine with File menu changes)
    graph_changed || node_editor.graph_modified
}

fn main() {
    env_logger::init();
    log::info!("Starting Phosphlux");

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll); // Continuous rendering

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
