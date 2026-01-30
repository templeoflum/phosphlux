// ============================================
// PAIK/ABE INPUT MATRIX - Standalone Workbench Version
// ============================================
//
// Adapted for device workbench testing with single-input texture.
// Derives 7 "virtual channels" from input 0 to test matrix mixing.
//
// Uses the 7-input binding layout but only samples input0:
//   @binding(0-6): Input textures (only 0 used)
//   @binding(7): Shared sampler
//   @binding(8): Params uniform
//
// Virtual channel derivation:
//   Ch1: Luminance (Y)
//   Ch2: Red component
//   Ch3: Green component
//   Ch4: Blue component
//   Ch5: Inverted luminance (1-Y)
//   Ch6: Luminance squared (contrast enhancement)
//   Ch7: Mid-gray offset (0.5)
//
// Parameters:
// - gain: Red channel output gain (r_mix, 0.0-2.0)
// - offset: Green channel output gain (g_mix, 0.0-2.0, shifted +1)
// - mix: Blue channel output gain (b_mix, 0.0-2.0)
// - fold: Rotation angle (0.0-1.0, maps to 0-2π)

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Params {
    gain: f32,      // r_mix
    offset: f32,    // g_mix (shifted)
    mix: f32,       // b_mix
    fold: f32,      // rotation
    bias: f32,
    frequency: f32,
    phase: f32,
    threshold: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
    _pad3: f32,
    _pad4: f32,
    _pad5: f32,
    _pad6: f32,
    _pad7: f32,
};

// 7 input textures (only input0 is used in standalone mode)
@group(0) @binding(0) var input0: texture_2d<f32>;
@group(0) @binding(1) var input1: texture_2d<f32>;
@group(0) @binding(2) var input2: texture_2d<f32>;
@group(0) @binding(3) var input3: texture_2d<f32>;
@group(0) @binding(4) var input4: texture_2d<f32>;
@group(0) @binding(5) var input5: texture_2d<f32>;
@group(0) @binding(6) var input6: texture_2d<f32>;

// Shared sampler
@group(0) @binding(7) var input_sampler: sampler;

// Parameters
@group(0) @binding(8) var<uniform> params: Params;

// Matrix row: sum weighted inputs with CA3054-style soft limiting
fn input_matrix_row(ch: array<f32, 7>, w: array<f32, 7>) -> f32 {
    var sum: f32 = 0.0;
    for (var i: u32 = 0u; i < 7u; i = i + 1u) {
        sum += (ch[i] - 0.5) * w[i];
    }
    // Soft limiting via tanh (models CA3054 differential amp characteristic)
    let limited = tanh(sum * 2.0) * 0.5;
    return clamp(limited + 0.5, 0.0, 1.0);
}

// Extract luminance from RGB (Rec. 601)
fn input_matrix_luma(rgb: vec3<f32>) -> f32 {
    return dot(rgb, vec3<f32>(0.299, 0.587, 0.114));
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
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
    // Sample only input0 for standalone mode
    let color = textureSample(input0, input_sampler, input.uv);
    let rgb = color.rgb;
    let luma = input_matrix_luma(rgb);

    // Map params to mixer controls
    let r_mix = params.gain;
    let g_mix = params.offset + 1.0;  // offset is -1 to 1, shift to 0-2 range
    let b_mix = params.mix;
    let rotation = params.fold;

    // Derive 7 virtual channels from single input
    var channels: array<f32, 7>;
    channels[0] = luma;                      // Ch1: Luminance
    channels[1] = rgb.r;                     // Ch2: Red
    channels[2] = rgb.g;                     // Ch3: Green
    channels[3] = rgb.b;                     // Ch4: Blue
    channels[4] = 1.0 - luma;               // Ch5: Inverted luma
    channels[5] = luma * luma;              // Ch6: Luma squared (contrast)
    channels[6] = 0.5;                       // Ch7: Mid-gray offset

    // Rotation angle (0-1 maps to 0-2π)
    let rot = rotation * 6.283185;
    let cos_r = cos(rot);
    let sin_r = sin(rot);

    // R channel weights
    var r_weights: array<f32, 7>;
    r_weights[0] = 0.4 * cos_r * r_mix;
    r_weights[1] = 0.6 * r_mix;
    r_weights[2] = -0.2 * sin_r * r_mix;
    r_weights[3] = 0.0;
    r_weights[4] = 0.1;
    r_weights[5] = 0.2;
    r_weights[6] = 0.0;

    // G channel weights
    var g_weights: array<f32, 7>;
    g_weights[0] = 0.4 * cos_r * g_mix;
    g_weights[1] = -0.2 * sin_r * g_mix;
    g_weights[2] = 0.6 * g_mix;
    g_weights[3] = 0.0;
    g_weights[4] = 0.1;
    g_weights[5] = 0.2;
    g_weights[6] = 0.0;

    // B channel weights
    var b_weights: array<f32, 7>;
    b_weights[0] = 0.4 * cos_r * b_mix;
    b_weights[1] = 0.0;
    b_weights[2] = -0.2 * sin_r * b_mix;
    b_weights[3] = 0.6 * b_mix;
    b_weights[4] = 0.1;
    b_weights[5] = 0.2;
    b_weights[6] = 0.0;

    // Apply matrix mixing
    let r_out = input_matrix_row(channels, r_weights);
    let g_out = input_matrix_row(channels, g_weights);
    let b_out = input_matrix_row(channels, b_weights);

    return vec4<f32>(r_out, g_out, b_out, 1.0);
}
