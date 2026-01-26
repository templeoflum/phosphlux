//! Test Signal Generators
//!
//! Source devices that generate test patterns from UV coordinates.
//! No video input required — these are the starting points for any signal chain.

use crate::gpu::{DeviceParameter, Port, ShaderDevice, SignalType};

/// Test Signal Generator
///
/// Generates standard test patterns: H ramp, V ramp, colorbars,
/// checkerboard, radial, and oscillator patterns.
pub struct TestSignalGenerator {
    name: String,
}

impl TestSignalGenerator {
    pub fn new() -> Self {
        Self {
            name: "Test Signal".to_string(),
        }
    }
}

impl Default for TestSignalGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ShaderDevice for TestSignalGenerator {
    fn device_id(&self) -> &'static str {
        "test_signal"
    }

    fn device_name(&self) -> &str {
        &self.name
    }

    fn shader_code(&self) -> &str {
        r#"
// Test Signal Generator
// Generates standard test patterns from UV coordinates

fn device_test_signal_process(
    uv: vec2<f32>,
    time: f32,
    mode: f32,
    frequency: f32
) -> vec4<f32> {
    let m = u32(mode);

    // Mode 0: Horizontal ramp (grayscale left-to-right)
    if m == 0u {
        return vec4<f32>(vec3<f32>(uv.x), 1.0);
    }

    // Mode 1: Vertical ramp (grayscale top-to-bottom)
    if m == 1u {
        return vec4<f32>(vec3<f32>(uv.y), 1.0);
    }

    // Mode 2: SMPTE-style color bars
    if m == 2u {
        let bar = u32(uv.x * 8.0);
        var color = vec3<f32>(0.0);
        if bar == 0u { color = vec3<f32>(1.0, 1.0, 1.0); }      // White
        else if bar == 1u { color = vec3<f32>(1.0, 1.0, 0.0); }  // Yellow
        else if bar == 2u { color = vec3<f32>(0.0, 1.0, 1.0); }  // Cyan
        else if bar == 3u { color = vec3<f32>(0.0, 1.0, 0.0); }  // Green
        else if bar == 4u { color = vec3<f32>(1.0, 0.0, 1.0); }  // Magenta
        else if bar == 5u { color = vec3<f32>(1.0, 0.0, 0.0); }  // Red
        else if bar == 6u { color = vec3<f32>(0.0, 0.0, 1.0); }  // Blue
        else { color = vec3<f32>(0.0); }                          // Black
        return vec4<f32>(color, 1.0);
    }

    // Mode 3: Checkerboard
    if m == 3u {
        let cx = u32(uv.x * frequency * 2.0);
        let cy = u32(uv.y * frequency * 2.0);
        let check = f32((cx + cy) % 2u);
        return vec4<f32>(vec3<f32>(check), 1.0);
    }

    // Mode 4: Radial gradient
    if m == 4u {
        let centered = uv - 0.5;
        let dist = length(centered) * 2.0;
        return vec4<f32>(vec3<f32>(clamp(1.0 - dist, 0.0, 1.0)), 1.0);
    }

    // Mode 5: Horizontal oscillator
    if m == 5u {
        let v = sin(uv.x * frequency * 6.283 + time * 2.0) * 0.5 + 0.5;
        return vec4<f32>(vec3<f32>(v), 1.0);
    }

    // Mode 6: Vertical oscillator
    if m == 6u {
        let v = sin(uv.y * frequency * 6.283 + time * 2.0) * 0.5 + 0.5;
        return vec4<f32>(vec3<f32>(v), 1.0);
    }

    // Default: 50% gray
    return vec4<f32>(vec3<f32>(0.5), 1.0);
}
"#
    }

    fn parameters(&self) -> Vec<DeviceParameter> {
        vec![
            DeviceParameter::new("mode", "Pattern", 0.0, 6.0, 0.0),
            DeviceParameter::new("frequency", "Frequency", 1.0, 20.0, 4.0)
                .with_unit("Hz"),
        ]
    }

    fn inputs(&self) -> Vec<Port> {
        // Source device — no inputs
        vec![]
    }

    fn outputs(&self) -> Vec<Port> {
        vec![Port {
            name: "video_out".to_string(),
            signal_type: SignalType::Rgb,
        }]
    }
}
