// ============================================
// PED COLORIZER - Workbench Test Version
// ============================================
//
// Amplitude window colorizer based on CA3019 matched diode array.
// Uses 7-input binding layout for workbench compatibility.
//
// Parameters:
// - gain: Threshold 1 (0.0-1.0)
// - offset: Threshold 2 (0.0-1.0)
// - mix: Threshold 3 (0.0-1.0)
// - fold: Softness (0.0-1.0)
// - bias: Clipping level (0.0-1.0)

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Params {
    gain: f32,      // threshold_1
    offset: f32,    // threshold_2
    mix: f32,       // threshold_3
    fold: f32,      // softness
    bias: f32,      // clipping
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

// CA3019 diode characteristics
const PED_DIODE_SHARPNESS: f32 = 40.0;

// CA3019 diode threshold detection
fn ped_diode_detect(signal: f32, threshold: f32, softness: f32) -> f32 {
    let diff = (signal - threshold) / max(softness * 0.1 + 0.01, 0.01);
    let effective_sharpness = PED_DIODE_SHARPNESS * (1.0 - softness * 0.8);
    return 1.0 / (1.0 + exp(-diff * effective_sharpness));
}

// Amplitude window using matched diode pair
fn ped_window(signal: f32, low: f32, high: f32, softness: f32) -> f32 {
    let above_low = ped_diode_detect(signal, low, softness);
    let above_high = ped_diode_detect(signal, high, softness);
    return above_low * (1.0 - above_high);
}

// Paik/Abe color palette
fn ped_palette(index: u32) -> vec3<f32> {
    switch(index) {
        case 0u: { return vec3<f32>(0.1, 0.0, 0.8); }   // Deep blue (shadows)
        case 1u: { return vec3<f32>(0.9, 0.0, 0.6); }   // Magenta (low-mids)
        case 2u: { return vec3<f32>(1.0, 0.9, 0.0); }   // Yellow (high-mids)
        case 3u: { return vec3<f32>(0.0, 0.9, 0.8); }   // Cyan (highlights)
        default: { return vec3<f32>(0.5); }
    }
}

// Apply clipping (CLP pin 8)
fn ped_clipping(signal: f32, clip_level: f32) -> f32 {
    let max_level = max(clip_level, 0.01);
    return min(signal, max_level) / max_level;
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
    let luma_raw = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));

    // Map params
    let t1 = params.gain * 0.5;              // 0-0.5 range for threshold 1
    let t2 = params.offset * 0.5 + 0.25;     // 0.25-0.75 range for threshold 2
    let t3 = params.mix * 0.5 + 0.5;         // 0.5-1.0 range for threshold 3
    let softness = params.fold;
    let clipping = params.bias + 0.5;        // 0.5-1.5 range

    // Apply clipping
    let luma = ped_clipping(luma_raw, clipping);

    // Sort thresholds
    var th1 = min(min(t1, t2), t3);
    var th3 = max(max(t1, t2), t3);
    var th2 = t1 + t2 + t3 - th1 - th3;

    // Create amplitude windows
    let w0 = ped_window(luma, 0.0, th1, softness);
    let w1 = ped_window(luma, th1, th2, softness);
    let w2 = ped_window(luma, th2, th3, softness);
    let w3 = ped_window(luma, th3, 1.0, softness);

    // Mix colors
    var out_color = vec3<f32>(0.0);
    out_color = out_color + ped_palette(0u) * w0;
    out_color = out_color + ped_palette(1u) * w1;
    out_color = out_color + ped_palette(2u) * w2;
    out_color = out_color + ped_palette(3u) * w3;

    return vec4<f32>(clamp(out_color, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
