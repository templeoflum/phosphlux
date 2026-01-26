//! Paik/Abe Video Synthesizer Modules
//!
//! The Paik/Abe Video Synthesizer (1969-1970) was created by Nam June Paik
//! and Shuya Abe. It was one of the first dedicated video synthesis instruments.
//!
//! Key characteristics:
//! - Built around LM741 op-amps (slow slew rate = character)
//! - Triangle wave folding for colorization
//! - Multiple stages of distortion/folding
//! - Warm, organic color shifts from component tolerances
//!
//! Reference: "Paik/Abe Video Synthesizer" documentation, EAI archives

use crate::gpu::{DeviceParameter, Port, ProcessingMode, ShaderDevice, SignalType};

/// The Paik/Abe Colorizer
///
/// This is the heart of the Paik/Abe synthesizer - converts luminance
/// to color through waveshaping. Uses op-amp saturation and triangle
/// folding to create the characteristic psychedelic color shifts.
pub struct PaikAbeColorizer {
    name: String,
}

impl PaikAbeColorizer {
    pub fn new() -> Self {
        Self {
            name: "Paik/Abe Colorizer".to_string(),
        }
    }
}

impl Default for PaikAbeColorizer {
    fn default() -> Self {
        Self::new()
    }
}

impl ShaderDevice for PaikAbeColorizer {
    fn device_id(&self) -> &'static str {
        "paik_abe_colorizer"
    }

    fn device_name(&self) -> &str {
        &self.name
    }

    fn shader_code(&self) -> &str {
        include_str!("colorizer.wgsl")
    }

    fn parameters(&self) -> Vec<DeviceParameter> {
        // These correspond to the physical pots on the original hardware
        vec![
            DeviceParameter::new("gain_r", "Red Gain", 0.5, 8.0, 2.0),
            DeviceParameter::new("gain_g", "Green Gain", 0.5, 8.0, 2.5),
            DeviceParameter::new("gain_b", "Blue Gain", 0.5, 8.0, 3.0),
            DeviceParameter::new("fold_depth", "Fold Depth", 1.0, 5.0, 2.0),
            DeviceParameter::new("saturation", "Saturation", 0.0, 2.0, 1.0),
            DeviceParameter::new("slew", "Slew (741 Character)", 0.0, 1.0, 0.3),
        ]
    }

    fn inputs(&self) -> Vec<Port> {
        vec![Port {
            name: "video_in".to_string(),
            signal_type: SignalType::Rgb,
        }]
    }

    fn outputs(&self) -> Vec<Port> {
        vec![Port {
            name: "video_out".to_string(),
            signal_type: SignalType::Rgb,
        }]
    }
}

/// The Paik/Abe Wobulator
///
/// Creates oscillating distortions in the horizontal domain.
/// Named after the "wobbly" effect it creates.
pub struct PaikAbeWobulator {
    name: String,
}

impl PaikAbeWobulator {
    pub fn new() -> Self {
        Self {
            name: "Paik/Abe Wobulator".to_string(),
        }
    }
}

impl Default for PaikAbeWobulator {
    fn default() -> Self {
        Self::new()
    }
}

impl ShaderDevice for PaikAbeWobulator {
    fn device_id(&self) -> &'static str {
        "paik_abe_wobulator"
    }

    fn device_name(&self) -> &str {
        &self.name
    }

    fn shader_code(&self) -> &str {
        r#"
// Paik/Abe Wobulator — Scan Modulator
// CRT raster displacement via oscillators + video-driven modulation.
// Reference: research/components/functional/wobbulator/v1/

fn device_paik_abe_wobulator_process(
    uv: vec2<f32>,
    time: f32,
    h_freq: f32,
    h_amp: f32,
    v_freq: f32,
    v_amp: f32,
    video_mod: f32,
    speed: f32,
    phase: f32
) -> vec4<f32> {
    // Sample input at current position for video-driven displacement
    let current = textureSample(seg_input_tex, seg_input_sampler, uv);
    let luma = to_luma(current.rgb);

    // Center luma around 0.5: mid-gray = no displacement,
    // bright = positive, dark = negative (authentic Paik/Abe behavior)
    let luma_centered = luma - 0.5;

    // Horizontal oscillator: displaces X based on Y position
    let h_phase = uv.y * h_freq + time * speed + phase;
    let h_disp = sin(h_phase * 6.283) * h_amp;

    // Vertical oscillator: displaces Y based on X position
    let v_phase = uv.x * v_freq + time * speed * 0.7 + phase * 1.3;
    let v_disp = sin(v_phase * 6.283) * v_amp;

    // Video-driven radial displacement
    let video_disp = luma_centered * video_mod;

    // Combine oscillator + video displacement
    let displaced_uv = clamp(
        uv + vec2<f32>(h_disp + video_disp, v_disp + video_disp * 0.5),
        vec2<f32>(0.0),
        vec2<f32>(1.0)
    );

    // Sample upstream signal at displaced position
    return textureSample(seg_input_tex, seg_input_sampler, displaced_uv);
}
"#
    }

    fn parameters(&self) -> Vec<DeviceParameter> {
        vec![
            DeviceParameter::new("h_freq", "H Frequency", 0.5, 20.0, 3.0),
            DeviceParameter::new("h_amp", "H Amplitude", 0.0, 0.2, 0.05),
            DeviceParameter::new("v_freq", "V Frequency", 0.5, 20.0, 2.0),
            DeviceParameter::new("v_amp", "V Amplitude", 0.0, 0.2, 0.04),
            DeviceParameter::new("video_mod", "Video Mod", 0.0, 0.5, 0.0),
            DeviceParameter::new("speed", "Speed", 0.0, 5.0, 0.5),
            DeviceParameter::new("phase", "Phase", 0.0, 1.0, 0.0),
        ]
    }

    fn inputs(&self) -> Vec<Port> {
        vec![Port {
            name: "video_in".to_string(),
            signal_type: SignalType::Rgb,
        }]
    }

    fn outputs(&self) -> Vec<Port> {
        vec![Port {
            name: "video_out".to_string(),
            signal_type: SignalType::Rgb,
        }]
    }

    fn processing_mode(&self) -> ProcessingMode {
        ProcessingMode::ScanModulator
    }
}
