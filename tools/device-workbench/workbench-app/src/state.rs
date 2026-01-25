//! Shared application state

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Parameter definition with value and range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDef {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub default: f32,
}

impl ParamDef {
    pub fn new(default: f32, min: f32, max: f32) -> Self {
        Self {
            value: default,
            min,
            max,
            default,
        }
    }
}

/// Main application state shared between render loop and API
#[derive(Debug)]
pub struct AppState {
    // Shader
    pub shader_name: String,
    pub shader_code: String,
    pub shader_error: Option<String>,
    pub shader_dirty: bool, // Flag to trigger recompile

    // Parameters
    pub params: HashMap<String, ParamDef>,

    // Images
    pub input_image: Option<Vec<u8>>,      // PNG bytes
    pub reference_image: Option<Vec<u8>>,  // PNG bytes
    pub output_image: Option<Vec<u8>>,     // Last rendered frame

    // Metrics
    pub fps: f32,
    pub frame_time_ms: f32,
    pub gpu_time_ms: f32,
    pub shader_compile_ms: f32,

    // Status
    pub status: String,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            shader_name: "none".to_string(),
            shader_code: DEFAULT_SHADER.to_string(),
            shader_error: None,
            shader_dirty: true,

            params: Self::default_params(),

            input_image: None,
            reference_image: None,
            output_image: None,

            fps: 0.0,
            frame_time_ms: 0.0,
            gpu_time_ms: 0.0,
            shader_compile_ms: 0.0,

            status: "ready".to_string(),
        }
    }

    fn default_params() -> HashMap<String, ParamDef> {
        let mut params = HashMap::new();
        params.insert("gain".to_string(), ParamDef::new(1.0, 0.0, 2.0));
        params.insert("offset".to_string(), ParamDef::new(0.0, -1.0, 1.0));
        params.insert("mix".to_string(), ParamDef::new(1.0, 0.0, 1.0));
        params
    }

    pub fn set_shader(&mut self, name: String, code: String) {
        self.shader_name = name;
        self.shader_code = code;
        self.shader_error = None;
        self.shader_dirty = true;
    }

    pub fn set_param(&mut self, name: &str, value: f32) -> bool {
        if let Some(param) = self.params.get_mut(name) {
            param.value = value.clamp(param.min, param.max);
            true
        } else {
            false
        }
    }

    pub fn get_params_for_gpu(&self) -> [f32; 16] {
        // Pack parameters into a fixed-size array for GPU uniform
        let mut data = [0.0f32; 16];

        // Standard params
        data[0] = self.params.get("gain").map(|p| p.value).unwrap_or(1.0);
        data[1] = self.params.get("offset").map(|p| p.value).unwrap_or(0.0);
        data[2] = self.params.get("mix").map(|p| p.value).unwrap_or(1.0);

        // Additional params as needed
        data[3] = self.params.get("fold").map(|p| p.value).unwrap_or(0.0);
        data[4] = self.params.get("bias").map(|p| p.value).unwrap_or(0.0);
        data[5] = self.params.get("frequency").map(|p| p.value).unwrap_or(1.0);
        data[6] = self.params.get("phase").map(|p| p.value).unwrap_or(0.0);
        data[7] = self.params.get("threshold").map(|p| p.value).unwrap_or(0.5);

        data
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Default passthrough shader
pub const DEFAULT_SHADER: &str = r#"
// Default passthrough shader
// Replace this with your device emulation shader

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Params {
    gain: f32,
    offset: f32,
    mix: f32,
    fold: f32,
    bias: f32,
    frequency: f32,
    phase: f32,
    threshold: f32,
    // Padding to reach 64 bytes (16-byte aligned)
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
    _pad3: f32,
    _pad4: f32,
    _pad5: f32,
    _pad6: f32,
    _pad7: f32,
};

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;
@group(0) @binding(2) var<uniform> params: Params;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Full-screen triangle
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0)
    );

    var uvs = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(2.0, 1.0),
        vec2<f32>(0.0, -1.0)
    );

    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    output.uv = uvs[vertex_index];
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(input_texture, input_sampler, input.uv);

    // Simple gain/offset processing
    var processed = color.rgb * params.gain + params.offset;

    // Mix with original
    let output = mix(color.rgb, processed, params.mix);

    return vec4<f32>(output, 1.0);
}
"#;
