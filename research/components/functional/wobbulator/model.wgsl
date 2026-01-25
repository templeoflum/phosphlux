// Wobbulator Model - CRT Scan Displacement
// Paik/Abe Video Synthesizer scan modulation effect
//
// The Wobbulator modulates the horizontal and vertical scan position,
// creating warping, rippling, and displacement effects characteristic
// of early video art.
//
// Historical: Nam June Paik originally created this effect by placing
// magnets on CRT screens. Shuya Abe refined it with electronic oscillators
// connected to the deflection coils.

// ============================================================================
// OSCILLATOR WAVEFORMS
// ============================================================================

const PI: f32 = 3.14159265359;
const TAU: f32 = 6.28318530718;

/// Sine wave oscillator
fn osc_sine(phase: f32) -> f32 {
    return sin(phase * TAU);
}

/// Triangle wave oscillator
fn osc_triangle(phase: f32) -> f32 {
    let p = fract(phase);
    return abs(p * 4.0 - 2.0) - 1.0;
}

/// Sawtooth wave oscillator
fn osc_sawtooth(phase: f32) -> f32 {
    return fract(phase) * 2.0 - 1.0;
}

/// Square wave oscillator
fn osc_square(phase: f32) -> f32 {
    return select(-1.0, 1.0, fract(phase) < 0.5);
}

/// Selectable waveform oscillator
/// waveform: 0=sine, 1=triangle, 2=sawtooth, 3=square
fn oscillator(phase: f32, waveform: u32) -> f32 {
    switch(waveform) {
        case 0u: { return osc_sine(phase); }
        case 1u: { return osc_triangle(phase); }
        case 2u: { return osc_sawtooth(phase); }
        case 3u: { return osc_square(phase); }
        default: { return osc_sine(phase); }
    }
}

// ============================================================================
// DISPLACEMENT FUNCTIONS
// ============================================================================

/// Basic horizontal displacement
/// Displaces X position based on Y coordinate (creates vertical bands)
fn displace_h(uv: vec2<f32>, frequency: f32, amplitude: f32, phase: f32) -> f32 {
    let mod_phase = uv.y * frequency + phase;
    return osc_sine(mod_phase) * amplitude;
}

/// Basic vertical displacement
/// Displaces Y position based on X coordinate (creates horizontal bands)
fn displace_v(uv: vec2<f32>, frequency: f32, amplitude: f32, phase: f32) -> f32 {
    let mod_phase = uv.x * frequency + phase;
    return osc_sine(mod_phase) * amplitude;
}

/// Combined H+V displacement with selectable waveforms
fn displace_hv(
    uv: vec2<f32>,
    h_freq: f32, h_amp: f32, h_phase: f32, h_wave: u32,
    v_freq: f32, v_amp: f32, v_phase: f32, v_wave: u32
) -> vec2<f32> {
    let h_mod = oscillator(uv.y * h_freq + h_phase, h_wave) * h_amp;
    let v_mod = oscillator(uv.x * v_freq + v_phase, v_wave) * v_amp;
    return vec2<f32>(h_mod, v_mod);
}

/// Video-driven displacement (luminance modulates position)
fn displace_video(luminance: f32, h_amount: f32, v_amount: f32) -> vec2<f32> {
    // Center around 0.5 so mid-gray = no displacement
    let luma_centered = luminance - 0.5;
    return vec2<f32>(luma_centered * h_amount, luma_centered * v_amount);
}

// ============================================================================
// WOBBULATOR PRESETS
// ============================================================================

/// Classic Paik wobble - slow sine modulation
fn wobble_paik(uv: vec2<f32>, time: f32, intensity: f32) -> vec2<f32> {
    let h_disp = sin(uv.y * 3.0 + time * 0.5) * intensity * 0.1;
    let v_disp = sin(uv.x * 2.0 + time * 0.3) * intensity * 0.08;
    return vec2<f32>(h_disp, v_disp);
}

/// Ripple effect - circular displacement from center
fn wobble_ripple(uv: vec2<f32>, time: f32, frequency: f32, amplitude: f32) -> vec2<f32> {
    let center = vec2<f32>(0.5, 0.5);
    let delta = uv - center;
    let dist = length(delta);
    let angle = atan2(delta.y, delta.x);

    let ripple = sin(dist * frequency * TAU - time) * amplitude;

    return vec2<f32>(cos(angle) * ripple, sin(angle) * ripple);
}

/// Barrel distortion (CRT-like)
fn wobble_barrel(uv: vec2<f32>, amount: f32) -> vec2<f32> {
    let center = vec2<f32>(0.5, 0.5);
    let delta = uv - center;
    let dist_sq = dot(delta, delta);

    // Barrel: push outward at edges
    let distortion = 1.0 + dist_sq * amount;

    return delta * (distortion - 1.0);
}

/// Pincushion distortion (inverse barrel)
fn wobble_pincushion(uv: vec2<f32>, amount: f32) -> vec2<f32> {
    return wobble_barrel(uv, -amount);
}

/// S-curve horizontal distortion
fn wobble_s_curve(uv: vec2<f32>, amount: f32) -> vec2<f32> {
    let y_centered = uv.y - 0.5;
    let h_disp = sin(y_centered * PI) * amount;
    return vec2<f32>(h_disp, 0.0);
}

// ============================================================================
// FULL WOBBULATOR SYSTEM
// ============================================================================

/// Complete wobbulator with all modulation options
struct WobbulatorParams {
    // Horizontal modulation
    h_frequency: f32,
    h_amplitude: f32,
    h_phase: f32,
    h_waveform: u32,

    // Vertical modulation
    v_frequency: f32,
    v_amplitude: f32,
    v_phase: f32,
    v_waveform: u32,

    // Video modulation
    video_h: f32,
    video_v: f32,

    // Time for animation
    time: f32,
}

/// Apply full wobbulator displacement
fn wobbulator(uv: vec2<f32>, luminance: f32, params: WobbulatorParams) -> vec2<f32> {
    // Oscillator-based displacement
    let osc_disp = displace_hv(
        uv,
        params.h_frequency, params.h_amplitude, params.h_phase + params.time, params.h_waveform,
        params.v_frequency, params.v_amplitude, params.v_phase + params.time * 0.7, params.v_waveform
    );

    // Video-driven displacement
    let video_disp = displace_video(luminance, params.video_h, params.video_v);

    // Combine
    return osc_disp + video_disp;
}

/// Apply displacement to UV coordinates with edge handling
fn apply_displacement(uv: vec2<f32>, displacement: vec2<f32>, edge_mode: u32) -> vec2<f32> {
    let new_uv = uv + displacement;

    switch(edge_mode) {
        // Clamp to edges
        case 0u: { return clamp(new_uv, vec2<f32>(0.0), vec2<f32>(1.0)); }
        // Wrap around
        case 1u: { return fract(new_uv); }
        // Mirror
        case 2u: {
            let wrapped = fract(new_uv);
            let flip = vec2<u32>(u32(new_uv.x) & 1u, u32(new_uv.y) & 1u);
            return vec2<f32>(
                select(wrapped.x, 1.0 - wrapped.x, flip.x == 1u),
                select(wrapped.y, 1.0 - wrapped.y, flip.y == 1u)
            );
        }
        default: { return new_uv; }
    }
}
