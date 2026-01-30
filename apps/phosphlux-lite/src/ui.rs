//! User interface using egui

use crate::app::{App, SelectedStage};
use crate::synth::*;
use egui::{Color32, RichText, Ui};

/// Draw the complete UI
pub fn draw_ui(ctx: &egui::Context, app: &mut App) {
    // Top panel with title and presets
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("PHOSPHLUX LITE");
            ui.separator();

            // Preset selector
            let preset_name = app
                .current_preset
                .and_then(|i| app.presets.get(i))
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "(modified)".to_string());

            let preset_names: Vec<String> = app.presets.iter().map(|p| p.name.clone()).collect();
            let current = app.current_preset;

            let mut selected_preset: Option<usize> = None;
            egui::ComboBox::from_label("")
                .selected_text(&preset_name)
                .width(150.0)
                .show_ui(ui, |ui| {
                    for (i, name) in preset_names.iter().enumerate() {
                        if ui.selectable_label(current == Some(i), name).clicked() {
                            selected_preset = Some(i);
                        }
                    }
                });

            if let Some(idx) = selected_preset {
                app.load_preset(idx);
            }

            if ui.button("Randomize").clicked() {
                app.randomize();
            }
        });
    });

    // Bottom panel with stage tabs
    egui::TopBottomPanel::bottom("stage_panel")
        .min_height(200.0)
        .show(ctx, |ui| {
            // Stage tab bar
            ui.horizontal(|ui| {
                let stages = [
                    (SelectedStage::Input, "INPUT"),
                    (SelectedStage::Geometry, "GEOM"),
                    (SelectedStage::Amplitude, "AMP"),
                    (SelectedStage::Colorize, "COLOR"),
                    (SelectedStage::Mixer, "MIX"),
                    (SelectedStage::Feedback, "FEEDBACK"),
                    (SelectedStage::Output, "OUTPUT"),
                ];

                for (stage, label) in stages {
                    let selected = app.selected_stage == stage;
                    let text = if selected {
                        RichText::new(label).strong().color(Color32::WHITE)
                    } else {
                        RichText::new(label).color(Color32::GRAY)
                    };

                    if ui.selectable_label(selected, text).clicked() {
                        app.selected_stage = stage;
                    }
                }
            });

            ui.separator();

            // Stage-specific controls
            let modified = match app.selected_stage {
                SelectedStage::Input => draw_input_stage(ui, &mut app.synth.input),
                SelectedStage::Geometry => draw_geometry_stage(ui, &mut app.synth.geometry),
                SelectedStage::Amplitude => draw_amplitude_stage(ui, &mut app.synth.amplitude),
                SelectedStage::Colorize => draw_colorize_stage(ui, &mut app.synth.colorize),
                SelectedStage::Mixer => draw_mixer_stage(ui, &mut app.synth.mixer),
                SelectedStage::Feedback => draw_feedback_stage(ui, &mut app.synth.feedback),
                SelectedStage::Output => draw_output_stage(ui, &mut app.synth.output),
            };

            if modified {
                app.mark_modified();
            }
        });
}

fn draw_input_stage(ui: &mut Ui, input: &mut InputStage) -> bool {
    let mut modified = false;

    ui.columns(2, |cols| {
        // Left column
        cols[0].label("Source A:");
        modified |= source_combo(&mut cols[0], "source_a", &mut input.source_a);

        cols[0].add_space(8.0);
        cols[0].label("Source B:");
        modified |= source_combo(&mut cols[0], "source_b", &mut input.source_b);

        // Right column
        cols[1].label("Mix A/B:");
        modified |= cols[1]
            .add(egui::Slider::new(&mut input.mix, 0.0..=1.0))
            .changed();

        cols[1].label("Frequency:");
        modified |= cols[1]
            .add(egui::Slider::new(&mut input.frequency, 0.5..=20.0))
            .changed();

        cols[1].label("Phase:");
        modified |= cols[1]
            .add(egui::Slider::new(&mut input.phase, 0.0..=1.0))
            .changed();

        cols[1].label("Rotation:");
        modified |= cols[1]
            .add(egui::Slider::new(&mut input.rotation, 0.0..=1.0))
            .changed();
    });

    modified
}

