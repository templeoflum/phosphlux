// Folding Amplifier - Paik/Abe Style Wavefolder
// Nonlinear waveshaper that creates wrap-around distortion
//
// Workbench Parameters:
//   gain      → Fold amount (1 = no fold, higher = more folds)
//   offset    → DC bias/asymmetry
//   mix       → Dry/wet blend
//   fold      → Softness (0 = hard triangle, 1 = soft sine)
//   bias      → Luminance vs RGB mode (0 = RGB, 1 = luma only)
//   frequency → Phase offset per channel (color fringing)

struct Params {
    gain: f32,      // Fold amount
    offset: f32,    // DC bias
    mix: f32,       // Dry/wet
    fold: f32,      // Softness
    bias: f32,      // Luma mode
    frequency: f32, // Phase offset
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
// FOLDING FUNCTIONS
// ============================================================================

/// Basic triangle fold
fn fold_triangle(x: f32) -> f32 {
    let saw = fract(x);
    return abs(saw * 2.0 - 1.0);
}

/// Soft sine-based fold
fn fold_sine(x: f32) -> f32 {
    return abs(sin(x * 3.14159));
}

/// Adjustable fold with softness
fn fold_adjustable(x: f32, gain: f32, softness: f32, bias: f32) -> f32 {
    // Apply bias
    let biased = x + bias;

    // Amplify
    let amplified = biased * gain;

    // Hard fold (triangle)
    let hard = fold_triangle(amplified);

    // Soft fold (sine)
    let soft = fold_sine(amplified);

    // Blend
    let folded = mix(hard, soft, softness);

    // Add slight transistor-like compression
    return tanh(folded * 1.3) / tanh(1.3);
}

/// Paik/Abe style fold with characteristic warmth
fn fold_paik_abe(x: f32, gain: f32, bias: f32, softness: f32) -> f32 {
    let biased = x + bias * 0.5;
    let amplified = biased * gain;

    // Multi-stage soft folding
    var sig = amplified;

    // First fold
    sig = fold_triangle(sig);

    // Soft saturation
    sig = tanh(sig * (2.0 - softness)) / tanh(2.0 - softness);

    // Second subtle fold for harmonics
    if (gain > 2.0) {
        sig = mix(sig, fold_triangle(sig * 1.5), (gain - 2.0) * 0.2);
    }

    return sig;
}

// ============================================================================
// MAIN SHADER
// ============================================================================

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let input_color = textureSample(input_texture, input_sampler, in.uv);
    let rgb = input_color.rgb;

    // Get params with defaults
    let fold_gain = select(params.gain, 2.0, params.gain == 0.0);
    let dc_bias = select(params.offset, 0.0, params.offset == 0.0) * 0.5 - 0.25;
    let dry_wet = select(params.mix, 1.0, params.mix == 0.0);
    let softness = select(params.fold, 0.3, params.fold == 0.0);
    let luma_mode = select(params.bias, 0.0, params.bias == 0.0);
    let phase_offset = select(params.frequency, 0.0, params.frequency == 0.0) * 0.2;

    var folded: vec3<f32>;

    if (luma_mode > 0.5) {
        // Luminance-only folding
        let luma = dot(rgb, vec3<f32>(0.299, 0.587, 0.114));
        let folded_luma = fold_paik_abe(luma, fold_gain, dc_bias, softness);

        // Scale RGB to match
        let scale = select(folded_luma / luma, 1.0, luma < 0.001);
        folded = clamp(rgb * scale, vec3<f32>(0.0), vec3<f32>(1.0));
    } else {
        // RGB folding with optional phase offset
        folded = vec3<f32>(
            fold_paik_abe(rgb.r, fold_gain, dc_bias, softness),
            fold_paik_abe(rgb.g, fold_gain, dc_bias + phase_offset, softness),
            fold_paik_abe(rgb.b, fold_gain, dc_bias + phase_offset * 2.0, softness)
        );
    }

    // Dry/wet mix
    let output_color = mix(rgb, folded, dry_wet);

    return vec4<f32>(output_color, 1.0);
}

// ============================================================================
// VERTEX SHADER
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
