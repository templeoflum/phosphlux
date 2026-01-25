// ============================================
// PAIK/ABE COLORIZER
// ============================================
//
// Authentic emulation of the colorizer section from the
// Paik/Abe Video Synthesizer (1969-1970).
//
// Circuit characteristics:
// - Uses LM741 op-amps (0.5 V/µs slew rate)
// - Triangle wave folding creates harmonics
// - Each RGB channel has independent gain
// - Saturation from op-amp rail limiting
//
// The "Paik/Abe look" comes from:
// 1. Slow slew rate softening transients
// 2. Asymmetric folding from component tolerances
// 3. Warm color shifts from thermal drift

// Device parameters (bound by shader composer)
// params.paik_abe_colorizer_X_gain_r
// params.paik_abe_colorizer_X_gain_g
// params.paik_abe_colorizer_X_gain_b
// params.paik_abe_colorizer_X_fold_depth
// params.paik_abe_colorizer_X_saturation
// params.paik_abe_colorizer_X_slew

// Op-amp saturation with LM741 characteristics
fn paik_abe_opamp_sat(v_in: f32, slew_factor: f32) -> f32 {
    // LM741 can only swing to within ~2V of rails on ±15V supply
    // Normalized: saturates at ~0.87 of full range
    let v_sat = 0.87;

    // Soft saturation curve (tanh-like)
    let gain = 3.0; // How sharp the knee is
    let normalized = v_in * 2.0 - 1.0; // Center around 0
    var saturated = tanh(normalized * gain) / tanh(gain);

    // Add slew rate limiting effect
    // Higher slew_factor = more "741 character"
    let slew_smooth = 1.0 - slew_factor * 0.3;
    saturated = saturated * slew_smooth + normalized * (1.0 - slew_smooth);

    return (saturated + 1.0) * 0.5 * v_sat + (1.0 - v_sat) * 0.5;
}

// Triangle wave folding (the core Paik/Abe effect)
fn paik_abe_fold(x: f32, gain: f32, depth: f32) -> f32 {
    var v = x * gain;

    // Multi-stage folding for richer harmonics
    for (var i = 0u; i < u32(depth); i = i + 1u) {
        // Triangle fold
        v = abs(fract(v * 0.5 + 0.25) * 2.0 - 1.0);

        // Slight asymmetry (simulates component tolerance)
        v = v * 0.98 + 0.01;
    }

    return v;
}

// Main colorizer processing
fn device_paik_abe_colorizer_process(
    uv: vec2<f32>,
    time: f32,
    video_in: vec4<f32>
) -> vec4<f32> {
    // Extract luminance from input
    let luma = to_luma(video_in.rgb);

    // Default parameters (will be overridden by uniform when bound)
    let gain_r = 2.0;
    let gain_g = 2.5;
    let gain_b = 3.0;
    let fold_depth = 2.0;
    let saturation = 1.0;
    let slew = 0.3;

    // Process each channel through folding stages
    var r = paik_abe_fold(luma, gain_r, fold_depth);
    var g = paik_abe_fold(luma, gain_g, fold_depth);
    var b = paik_abe_fold(luma, gain_b, fold_depth);

    // Apply op-amp saturation (the "warmth")
    r = paik_abe_opamp_sat(r, slew);
    g = paik_abe_opamp_sat(g, slew);
    b = paik_abe_opamp_sat(b, slew);

    // Saturation adjustment (mix with luma)
    let output_luma = (r + g + b) / 3.0;
    r = mix(output_luma, r, saturation);
    g = mix(output_luma, g, saturation);
    b = mix(output_luma, b, saturation);

    // Slight warmth bias (simulates thermal drift)
    let warmth = 0.02;
    r = r + warmth;
    b = b - warmth * 0.5;

    return vec4<f32>(video_clamp3(vec3<f32>(r, g, b)), 1.0);
}

// Variant with explicit parameter passing (for workbench testing)
fn paik_abe_colorizer_with_params(
    uv: vec2<f32>,
    time: f32,
    video_in: vec4<f32>,
    gain_r: f32,
    gain_g: f32,
    gain_b: f32,
    fold_depth: f32,
    saturation: f32,
    slew: f32
) -> vec4<f32> {
    let luma = to_luma(video_in.rgb);

    var r = paik_abe_fold(luma, gain_r, fold_depth);
    var g = paik_abe_fold(luma, gain_g, fold_depth);
    var b = paik_abe_fold(luma, gain_b, fold_depth);

    r = paik_abe_opamp_sat(r, slew);
    g = paik_abe_opamp_sat(g, slew);
    b = paik_abe_opamp_sat(b, slew);

    let output_luma = (r + g + b) / 3.0;
    r = mix(output_luma, r, saturation);
    g = mix(output_luma, g, saturation);
    b = mix(output_luma, b, saturation);

    let warmth = 0.02;
    r = r + warmth;
    b = b - warmth * 0.5;

    return vec4<f32>(video_clamp3(vec3<f32>(r, g, b)), 1.0);
}
