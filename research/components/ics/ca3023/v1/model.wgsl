// CA3023 High-Gain Wideband Amplifier Model
// Two-stage broadband amplifier from RCA CA3020 family
//
// This models the CA3023's key behaviors:
// - Two cascaded gain stages (30-50 dB total)
// - Bandpass frequency response peaked around NTSC 3.58 MHz subcarrier
// - Phase shift control for color angle adjustment
// - Black level clamping / DC restoration
// - Soft saturation at high drive levels
//
// In the Paik/Abe synthesizer, the CA3023 serves as the color subcarrier
// amplifier (schematic pages 3, 4, 26, 30), handling 3.58 MHz NTSC
// subcarrier amplification, clamp pulse generation, and phase control.

const PI: f32 = 3.14159265359;

// --- Two-stage cascaded amplifier ---
// Models the CA3023's two internal gain stages in series.
// Each stage has independent gain and both contribute to soft saturation.
//
// input: signal to amplify
// gain1: first stage gain (typically 10-30)
// gain2: second stage gain (typically 5-20)
fn ca3023_two_stage_amp(input: f32, gain1: f32, gain2: f32) -> f32 {
    // Stage 1: high-impedance input, moderate gain
    let stage1 = tanh(input * gain1 / 1.5) * 1.5;
    // Stage 2: drives output with additional gain
    let stage2 = tanh(stage1 * gain2 / 2.0) * 2.0;
    return stage2;
}

// --- Bandpass frequency response ---
// Approximates the CA3023's peaked frequency response using spatial filtering.
// The bandpass characteristic emphasizes edges (high spatial frequency content)
// while attenuating very low and very high frequencies.
//
// In the frequency domain, this is a resonant peak around the center frequency.
// In the spatial domain, we approximate this with a difference-of-gaussians approach:
// edge emphasis (highpass) combined with blur (lowpass) = bandpass.
//
// input: current pixel value
// neighbor_avg: average of surrounding pixels (lowpass component)
// q: quality factor (sharpness of bandpass peak, 1-20)
fn ca3023_bandpass(input: f32, neighbor_avg: f32, q: f32) -> f32 {
    // Highpass: subtract lowpass from signal
    let highpass = input - neighbor_avg;
    // Bandpass: mix of original and edge-emphasized signal
    // Higher Q = more edge emphasis = tighter bandpass
    let emphasis = clamp(q / 10.0, 0.1, 2.0);
    return input + highpass * emphasis;
}

// --- Phase shift / color rotation ---
// Rotates the chrominance signal by a specified angle.
// For NTSC, the color subcarrier phase angle determines hue.
// This operates on a 2-component signal (I/Q or two chroma axes).
//
// input: signal value (one axis of color space)
// input_quad: quadrature signal (90-degree shifted component)
// angle: phase rotation in radians
fn ca3023_phase_shift(input: f32, input_quad: f32, angle: f32) -> f32 {
    return input * cos(angle) - input_quad * sin(angle);
}

// --- Phase shift returning both components ---
fn ca3023_phase_shift_iq(i: f32, q: f32, angle: f32) -> vec2<f32> {
    let cos_a = cos(angle);
    let sin_a = sin(angle);
    return vec2<f32>(
        i * cos_a - q * sin_a,
        i * sin_a + q * cos_a
    );
}

// --- Black level clamp / DC restoration ---
// Restores DC reference (black level) that may have been lost through
// AC-coupled stages. Clamps the signal minimum to the specified level.
//
// input: signal to clamp
// level: target black level (0.0 = blanking, typically 0.0-0.1)
fn ca3023_clamp(input: f32, level: f32) -> f32 {
    return max(input, level);
}

// --- Saturation with drive control ---
// Higher drive pushes the two-stage amp into nonlinear territory,
// creating harmonic distortion characteristic of overdriven subcarrier amps.
//
// input: signal to saturate
// drive: how hard the amp is driven (0.5 = linear, 2.0 = heavy saturation)
fn ca3023_drive_saturate(input: f32, drive: f32) -> f32 {
    let d = max(drive, 0.1);
    return tanh(input * d) / tanh(d);
}

// --- Full CA3023 pipeline ---
// Applies all stages for a complete subcarrier amplifier model.
fn ca3023_process(input: f32, gain1: f32, gain2: f32, clamp_level: f32) -> f32 {
    var sig = ca3023_two_stage_amp(input, gain1, gain2);
    sig = ca3023_clamp(sig, clamp_level);
    return sig;
}
