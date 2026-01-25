//! Control Panel UI
//!
//! Interactive parameter controls using egui

use egui::{Context, Slider, Window, ComboBox, CollapsingHeader, Color32, RichText};
use crate::gpu::renderer::FeedbackUniforms;

/// Synthesis parameters that can be controlled via UI
#[derive(Clone, Debug)]
pub struct SynthParams {
    // Feedback section
    pub feedback: FeedbackUniforms,

    // Oscillator section
    pub osc1_freq: f32,
    pub osc1_phase: f32,
    pub osc2_freq: f32,
    pub osc2_phase: f32,

    // Noise section
    pub noise_mix: f32,
    pub noise_type: NoiseType,

    // Keyer section
    pub key_enabled: bool,
    pub key_clip_black: f32,
    pub key_clip_white: f32,
    pub key_invert: bool,

    // Colorizer section
    pub colorizer_type: ColorizerType,
    pub colorizer_levels: f32,

    // Rutt/Etra section
    pub rutt_etra_enabled: bool,
    pub rutt_etra_amount: f32,

    // CRT section
    pub scanline_intensity: f32,
    pub curvature: f32,
    pub vignette_amount: f32,

    // Shape section
    pub shape_type: ShapeType,
    pub shape_size: f32,
    pub shape_softness: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NoiseType {
    White,
    Pink,
    Brown,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColorizerType {
    Spectrum,
    Threshold,
    Grayscale,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShapeType {
    None,
    Circle,
    Rectangle,
    Diamond,
    Star,
    Bars,
    Checker,
}

impl Default for SynthParams {
    fn default() -> Self {
        Self {
            feedback: FeedbackUniforms::default(),
            osc1_freq: 4.0,
            osc1_phase: 0.0,
            osc2_freq: 3.0,
            osc2_phase: 0.0,
            noise_mix: 0.1,
            noise_type: NoiseType::Pink,
            key_enabled: false,
            key_clip_black: 0.2,
            key_clip_white: 0.8,
            key_invert: false,
            colorizer_type: ColorizerType::Spectrum,
            colorizer_levels: 8.0,
            rutt_etra_enabled: true,
            rutt_etra_amount: 0.15,
            scanline_intensity: 0.12,
            curvature: 0.1,
            vignette_amount: 0.4,
            shape_type: ShapeType::Circle,
            shape_size: 0.3,
            shape_softness: 0.05,
        }
    }
}

/// Main control panel
pub struct ControlPanel {
    pub params: SynthParams,
    pub show_feedback: bool,
    pub show_oscillators: bool,
    pub show_noise: bool,
    pub show_keyer: bool,
    pub show_colorizer: bool,
    pub show_rutt_etra: bool,
    pub show_crt: bool,
    pub show_shapes: bool,
    pub show_presets: bool,
    pub preset_name: String,
    pub compact_mode: bool,
}

impl Default for ControlPanel {
    fn default() -> Self {
        Self {
            params: SynthParams::default(),
            show_feedback: true,
            show_oscillators: false,
            show_noise: false,
            show_keyer: true,
            show_colorizer: false,
            show_rutt_etra: true,
            show_crt: false,
            show_shapes: false,
            show_presets: true,
            preset_name: String::from("My Preset"),
            compact_mode: false,
        }
    }
}

impl ControlPanel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Render the control panel UI
    pub fn ui(&mut self, ctx: &Context) {
        Window::new("🎛 Analog Video Synth")
            .default_pos([10.0, 10.0])
            .default_width(280.0)
            .resizable(true)
            .collapsible(true)
            .title_bar(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Reset All").clicked() {
                        self.params = SynthParams::default();
                    }
                    ui.checkbox(&mut self.compact_mode, "Compact");
                    ui.separator();
                    ui.label(RichText::new("60 FPS").color(Color32::GREEN));
                });

                // Apply compact spacing if enabled
                if self.compact_mode {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.spacing_mut().slider_width = 100.0;
                }

                ui.separator();

                // Presets section
                CollapsingHeader::new("📁 Presets")
                    .default_open(self.show_presets)
                    .show(ui, |ui| {
                        self.presets_ui(ui);
                    });

                // Feedback section
                CollapsingHeader::new("🔄 Feedback")
                    .default_open(self.show_feedback)
                    .show(ui, |ui| {
                        self.feedback_ui(ui);
                    });

                // Keyer section
                CollapsingHeader::new("🔑 Luma Key")
                    .default_open(self.show_keyer)
                    .show(ui, |ui| {
                        self.keyer_ui(ui);
                    });

                // Rutt/Etra section
                CollapsingHeader::new("📺 Rutt/Etra")
                    .default_open(self.show_rutt_etra)
                    .show(ui, |ui| {
                        self.rutt_etra_ui(ui);
                    });

                // Oscillators section
                CollapsingHeader::new("〰 Oscillators")
                    .default_open(self.show_oscillators)
                    .show(ui, |ui| {
                        self.oscillators_ui(ui);
                    });

                // Noise section
                CollapsingHeader::new("📡 Noise")
                    .default_open(self.show_noise)
                    .show(ui, |ui| {
                        self.noise_ui(ui);
                    });

                // Colorizer section
                CollapsingHeader::new("🎨 Colorizer")
                    .default_open(self.show_colorizer)
                    .show(ui, |ui| {
                        self.colorizer_ui(ui);
                    });

                // Shapes section
                CollapsingHeader::new("⬡ Shapes")
                    .default_open(self.show_shapes)
                    .show(ui, |ui| {
                        self.shapes_ui(ui);
                    });

                // CRT section
                CollapsingHeader::new("📟 CRT Effects")
                    .default_open(self.show_crt)
                    .show(ui, |ui| {
                        self.crt_ui(ui);
                    });
            });
    }

    fn presets_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut self.preset_name);
        });

        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                log::info!("Saving preset: {}", self.preset_name);
                // TODO: Implement preset saving
            }
            if ui.button("Load").clicked() {
                log::info!("Loading preset: {}", self.preset_name);
                // TODO: Implement preset loading
            }
        });

        ui.separator();
        ui.label("Quick Presets:");

        ui.horizontal(|ui| {
            if ui.button("Spiral").clicked() {
                self.apply_preset_spiral();
            }
            if ui.button("Tunnel").clicked() {
                self.apply_preset_tunnel();
            }
            if ui.button("Chaos").clicked() {
                self.apply_preset_chaos();
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Drift").clicked() {
                self.apply_preset_drift();
            }
            if ui.button("Kaleidoscope").clicked() {
                self.apply_preset_kaleidoscope();
            }
        });
    }

    fn feedback_ui(&mut self, ui: &mut egui::Ui) {
        let fb = &mut self.params.feedback;

        ui.add(Slider::new(&mut fb.mix, 0.0..=0.99).text("Mix"));
        ui.add(Slider::new(&mut fb.zoom, 0.9..=1.1).text("Zoom"));
        ui.add(Slider::new(&mut fb.rotation, -0.1..=0.1).text("Rotation"));
        ui.add(Slider::new(&mut fb.hue_shift, -0.05..=0.05).text("Hue Shift"));
        ui.add(Slider::new(&mut fb.decay, 0.9..=1.0).text("Decay"));
        ui.add(Slider::new(&mut fb.saturation, 0.0..=2.0).text("Saturation"));

        ui.separator();
        ui.label("Offset:");
        ui.horizontal(|ui| {
            ui.add(Slider::new(&mut fb.offset_x, -0.1..=0.1).text("X"));
            ui.add(Slider::new(&mut fb.offset_y, -0.1..=0.1).text("Y"));
        });
    }

    fn keyer_ui(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.params.key_enabled, "Enable Luma Key");

        if self.params.key_enabled {
            ui.add(Slider::new(&mut self.params.key_clip_black, 0.0..=1.0).text("Clip Black"));
            ui.add(Slider::new(&mut self.params.key_clip_white, 0.0..=1.0).text("Clip White"));
            ui.checkbox(&mut self.params.key_invert, "Invert Key");
        }
    }

    fn rutt_etra_ui(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.params.rutt_etra_enabled, "Enable Z-Displacement");

        if self.params.rutt_etra_enabled {
            ui.add(Slider::new(&mut self.params.rutt_etra_amount, 0.0..=0.5).text("Amount"));
        }
    }

    fn oscillators_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Oscillator 1 (Horizontal):");
        ui.add(Slider::new(&mut self.params.osc1_freq, 0.1..=20.0).text("Frequency"));
        ui.add(Slider::new(&mut self.params.osc1_phase, 0.0..=1.0).text("Phase"));

        ui.separator();

        ui.label("Oscillator 2 (Vertical):");
        ui.add(Slider::new(&mut self.params.osc2_freq, 0.1..=20.0).text("Frequency"));
        ui.add(Slider::new(&mut self.params.osc2_phase, 0.0..=1.0).text("Phase"));
    }

    fn noise_ui(&mut self, ui: &mut egui::Ui) {
        ui.add(Slider::new(&mut self.params.noise_mix, 0.0..=1.0).text("Mix"));

        ComboBox::from_label("Type")
            .selected_text(format!("{:?}", self.params.noise_type))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.params.noise_type, NoiseType::White, "White");
                ui.selectable_value(&mut self.params.noise_type, NoiseType::Pink, "Pink");
                ui.selectable_value(&mut self.params.noise_type, NoiseType::Brown, "Brown");
            });
    }

    fn colorizer_ui(&mut self, ui: &mut egui::Ui) {
        ComboBox::from_label("Type")
            .selected_text(format!("{:?}", self.params.colorizer_type))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.params.colorizer_type, ColorizerType::Spectrum, "Spectrum");
                ui.selectable_value(&mut self.params.colorizer_type, ColorizerType::Threshold, "Threshold");
                ui.selectable_value(&mut self.params.colorizer_type, ColorizerType::Grayscale, "Grayscale");
            });

        if self.params.colorizer_type == ColorizerType::Threshold {
            ui.add(Slider::new(&mut self.params.colorizer_levels, 2.0..=16.0).text("Levels"));
        }
    }

    fn shapes_ui(&mut self, ui: &mut egui::Ui) {
        ComboBox::from_label("Shape")
            .selected_text(format!("{:?}", self.params.shape_type))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.params.shape_type, ShapeType::None, "None");
                ui.selectable_value(&mut self.params.shape_type, ShapeType::Circle, "Circle");
                ui.selectable_value(&mut self.params.shape_type, ShapeType::Rectangle, "Rectangle");
                ui.selectable_value(&mut self.params.shape_type, ShapeType::Diamond, "Diamond");
                ui.selectable_value(&mut self.params.shape_type, ShapeType::Star, "Star");
                ui.selectable_value(&mut self.params.shape_type, ShapeType::Bars, "Bars");
                ui.selectable_value(&mut self.params.shape_type, ShapeType::Checker, "Checker");
            });

        if self.params.shape_type != ShapeType::None {
            ui.add(Slider::new(&mut self.params.shape_size, 0.05..=0.8).text("Size"));
            ui.add(Slider::new(&mut self.params.shape_softness, 0.0..=0.2).text("Softness"));
        }
    }

    fn crt_ui(&mut self, ui: &mut egui::Ui) {
        ui.add(Slider::new(&mut self.params.scanline_intensity, 0.0..=0.5).text("Scanlines"));
        ui.add(Slider::new(&mut self.params.curvature, 0.0..=0.3).text("Curvature"));
        ui.add(Slider::new(&mut self.params.vignette_amount, 0.0..=1.0).text("Vignette"));
    }

    // Preset implementations
    fn apply_preset_spiral(&mut self) {
        self.params.feedback = FeedbackUniforms {
            mix: 0.85,
            zoom: 0.98,
            rotation: 0.015,
            hue_shift: 0.003,
            offset_x: 0.0,
            offset_y: 0.0,
            saturation: 1.0,
            decay: 0.99,
        };
        log::info!("Applied preset: Spiral");
    }

    fn apply_preset_tunnel(&mut self) {
        self.params.feedback = FeedbackUniforms {
            mix: 0.9,
            zoom: 0.95,
            rotation: 0.0,
            hue_shift: 0.005,
            offset_x: 0.0,
            offset_y: 0.0,
            saturation: 1.2,
            decay: 0.98,
        };
        log::info!("Applied preset: Tunnel");
    }

    fn apply_preset_chaos(&mut self) {
        self.params.feedback = FeedbackUniforms {
            mix: 0.92,
            zoom: 0.97,
            rotation: 0.02,
            hue_shift: 0.01,
            offset_x: 0.003,
            offset_y: 0.002,
            saturation: 1.3,
            decay: 0.995,
        };
        log::info!("Applied preset: Chaos");
    }

    fn apply_preset_drift(&mut self) {
        self.params.feedback = FeedbackUniforms {
            mix: 0.8,
            zoom: 0.995,
            rotation: 0.002,
            hue_shift: 0.001,
            offset_x: 0.001,
            offset_y: 0.0,
            saturation: 0.9,
            decay: 0.97,
        };
        log::info!("Applied preset: Drift");
    }

    fn apply_preset_kaleidoscope(&mut self) {
        self.params.feedback = FeedbackUniforms {
            mix: 0.88,
            zoom: 0.99,
            rotation: 0.025,
            hue_shift: 0.008,
            offset_x: 0.0,
            offset_y: 0.0,
            saturation: 1.5,
            decay: 0.99,
        };
        log::info!("Applied preset: Kaleidoscope");
    }
}
