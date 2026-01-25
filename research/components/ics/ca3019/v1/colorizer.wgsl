// CA3019 PED Control - Paik/Abe Amplitude Window Colorizer
// Based on schematic page 28 of Paik-Abe Video Synthesizer Technical Documentation
//
// This is the CORE colorizer effect of the Paik/Abe Video Synthesizer.
// The CA3019 matched diode array creates precise amplitude windows that
// map luminance levels to distinct colors.
//
// Workbench Parameters:
//   gain   → Threshold 1 (0.0-1.0, default 0.25)
//   offset → Threshold 2 (0.0-1.0, default 0.50)
//   mix    → Threshold 3 (0.0-1.0, default 0.75)
//   bias   → Edge softness (0.0=hard, 1.0=soft, default 0.25)
//
// Workbench binding order:
//   Binding 0: Input texture
//   Binding 1: Sampler
//   Binding 2: Uniforms (Params struct)

struct Params {
    gain: f32,      // Threshold 1
    offset: f32,    // Threshold 2
    mix: f32,       // Threshold 3
    fold: f32,      // unused
    bias: f32,      // Softness
    frequency: f32, // unused
    phase: f32,     // unused
    threshold: f32, // unused
    _pad0: f32, _pad1: f32, _pad2: f32, _pad3: f32,
    _pad4: f32, _pad5: f32, _pad6: f32, _pad7: f32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;
@group(0) @binding(2) var<uniform> params: Params;

// ============================================================================
// CA3019 AMPLITUDE WINDOW DETECTION
// ============================================================================

/// Amplitude window using CA3019 matched diode pair
/// The matched diodes create precise threshold detection with sharp transitions.
///
/// Parameters:
///   signal: Input luminance (0-1)
///   low: Lower threshold
///   high: Upper threshold
///   soft: Edge softness (0-1)
///
/// Returns: Window membership (0-1)
fn amplitude_window(signal: f32, low: f32, high: f32, soft: f32) -> f32 {
    // Edge width based on softness parameter
    // Hard edges (soft=0) give sharp posterization
    // Soft edges (soft=1) give smooth color blending
    let edge = max(soft * 0.08, 0.005);

    // Smoothstep creates the S-curve transition characteristic
    // of the CA3019 diode response
    let low_edge = smoothstep(low - edge, low + edge, signal);
    let high_edge = smoothstep(high - edge, high + edge, signal);

    return low_edge * (1.0 - high_edge);
}

/// Diode conducting detection (sigmoid approximation)
/// Models the exponential turn-on of a silicon diode
fn diode_conducting(signal: f32, threshold: f32) -> f32 {
    let diff = signal - threshold;
    let sharpness = 30.0;  // Matched diode sharpness
    return 1.0 / (1.0 + exp(-diff * sharpness));
}

// ============================================================================
// COLOR PALETTES
// ============================================================================

/// Classic Paik/Abe video art palette
/// High saturation, psychedelic colors typical of 1960s-70s video art
fn paik_abe_palette(index: u32) -> vec3<f32> {
    switch(index) {
        case 0u: { return vec3<f32>(0.15, 0.0, 0.35); }   // Deep purple (blacks)
        case 1u: { return vec3<f32>(0.0, 0.25, 0.9); }    // Electric blue
        case 2u: { return vec3<f32>(0.0, 0.95, 0.4); }    // Neon green
        case 3u: { return vec3<f32>(1.0, 0.85, 0.1); }    // Golden yellow
        default: { return vec3<f32>(1.0); }               // White fallback
    }
}

/// Alternative warm palette
fn warm_palette(index: u32) -> vec3<f32> {
    switch(index) {
        case 0u: { return vec3<f32>(0.1, 0.0, 0.15); }    // Dark magenta
        case 1u: { return vec3<f32>(0.6, 0.0, 0.3); }     // Deep red
        case 2u: { return vec3<f32>(1.0, 0.4, 0.0); }     // Orange
        case 3u: { return vec3<f32>(1.0, 0.95, 0.5); }    // Warm yellow
        default: { return vec3<f32>(1.0); }
    }
}

/// Cool palette
fn cool_palette(index: u32) -> vec3<f32> {
    switch(index) {
        case 0u: { return vec3<f32>(0.0, 0.05, 0.15); }   // Deep blue
        case 1u: { return vec3<f32>(0.0, 0.3, 0.5); }     // Ocean blue
        case 2u: { return vec3<f32>(0.0, 0.7, 0.6); }     // Cyan
        case 3u: { return vec3<f32>(0.7, 1.0, 0.9); }     // Light cyan
        default: { return vec3<f32>(1.0); }
    }
}

// ============================================================================
// MAIN COLORIZER
// ============================================================================

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample input texture
    let input_color = textureSample(input_texture, input_sampler, in.uv);

    // Extract luminance using standard Rec. 601 coefficients
    let luminance = dot(input_color.rgb, vec3<f32>(0.299, 0.587, 0.114));

    // Map params to thresholds with sensible defaults
    let t1 = select(params.gain, 0.25, params.gain == 0.0);
    let t2 = select(params.offset, 0.50, params.offset == 0.0);
    let t3 = select(params.mix, 0.75, params.mix == 0.0);
    let soft = select(params.bias, 0.25, params.bias == 0.0);

    // Sort thresholds to ensure proper window ordering
    // This allows free adjustment without worrying about order
    var th1 = min(min(t1, t2), t3);
    var th3 = max(max(t1, t2), t3);
    var th2 = t1 + t2 + t3 - th1 - th3;

    // Create 4 amplitude windows using CA3019 diode array detection
    // Window 0: Black to threshold 1 (darkest tones)
    // Window 1: Threshold 1 to 2 (dark midtones)
    // Window 2: Threshold 2 to 3 (bright midtones)
    // Window 3: Threshold 3 to white (highlights)
    let w0 = amplitude_window(luminance, 0.0, th1, soft);
    let w1 = amplitude_window(luminance, th1, th2, soft);
    let w2 = amplitude_window(luminance, th2, th3, soft);
    let w3 = amplitude_window(luminance, th3, 1.0, soft);

    // Mix colors based on window membership
    var color = vec3<f32>(0.0);
    color += paik_abe_palette(0u) * w0;
    color += paik_abe_palette(1u) * w1;
    color += paik_abe_palette(2u) * w2;
    color += paik_abe_palette(3u) * w3;

    return vec4<f32>(color, 1.0);
}

// ============================================================================
// VERTEX SHADER - Fullscreen Triangle
// ============================================================================

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var out: VertexOutput;

    // Fullscreen triangle technique - single oversized triangle
    // More efficient than quad (avoids diagonal seam artifacts)
    let x = f32((vi & 1u) << 2u) - 1.0;  // -1, 3, -1
    let y = f32((vi & 2u) << 1u) - 1.0;  // -1, -1, 3

    out.position = vec4<f32>(x, y, 0.0, 1.0);

    // UV from position: map [-1,1] to [0,1], flip Y for texture coords
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);

    return out;
}
