// ============================================
// PHOSPHLUX v2 - Analog Component Models
// ============================================
//
// GPU implementations of real analog components.
// These form the building blocks for device emulations.
//
// Each model captures authentic behavior including:
// - Saturation and clipping characteristics
// - Frequency-dependent gain
// - Slew rate limiting
// - Thermal/tolerance variation

// === CONSTANTS ===

const PI: f32 = 3.14159265359;
const TWO_PI: f32 = 6.28318530718;

// Thermal voltage at 25°C (for transistor equations)
const VT: f32 = 0.026;

// === OP-AMP MODELS ===

// Op-amp parameters structure
struct OpAmpParams {
    // Supply rails
    v_pos: f32,
    v_neg: f32,
    // Saturation offset from rails
    sat_offset: f32,
    // Slew rate in normalized units per sample
    slew_rate: f32,
    // Open loop gain (for saturation curve shape)
    gain: f32,
}

// LM741 parameters (slow, characterful)
fn opamp_lm741() -> OpAmpParams {
    return OpAmpParams(
        1.0,     // v_pos (normalized)
        -1.0,    // v_neg
        0.133,   // sat_offset (2V / 15V)
        0.033,   // slew_rate (0.5 V/µs normalized)
        10.0     // effective gain for curve shape
    );
}

// TL072 parameters (faster, cleaner)
fn opamp_tl072() -> OpAmpParams {
    return OpAmpParams(
        1.0,
        -1.0,
        0.1,
        0.87,    // 13 V/µs
        20.0
    );
}

// LM6172 parameters (video speed)
fn opamp_lm6172() -> OpAmpParams {
    return OpAmpParams(
        1.0,
        -1.0,
        0.067,   // Can get closer to rails
        1.0,     // Effectively instant at video rates
        5.0      // Lower gain but much faster
    );
}

// Op-amp saturation curve
// Models the soft clipping as output approaches rails
fn opamp_saturate(v_in: f32, params: OpAmpParams) -> f32 {
    let v_max = params.v_pos - params.sat_offset;
    let v_min = params.v_neg + params.sat_offset;
    let range = v_max - v_min;
    let center = (v_max + v_min) / 2.0;

    // Normalize to [-1, 1]
    let normalized = (v_in - center) / (range / 2.0);

    // Soft saturation using tanh-like curve
    // The gain parameter controls how sharp the knee is
    let saturated = tanh(normalized * params.gain) / tanh(params.gain);

    // Scale back
    return saturated * (range / 2.0) + center;
}

// Slew-rate limited op-amp output
// prev: previous output value
// desired: desired output
// dt: time step (normalized, typically 1.0 for per-sample)
fn opamp_slew(prev: f32, desired: f32, slew_rate: f32) -> f32 {
    let delta = desired - prev;
    let max_delta = slew_rate;

    if abs(delta) <= max_delta {
        return desired;
    } else {
        return prev + sign(delta) * max_delta;
    }
}

// Complete op-amp model
// v_plus: non-inverting input
// v_minus: inverting input
// prev_out: previous output (for slew limiting)
// params: op-amp parameters
fn opamp_compute(v_plus: f32, v_minus: f32, prev_out: f32, params: OpAmpParams) -> f32 {
    // Differential input
    let v_diff = v_plus - v_minus;

    // Amplify (in practice, feedback sets closed-loop gain)
    let v_ideal = v_diff * params.gain;

    // Saturate at rails
    let v_sat = opamp_saturate(v_ideal, params);

    // Apply slew rate limiting
    return opamp_slew(prev_out, v_sat, params.slew_rate);
}

// === TRANSISTOR MODELS ===

// Simplified transistor behavior for shader use
// Returns collector current fraction [0, 1]
fn transistor_npn(v_base: f32, v_emitter: f32, v_collector: f32, beta: f32) -> f32 {
    let vbe = v_base - v_emitter;
    let vce = v_collector - v_emitter;

    // Cutoff
    if vbe < 0.5 {
        return 0.0;
    }

    // Saturation
    if vce < 0.1 {
        return vce / 0.1;
    }

    // Active region - exponential characteristic
    let ib = exp((vbe - 0.65) / VT) - 1.0;
    return clamp(ib * beta, 0.0, 1.0);
}

