// CA3054 Dual Differential Amplifier Model
// Two independent matched differential amplifier stages
//
// Used in Paik/Abe Video Synthesizer for:
// - Keying signal processing (page 19)
// - Matrix mixing with weighted inputs
// - Hard/soft video keying
//
// Each amplifier stage provides:
// - High input impedance differential input
// - Voltage-controlled gain via tail current
// - Soft limiting characteristic (tanh transfer function)
// - Linear region for small signals, compressed for large

// ============================================================================
// DIFFERENTIAL AMPLIFIER CORE
// ============================================================================

/// Thermal voltage constant (26mV at room temperature)
const VT: f32 = 0.026;

/// Single differential amplifier stage
/// Models the core transfer function of one CA3054 diff amp
///
/// Parameters:
///   v_plus: Non-inverting input voltage (0-1 normalized)
///   v_minus: Inverting input voltage (0-1 normalized)
///   tail_current: Bias current controlling gain (0-1)
///   gain: Output stage gain multiplier
///
/// Returns: Output voltage (centered around 0.5 for bipolar, 0-1 range)
fn diff_amp(v_plus: f32, v_minus: f32, tail_current: f32, gain: f32) -> f32 {
    let v_diff = v_plus - v_minus;

    // Tanh transfer function models the diff pair's soft limiting
    // At small signals: linear with slope proportional to tail current
    // At large signals: smoothly limits to +/- tail_current
    let normalized_diff = v_diff / (2.0 * VT * 10.0);  // Scale for video levels
    let diff_output = tanh(normalized_diff) * tail_current;

    // Apply gain and offset to 0-1 range
    return clamp(0.5 + diff_output * gain, 0.0, 1.0);
}

/// Differential amplifier with adjustable softness
/// Softness controls the transition region width
///
/// Parameters:
///   v_plus, v_minus: Input voltages
///   tail_current: Gain/bias control
///   softness: 0 = hard switching, 1 = very soft/linear
fn diff_amp_soft(v_plus: f32, v_minus: f32, tail_current: f32, softness: f32) -> f32 {
    let v_diff = v_plus - v_minus;

    // Softness controls the "knee" of the tanh curve
    // Lower values = sharper transition (more like comparator)
    // Higher values = smoother transition (more linear)
    let sharpness = mix(50.0, 5.0, softness);
    let normalized_diff = v_diff * sharpness;

    return tanh(normalized_diff) * tail_current;
}

// ============================================================================
// VIDEO KEYING FUNCTIONS
// ============================================================================

/// Hard key - binary switching between two sources
/// Like a digital switch controlled by key signal
///
/// Parameters:
///   source_a: First video source
///   source_b: Second video source
///   key_signal: Control signal (0 = source_a, 1 = source_b)
///   threshold: Switching point (0-1)
fn hard_key(source_a: f32, source_b: f32, key_signal: f32, threshold: f32) -> f32 {
    // Use diff amp as comparator
    let key_diff = diff_amp(key_signal, threshold, 1.0, 10.0);

    // Binary selection
    return select(source_a, source_b, key_diff > 0.5);
}

/// Soft key - smooth crossfade between two sources
/// Creates dissolve effect controlled by key signal
///
/// Parameters:
///   source_a: First video source
///   source_b: Second video source
///   key_signal: Control signal for blend amount
///   softness: Transition softness (0 = sharp, 1 = gradual)
fn soft_key(source_a: f32, source_b: f32, key_signal: f32, softness: f32) -> f32 {
    // Use soft diff amp for smooth transition
    let blend = diff_amp_soft(key_signal, 0.5, 1.0, softness);

    // Map from -1..1 to 0..1 for blend factor
    let blend_factor = clamp(blend * 0.5 + 0.5, 0.0, 1.0);

    return mix(source_a, source_b, blend_factor);
}

/// Luminance key - key based on brightness of key source
/// Classic video keying technique
///
/// Parameters:
///   background: Background video
///   foreground: Foreground video
///   key_source: Source for key generation (often same as foreground)
///   clip_level: Black level threshold
///   gain: Key gain (sharpness)
fn luma_key(background: f32, foreground: f32, key_source: f32,
            clip_level: f32, key_gain: f32) -> f32 {
    // Generate key from luminance
    let key_raw = (key_source - clip_level) * key_gain;
    let key = clamp(key_raw, 0.0, 1.0);

    return mix(background, foreground, key);
}

