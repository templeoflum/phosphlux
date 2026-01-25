// CA3080 Operational Transconductance Amplifier Model
// Generated from spec

const CA3080_GM_MAX: f32 = 9600.0;  // uS

// OTA: Iout = gm * (V+ - V-)
// gm is controlled by Iabc (amplifier bias current)
fn ca3080_ota(v_plus: f32, v_minus: f32, i_abc: f32) -> f32 {
    let gm = i_abc * CA3080_GM_MAX / 1000.0;  // Scale bias to gm
    let v_diff = v_plus - v_minus;
    // Soft limiting for large signals
    return tanh(v_diff * gm) * gm;
}

// VCA application: voltage controlled amplifier
fn ca3080_vca(signal: f32, cv: f32) -> f32 {
    let gain = clamp(cv, 0.0, 1.0);
    return signal * gain;
}
