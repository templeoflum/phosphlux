// ============================================
// PAIK/ABE OUTPUT ENCODER
// ============================================
//
// Final output stage of the Paik/Abe Video Synthesizer.
// This stage provides the characteristic "warmth" and analog feel
// through op-amp saturation, slew limiting, and thermal drift.
//
// The Paik/Abe used LM741 op-amps which have:
// - Limited slew rate (0.5 V/µs) - softens transients
// - Rail-limited output (~87% of supply)
// - Slight thermal drift - adds organic warmth
//
// Circuit behavior:
// - Op-amp saturation at extremes
// - Slew rate limiting smooths harsh edges
// - Thermal bias shifts color temperature slightly warm
// - Saturation control adjusts color intensity
//
// Reference: Extracted from colorizer.wgsl op-amp modeling
//
// Parameters:
// - saturation: Color saturation (0.0-2.0, default 1.0)
// - slew: LM741 slew rate character (0.0-1.0, default 0.3)
// - warmth: Thermal drift warmth bias (0.0-1.0, default 0.5)

// Op-amp saturation with LM741 characteristics
fn output_opamp_saturation(v_in: f32, slew_factor: f32) -> f32 {
    // LM741 can only swing to within ~2V of rails on ±15V supply
    // Normalized: saturates at ~0.87 of full range
    let v_sat = 0.87;

    // Soft saturation curve (tanh-like)
    let gain = 3.0;
    let normalized = v_in * 2.0 - 1.0;
    var saturated = tanh(normalized * gain) / tanh(gain);

    // Add slew rate limiting effect
    // Higher slew_factor = more "741 character"
    let slew_smooth = 1.0 - slew_factor * 0.3;
    saturated = saturated * slew_smooth + normalized * (1.0 - slew_smooth);

    return (saturated + 1.0) * 0.5 * v_sat + (1.0 - v_sat) * 0.5;
}

// Main output encoder processing function
fn device_paik_abe_output_encoder_process(
    uv: vec2<f32>,
    time: f32,
    video_in: vec4<f32>,
    saturation: f32,
    slew: f32,
    warmth: f32
) -> vec4<f32> {
    var rgb = video_in.rgb;

    // Apply op-amp saturation to each channel
    var r = output_opamp_saturation(rgb.r, slew);
    var g = output_opamp_saturation(rgb.g, slew);
    var b = output_opamp_saturation(rgb.b, slew);

    // Saturation adjustment (mix with luminance)
    let output_luma = dot(vec3<f32>(r, g, b), vec3<f32>(0.299, 0.587, 0.114));
    r = mix(output_luma, r, saturation);
    g = mix(output_luma, g, saturation);
    b = mix(output_luma, b, saturation);

    // Thermal drift warmth bias
    // warmth 0.5 is neutral, <0.5 is cool, >0.5 is warm
    let warmth_amount = (warmth - 0.5) * 0.1;
    r = r + warmth_amount;
    b = b - warmth_amount * 0.5;

    return vec4<f32>(clamp(vec3<f32>(r, g, b), vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
