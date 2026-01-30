// ============================================
// FOLDING AMP - Workbench Test Version
// ============================================
//
// Video wavefolder for luminance manipulation.
// Uses 7-input binding layout for workbench compatibility.
//
// Parameters:
// - gain: Fold amount (1.0-8.0)
// - offset: Depth/stages (mapped to 1.0-5.0)
// - mix: Output mix with original
// - fold: Bias/asymmetry (0.0-1.0)

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Params {
    gain: f32,
    offset: f32,    // depth
    mix: f32,
    fold: f32,      // bias
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

// Triangle fold - hard corners
fn fold_triangle(x: f32) -> f32 {
    return abs(fract(x * 0.5 + 0.25) * 2.0 - 1.0);
}

// Multi-stage folding with transistor-like soft compression
fn fold_channel(x: f32, gain: f32, depth: f32, bias: f32) -> f32 {
    let bias_offset = (bias - 0.5) * 0.5;
    var v = x + bias_offset;
    v = v * gain;

    let stages = u32(clamp(depth, 1.0, 5.0));
    for (var i = 0u; i < stages; i = i + 1u) {
        v = fold_triangle(v);
        v = v * 0.98 + 0.01;
    }

    // Soft compression (transistor saturation)
    return tanh(v * 1.5) / tanh(1.5);
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

    // Extract luminance
    let luma = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));

    // Map params
    let gain = max(params.gain, 1.0);           // 1.0-8.0
    let depth = params.offset * 4.0 + 1.0;      // 1.0-5.0
    let bias = params.fold * 0.5 + 0.25;        // 0.25-0.75

    // Fold
    let folded = fold_channel(luma, gain, depth, bias);

    // Mix with original
    let output_luma = mix(luma, folded, params.mix);

    return vec4<f32>(output_luma, output_luma, output_luma, 1.0);
}
