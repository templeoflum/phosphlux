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

use crate::gpu::{DeviceParameter, Port, ShaderDevice, SignalType};

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
// Paik/Abe Wobulator
// Horizontal displacement based on luminance

fn device_paik_abe_wobulator_process(
    uv: vec2<f32>,
    time: f32,
    video_in: vec4<f32>,
    frequency: f32,
    amplitude: f32,
    phase: f32
) -> vec4<f32> {
    // Get displacement amount from input luminance
    let luma = to_luma(video_in.rgb);

    // Calculate horizontal displacement
    let displacement = sin(uv.y * frequency * 6.283 + time * 2.0 + phase * 6.283) * amplitude * luma;

    // Apply displacement (modulate color based on displacement)
    let wobbled = video_in.rgb * (1.0 + displacement * 2.0);

    return vec4<f32>(video_clamp3(wobbled), 1.0);
}
"#
    }

    fn parameters(&self) -> Vec<DeviceParameter> {
        vec![
            DeviceParameter::new("frequency", "Wobble Freq", 0.5, 20.0, 4.0),
            DeviceParameter::new("amplitude", "Amplitude", 0.0, 0.1, 0.02),
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
}
