// ============================================
// PAIK/ABE INPUT MATRIX - Full 7-Input Version
// ============================================
//
// Authentic 7-input matrix mixer for the Device Workbench.
// Samples from 7 actual texture inputs, matching the original
// Paik/Abe Video Synthesizer architecture.
//
// Bindings (matching workbench 7-input layout):
//   @binding(0-6): Input textures 0-6
//   @binding(7): Shared sampler
//   @binding(8): Params uniform
//
// Parameters:
// - gain: Red channel output gain (r_mix, 0.0-2.0)
// - offset: Green channel output gain (g_mix, 0.0-2.0, shifted +1)
// - mix: Blue channel output gain (b_mix, 0.0-2.0)
// - fold: Rotation angle (0.0-1.0, maps to 0-2π)
// - bias: Master output bias (-0.5 to 0.5)
// - frequency: Soft-limit sharpness (0.5-4.0, default 2.0)

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Params {
    gain: f32,      // r_mix
    offset: f32,    // g_mix (shifted)
    mix: f32,       // b_mix
    fold: f32,      // rotation
    bias: f32,      // output bias
    frequency: f32, // soft-limit sharpness
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

// 7 input textures
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

// CA3054 differential amplifier soft limiting via tanh
// Models the thermal voltage characteristic of the transistor pair
fn ca3054_soft_limit(x: f32, sharpness: f32) -> f32 {
    return tanh(x * sharpness) * 0.5;
}

// Matrix row: sum weighted inputs with CA3054-style soft limiting
fn input_matrix_row(ch: array<f32, 7>, w: array<f32, 7>, sharpness: f32) -> f32 {
    var sum: f32 = 0.0;
    for (var i: u32 = 0u; i < 7u; i = i + 1u) {
        // Center around 0.5 (video black level)
        sum += (ch[i] - 0.5) * w[i];
    }
    // Soft limiting via tanh (models CA3054 differential amp characteristic)
    let limited = ca3054_soft_limit(sum, sharpness);
    return clamp(limited + 0.5, 0.0, 1.0);
}

// Extract luminance from RGB sample (for single-channel inputs)
fn sample_luma(tex: texture_2d<f32>, samp: sampler, uv: vec2<f32>) -> f32 {
    let rgb = textureSample(tex, samp, uv).rgb;
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
    let uv = input.uv;

    // Sample all 7 inputs (extract luminance for matrix processing)
    var channels: array<f32, 7>;
    channels[0] = sample_luma(input0, input_sampler, uv);
    channels[1] = sample_luma(input1, input_sampler, uv);
    channels[2] = sample_luma(input2, input_sampler, uv);
    channels[3] = sample_luma(input3, input_sampler, uv);
    channels[4] = sample_luma(input4, input_sampler, uv);
    channels[5] = sample_luma(input5, input_sampler, uv);
    channels[6] = sample_luma(input6, input_sampler, uv);

    // Map params to mixer controls
    let r_mix = params.gain;
    let g_mix = params.offset + 1.0;  // offset is -1 to 1, shift to 0-2 range
    let b_mix = params.mix;
    let rotation = params.fold;
    let output_bias = params.bias;
    let sharpness = max(params.frequency, 0.5) + 1.5;  // Map 0-1 to ~1.5-2.5

    // Rotation angle (0-1 maps to 0-2π)
    let rot = rotation * 6.283185;
    let cos_r = cos(rot);
    let sin_r = sin(rot);

    // Matrix weights for R output
    // Ch0-2: Primary color sources
    // Ch3-4: Secondary/keying sources
    // Ch5-6: Modulation/offset
    var r_weights: array<f32, 7>;
    r_weights[0] = 0.6 * r_mix;                    // Primary R input
    r_weights[1] = 0.2 * cos_r * r_mix;            // Cross-feed from ch1
    r_weights[2] = -0.2 * sin_r * r_mix;           // Cross-feed from ch2 (rotation)
    r_weights[3] = 0.1;                             // Key input
    r_weights[4] = 0.05;                            // Modulation
    r_weights[5] = 0.15;                            // Secondary modulation
    r_weights[6] = 0.0;                             // Offset channel

    // Matrix weights for G output
    var g_weights: array<f32, 7>;
    g_weights[0] = 0.2 * cos_r * g_mix;
    g_weights[1] = 0.6 * g_mix;                    // Primary G input
    g_weights[2] = 0.2 * sin_r * g_mix;
    g_weights[3] = 0.1;
    g_weights[4] = 0.15;
    g_weights[5] = 0.05;
    g_weights[6] = 0.0;

    // Matrix weights for B output
    var b_weights: array<f32, 7>;
    b_weights[0] = -0.2 * sin_r * b_mix;
    b_weights[1] = 0.2 * cos_r * b_mix;
    b_weights[2] = 0.6 * b_mix;                    // Primary B input
    b_weights[3] = 0.05;
    b_weights[4] = 0.1;
    b_weights[5] = 0.15;
    b_weights[6] = 0.0;

    // Apply matrix mixing with CA3054 soft limiting
    var r_out = input_matrix_row(channels, r_weights, sharpness);
    var g_out = input_matrix_row(channels, g_weights, sharpness);
    var b_out = input_matrix_row(channels, b_weights, sharpness);

    // Apply output bias
    r_out = clamp(r_out + output_bias, 0.0, 1.0);
    g_out = clamp(g_out + output_bias, 0.0, 1.0);
    b_out = clamp(b_out + output_bias, 0.0, 1.0);

    return vec4<f32>(r_out, g_out, b_out, 1.0);
}
