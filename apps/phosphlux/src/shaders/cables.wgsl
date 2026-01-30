// ============================================
// CABLE DEGRADATION FUNCTIONS
// ============================================
//
// Models signal degradation introduced by different cable/connector types.
// Each cable type has distinct characteristics based on physical properties.
//
// Specification: docs/CABLE_DEGRADATION_SPEC.md
// Sources: NTSC standard, YIQ color space (Wikipedia), RG-59 attenuation data
//
// Usage: Shader composer inserts appropriate cable function between
// device outputs and inputs based on Connection.cable_type
//
// v0.13 - Research-based implementation

// ─── Color Space Constants ─────────────────────────────────────────
// From 1953 NTSC colorimetry specification

// RGB to YIQ conversion
const RGB_TO_Y: vec3<f32> = vec3<f32>(0.299, 0.587, 0.114);
const RGB_TO_I: vec3<f32> = vec3<f32>(0.596, -0.274, -0.322);
const RGB_TO_Q: vec3<f32> = vec3<f32>(0.211, -0.523, 0.312);

// YIQ to RGB conversion
const YIQ_TO_R: vec3<f32> = vec3<f32>(1.0, 0.956, 0.621);
const YIQ_TO_G: vec3<f32> = vec3<f32>(1.0, -0.272, -0.647);
const YIQ_TO_B: vec3<f32> = vec3<f32>(1.0, -1.106, 1.703);

// RGB to YPbPr conversion (for component video)
const RGB_TO_Y_COMPONENT: vec3<f32> = vec3<f32>(0.299, 0.587, 0.114);
const RGB_TO_PB: vec3<f32> = vec3<f32>(-0.169, -0.331, 0.500);
const RGB_TO_PR: vec3<f32> = vec3<f32>(0.500, -0.419, -0.081);

// YPbPr to RGB conversion
const YPBPR_TO_R: vec3<f32> = vec3<f32>(1.0, 0.0, 1.402);
const YPBPR_TO_G: vec3<f32> = vec3<f32>(1.0, -0.344, -0.714);
const YPBPR_TO_B: vec3<f32> = vec3<f32>(1.0, 1.772, 0.0);

// ─── NTSC Timing Constants ─────────────────────────────────────────
const NTSC_SUBCARRIER_HZ: f32 = 3579545.0;  // 3.579545 MHz (exactly 315/88 MHz)
const NTSC_LINE_RATE_HZ: f32 = 15734.0;     // 15.734 kHz horizontal
const NTSC_FIELD_RATE_HZ: f32 = 59.94;      // 59.94 Hz field rate
const NTSC_CYCLES_PER_LINE: f32 = 227.5;    // Subcarrier cycles per line

// ─── Bandwidth Parameters ──────────────────────────────────────────
// Based on NTSC specification bandwidth limits

// Composite video bandwidth limits
// I channel: 1.3 MHz (vestigial sideband) → normalized blur radius
// Q channel: 0.4 MHz (double sideband) → wider blur
// Composite video degradation constants
// Based on NTSC bandwidth limits: Y=4.2MHz, I=1.3MHz, Q=0.4MHz
const COMPOSITE_BLUR_I: f32 = 0.003;   // I channel blur radius (normalized)
const COMPOSITE_BLUR_Q: f32 = 0.008;   // Q channel blur radius (normalized)
const COMPOSITE_DOT_CRAWL_AMP: f32 = 0.03;  // Dot crawl amplitude

// S-Video: separate Y/C, no crosstalk
// Chroma bandwidth: ~1.5 MHz (both channels)
const SVIDEO_BLUR_CHROMA: f32 = 0.002;

// Component video: wide bandwidth per channel
// Y: ~6 MHz, Pb/Pr: ~3 MHz for SD
const COMPONENT_BLUR_BASE: f32 = 0.0005;
const COMPONENT_LENGTH_FACTOR: f32 = 0.00002;

// RGB: direct signals, cable-length dependent only
const RGB_BLUR_BASE: f32 = 0.0002;
const RGB_LENGTH_FACTOR: f32 = 0.00003;

// ─── Helper: Simple Box Blur ───────────────────────────────────────
// Approximates bandwidth limiting with spatial blur
// In a real implementation, this would sample neighboring pixels
// For now, we simulate the effect on the current pixel's color channels