// ============================================================================
// MATRIX MIXING FUNCTIONS
// ============================================================================

/// Single matrix mixer input with weighted contribution
/// Part of RGB matrix mixing system
///
/// Parameters:
///   input: Input signal (0-1)
///   weight: Mixing weight (-1 to 1, negative inverts)
///   bias_current: Gain control via diff amp bias
fn matrix_input(input: f32, weight: f32, bias_current: f32) -> f32 {
    // Diff amp provides voltage-controlled gain
    let amplified = diff_amp(input, 0.5, bias_current, abs(weight));

    // Handle polarity
    if (weight < 0.0) {
        return 1.0 - amplified;  // Invert
    }
    return amplified;
}

/// 3-input matrix mixer row
/// Combines 3 inputs with individual weights for one output channel
///
/// Parameters:
///   in1, in2, in3: Input signals
///   w1, w2, w3: Mixing weights
fn matrix_row_3(in1: f32, in2: f32, in3: f32,
                w1: f32, w2: f32, w3: f32) -> f32 {
    // Sum weighted inputs (centered around 0.5)
    let sum = (in1 - 0.5) * w1 + (in2 - 0.5) * w2 + (in3 - 0.5) * w3;

    // Soft limit the sum using diff amp characteristic
    let limited = tanh(sum * 2.0) * 0.5;

    return clamp(limited + 0.5, 0.0, 1.0);
}

/// 7-input matrix mixer row (Paik/Abe has 7 channels)
/// Full matrix row for one RGB output
fn matrix_row_7(inputs: array<f32, 7>, weights: array<f32, 7>) -> f32 {
    var sum: f32 = 0.0;

    for (var i: u32 = 0u; i < 7u; i = i + 1u) {
        sum += (inputs[i] - 0.5) * weights[i];
    }

    // Soft limiting
    let limited = tanh(sum * 2.0) * 0.5;

    return clamp(limited + 0.5, 0.0, 1.0);
}

// ============================================================================
// FULL MATRIX MIXER (3x7 for RGB from 7 channels)
// ============================================================================

/// RGB output structure
struct RGBOutput {
    r: f32,
    g: f32,
    b: f32,
}

/// Full 7-channel to RGB matrix mixer
/// Implements the Paik/Abe matrix mixing amp (page 8)
///
/// Parameters:
///   inputs: 7 input channels
///   r_weights: Weights for red channel
///   g_weights: Weights for green channel
///   b_weights: Weights for blue channel
fn matrix_mixer_7x3(
    inputs: array<f32, 7>,
    r_weights: array<f32, 7>,
    g_weights: array<f32, 7>,
    b_weights: array<f32, 7>
) -> RGBOutput {
    var output: RGBOutput;

    output.r = matrix_row_7(inputs, r_weights);
    output.g = matrix_row_7(inputs, g_weights);
    output.b = matrix_row_7(inputs, b_weights);

    return output;
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Voltage-controlled amplifier using diff amp
/// Gain controlled by external CV signal
fn vca_diff_amp(input: f32, cv: f32) -> f32 {
    // CV controls tail current of diff amp
    let tail = clamp(cv, 0.0, 1.0);
    return diff_amp(input, 0.5, tail, 1.0);
}

/// Comparator with hysteresis using dual diff amp
/// Uses both amps in CA3054 for Schmitt trigger behavior
fn comparator_hysteresis(input: f32, threshold: f32, hysteresis: f32) -> f32 {
    let upper = threshold + hysteresis * 0.5;
    let lower = threshold - hysteresis * 0.5;

    // First amp: compare to upper threshold
    let above_upper = diff_amp(input, upper, 1.0, 20.0);

    // Second amp: compare to lower threshold
    let above_lower = diff_amp(input, lower, 1.0, 20.0);

    // SR latch behavior approximation
    if (above_upper > 0.9) {
        return 1.0;
    } else if (above_lower < 0.1) {
        return 0.0;
    }
    return 0.5;  // In hysteresis band
}

/// Signal balance control
/// Adjusts balance between two signals (pan-like)
fn signal_balance(left: f32, right: f32, balance: f32) -> f32 {
    // Balance: -1 = full left, 0 = center, 1 = full right
    let left_gain = clamp(1.0 - balance, 0.0, 1.0);
    let right_gain = clamp(1.0 + balance, 0.0, 1.0);

    return left * left_gain + right * right_gain;
}
