// ============================================
// PAIK/ABE INPUT MATRIX
// ============================================
//
// 7-channel to RGB matrix mixer based on CA3054 differential amplifiers.
// This is the input stage of the Paik/Abe Video Synthesizer.
//
// The real hardware takes 7 video inputs and creates weighted RGB outputs.
// In this emulation, we derive the 7 channels from a single input:
//   Ch1: Luminance
//   Ch2: Red component
//   Ch3: Green component
//   Ch4: Blue component
//   Ch5: Inverted luminance
//   Ch6: Luminance squared (creates contrast)
//   Ch7: Mid-gray offset
//
// The matrix mixing uses CA3054-style soft limiting for authentic behavior.
//
// Reference: research/components/ics/ca3054/v1/matrix_mixer.wgsl
//
// Parameters:
// - r_mix: Red channel output gain (0.0-2.0, default 1.0)
// - g_mix: Green channel output gain (0.0-2.0, default 1.0)
// - b_mix: Blue channel output gain (0.0-2.0, default 1.0)
// - rotation: Matrix rotation angle (0.0-1.0, default 0.0) - rotates color space

// CA3054 thermal voltage for soft limiting
const INPUT_MATRIX_VT: f32 = 0.026;

// Matrix row: sum weighted inputs with CA3054-style soft limiting
fn input_matrix_row(ch: array<f32, 7>, w: array<f32, 7>) -> f32 {
    var sum: f32 = 0.0;
    for (var i: u32 = 0u; i < 7u; i = i + 1u) {
        sum += (ch[i] - 0.5) * w[i];
    }
    // Soft limiting via tanh (models CA3054 differential amp characteristic)
    let limited = tanh(sum * 2.0) * 0.5;
    return clamp(limited + 0.5, 0.0, 1.0);
}

// Extract luminance from RGB (Rec. 601)
fn input_matrix_luma(rgb: vec3<f32>) -> f32 {
    return dot(rgb, vec3<f32>(0.299, 0.587, 0.114));
}

// Main input matrix processing function
fn device_paik_abe_input_matrix_process(
    uv: vec2<f32>,
    time: f32,
    video_in: vec4<f32>,
    r_mix: f32,
    g_mix: f32,
    b_mix: f32,
    rotation: f32
) -> vec4<f32> {
    let rgb = video_in.rgb;
    let luma = input_matrix_luma(rgb);

    // Extract 7 channels from input
    var channels: array<f32, 7>;
    channels[0] = luma;                      // Ch1: Luminance
    channels[1] = rgb.r;                     // Ch2: Red
    channels[2] = rgb.g;                     // Ch3: Green
    channels[3] = rgb.b;                     // Ch4: Blue
    channels[4] = 1.0 - luma;               // Ch5: Inverted luma
    channels[5] = luma * luma;              // Ch6: Luma squared (contrast)
    channels[6] = 0.5;                       // Ch7: Mid-gray offset

    // Rotation angle (0-1 maps to 0-2π)
    let rot = rotation * 6.283185;
    let cos_r = cos(rot);
    let sin_r = sin(rot);

    // R channel weights: primarily from luma and red
    var r_weights: array<f32, 7>;
    r_weights[0] = 0.4 * cos_r * r_mix;     // Luma
    r_weights[1] = 0.6 * r_mix;              // Red
    r_weights[2] = -0.2 * sin_r * r_mix;    // Green (subtract with rotation)
    r_weights[3] = 0.0;                      // Blue
    r_weights[4] = 0.1;                      // Inv luma
    r_weights[5] = 0.2;                      // Luma^2 (contrast)
    r_weights[6] = 0.0;                      // Offset

    // G channel weights: primarily from luma and green
    var g_weights: array<f32, 7>;
    g_weights[0] = 0.4 * cos_r * g_mix;
    g_weights[1] = -0.2 * sin_r * g_mix;
    g_weights[2] = 0.6 * g_mix;
    g_weights[3] = 0.0;
    g_weights[4] = 0.1;
    g_weights[5] = 0.2;
    g_weights[6] = 0.0;

    // B channel weights: primarily from luma and blue
    var b_weights: array<f32, 7>;
    b_weights[0] = 0.4 * cos_r * b_mix;
    b_weights[1] = 0.0;
    b_weights[2] = -0.2 * sin_r * b_mix;
    b_weights[3] = 0.6 * b_mix;
    b_weights[4] = 0.1;
    b_weights[5] = 0.2;
    b_weights[6] = 0.0;

    // Apply matrix mixing
    let r_out = input_matrix_row(channels, r_weights);
    let g_out = input_matrix_row(channels, g_weights);
    let b_out = input_matrix_row(channels, b_weights);

    return vec4<f32>(r_out, g_out, b_out, 1.0);
}