fn bandwidth_limit_channel(value: f32, blur_amount: f32, uv_offset: f32) -> f32 {
    // Simplified model: attenuate based on blur amount
    // Real implementation would convolve with neighboring pixels
    // This approximation reduces high-frequency content
    let attenuation = 1.0 - blur_amount * 10.0;
    return value * max(attenuation, 0.5);
}

// ─── Composite Video ─────────────────────────────────────────────────
// Single RCA cable carrying combined luma + chroma
//
// Characteristics (from NTSC spec):
// - Subcarrier: 3.579545 MHz
// - Luma bandwidth: ~4.2 MHz
// - I channel bandwidth: 1.3 MHz
// - Q channel bandwidth: 0.4 MHz
// - Chroma/luma crosstalk causes dot crawl artifact
//
// Reference: docs/CABLE_DEGRADATION_SPEC.md Section 3.1

fn composite_degrade(color: vec4<f32>, uv: vec2<f32>, time: f32) -> vec4<f32> {
    // Convert RGB to YIQ using NTSC matrices
    let y = dot(color.rgb, RGB_TO_Y);
    let i = dot(color.rgb, RGB_TO_I);
    let q = dot(color.rgb, RGB_TO_Q);

    // Chroma bandwidth limiting
    // I channel: 1.3 MHz → ~0.003 normalized blur
    // Q channel: 0.4 MHz → ~0.008 normalized blur (Q has narrower bandwidth)
    // The Q channel loses more high-frequency detail than I
    let i_degraded = i * (1.0 - COMPOSITE_BLUR_I * 5.0);  // Slight I loss
    let q_degraded = q * (1.0 - COMPOSITE_BLUR_Q * 5.0);  // More Q loss

    // Dot crawl artifact
    // Occurs at subcarrier frequency (227.5 cycles per line)
    // Visible as moving checkerboard pattern at color transitions
    // Amplitude depends on chroma magnitude (only visible where color changes)
    let chroma_magnitude = sqrt(i * i + q * q);
    let subcarrier_phase = uv.x * NTSC_CYCLES_PER_LINE + time * NTSC_FIELD_RATE_HZ;
    let dot_crawl = sin(subcarrier_phase * 6.283185) * COMPOSITE_DOT_CRAWL_AMP * chroma_magnitude;

    // Convert back to RGB with artifacts
    // The dot crawl adds to luma, creating the visible artifact
    let y_with_crawl = y + dot_crawl;

    let r = dot(vec3<f32>(y_with_crawl, i_degraded, q_degraded), YIQ_TO_R);
    let g = dot(vec3<f32>(y_with_crawl, i_degraded, q_degraded), YIQ_TO_G);
    let b = dot(vec3<f32>(y_with_crawl, i_degraded, q_degraded), YIQ_TO_B);

    return vec4<f32>(clamp(vec3<f32>(r, g, b), vec3<f32>(0.0), vec3<f32>(1.0)), color.a);
}

// ─── S-Video ─────────────────────────────────────────────────────────
// Separate Y (luma) and C (chroma) signals on different wires
//
// Characteristics:
// - NO chroma/luma crosstalk (signals physically separated)
// - NO dot crawl artifact
// - Luma bandwidth: ~5-7 MHz (full detail)
// - Chroma bandwidth: ~1.5 MHz (both I and Q)
//
// Reference: docs/CABLE_DEGRADATION_SPEC.md Section 3.2

fn svideo_degrade(color: vec4<f32>, uv: vec2<f32>, time: f32) -> vec4<f32> {
    // Convert RGB to YIQ
    let y = dot(color.rgb, RGB_TO_Y);
    let i = dot(color.rgb, RGB_TO_I);
    let q = dot(color.rgb, RGB_TO_Q);

    // Luma passes through with minimal degradation
    // Y channel has full ~5-7 MHz bandwidth
    let y_out = y;

    // Chroma bandwidth limiting
    // Both I and Q limited to ~1.5 MHz (equiband mode)
    // Less degradation than composite because:
    // 1. Equal treatment of I and Q
    // 2. Better overall chroma bandwidth
    let i_degraded = i * (1.0 - SVIDEO_BLUR_CHROMA * 3.0);
    let q_degraded = q * (1.0 - SVIDEO_BLUR_CHROMA * 3.0);

    // NO dot crawl - this is the key difference from composite
    // Y and C are on separate wires, so no crosstalk

    // Convert back to RGB
    let r = dot(vec3<f32>(y_out, i_degraded, q_degraded), YIQ_TO_R);
    let g = dot(vec3<f32>(y_out, i_degraded, q_degraded), YIQ_TO_G);
    let b = dot(vec3<f32>(y_out, i_degraded, q_degraded), YIQ_TO_B);

    return vec4<f32>(clamp(vec3<f32>(r, g, b), vec3<f32>(0.0), vec3<f32>(1.0)), color.a);
}

