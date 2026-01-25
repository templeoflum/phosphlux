// LM311 Comparator-based Video Keyer
// Hard threshold keying with window comparator option
//
// The LM311 is a classic voltage comparator used extensively in
// video synthesis for keying, level detection, and sync generation.
// This shader demonstrates threshold and window comparator modes.

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Params {
    gain: f32,       // Threshold level (0-1)
    offset: f32,     // Window width (for window mode)
    mix: f32,        // Key mix amount
    fold: f32,       // Invert key (>0.5 inverts)
    bias: f32,       // Softness (0=hard, 1=soft)
    frequency: f32,  // Not used
    phase: f32,      // Not used
    threshold: f32,  // Secondary threshold for window
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

// === LM311 Comparator Model ===
// From research/components/ics/lm311/model.wgsl

// Hard comparator: output high when V+ > V-
fn lm311_compare(v_plus: f32, v_minus: f32) -> f32 {
    return select(0.0, 1.0, v_plus > v_minus);
}

// Soft comparator with adjustable sharpness
fn lm311_compare_soft(v_plus: f32, v_minus: f32, sharpness: f32) -> f32 {
    let diff = v_plus - v_minus;
    return clamp(diff * sharpness + 0.5, 0.0, 1.0);
}

// Window comparator: output high when signal is between thresholds
fn lm311_window(signal: f32, low: f32, high: f32) -> f32 {
    let above_low = select(0.0, 1.0, signal > low);
    let below_high = select(0.0, 1.0, signal < high);
    return above_low * below_high;
}

// Soft window comparator
fn lm311_window_soft(signal: f32, low: f32, high: f32, sharpness: f32) -> f32 {
    let above_low = clamp((signal - low) * sharpness + 0.5, 0.0, 1.0);
    let below_high = clamp((high - signal) * sharpness + 0.5, 0.0, 1.0);
    return above_low * below_high;
}

// === Shader ===

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
    let color = textureSample(input_texture, input_sampler, input.uv);

    // Convert to luminance for keying
    let luma = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));

    // Calculate thresholds
    let threshold_low = params.gain;
    let threshold_high = params.gain + params.offset;

    // Sharpness from bias (inverted: 0=soft, 1=hard)
    let sharpness = 5.0 + (1.0 - params.bias) * 95.0;  // 5 to 100

    // Generate key signal
    var key: f32;
    if (params.offset > 0.01) {
        // Window mode: key between two thresholds
        key = lm311_window_soft(luma, threshold_low, threshold_high, sharpness);
    } else {
        // Simple threshold mode
        key = lm311_compare_soft(luma, threshold_low, sharpness);
    }

    // Invert if fold > 0.5
    if (params.fold > 0.5) {
        key = 1.0 - key;
    }

    // Apply key: show white where key is active, original elsewhere
    let keyed = vec3<f32>(key);

    // Mix key visualization with original
    let output = mix(color.rgb, keyed, params.mix);

    return vec4<f32>(output, 1.0);
}
