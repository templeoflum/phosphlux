// Paik/Abe PED Control (Pedestal Control) - Amplitude Window Colorizer
// Based on schematic page 28 of Paik-Abe Video Synthesizer Technical Documentation
//
// This is the CORE of the Paik/Abe colorizer effect.
// Uses CA3019 matched diode array for precise amplitude window detection.
//
// Signal flow from schematic:
//   VID IN → Amplifiers → CA3019 (with PED CTL thresholds) → Output stages
//   CHROMA IN → CA3019 → Mixed with video based on window
//   BLK IN → Blanking control

struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    // PED Control thresholds (from front panel pots on schematic)
    ped_ctl_1: f32,    // First threshold level
    ped_ctl_2: f32,    // Second threshold level
    ped_ctl_3: f32,    // Third threshold level
    clip_level: f32,   // Clip control (pin 8)
    chroma_gain: f32,  // Chroma input gain
    // Color assignments for each window
    color_1: vec3<f32>,
    color_2: vec3<f32>,
    color_3: vec3<f32>,
    color_4: vec3<f32>,
}

@group(0) @binding(0) var<uniform> u: Uniforms;

// ============================================================================
// CA3019 DIODE ARRAY FUNCTIONS (from model.wgsl)
// ============================================================================

const DIODE_VT: f32 = 0.026;
const DIODE_VF: f32 = 0.65;

fn diode_conducting(signal: f32, threshold: f32) -> f32 {
    let diff = signal - threshold;
    let sharpness = 30.0;
    return 1.0 / (1.0 + exp(-diff * sharpness));
}

fn amplitude_window(signal: f32, low: f32, high: f32) -> f32 {
    let above_low = diode_conducting(signal, low);
    let below_high = diode_conducting(high, signal);
    return above_low * below_high;
}

fn amplitude_window_soft(signal: f32, low: f32, high: f32, softness: f32) -> f32 {
    let edge_width = max(softness * 0.1, 0.001);
    let low_edge = smoothstep(low - edge_width, low + edge_width, signal);
    let high_edge = smoothstep(high - edge_width, high + edge_width, signal);
    return low_edge * (1.0 - high_edge);
}

// ============================================================================
// PED CONTROL CIRCUIT EMULATION
// ============================================================================

/// Main PED Control function
/// Takes luminance input and outputs colorized RGB based on amplitude windows
fn ped_control(
    luminance: f32,
    ped_1: f32,        // Threshold 1 (0-1)
    ped_2: f32,        // Threshold 2 (0-1)
    ped_3: f32,        // Threshold 3 (0-1)
    colors: array<vec3<f32>, 4>,
    softness: f32      // Edge softness (0 = hard, 1 = soft)
) -> vec3<f32> {
    // Sort thresholds to ensure proper ordering
    var t1 = min(min(ped_1, ped_2), ped_3);
    var t3 = max(max(ped_1, ped_2), ped_3);
    var t2 = ped_1 + ped_2 + ped_3 - t1 - t3;

    // Create 4 amplitude windows using CA3019 diode detection
    // Window 0: black to threshold 1
    // Window 1: threshold 1 to threshold 2
    // Window 2: threshold 2 to threshold 3
    // Window 3: threshold 3 to white

    let w0 = amplitude_window_soft(luminance, 0.0, t1, softness);
    let w1 = amplitude_window_soft(luminance, t1, t2, softness);
    let w2 = amplitude_window_soft(luminance, t2, t3, softness);
    let w3 = amplitude_window_soft(luminance, t3, 1.0, softness);

    // Mix colors based on window membership
    var color = vec3<f32>(0.0);
    color = color + colors[0] * w0;
    color = color + colors[1] * w1;
    color = color + colors[2] * w2;
    color = color + colors[3] * w3;

    return color;
}

/// PED Control with chroma mixing (as shown in schematic)
/// Chroma input gets gated by the amplitude windows
fn ped_control_with_chroma(
    luminance: f32,
    chroma: vec3<f32>,
    ped_1: f32,
    ped_2: f32,
    ped_3: f32,
    chroma_gain: f32
) -> vec3<f32> {
    // Sort thresholds
    var t1 = min(min(ped_1, ped_2), ped_3);
    var t3 = max(max(ped_1, ped_2), ped_3);
    var t2 = ped_1 + ped_2 + ped_3 - t1 - t3;

    // Middle window gates the chroma input
    let chroma_window = amplitude_window(luminance, t1, t3);

    // Output is luminance + chroma gated by window
    let luma_out = vec3<f32>(luminance);
    let chroma_out = chroma * chroma_window * chroma_gain;

    return luma_out + chroma_out;
}

// ============================================================================
// TEST SHADER - DEMONSTRATES PED CONTROL COLORIZER
// ============================================================================

@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = frag_coord.xy / u.resolution;

    // Generate test luminance ramp (vertical gradient)
    let luminance = uv.y;

    // Default color palette (Paik/Abe style - high saturation)
    var colors: array<vec3<f32>, 4>;
    colors[0] = u.color_1;  // Darkest region
    colors[1] = u.color_2;  // Low-mid region
    colors[2] = u.color_3;  // Mid-high region
    colors[3] = u.color_4;  // Brightest region

    // Apply PED control colorization
    let softness = 0.3;  // Slight edge softness for authentic analog feel
    let colorized = ped_control(
        luminance,
        u.ped_ctl_1,
        u.ped_ctl_2,
        u.ped_ctl_3,
        colors,
        softness
    );

    return vec4<f32>(colorized, 1.0);
}

// ============================================================================
// VERTEX SHADER
// ============================================================================

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
    );

    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    output.uv = positions[vertex_index] * 0.5 + 0.5;
    return output;
}
