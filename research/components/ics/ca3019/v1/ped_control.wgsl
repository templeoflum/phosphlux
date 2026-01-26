// Paik/Abe PED Control (Pedestal Control) - Amplitude Window Colorizer
// Based on schematic page 28 of Paik-Abe Video Synthesizer Technical Documentation
//
// This is the CORE of the Paik/Abe colorizer effect.
// Uses CA3019 matched diode array for precise amplitude window detection.
//
// Signal flow from schematic:
//   VID IN -> Amplifiers -> CA3019 (with PED CTL thresholds) -> Output stages
//   CHROMA IN -> CA3019 -> Mixed with video based on window
//   BLK IN -> Blanking control
//
// Workbench Parameters:
//   gain      -> PED threshold 1 (lowest window boundary)
//   offset    -> PED threshold 2 (middle window boundary)
//   mix       -> PED threshold 3 (upper window boundary)
//   fold      -> Edge softness (0 = hard, 1 = soft analog edges)
//   bias      -> Color palette selector (0..1 selects palette)
//   frequency -> Chroma gain (amount of colorization vs luma)

struct Params {
    gain: f32,      // PED threshold 1
    offset: f32,    // PED threshold 2
    mix: f32,       // PED threshold 3
    fold: f32,      // Softness
    bias: f32,      // Palette selector
    frequency: f32, // Chroma gain
    phase: f32,
    threshold: f32,
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
// COLOR PALETTES
// ============================================================================

/// Built-in color palettes for the 4 amplitude windows.
/// Palette selection via the bias parameter (0..1 range, 6 palettes).
/// These represent the high-saturation colors typical of Paik/Abe output.

fn get_palette(index: u32) -> array<vec3<f32>, 4> {
    var colors: array<vec3<f32>, 4>;

    switch index {
        // Palette 0: Classic Paik/Abe (blue-magenta-yellow-cyan)
        case 0u: {
            colors[0] = vec3<f32>(0.1, 0.0, 0.8);   // Deep blue
            colors[1] = vec3<f32>(0.9, 0.0, 0.6);   // Magenta
            colors[2] = vec3<f32>(1.0, 0.9, 0.0);   // Yellow
            colors[3] = vec3<f32>(0.0, 0.9, 0.8);   // Cyan
        }
        // Palette 1: Warm (red-orange-yellow-white)
        case 1u: {
            colors[0] = vec3<f32>(0.5, 0.0, 0.0);   // Dark red
            colors[1] = vec3<f32>(1.0, 0.3, 0.0);   // Orange
            colors[2] = vec3<f32>(1.0, 0.9, 0.2);   // Yellow
            colors[3] = vec3<f32>(1.0, 1.0, 0.9);   // Warm white
        }
        // Palette 2: Cool (purple-blue-cyan-green)
        case 2u: {
            colors[0] = vec3<f32>(0.3, 0.0, 0.5);   // Purple
            colors[1] = vec3<f32>(0.0, 0.2, 0.9);   // Blue
            colors[2] = vec3<f32>(0.0, 0.8, 0.7);   // Cyan
            colors[3] = vec3<f32>(0.2, 1.0, 0.3);   // Green
        }
        // Palette 3: Psychedelic (green-magenta-orange-blue)
        case 3u: {
            colors[0] = vec3<f32>(0.0, 0.8, 0.2);   // Green
            colors[1] = vec3<f32>(0.9, 0.0, 0.9);   // Magenta
            colors[2] = vec3<f32>(1.0, 0.5, 0.0);   // Orange
            colors[3] = vec3<f32>(0.0, 0.3, 1.0);   // Blue
        }
        // Palette 4: Monochrome (black-dark gray-light gray-white)
        case 4u: {
            colors[0] = vec3<f32>(0.0, 0.0, 0.0);
            colors[1] = vec3<f32>(0.33, 0.33, 0.33);
            colors[2] = vec3<f32>(0.66, 0.66, 0.66);
            colors[3] = vec3<f32>(1.0, 1.0, 1.0);
        }
        // Palette 5: Complementary (red-cyan-green-magenta)
        default: {
            colors[0] = vec3<f32>(0.9, 0.1, 0.1);   // Red
            colors[1] = vec3<f32>(0.1, 0.9, 0.9);   // Cyan
            colors[2] = vec3<f32>(0.1, 0.9, 0.1);   // Green
            colors[3] = vec3<f32>(0.9, 0.1, 0.9);   // Magenta
        }
    }

    return colors;
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
// MAIN SHADER
// ============================================================================

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample input texture
    let input_color = textureSample(input_texture, input_sampler, in.uv);
    let rgb = input_color.rgb;

    // Extract luminance for PED window detection
    let luminance = dot(rgb, vec3<f32>(0.299, 0.587, 0.114));

    // Get parameters with defaults
    let ped_1 = select(params.gain, 0.25, params.gain == 0.0);
    let ped_2 = select(params.offset, 0.5, params.offset == 0.0);
    let ped_3 = select(params.mix, 0.75, params.mix == 0.0);
    let softness = select(params.fold, 0.3, params.fold == 0.0);
    let palette_sel = select(params.bias, 0.0, params.bias == 0.0);
    let chroma_gain = select(params.frequency, 1.0, params.frequency == 0.0);

    // Select color palette (6 palettes mapped across 0..1)
    let palette_index = u32(floor(palette_sel * 5.99));
    let colors = get_palette(palette_index);

    // Apply PED control colorization
    let colorized = ped_control(luminance, ped_1, ped_2, ped_3, colors, softness);

    // Blend between full colorization and luminance-preserving mix
    // chroma_gain=1 is full color replacement, 0 preserves original
    let output_color = mix(rgb, colorized, chroma_gain);

    return vec4<f32>(output_color, 1.0);
}

// ============================================================================
// VERTEX SHADER - Fullscreen Triangle
// ============================================================================

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var out: VertexOutput;

    let x = f32((vi & 1u) << 2u) - 1.0;
    let y = f32((vi & 2u) << 1u) - 1.0;

    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);

    return out;
}
