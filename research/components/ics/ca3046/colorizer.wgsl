// CA3046-based Colorizer
// Uses transistor array differential pair for amplitude windows
// Emulates the Paik/Abe colorizer technique
//
// This shader demonstrates the CA3046's differential pair used as
// amplitude window comparators to create color banding effects.

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

// === CA3046 Component Model ===
// From research/components/ics/ca3046/model.wgsl

const CA3046_VT: f32 = 0.026;  // Thermal voltage ~26mV

// Differential pair transfer function
// Classic long-tailed pair - output is tanh of voltage difference
fn differential_pair(v1: f32, v2: f32, tail_current: f32) -> f32 {
    let vdiff = v1 - v2;
    return tanh(vdiff / (2.0 * CA3046_VT)) * tail_current;
}

// Amplitude window comparator
// Uses matched transistors to create sharp threshold
fn amplitude_window(signal: f32, threshold_low: f32, threshold_high: f32) -> f32 {
    let above_low = differential_pair(signal, threshold_low, 1.0);
    let below_high = differential_pair(threshold_high, signal, 1.0);
    // Both conditions must be true (AND)
    return clamp((above_low + 1.0) * (below_high + 1.0) / 4.0, 0.0, 1.0);
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

    // Convert to luminance
    let luma = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));

    // Apply gain and offset
    let signal = luma * params.gain + params.offset;

    // Create color bands using amplitude windows
    // threshold param controls band width, frequency controls number of bands
    let band_width = 0.15 + params.threshold * 0.3;  // 0.15 to 0.45
    let num_bands = 3.0 + params.frequency * 5.0;     // 3 to 8 bands

    // Calculate which bands are active
    var r = 0.0;
    var g = 0.0;
    var b = 0.0;

    // Band 1: Low luminance -> Blue
    let band1 = amplitude_window(signal, 0.0, 0.33);
    b += band1;

    // Band 2: Mid-low luminance -> Cyan
    let band2 = amplitude_window(signal, 0.2, 0.5);
    g += band2 * 0.8;
    b += band2 * 0.5;

    // Band 3: Mid luminance -> Green
    let band3 = amplitude_window(signal, 0.35, 0.65);
    g += band3;

    // Band 4: Mid-high luminance -> Yellow
    let band4 = amplitude_window(signal, 0.5, 0.8);
    r += band4 * 0.9;
    g += band4 * 0.7;

    // Band 5: High luminance -> Red
    let band5 = amplitude_window(signal, 0.65, 1.0);
    r += band5;

    // Combine and normalize
    let colorized = vec3<f32>(
        clamp(r, 0.0, 1.0),
        clamp(g, 0.0, 1.0),
        clamp(b, 0.0, 1.0)
    );

    // Mix with original based on mix param
    let output = mix(color.rgb, colorized, params.mix);

    return vec4<f32>(output, 1.0);
}
