// ============================================
// PAIK/ABE PED COLORIZER
// ============================================
//
// Amplitude window colorizer based on CA3019 matched diode array.
// This is the core of the Paik/Abe color generation - it maps luminance
// levels to specific colors based on amplitude windows (PED = pedestal).
//
// The CA3019 diode array provides precise amplitude detection:
// - Window 0: Black to threshold 1 (shadows)
// - Window 1: Threshold 1 to threshold 2 (low-mids)
// - Window 2: Threshold 2 to threshold 3 (high-mids)
// - Window 3: Threshold 3 to white (highlights)
//
// Each window maps to a different color, creating the characteristic
// Paik/Abe posterized psychedelic look.
//
// Reference: research/components/ics/ca3019/v1/ped_control.wgsl
//
// Parameters:
// - threshold_1: Low window boundary (0.0-1.0, default 0.25)
// - threshold_2: Mid window boundary (0.0-1.0, default 0.50)
// - threshold_3: High window boundary (0.0-1.0, default 0.75)
// - softness: Edge softness (0.0-1.0, default 0.25) - 0=hard edges, 1=soft analog
// - clipping: Signal clipping level (0.0-1.0, default 1.0) - CLP pin 8 from schematic

// CA3019 diode characteristics (from datasheet)
// Thermal voltage at room temperature
const PED_DIODE_VT: f32 = 0.026;
// Forward voltage drop
const PED_DIODE_VF: f32 = 0.65;
// Normalized sharpness factor (scales Vt to 0-1 signal range)
// Real diode: ~38.5 (1/Vt), we scale for video signal range
const PED_DIODE_SHARPNESS: f32 = 40.0;

// CA3019 diode threshold detection
// Models the exponential turn-on characteristic of matched diodes
// Returns 0-1 indicating how much signal exceeds threshold
fn ped_ca3019_diode_detect(signal: f32, threshold: f32, softness: f32) -> f32 {
    // Difference from threshold, scaled by softness
    // Lower softness = sharper transition (more like ideal diode)
    let diff = (signal - threshold) / max(softness * 0.1 + 0.01, 0.01);

    // Exponential diode characteristic: I = Is * (exp(V/Vt) - 1)
    // Approximated as sigmoid for numerical stability
    // sharpness derived from 1/Vt characteristic
    let effective_sharpness = PED_DIODE_SHARPNESS * (1.0 - softness * 0.8);
    return 1.0 / (1.0 + exp(-diff * effective_sharpness));
}

// Amplitude window using CA3019 matched diode pair
// One diode detects "above low", another detects "above high"
// Window = (above low) AND NOT (above high)
fn ped_amplitude_window_ca3019(signal: f32, low: f32, high: f32, softness: f32) -> f32 {
    let above_low = ped_ca3019_diode_detect(signal, low, softness);
    let above_high = ped_ca3019_diode_detect(signal, high, softness);
    return above_low * (1.0 - above_high);
}

// Classic Paik/Abe color palette for the 4 amplitude windows
fn ped_paik_abe_palette() -> array<vec3<f32>, 4> {
    var colors: array<vec3<f32>, 4>;
    colors[0] = vec3<f32>(0.1, 0.0, 0.8);   // Deep blue (shadows)
    colors[1] = vec3<f32>(0.9, 0.0, 0.6);   // Magenta (low-mids)
    colors[2] = vec3<f32>(1.0, 0.9, 0.0);   // Yellow (high-mids)
    colors[3] = vec3<f32>(0.0, 0.9, 0.8);   // Cyan (highlights)
    return colors;
}

// PED Control main function
// Maps luminance to colors based on amplitude windows
fn ped_control_colorize(
    luminance: f32,
    t1: f32,
    t2: f32,
    t3: f32,
    colors: array<vec3<f32>, 4>,
    softness: f32
) -> vec3<f32> {
    // Sort thresholds to ensure proper ordering
    var th1 = min(min(t1, t2), t3);
    var th3 = max(max(t1, t2), t3);
    var th2 = t1 + t2 + t3 - th1 - th3;

    // Create 4 amplitude windows using CA3019 matched diode detection
    let w0 = ped_amplitude_window_ca3019(luminance, 0.0, th1, softness);
    let w1 = ped_amplitude_window_ca3019(luminance, th1, th2, softness);
    let w2 = ped_amplitude_window_ca3019(luminance, th2, th3, softness);
    let w3 = ped_amplitude_window_ca3019(luminance, th3, 1.0, softness);

    // Mix colors based on window membership
    var color = vec3<f32>(0.0);
    color = color + colors[0] * w0;
    color = color + colors[1] * w1;
    color = color + colors[2] * w2;
    color = color + colors[3] * w3;

    return color;
}

// Apply clipping to signal (CLP pin 8)
// Models the signal limiting behavior before amplitude detection
fn ped_apply_clipping(signal: f32, clip_level: f32) -> f32 {
    // clip_level: 1.0 = no clipping, 0.0 = clip to 0
    // Clipping limits the maximum signal level, compressing highlights
    let max_level = max(clip_level, 0.01);
    return min(signal, max_level) / max_level;
}

// Main PED colorizer processing function
fn device_paik_abe_ped_colorizer_process(
    uv: vec2<f32>,
    time: f32,
    video_in: vec4<f32>,
    threshold_1: f32,
    threshold_2: f32,
    threshold_3: f32,
    softness: f32,
    clipping: f32
) -> vec4<f32> {
    // Extract luminance from input (Rec. 601)
    let luma_raw = dot(video_in.rgb, vec3<f32>(0.299, 0.587, 0.114));

    // Apply clipping (CLP pin 8) - limits signal before window detection
    let luma = ped_apply_clipping(luma_raw, clipping);

    // Get Paik/Abe color palette
    let colors = ped_paik_abe_palette();

    // Apply PED control colorization
    let colorized = ped_control_colorize(
        luma,
        threshold_1,
        threshold_2,
        threshold_3,
        colors,
        softness
    );

    return vec4<f32>(clamp(colorized, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
