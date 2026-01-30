//! Preset system for saving/loading synthesizer state

use crate::synth::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub description: String,
    pub state: SynthState,
}

impl Preset {
    pub fn new(name: &str, description: &str, state: SynthState) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            state,
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, json)
    }

    pub fn load(path: &Path) -> Result<Self, std::io::Error> {
        let json = std::fs::read_to_string(path)?;
        serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

/// Built-in presets
pub fn builtin_presets() -> Vec<Preset> {
    vec![
        // Classic feedback spiral
        Preset::new(
            "Feedback Spiral",
            "Classic video feedback with zoom and rotation",
            SynthState {
                input: InputStage {
                    source_a: InputSource::RampH,
                    source_b: InputSource::RampV,
                    mix: 0.5,
                    frequency: 1.0,
                    ..Default::default()
                },
                geometry: GeometryStage::default(),
                amplitude: AmplitudeStage::default(),
                colorize: ColorizeStage {
                    mode: ColorMode::Spectrum,
                    saturation: 1.2,
                    ..Default::default()
                },
                mixer: MixerStage {
                    feedback_mix: 0.8,
                    blend_mode: BlendMode::Screen,
                    ..Default::default()
                },
                feedback: FeedbackStage {
                    enabled: true,
                    zoom: 1.03,
                    rotation: 0.02,
                    hue_shift: 0.01,
                    decay: 0.96,
                    ..Default::default()
                },
                output: OutputStage {
                    mode: OutputMode::CRT,
                    scanlines: 0.1,
                    bloom: 0.3,
                    ..Default::default()
                },
            },
        ),

        // Wobbulator pattern
        Preset::new(
            "Wobbulator",
            "Paik/Abe style wobbulator distortion",
            SynthState {
                input: InputStage {
                    source_a: InputSource::OscH,
                    source_b: InputSource::OscV,
                    mix: 0.0,
                    frequency: 8.0,
                    ..Default::default()
                },
                geometry: GeometryStage {
                    wobbulate_h: 0.3,
                    wobbulate_v: 0.2,
                    wobble_freq: 6.0,
                    ..Default::default()
                },
                amplitude: AmplitudeStage {
                    fold_gain: 3.0,
                    fold_mix: 0.4,
                    ..Default::default()
                },
                colorize: ColorizeStage {
                    mode: ColorMode::Spectrum,
                    hue_offset: 0.3,
                    ..Default::default()
                },
                mixer: MixerStage {
                    feedback_mix: 0.2,
                    ..Default::default()
                },
                feedback: FeedbackStage {
                    enabled: true,
                    zoom: 1.01,
                    decay: 0.9,
                    ..Default::default()
                },
                output: OutputStage {
                    mode: OutputMode::CRT,
                    ..Default::default()
                },
            },
        ),

        // Rutt/Etra style displacement
        Preset::new(
            "Rutt/Etra Terrain",
            "Z-axis displacement creating 3D terrain",
            SynthState {
                input: InputStage {
                    source_a: InputSource::NoisePink,
                    source_b: InputSource::RampH,
                    mix: 0.3,
                    frequency: 2.0,
                    ..Default::default()
                },
                geometry: GeometryStage {
                    z_displacement: 0.25,
                    ..Default::default()
                },
                amplitude: AmplitudeStage {
                    quantize_levels: 16.0,
                    quantize_mix: 0.3,
                    ..Default::default()
                },
                colorize: ColorizeStage {
                    mode: ColorMode::Threshold,
                    levels: 12.0,
                    ..Default::default()
                },
                mixer: MixerStage {
                    feedback_mix: 0.1,
                    ..Default::default()
                },
                feedback: FeedbackStage {
                    enabled: true,
                    zoom: 1.0,
                    decay: 0.85,
                    ..Default::default()
                },
                output: OutputStage {
                    mode: OutputMode::CRT,
                    scanlines: 0.2,
                    ..Default::default()
                },
            },
        ),

        // VHS degradation look
        Preset::new(
            "VHS Memory",
            "Degraded VHS tape aesthetic",
            SynthState {
                input: InputStage {
                    source_a: InputSource::RampV,
                    source_b: InputSource::NoiseWhite,
                    mix: 0.1,
                    ..Default::default()
                },
                geometry: GeometryStage {
                    wobbulate_h: 0.02,
                    wobble_freq: 0.5,
                    ..Default::default()
                },
                amplitude: AmplitudeStage {
                    soft_clip: 0.3,
                    ..Default::default()
                },
                colorize: ColorizeStage {
                    mode: ColorMode::Spectrum,
                    saturation: 0.7,
                    ..Default::default()
                },
                mixer: MixerStage {
                    feedback_mix: 0.6,
                    blend_mode: BlendMode::Mix,
                    ..Default::default()
                },
                feedback: FeedbackStage {
                    enabled: true,
                    zoom: 1.005,
                    hue_shift: 0.002,
                    decay: 0.92,
                    saturation: 0.95,
                    ..Default::default()
                },
                output: OutputStage {
                    mode: OutputMode::VHS,
                    scanlines: 0.05,
                    noise: 0.08,
                    tracking: 0.3,
                    chroma_shift: 0.008,
                    tape_wobble: 0.4,
                    ..Default::default()
                },
            },
        ),

        // Colorizer bands
        Preset::new(
            "Colorizer Bands",
            "Jones-style banded colorization",
            SynthState {
                input: InputStage {
                    source_a: InputSource::RampH,
                    source_b: InputSource::ShapeCircle,
                    mix: 0.4,
                    ..Default::default()
                },
                geometry: GeometryStage::default(),
                amplitude: AmplitudeStage {
                    quantize_levels: 6.0,
                    quantize_mix: 1.0,
                    ..Default::default()
                },
                colorize: ColorizeStage {
                    mode: ColorMode::Threshold,
                    levels: 6.0,
                    saturation: 1.5,
                    ..Default::default()
                },
                mixer: MixerStage {
                    feedback_mix: 0.0,
                    ..Default::default()
                },
                feedback: FeedbackStage {
                    enabled: false,
                    ..Default::default()
                },
                output: OutputStage {
                    mode: OutputMode::CRT,
                    scanlines: 0.15,
                    bloom: 0.4,
                    ..Default::default()
                },
            },
        ),

        // Noise meditation
        Preset::new(
            "Noise Meditation",
            "Slow-moving noise patterns",
            SynthState {
                input: InputStage {
                    source_a: InputSource::NoiseBrown,
                    source_b: InputSource::NoisePink,
                    mix: 0.5,
                    frequency: 1.0,
                    ..Default::default()
                },
                geometry: GeometryStage {
                    scale: 1.5,
                    ..Default::default()
                },
                amplitude: AmplitudeStage {
                    soft_clip: 0.5,
                    ..Default::default()
                },
                colorize: ColorizeStage {
                    mode: ColorMode::Gradient,
                    gradient_start: [0.1, 0.0, 0.2],
                    gradient_end: [0.8, 0.4, 0.1],
                    ..Default::default()
                },
                mixer: MixerStage {
                    feedback_mix: 0.7,
                    blend_mode: BlendMode::Screen,
                    ..Default::default()
                },
                feedback: FeedbackStage {
                    enabled: true,
                    zoom: 1.01,
                    rotation: 0.005,
                    hue_shift: 0.005,
                    decay: 0.98,
                    ..Default::default()
                },
                output: OutputStage {
                    mode: OutputMode::CRT,
                    scanlines: 0.08,
                    bloom: 0.5,
                    vignette: 0.5,
                    ..Default::default()
                },
            },
        ),
    ]
}
