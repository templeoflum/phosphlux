// CA3019 - Ultra-fast Low-Capacitance Matched Diode Array
// RCA/Intersil - 6 diodes (quad + 2 individual)
//
// Core component of Paik/Abe Video Synthesizer amplitude window detection
// Used in PED Control (colorizer) and Burst Generator circuits

// Physical constants
const DIODE_VT: f32 = 0.026;       // Thermal voltage at 25°C (~26mV)
const DIODE_VF: f32 = 0.65;        // Forward voltage (typical)
const DIODE_N: f32 = 1.0;          // Ideality factor (matched diodes)
const DIODE_IS: f32 = 1e-14;       // Saturation current

// ============================================================================
// CORE DIODE TRANSFER FUNCTIONS
// ============================================================================

/// Ideal diode exponential characteristic (normalized for 0-1 signals)
/// Returns normalized current for given forward voltage
fn diode_exponential(vf: f32) -> f32 {
    // I = Is * (exp(V / (n * Vt)) - 1)
    // Normalized for video signals (0-1 range maps to 0-1V)
    let v_scaled = vf * DIODE_VF;  // Scale 0-1 to actual voltage range
    return clamp(exp(v_scaled / (DIODE_N * DIODE_VT)) - 1.0, 0.0, 1.0);
}

/// Practical diode response for video signals
/// Smooth transition around threshold, unity gain above
fn diode_forward(signal: f32, threshold: f32) -> f32 {
    // Soft threshold using diode characteristic
    // signal > threshold: conducts (returns signal - threshold)
    // signal < threshold: blocks (returns ~0)
    let diff = signal - threshold;

    // Sharpness factor controls transition steepness
    // Higher = sharper threshold (more like ideal diode)
    let sharpness = 20.0;

    // Smooth approximation of max(0, diff)
    let softplus = log(1.0 + exp(diff * sharpness)) / sharpness;
    return softplus;
}

/// Diode "on" state detector - returns 0-1 based on conduction
fn diode_conducting(signal: f32, threshold: f32) -> f32 {
    let diff = signal - threshold;
    let sharpness = 30.0;  // Sharp transition for window detection
    return 1.0 / (1.0 + exp(-diff * sharpness));
}

// ============================================================================
// AMPLITUDE WINDOW DETECTION
// ============================================================================

/// Single amplitude window using matched diode pair
/// Returns 1.0 when signal is within window, 0.0 outside
/// Soft transitions at boundaries due to diode characteristics
fn amplitude_window(signal: f32, low_threshold: f32, high_threshold: f32) -> f32 {
    // D1: conducts when signal > low (signal passed lower limit)
    // D2: conducts when signal < high (signal below upper limit)
    // Window = D1 conducting AND D2 NOT conducting from above

    let above_low = diode_conducting(signal, low_threshold);
    let below_high = diode_conducting(high_threshold, signal);

    return above_low * below_high;
}

/// Hard amplitude window (ideal comparators, no soft edges)
fn amplitude_window_hard(signal: f32, low: f32, high: f32) -> f32 {
    if signal > low && signal < high {
        return 1.0;
    }
    return 0.0;
}

/// Amplitude window with adjustable edge softness
fn amplitude_window_soft(signal: f32, low: f32, high: f32, softness: f32) -> f32 {
    // softness: 0.0 = hard edges, 1.0 = very soft edges
    let edge_width = max(softness * 0.1, 0.001);

    let low_edge = smoothstep(low - edge_width, low + edge_width, signal);
    let high_edge = smoothstep(high - edge_width, high + edge_width, signal);

    return low_edge * (1.0 - high_edge);
}

// ============================================================================
// MULTI-LEVEL AMPLITUDE CLASSIFICATION
// ============================================================================

/// Classify signal into N discrete levels using cascaded windows
/// Returns level index (0 to num_levels-1) as normalized float
fn amplitude_classify(signal: f32, num_levels: u32) -> f32 {
    var level: f32 = 0.0;
    let step = 1.0 / f32(num_levels);

    for (var i: u32 = 0u; i < num_levels; i = i + 1u) {
        let threshold = f32(i + 1u) * step;
        level = level + diode_conducting(signal, threshold);
    }

    return level / f32(num_levels);
}

/// Get window membership for specific level in multi-level system
fn amplitude_level_window(signal: f32, level: u32, num_levels: u32) -> f32 {
    let step = 1.0 / f32(num_levels);
    let low = f32(level) * step;
    let high = f32(level + 1u) * step;

    return amplitude_window(signal, low, high);
}

// ============================================================================
// PAIK/ABE SPECIFIC FUNCTIONS
// ============================================================================

/// Burst gate using CA3019 (from page 27 schematic)
/// Gates the 3.58MHz color burst signal during blanking
fn burst_gate(subcarrier: f32, gate_pulse: f32) -> f32 {
    // Gate pulse controls diode conduction
    // When gate is high, subcarrier passes through
    let gate_threshold = 0.5;
    let gate_on = diode_conducting(gate_pulse, gate_threshold);

    return subcarrier * gate_on;
}

/// PED Control amplitude window (from page 28 schematic)
/// Creates color bands based on luminance amplitude
fn ped_control_window(
    luminance: f32,
    ped_low: f32,      // PED CTL low threshold
    ped_high: f32,     // PED CTL high threshold
    chroma_in: f32     // Chroma input to mix
) -> f32 {
    // Detect if luminance is within pedestal window
    let in_window = amplitude_window(luminance, ped_low, ped_high);

    // Mix chroma based on window membership
    return chroma_in * in_window;
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Diode OR gate - conducts if ANY input exceeds threshold
fn diode_or(a: f32, b: f32, threshold: f32) -> f32 {
    let a_on = diode_conducting(a, threshold);
    let b_on = diode_conducting(b, threshold);
    return max(a_on, b_on);
}

/// Diode AND gate - conducts if ALL inputs exceed threshold
fn diode_and(a: f32, b: f32, threshold: f32) -> f32 {
    let a_on = diode_conducting(a, threshold);
    let b_on = diode_conducting(b, threshold);
    return a_on * b_on;
}

/// Matched diode voltage reference
/// Creates stable voltage level using forward voltage drop
fn diode_reference(supply: f32, num_diodes: u32) -> f32 {
    // Each diode drops ~Vf when conducting
    return f32(num_diodes) * DIODE_VF / supply;
}
