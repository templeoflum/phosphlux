// LM311 Comparator Model
// Generated from spec

// Hard comparator: output high when V+ > V-
fn lm311_compare(v_plus: f32, v_minus: f32) -> f32 {
    return select(0.0, 1.0, v_plus > v_minus);
}

// Soft comparator with adjustable sharpness
fn lm311_compare_soft(v_plus: f32, v_minus: f32, sharpness: f32) -> f32 {
    let diff = v_plus - v_minus;
    return clamp(diff * sharpness + 0.5, 0.0, 1.0);
}

// Window comparator: output high when signal is between thresholds
fn lm311_window(signal: f32, low: f32, high: f32) -> f32 {
    let above_low = select(0.0, 1.0, signal > low);
    let below_high = select(0.0, 1.0, signal < high);
    return above_low * below_high;
}
