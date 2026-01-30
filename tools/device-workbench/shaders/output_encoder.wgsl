// ============================================
// OUTPUT ENCODER - Workbench Test Version
// ============================================
//
// Final output stage with LM741 op-amp characteristics.
// Uses 7-input binding layout for workbench compatibility.
//
// Parameters:
// - gain: Saturation (0.0-2.0)
// - offset: Slew rate character (0.0-1.0)
// - mix: Warmth bias (0.0-1.0, 0.5=neutral)

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Params {
    gain: f32,      // saturation
    offset: f32,    // slew
    mix: f32,       // warmth
    fold: f32,
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

// LM741 op-amp saturation
fn opamp_sat(v_in: f32, slew_factor: f32) -> f32 {
    let v_sat = 0.87;
    let gain = 3.0;
    let normalized = v_in * 2.0 - 1.0;
    var saturated = tanh(normalized * gain) / tanh(gain);
    let slew_smooth = 1.0 - slew_factor * 0.3;
    saturated = saturated * slew_smooth + normalized * (1.0 - slew_smooth);
    return (saturated + 1.0) * 0.5 * v_sat + (1.0 - v_sat) * 0.5;
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
    var rgb = color.rgb;

    // Map params
    let saturation = params.gain;              // 0-2
    let slew = params.offset;                  // 0-1
    let warmth = params.mix;                   // 0-1

    // Apply op-amp saturation
    var r = opamp_sat(rgb.r, slew);
    var g = opamp_sat(rgb.g, slew);
    var b = opamp_sat(rgb.b, slew);

    // Saturation adjustment
    let output_luma = dot(vec3<f32>(r, g, b), vec3<f32>(0.299, 0.587, 0.114));
    r = mix(output_luma, r, saturation);
    g = mix(output_luma, g, saturation);
    b = mix(output_luma, b, saturation);

    // Thermal drift warmth
    let warmth_amount = (warmth - 0.5) * 0.1;
    r = r + warmth_amount;
    b = b - warmth_amount * 0.5;

    return vec4<f32>(clamp(vec3<f32>(r, g, b), vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