fn source_combo(ui: &mut Ui, id: &str, source: &mut InputSource) -> bool {
    let mut changed = false;
    egui::ComboBox::from_id_salt(id)
        .selected_text(format!("{:?}", source))
        .show_ui(ui, |ui| {
            let sources = [
                InputSource::RampH,
                InputSource::RampV,
                InputSource::OscH,
                InputSource::OscV,
                InputSource::NoiseWhite,
                InputSource::NoisePink,
                InputSource::NoiseBrown,
                InputSource::ShapeCircle,
                InputSource::ShapeRect,
                InputSource::ShapeDiamond,
                InputSource::Checkerboard,
            ];
            for s in sources {
                if ui.selectable_label(*source == s, format!("{:?}", s)).clicked() {
                    *source = s;
                    changed = true;
                }
            }
        });
    changed
}

fn draw_geometry_stage(ui: &mut Ui, geo: &mut GeometryStage) -> bool {
    let mut modified = false;

    ui.columns(2, |cols| {
        cols[0].label("Wobble H:");
        modified |= cols[0]
            .add(egui::Slider::new(&mut geo.wobbulate_h, 0.0..=1.0))
            .changed();

        cols[0].label("Wobble V:");
        modified |= cols[0]
            .add(egui::Slider::new(&mut geo.wobbulate_v, 0.0..=1.0))
            .changed();

        cols[0].label("Wobble Freq:");
        modified |= cols[0]
            .add(egui::Slider::new(&mut geo.wobble_freq, 1.0..=20.0))
            .changed();

        cols[0].label("Z Displacement:");
        modified |= cols[0]
            .add(egui::Slider::new(&mut geo.z_displacement, 0.0..=0.5))
            .changed();

        cols[1].label("Lissajous X:");
        modified |= cols[1]
            .add(egui::Slider::new(&mut geo.lissajous_x, 0.0..=1.0))
            .changed();

        cols[1].label("Lissajous Y:");
        modified |= cols[1]
            .add(egui::Slider::new(&mut geo.lissajous_y, 0.0..=1.0))
            .changed();

        cols[1].label("Rotation:");
        modified |= cols[1]
            .add(egui::Slider::new(&mut geo.rotation, 0.0..=1.0))
            .changed();

        cols[1].label("Scale:");
        modified |= cols[1]
            .add(egui::Slider::new(&mut geo.scale, 0.5..=2.0))
            .changed();
    });

    modified
}

fn draw_amplitude_stage(ui: &mut Ui, amp: &mut AmplitudeStage) -> bool {
    let mut modified = false;

    ui.columns(2, |cols| {
        cols[0].label("Fold Gain:");
        modified |= cols[0]
            .add(egui::Slider::new(&mut amp.fold_gain, 1.0..=8.0))
            .changed();

        cols[0].label("Fold Mix:");
        modified |= cols[0]
            .add(egui::Slider::new(&mut amp.fold_mix, 0.0..=1.0))
            .changed();

        cols[0].label("Quantize Levels:");
        modified |= cols[0]
            .add(egui::Slider::new(&mut amp.quantize_levels, 2.0..=32.0))
            .changed();

        cols[0].label("Quantize Mix:");
        modified |= cols[0]
            .add(egui::Slider::new(&mut amp.quantize_mix, 0.0..=1.0))
            .changed();

        cols[1].label("Soft Clip:");
        modified |= cols[1]
            .add(egui::Slider::new(&mut amp.soft_clip, 0.0..=1.0))
            .changed();

        cols[1].label("Solarize:");
        modified |= cols[1]
            .add(egui::Slider::new(&mut amp.solarize, 0.0..=1.0))
            .changed();

        cols[1].label("Gate:");
        modified |= cols[1]
            .add(egui::Slider::new(&mut amp.gate_threshold, 0.0..=1.0))
            .changed();

        let mut invert_bool = amp.invert > 0.5;
        if cols[1].checkbox(&mut invert_bool, "Invert").changed() {
            amp.invert = if invert_bool { 1.0 } else { 0.0 };
            modified = true;
        }
    });

    modified
}

