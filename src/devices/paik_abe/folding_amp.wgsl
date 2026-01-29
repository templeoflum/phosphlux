// ============================================
// PAIK/ABE FOLDING AMPLIFIER
// ============================================
//
// Standalone folding amplifier extracted from the Paik/Abe Video Synthesizer.
// This is the heart of the Paik/Abe color generation effect.
//
// The folding amplifier takes an input signal and "folds" it back on itself
// when it exceeds certain thresholds, creating the characteristic psychedelic
// color banding effect. Different gain values for R/G/B channels create
// the distinctive color shifts.
//
// Circuit behavior:
// - Triangle wave folding creates harmonic overtones
// - Multiple stages produce richer, more complex patterns
// - Slight asymmetry from component tolerances adds organic character
// - Soft saturation at fold peaks models transistor behavior
//
// Reference: research/components/functional/folding_amp/v1/model.wgsl
//
// Parameters:
// - gain: Fold amount (1.0-8.0, default 2.0) - higher = more folds
// - depth: Number of folding stages (1.0-5.0, default 2.0)
// - bias: DC offset/asymmetry (0.0-1.0, default 0.5)

// Basic triangle fold - hard corners
fn paik_abe_fold_triangle(x: f32) -> f32 {
    // fract gives sawtooth, convert to triangle
    return abs(fract(x * 0.5 + 0.25) * 2.0 - 1.0);
}

// Paik/Abe style soft fold with transistor-like response
// Includes slight compression and asymmetry characteristic of the original
fn paik_abe_fold_channel(x: f32, gain: f32, depth: f32, bias: f32) -> f32 {
    // Center bias around 0.5 (parameter range 0-1, effective range -0.5 to 0.5)
    let bias_offset = (bias - 0.5) * 0.5;

    // Add bias for asymmetric folding
    var v = x + bias_offset;

    // Amplify by gain
    v = v * gain;

    // Multi-stage folding for richer harmonics
    let stages = u32(clamp(depth, 1.0, 5.0));
    for (var i = 0u; i < stages; i = i + 1u) {
        // Triangle fold
        v = paik_abe_fold_triangle(v);

        // Slight asymmetry per stage (simulates component tolerance)
        v = v * 0.98 + 0.01;
    }

    // Soft compression at peaks (transistor saturation characteristic)
    let compressed = tanh(v * 1.5) / tanh(1.5);

    return compressed;
}

// Main folding amplifier processing function
fn device_paik_abe_folding_amp_process(
    uv: vec2<f32>,
    time: f32,
    video_in: vec4<f32>,
    gain: f32,
    depth: f32,
    bias: f32
) -> vec4<f32> {
    // Extract luminance from input (Rec. 601 coefficients)
    let luma = dot(video_in.rgb, vec3<f32>(0.299, 0.587, 0.114));

    // Process luminance through folding amplifier
    // All channels folded identically - differential coloring happens
    // in the InputMatrix stage upstream or gain stages downstream
    let folded = paik_abe_fold_channel(luma, gain, depth, bias);

    // Output as grayscale (preserving fold information for downstream processing)
    // Downstream devices (PedColorizer, OutputEncoder) will add color
    return vec4<f32>(folded, folded, folded, 1.0);
}