// === DIODE MODELS ===

// Diode forward characteristic
// Returns forward current (normalized)
fn diode_forward(v_anode: f32, v_cathode: f32, v_forward: f32) -> f32 {
    let v = v_anode - v_cathode;
    if v < 0.0 {
        return 0.0; // Reverse biased
    }
    // Exponential knee
    return clamp(exp((v - v_forward) / (2.0 * VT)) - 1.0, 0.0, 1.0);
}

// Hard clipping diodes (like in guitar pedals)
fn diode_clip(v_in: f32, v_forward: f32) -> f32 {
    if v_in > v_forward {
        return v_forward + (v_in - v_forward) * 0.01; // Soft knee
    } else if v_in < -v_forward {
        return -v_forward + (v_in + v_forward) * 0.01;
    }
    return v_in;
}

// === RC FILTER MODELS ===

// Single-pole lowpass (RC filter)
// v_in: input voltage
// v_cap: current capacitor voltage (state)
// alpha: filter coefficient = dt / (RC + dt)
// Returns: new capacitor voltage (filtered output)
fn rc_lowpass(v_in: f32, v_cap: f32, alpha: f32) -> f32 {
    return v_cap + alpha * (v_in - v_cap);
}

// Single-pole highpass (RC filter)
// Returns: highpass output
fn rc_highpass(v_in: f32, v_in_prev: f32, v_out_prev: f32, alpha: f32) -> f32 {
    return alpha * (v_out_prev + v_in - v_in_prev);
}

// Calculate alpha from time constant and sample rate
// tau: RC time constant in seconds
// dt: sample period in seconds
fn rc_alpha(tau: f32, dt: f32) -> f32 {
    return dt / (tau + dt);
}

// === WAVESHAPING ===

// Triangle wave folding (Paik/Abe style)
// This is the classic "colorizer" effect
fn wave_fold(x: f32, gain: f32) -> f32 {
    var v = x * gain;
    // Triangle fold using fract
    v = abs(fract(v * 0.5 + 0.25) * 2.0 - 1.0);
    return v;
}

// Multi-stage folding (more complex harmonics)
fn wave_fold_multi(x: f32, gain: f32, stages: u32) -> f32 {
    var v = x;
    for (var i = 0u; i < stages; i = i + 1u) {
        v = wave_fold(v, gain);
        // Each stage reduces amplitude slightly
        v = v * 0.9;
    }
    return v;
}

// Soft clipping waveshaper (tube-like)
fn wave_soft_clip(x: f32, drive: f32) -> f32 {
    return tanh(x * drive) / tanh(drive);
}

// Hard clipping waveshaper
fn wave_hard_clip(x: f32, threshold: f32) -> f32 {
    return clamp(x, -threshold, threshold);
}

// Asymmetric clipping (like real diodes)
fn wave_asymmetric_clip(x: f32, pos_thresh: f32, neg_thresh: f32) -> f32 {
    if x > pos_thresh {
        return pos_thresh + (x - pos_thresh) * 0.1;
    } else if x < -neg_thresh {
        return -neg_thresh + (x + neg_thresh) * 0.1;
    }
    return x;
}

// === QUANTIZATION ===

// Level quantizer (Sandin Amplitude Classifier)
fn quantize_levels(x: f32, levels: f32) -> f32 {
    return floor(x * levels + 0.5) / levels;
}

// Posterize with adjustable gamma
fn quantize_posterize(x: f32, levels: f32, gamma: f32) -> f32 {
    let linear = pow(x, gamma);
    let quantized = quantize_levels(linear, levels);
    return pow(quantized, 1.0 / gamma);
}

// === COLORIZERS ===

