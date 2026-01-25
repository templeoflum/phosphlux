// Folding Amplifier Model
// Nonlinear waveshaper that folds signal back instead of clipping
//
// Classic video synthesis effect from Paik/Abe, Sandin, and others.
// Creates distinctive wrap-around distortion where bright areas
// fold back to dark, producing banding and psychedelic effects.

// ============================================================================
// CORE FOLDING FUNCTIONS
// ============================================================================

/// Basic triangle fold - hard corners
/// Input signal folds back at 0 and 1 boundaries
///
/// Parameters:
///   x: Input signal (can exceed 0-1 range)
///
/// Returns: Folded signal (always 0-1)
fn fold_triangle(x: f32) -> f32 {
    // fract gives sawtooth, convert to triangle
    let saw = fract(x);
    return abs(saw * 2.0 - 1.0);
}

/// Triangle fold with gain control
/// Higher gain = more folds
///
/// Parameters:
///   x: Input signal (0-1)
///   gain: Fold amount (1 = no fold, 2 = one fold, etc.)
fn fold_triangle_gain(x: f32, gain: f32) -> f32 {
    let amplified = x * gain;
    return fold_triangle(amplified);
}

/// Soft/sine fold - rounded corners like Buchla wavefolder
/// Creates smoother, more musical folding
///
/// Parameters:
///   x: Input signal (0-1)
///   gain: Fold amount
fn fold_sine(x: f32, gain: f32) -> f32 {
    let amplified = x * gain;
    // Sine gives smooth folding with rounded peaks
    return sin(amplified * 3.14159) * 0.5 + 0.5;
}

/// Serge-style sharp fold with adjustable stages
/// Each stage adds more harmonic content
///
/// Parameters:
///   x: Input signal
///   stages: Number of folding stages (1-4)
fn fold_serge(x: f32, stages: u32) -> f32 {
    var sig = x;

    for (var i: u32 = 0u; i < stages; i = i + 1u) {
        // Each stage: amplify and fold
        sig = sig * 2.0;
        if (sig > 1.0) {
            sig = 2.0 - sig;
        }
        if (sig < 0.0) {
            sig = -sig;
        }
    }

    return clamp(sig, 0.0, 1.0);
}

/// Paik/Abe style soft fold with transistor-like response
/// Includes slight compression and asymmetry
///
/// Parameters:
///   x: Input signal
///   gain: Fold amount
///   bias: DC offset/asymmetry (-0.5 to 0.5)
fn fold_paik_abe(x: f32, gain: f32, bias: f32) -> f32 {
    // Add bias for asymmetric folding
    let biased = x + bias;

    // Amplify
    let amplified = biased * gain;

    // Soft fold using tanh-wrapped triangle
    // This models the transistor's soft saturation
    let folded = fold_triangle(amplified);

    // Add slight compression at peaks (transistor saturation)
    let compressed = tanh(folded * 1.5) / tanh(1.5);

    return compressed;
}

/// Adjustable fold with softness control
/// Interpolates between hard triangle and soft sine
///
/// Parameters:
///   x: Input signal
///   gain: Fold amount (1-8 typical)
///   softness: Corner rounding (0 = hard, 1 = soft)
///   symmetry: Fold center point (0.5 = symmetric)
fn fold_adjustable(x: f32, gain: f32, softness: f32, symmetry: f32) -> f32 {
    // Offset by symmetry
    let centered = x - symmetry;

    // Amplify
    let amplified = centered * gain + symmetry;

    // Hard fold (triangle)
    let hard = fold_triangle(amplified);

    // Soft fold (sine-based)
    let phase = amplified * 3.14159;
    let soft = abs(sin(phase));

    // Blend based on softness
    return mix(hard, soft, softness);
}

// ============================================================================
// MULTI-CHANNEL FOLDING
// ============================================================================

/// Fold RGB channels independently
fn fold_rgb(rgb: vec3<f32>, gain: f32, softness: f32) -> vec3<f32> {
    return vec3<f32>(
        fold_adjustable(rgb.r, gain, softness, 0.5),
        fold_adjustable(rgb.g, gain, softness, 0.5),
        fold_adjustable(rgb.b, gain, softness, 0.5)
    );
}

/// Fold luminance only, preserve color ratios
fn fold_luminance(rgb: vec3<f32>, gain: f32, softness: f32) -> vec3<f32> {
    let luma = dot(rgb, vec3<f32>(0.299, 0.587, 0.114));
    let folded_luma = fold_adjustable(luma, gain, softness, 0.5);

    // Scale RGB to match new luminance
    let scale = select(folded_luma / luma, 1.0, luma < 0.001);

    return clamp(rgb * scale, vec3<f32>(0.0), vec3<f32>(1.0));
}

/// Fold with phase offset per channel (creates color fringing)
fn fold_rgb_phase(rgb: vec3<f32>, gain: f32, phase_offset: f32) -> vec3<f32> {
    return vec3<f32>(
        fold_adjustable(rgb.r, gain, 0.3, 0.5),
        fold_adjustable(rgb.g, gain, 0.3, 0.5 + phase_offset),
        fold_adjustable(rgb.b, gain, 0.3, 0.5 + phase_offset * 2.0)
    );
}

// ============================================================================
// SPECIAL EFFECTS
// ============================================================================

/// Feedback fold - output feeds back into input
/// Creates complex, chaotic patterns
fn fold_feedback(x: f32, gain: f32, feedback: f32, iterations: u32) -> f32 {
    var sig = x;

    for (var i: u32 = 0u; i < iterations; i = i + 1u) {
        let folded = fold_triangle_gain(sig, gain);
        sig = mix(x, folded, 1.0 - feedback * f32(i) / f32(iterations));
    }

    return sig;
}

/// Threshold fold - only folds signal above/below thresholds
fn fold_threshold(x: f32, gain: f32, low: f32, high: f32) -> f32 {
    if (x < low || x > high) {
        // Outside threshold: fold
        return fold_triangle_gain(x, gain);
    }
    // Inside threshold: pass through
    return x;
}

/// Modulated fold - gain varies with another signal
fn fold_modulated(x: f32, base_gain: f32, mod_signal: f32, mod_depth: f32) -> f32 {
    let gain = base_gain + mod_signal * mod_depth;
    return fold_adjustable(x, gain, 0.3, 0.5);
}