fn draw_colorize_stage(ui: &mut Ui, color: &mut ColorizeStage) -> bool {
    let mut modified = false;

    ui.columns(2, |cols| {
        cols[0].label("Mode:");
        egui::ComboBox::from_id_salt("color_mode")
            .selected_text(format!("{:?}", color.mode))
            .show_ui(&mut cols[0], |ui| {
                for mode in [
                    ColorMode::Spectrum,
                    ColorMode::Threshold,
                    ColorMode::Gradient,
                    ColorMode::Monochrome,
                ] {
                    if ui
                        .selectable_label(color.mode == mode, format!("{:?}", mode))
                        .clicked()
                    {
                        color.mode = mode;
                        modified = true;
                    }
                }
            });

        cols[0].label("Hue Offset:");
        modified |= cols[0]
            .add(egui::Slider::new(&mut color.hue_offset, 0.0..=1.0))
            .changed();

        cols[0].label("Saturation:");
        modified |= cols[0]
            .add(egui::Slider::new(&mut color.saturation, 0.0..=2.0))
            .changed();

        cols[1].label("Levels (threshold):");
        modified |= cols[1]
            .add(egui::Slider::new(&mut color.levels, 2.0..=32.0))
            .changed();

        // Gradient colors (only show when gradient mode)
        if color.mode == ColorMode::Gradient {
            cols[1].label("Gradient Start:");
            let mut start_color = Color32::from_rgb(
                (color.gradient_start[0] * 255.0) as u8,
                (color.gradient_start[1] * 255.0) as u8,
                (color.gradient_start[2] * 255.0) as u8,
            );
            if cols[1].color_edit_button_srgba(&mut start_color).changed() {
                color.gradient_start = [
                    start_color.r() as f32 / 255.0,
                    start_color.g() as f32 / 255.0,
                    start_color.b() as f32 / 255.0,
                ];
                modified = true;
            }

            cols[1].label("Gradient End:");
            let mut end_color = Color32::from_rgb(
                (color.gradient_end[0] * 255.0) as u8,
                (color.gradient_end[1] * 255.0) as u8,
                (color.gradient_end[2] * 255.0) as u8,
            );
            if cols[1].color_edit_button_srgba(&mut end_color).changed() {
                color.gradient_end = [
                    end_color.r() as f32 / 255.0,
                    end_color.g() as f32 / 255.0,
                    end_color.b() as f32 / 255.0,
                ];
                modified = true;
            }
        }
    });

    modified
}

fn draw_mixer_stage(ui: &mut Ui, mixer: &mut MixerStage) -> bool {
    let mut modified = false;

    ui.columns(2, |cols| {
        cols[0].label("Feedback Mix:");
        modified |= cols[0]
            .add(egui::Slider::new(&mut mixer.feedback_mix, 0.0..=1.0))
            .changed();

        cols[0].label("Blend Mode:");
        egui::ComboBox::from_id_salt("blend_mode")
            .selected_text(format!("{:?}", mixer.blend_mode))
            .show_ui(&mut cols[0], |ui| {
                for mode in [
                    BlendMode::Mix,
                    BlendMode::Add,
                    BlendMode::Multiply,
                    BlendMode::Screen,
                    BlendMode::Overlay,
                    BlendMode::Difference,
                    BlendMode::LumaKeyA,
                    BlendMode::LumaKeyB,
                ] {
                    if ui
                        .selectable_label(mixer.blend_mode == mode, format!("{:?}", mode))
                        .clicked()
                    {
                        mixer.blend_mode = mode;
                        modified = true;
                    }
                }
            });

        cols[0].label("Layer Opacity:");
        modified |= cols[0]
            .add(egui::Slider::new(&mut mixer.layer_opacity, 0.0..=1.0))
            .changed();

        cols[1].label("Key Threshold:");
        modified |= cols[1]
            .add(egui::Slider::new(&mut mixer.key_threshold, 0.0..=1.0))
            .changed();

        cols[1].label("Key Softness:");
        modified |= cols[1]
            .add(egui::Slider::new(&mut mixer.key_softness, 0.0..=0.5))
            .changed();

        modified |= cols[1].checkbox(&mut mixer.key_invert, "Invert Key").changed();
    });

    modified
}