// Rainbow/spectrum colorizer
fn colorize_spectrum(luma: f32) -> vec3<f32> {
    let h = luma * 6.0;
    let i = floor(h);
    let f = h - i;
    let q = 1.0 - f;

    let hi = i32(i) % 6;

    if hi == 0 { return vec3<f32>(1.0, f, 0.0); }
    if hi == 1 { return vec3<f32>(q, 1.0, 0.0); }
    if hi == 2 { return vec3<f32>(0.0, 1.0, f); }
    if hi == 3 { return vec3<f32>(0.0, q, 1.0); }
    if hi == 4 { return vec3<f32>(f, 0.0, 1.0); }
    return vec3<f32>(1.0, 0.0, q);
}

// Threshold colorizer (Dave Jones style)
// Maps luma ranges to distinct colors
fn colorize_threshold(luma: f32, thresholds: vec4<f32>, colors: array<vec3<f32>, 5>) -> vec3<f32> {
    if luma < thresholds.x {
        return colors[0];
    } else if luma < thresholds.y {
        return colors[1];
    } else if luma < thresholds.z {
        return colors[2];
    } else if luma < thresholds.w {
        return colors[3];
    }
    return colors[4];
}

// Heat map colorizer (thermal camera style)
fn colorize_heat(luma: f32) -> vec3<f32> {
    // Black -> Blue -> Cyan -> Green -> Yellow -> Red -> White
    let t = luma * 6.0;

    if t < 1.0 {
        return mix(vec3<f32>(0.0), vec3<f32>(0.0, 0.0, 1.0), t);
    } else if t < 2.0 {
        return mix(vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(0.0, 1.0, 1.0), t - 1.0);
    } else if t < 3.0 {
        return mix(vec3<f32>(0.0, 1.0, 1.0), vec3<f32>(0.0, 1.0, 0.0), t - 2.0);
    } else if t < 4.0 {
        return mix(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 1.0, 0.0), t - 3.0);
    } else if t < 5.0 {
        return mix(vec3<f32>(1.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), t - 4.0);
    }
    return mix(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(1.0, 1.0, 1.0), t - 5.0);
}

// === KEYERS ===

// Luma key with soft edges
fn luma_key(rgb: vec3<f32>, threshold: f32, softness: f32) -> f32 {
    let luma = to_luma(rgb);
    let low = threshold - softness * 0.5;
    let high = threshold + softness * 0.5;
    return smoothstep(low, high, luma);
}

// Chroma key (green screen)
fn chroma_key(rgb: vec3<f32>, key_color: vec3<f32>, tolerance: f32, softness: f32) -> f32 {
    let dist = distance(rgb, key_color);
    return smoothstep(tolerance, tolerance + softness, dist);
}

// Difference key
fn difference_key(a: vec3<f32>, b: vec3<f32>, threshold: f32, softness: f32) -> f32 {
    let diff = length(a - b);
    return smoothstep(threshold, threshold + softness, diff);
}

// === BLEND MODES ===

fn blend_add(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return min(a + b, vec3<f32>(1.0));
}

fn blend_multiply(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return a * b;
}

fn blend_screen(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return 1.0 - (1.0 - a) * (1.0 - b);
}

fn blend_overlay(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    let r = select(1.0 - 2.0 * (1.0 - a.r) * (1.0 - b.r), 2.0 * a.r * b.r, a.r < 0.5);
    let g = select(1.0 - 2.0 * (1.0 - a.g) * (1.0 - b.g), 2.0 * a.g * b.g, a.g < 0.5);
    let bl = select(1.0 - 2.0 * (1.0 - a.b) * (1.0 - b.b), 2.0 * a.b * b.b, a.b < 0.5);
    return vec3<f32>(r, g, bl);
}

fn blend_difference(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return abs(a - b);
}

// Mix with key signal
fn blend_key_mix(a: vec3<f32>, b: vec3<f32>, key: f32) -> vec3<f32> {
    return mix(a, b, key);
}
