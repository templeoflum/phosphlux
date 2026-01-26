// CA3022 Wideband Video Amplifier Model
// Low-power broadband amplifier from RCA CA3020 family
//
// This models the CA3022's key behaviors for video-rate signals:
// - Configurable voltage gain via external R2/R1 ratio
// - Single dominant pole bandwidth rolloff (~10 MHz)
// - Soft saturation at supply rail limits
// - Slew rate limiting on fast transitions
// - DC bias offset
//
// In the Paik/Abe synthesizer, the CA3022 serves as the chroma key
// video output amplifier (schematic pages 10, 20), buffering and
// amplifying the signal after keying/mixing stages.

// --- Core gain stage ---
// Amplifies input signal with configurable gain and DC offset.
// gain: voltage gain factor (R2/R1 ratio, typically 2-100)
// bias: DC operating point offset (-0.5 to +0.5 in normalized range)
fn ca3022_gain_stage(input: f32, gain: f32, bias: f32) -> f32 {
    return input * gain + bias;
}

// --- Bandwidth limiting ---
// Approximates single-pole RC lowpass rolloff using neighbor-pixel averaging.
// In the spatial domain, this is equivalent to temporal bandwidth limiting:
// high spatial frequency = high temporal frequency on a scanned display.
//
// input: current pixel value
// uv: texture coordinate (used for spatial context in application shaders)
// cutoff: normalized cutoff (0.0 = full bandwidth, 1.0 = heavy filtering)
//
// Note: This is a simplified model. The full application shader implements
// the actual spatial blur using texture samples. This function provides
// the mixing coefficient.
fn ca3022_bandwidth_coefficient(cutoff: f32) -> f32 {
    // Convert normalized cutoff to mix factor
    // 0.0 = no filtering (full bandwidth), 1.0 = maximum smoothing
    return clamp(cutoff, 0.0, 0.95);
}

// --- Slew rate limiting ---
// Limits the rate of change between consecutive samples.
// Models the CA3022's ~10 V/us slew rate on fast transitions.
//
// current: current sample value
// previous: previous sample value
// max_rate: maximum change per sample (normalized, ~0.01-0.5)
fn ca3022_slew_limit(current: f32, previous: f32, max_rate: f32) -> f32 {
    let delta = current - previous;
    let limited_delta = clamp(delta, -max_rate, max_rate);
    return previous + limited_delta;
}

// --- Soft saturation ---
// Models supply rail clipping using tanh() for natural compression.
// Real CA3022 clips softly as signal approaches supply rails.
//
// input: signal to saturate
// headroom: normalized headroom before clipping (0.5 = lots of room, 0.1 = heavy clipping)
fn ca3022_saturate(input: f32, headroom: f32) -> f32 {
    let h = max(headroom, 0.05);
    return tanh(input / h) * h;
}

// --- Noise floor ---
// Adds thermal noise scaled by signal level.
// Uses a deterministic pseudo-random based on position for GPU compatibility.
//
// signal: input signal level (noise scales with signal for shot noise component)
// seed: pseudo-random seed (typically derived from UV coordinates)
// amount: noise amplitude (0.0 = clean, 0.05 = subtle analog, 0.2 = noisy)
fn ca3022_noise(signal: f32, seed: f32, amount: f32) -> f32 {
    // Simple GPU-friendly hash for noise generation
    let n = fract(sin(seed * 12.9898 + signal * 78.233) * 43758.5453);
    // Mix thermal (constant) and shot (signal-dependent) noise
    let thermal = (n - 0.5) * 2.0;
    let shot = thermal * sqrt(max(abs(signal), 0.001));
    return signal + (thermal * 0.6 + shot * 0.4) * amount;
}

// --- Full CA3022 pipeline ---
// Applies all stages in sequence for a complete video amplifier model.
fn ca3022_process(input: f32, gain: f32, bias: f32, headroom: f32) -> f32 {
    var sig = ca3022_gain_stage(input, gain, bias);
    sig = ca3022_saturate(sig, headroom);
    return sig;
}