fn draw_feedback_stage(ui: &mut Ui, fb: &mut FeedbackStage) -> bool {
    let mut modified = false;

    modified |= ui.checkbox(&mut fb.enabled, "Enable Feedback").changed();

    if fb.enabled {
        ui.columns(2, |cols| {
            cols[0].label("Zoom:");
            modified |= cols[0]
                .add(egui::Slider::new(&mut fb.zoom, 0.9..=1.1))
                .changed();

            cols[0].label("Rotation:");
            modified |= cols[0]
                .add(egui::Slider::new(&mut fb.rotation, -0.1..=0.1))
                .changed();

            cols[0].label("Hue Shift:");
            modified |= cols[0]
                .add(egui::Slider::new(&mut fb.hue_shift, 0.0..=0.1))
                .changed();

            cols[0].label("Decay:");
            modified |= cols[0]
                .add(egui::Slider::new(&mut fb.decay, 0.8..=1.0))
                .changed();

            cols[1].label("Offset X:");
            modified |= cols[1]
                .add(egui::Slider::new(&mut fb.offset_x, -0.1..=0.1))
                .changed();

            cols[1].label("Offset Y:");
            modified |= cols[1]
                .add(egui::Slider::new(&mut fb.offset_y, -0.1..=0.1))
                .changed();

            cols[1].label("Saturation:");
            modified |= cols[1]
                .add(egui::Slider::new(&mut fb.saturation, 0.0..=2.0))
                .changed();
        });
    }

    modified
}

fn draw_output_stage(ui: &mut Ui, out: &mut OutputStage) -> bool {
    let mut modified = false;

    ui.columns(2, |cols| {
        cols[0].label("Mode:");
        egui::ComboBox::from_id_salt("output_mode")
            .selected_text(format!("{:?}", out.mode))
            .show_ui(&mut cols[0], |ui| {
                for mode in [
                    OutputMode::Clean,
                    OutputMode::CRT,
                    OutputMode::VHS,
                    OutputMode::Cable,
                ] {
                    if ui
                        .selectable_label(out.mode == mode, format!("{:?}", mode))
                        .clicked()
                    {
                        out.mode = mode;
                        modified = true;
                    }
                }
            });

        cols[0].label("Scanlines:");
        modified |= cols[0]
            .add(egui::Slider::new(&mut out.scanlines, 0.0..=0.5))
            .changed();

        cols[0].label("Curvature:");
        modified |= cols[0]
            .add(egui::Slider::new(&mut out.curvature, 0.0..=0.5))
            .changed();

        cols[0].label("Bloom:");
        modified |= cols[0]
            .add(egui::Slider::new(&mut out.bloom, 0.0..=1.0))
            .changed();

        cols[1].label("Vignette:");
        modified |= cols[1]
            .add(egui::Slider::new(&mut out.vignette, 0.0..=1.0))
            .changed();

        cols[1].label("Noise:");
        modified |= cols[1]
            .add(egui::Slider::new(&mut out.noise, 0.0..=0.5))
            .changed();

        // VHS-specific controls
        if out.mode == OutputMode::VHS {
            cols[1].label("Tracking:");
            modified |= cols[1]
                .add(egui::Slider::new(&mut out.tracking, 0.0..=1.0))
                .changed();

            cols[0].label("Chroma Shift:");
            modified |= cols[0]
                .add(egui::Slider::new(&mut out.chroma_shift, 0.0..=0.02))
                .changed();

            cols[0].label("Tape Wobble:");
            modified |= cols[0]
                .add(egui::Slider::new(&mut out.tape_wobble, 0.0..=1.0))
                .changed();
        }

        // Cable-specific controls
        if out.mode == OutputMode::Cable {
            cols[1].label("Bandwidth:");
            modified |= cols[1]
                .add(egui::Slider::new(&mut out.bandwidth, 0.5..=1.0))
                .changed();

            cols[1].label("Ghosting:");
            modified |= cols[1]
                .add(egui::Slider::new(&mut out.ghosting, 0.0..=0.3))
                .changed();
        }
    });

    modified
}