// ─── Component Video ─────────────────────────────────────────────────
// Three separate cables for Y, Pb, Pr
//
// Characteristics:
// - Wide bandwidth per channel (~6 MHz Y, ~3 MHz color difference)
// - No color subcarrier encoding (analog baseband)
// - Supports progressive scan and HD
// - Cable length affects HF rolloff
//
// Reference: docs/CABLE_DEGRADATION_SPEC.md Section 3.3

fn component_degrade(color: vec4<f32>, uv: vec2<f32>, cable_length_ft: f32) -> vec4<f32> {
    // Convert RGB to YPbPr (different from YIQ!)
    let y = dot(color.rgb, RGB_TO_Y_COMPONENT);
    let pb = dot(color.rgb, RGB_TO_PB);
    let pr = dot(color.rgb, RGB_TO_PR);

    // Cable-length dependent HF rolloff
    // Minimal at short lengths, increases with distance
    // RG-59 at video frequencies: ~1 dB/100ft at 5 MHz
    let length_factor = cable_length_ft * COMPONENT_LENGTH_FACTOR;
    let blur_amount = COMPONENT_BLUR_BASE + length_factor;

    // Apply slight bandwidth limiting to all channels
    let y_out = y * (1.0 - blur_amount * 2.0);
    let pb_out = pb * (1.0 - blur_amount * 3.0);  // Color difference slightly more affected
    let pr_out = pr * (1.0 - blur_amount * 3.0);

    // Convert back to RGB
    let r = dot(vec3<f32>(y_out, pb_out, pr_out), YPBPR_TO_R);
    let g = dot(vec3<f32>(y_out, pb_out, pr_out), YPBPR_TO_G);
    let b = dot(vec3<f32>(y_out, pb_out, pr_out), YPBPR_TO_B);

    return vec4<f32>(clamp(vec3<f32>(r, g, b), vec3<f32>(0.0), vec3<f32>(1.0)), color.a);
}

// ─── RGB ─────────────────────────────────────────────────────────────
// Direct RGB signals (VGA, SCART RGB mode)
//
// Characteristics:
// - Best quality (no color space conversion losses)
// - HF rolloff proportional to cable length
// - Equal treatment of all channels
// - Crosstalk negligible except on very long cables
//
// Reference: docs/CABLE_DEGRADATION_SPEC.md Section 3.4

fn rgb_degrade(color: vec4<f32>, uv: vec2<f32>, cable_length_ft: f32) -> vec4<f32> {
    // Cable-length dependent HF rolloff
    // RGB cables don't have color space conversion losses
    // Degradation comes only from cable characteristics
    let length_factor = cable_length_ft * RGB_LENGTH_FACTOR;
    let blur_amount = RGB_BLUR_BASE + length_factor;

    // Apply identical rolloff to all channels
    // This preserves color balance while reducing HF detail
    let attenuation = 1.0 - blur_amount * 2.0;

    let r_out = color.r * attenuation;
    let g_out = color.g * attenuation;
    let b_out = color.b * attenuation;

    // Optional: very slight crosstalk on long cables (>50ft)
    // Typically -40dB = 0.01 amplitude
    var crosstalk = 0.0;
    if (cable_length_ft > 50.0) {
        crosstalk = (cable_length_ft - 50.0) * 0.0001;
    }

    // Crosstalk adds small amount of adjacent channel
    let r_final = r_out + crosstalk * (g_out - r_out);
    let g_final = g_out + crosstalk * (r_out + b_out - 2.0 * g_out) * 0.5;
    let b_final = b_out + crosstalk * (g_out - b_out);

    return vec4<f32>(clamp(vec3<f32>(r_final, g_final, b_final), vec3<f32>(0.0), vec3<f32>(1.0)), color.a);
}

// ─── Ideal (No Degradation) ──────────────────────────────────────────
// Pass-through for testing and comparison

fn ideal_degrade(color: vec4<f32>, uv: vec2<f32>, time: f32) -> vec4<f32> {
    return color;
}
