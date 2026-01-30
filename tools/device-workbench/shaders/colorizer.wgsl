// ============================================
// WAVEFOLD COLORIZER - Workbench Test Version
// ============================================
//
// Triangle wave folding colorizer with per-channel gains.
// Uses 7-input binding layout for workbench compatibility.
//
// Parameters:
// - gain: Red channel gain (0.5-8.0)
// - offset: Green channel gain offset from red
// - mix: Blue channel gain offset from red
// - fold: Fold depth (1-5 stages)
// - bias: Saturation (0-2)
// - frequency: Slew rate (0-1)

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Params {
    gain: f32,      // gain_r base
    offset: f32,    // gain_g offset
    mix: f32,       // gain_b offset
    fold: f32,      // fold_depth
    bias: f32,      // saturation
    frequency: f32, // slew
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

@group(0) @binding(7) var input_sampler: sampler;
@group(0) @binding(8) var<uniform> params: Params;

// Op-amp saturation with soft limiting
fn opamp_sat(v_in: f32, slew_factor: f32) -> f32 {
    let v_sat = 0.87;
    let gain = 3.0;
    let normalized = v_in * 2.0 - 1.0;
    var saturated = tanh(normalized * gain) / tanh(gain);
    let slew_smooth = 1.0 - slew_factor * 0.3;
    saturated = saturated * slew_smooth + normalized * (1.0 - slew_smooth);
    return (saturated + 1.0) * 0.5 * v_sat + (1.0 - v_sat) * 0.5;
}

// Triangle wave folding
fn fold(x: f32, gain: f32, depth: f32) -> f32 {
    var v = x * gain;
    let stages = u32(clamp(depth, 1.0, 5.0));
    for (var i = 0u; i < stages; i = i + 1u) {
        v = abs(fract(v * 0.5 + 0.25) * 2.0 - 1.0);
        v = v * 0.98 + 0.01;
    }
    return v;
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
    let color = textureSample(input0, input_sampler, input.uv);
    let luma = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));

    // Map params to colorizer controls
    let gain_r = max(params.gain, 0.5);
    let gain_g = gain_r + params.offset * 2.0;      // offset adds 0-2 to base
    let gain_b = gain_r + params.mix * 3.0;         // mix adds 0-3 to base
    let fold_depth = params.fold * 4.0 + 1.0;       // 1-5 stages
    let saturation = params.bias * 2.0;             // 0-2
    let slew = params.frequency;                     // 0-1

    // Fold each channel with different gains
    var r = fold(luma, gain_r, fold_depth);
    var g = fold(luma, gain_g, fold_depth);
    var b = fold(luma, gain_b, fold_depth);

    // Apply op-amp saturation
    r = opamp_sat(r, slew);
    g = opamp_sat(g, slew);
    b = opamp_sat(b, slew);

    // Saturation adjustment
    let output_luma = (r + g + b) / 3.0;
    r = mix(output_luma, r, saturation);
    g = mix(output_luma, g, saturation);
    b = mix(output_luma, b, saturation);

    // Warmth bias
    r = r + 0.02;
    b = b - 0.01;

    return vec4<f32>(clamp(vec3<f32>(r, g, b), vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
