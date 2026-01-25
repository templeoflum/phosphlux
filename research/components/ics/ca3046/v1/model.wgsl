// CA3046 NPN Transistor Array Model
// 5 matched transistors on single die
//
// This models the fundamental BJT behavior for video-rate signals.
// For GPU emulation, we simplify the full SPICE model to key behaviors:
// - Exponential Vbe->Ic characteristic
// - Early effect (Vce dependence)
// - Saturation limiting
// - Matched transistor behavior (same Is, Vt for all 5)

struct TransistorParams {
    is_: f32,        // Saturation current (normalized)
    vt: f32,         // Thermal voltage ~26mV
    beta: f32,       // Current gain hFE
    vaf: f32,        // Early voltage
    vce_sat: f32,    // Saturation voltage
}

// Default CA3046 parameters (normalized for 0-1V video signals)
const CA3046_PARAMS = TransistorParams(
    1e-6,    // is_ (scaled for video levels)
    0.026,   // vt (thermal voltage)
    100.0,   // beta (typical hFE)
    100.0,   // vaf (Early voltage)
    0.1,     // vce_sat
);

// Single transistor transfer function
// Input: vbe (base-emitter voltage), vce (collector-emitter voltage)
// Output: ic (collector current, normalized)
fn bjt_npn(vbe: f32, vce: f32, params: TransistorParams) -> f32 {
    // Exponential characteristic: Ic = Is * exp(Vbe/Vt)
    let exp_term = exp(clamp(vbe, -0.5, 0.8) / params.vt);
    let ic_ideal = params.is_ * exp_term;

    // Early effect: multiply by (1 + Vce/Vaf)
    let early_factor = 1.0 + max(vce, 0.0) / params.vaf;

    // Saturation limiting (soft clip when Vce < Vce_sat)
    let sat_factor = tanh(max(vce, 0.0) / params.vce_sat);

    return ic_ideal * early_factor * sat_factor;
}

// Differential pair (Q1, Q2 share emitter)
// Classic long-tailed pair configuration
// Input: v1, v2 (base voltages)
// Output: differential current (i1 - i2), normalized to +-1
fn differential_pair(v1: f32, v2: f32, tail_current: f32) -> f32 {
    let vdiff = v1 - v2;
    // Tanh approximation of diff pair transfer function
    // Accurate for small signals, soft limits for large
    return tanh(vdiff / (2.0 * CA3046_PARAMS.vt)) * tail_current;
}

// Current mirror (Q3 as reference, Q4 as output)
// Output current tracks input current (with ratio)
fn current_mirror(i_ref: f32, ratio: f32) -> f32 {
    // Ideal mirror: Iout = Iref * ratio
    // Add slight mismatch for realism
    return i_ref * ratio;
}

// Exponential converter (for VCO, VCA control)
// Attempt to produce 1V/octave-like response
fn exponential_converter(v_control: f32) -> f32 {
    // Attempt to produce 1V/octave-like response
    let v_normalized = v_control * 10.0; // Scale input
    return exp(v_normalized * 0.0693); // ln(2) ≈ 0.693, scaled
}

// === Video-specific applications ===

// Amplitude classifier / window comparator
// Uses matched transistors to create sharp threshold
fn amplitude_window(signal: f32, threshold_low: f32, threshold_high: f32) -> f32 {
    let above_low = differential_pair(signal, threshold_low, 1.0);
    let below_high = differential_pair(threshold_high, signal, 1.0);
    // Both conditions must be true (AND)
    return clamp((above_low + 1.0) * (below_high + 1.0) / 4.0, 0.0, 1.0);
}

// Matrix mixer element (weighted sum with gain)
fn matrix_element(input: f32, weight: f32, gain: f32) -> f32 {
    let weighted = input * weight;
    // Use diff pair as voltage-controlled gain
    return differential_pair(weighted, 0.0, gain);
}
