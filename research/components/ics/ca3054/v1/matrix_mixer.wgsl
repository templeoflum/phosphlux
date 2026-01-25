// CA3054 Matrix Mixer - Paik/Abe 7-Channel to RGB
// Based on schematic page 8 - Matrix Mixing Amp
//
// The Paik/Abe synthesizer takes 7 video inputs and creates RGB output
// via a weighted mixing matrix. The CA3054 differential amplifiers
// provide the weighted summing with soft limiting.
//
// In this demo, we derive 7 "channels" from the input:
//   Ch1: Luminance
//   Ch2: Red component
//   Ch3: Green component
//   Ch4: Blue component
//   Ch5: Inverted luminance
//   Ch6: Edge detection (high-pass)
//   Ch7: Colorized luminance (from PED control)
//
// Workbench Parameters:
//   gain   → R channel intensity
//   offset → G channel intensity
//   mix    → B channel intensity
//   fold   → Matrix rotation angle
//   bias   → Edge detection amount
//   frequency → Color phase shift

struct Params {
    gain: f32,      // R intensity
    offset: f32,    // G intensity
    mix: f32,       // B intensity
    fold: f32,      // Matrix rotation
    bias: f32,      // Edge amount
    frequency: f32, // Color phase
    phase: f32,     // unused
    threshold: f32, // unused
    _pad0: f32, _pad1: f32, _pad2: f32, _pad3: f32,
    _pad4: f32, _pad5: f32, _pad6: f32, _pad7: f32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;
@group(0) @binding(2) var<uniform> params: Params;

// ============================================================================
// CA3054 DIFFERENTIAL AMPLIFIER FUNCTIONS
// ============================================================================

const VT: f32 = 0.026;

/// Differential amplifier transfer function
fn diff_amp(v_plus: f32, v_minus: f32, tail_current: f32, gain: f32) -> f32 {
    let v_diff = v_plus - v_minus;
    let normalized_diff = v_diff / (2.0 * VT * 10.0);
    let diff_output = tanh(normalized_diff) * tail_current;
    return clamp(0.5 + diff_output * gain, 0.0, 1.0);
}

/// Matrix row: sum weighted inputs with soft limiting
fn matrix_row(ch: array<f32, 7>, w: array<f32, 7>) -> f32 {
    var sum: f32 = 0.0;
    for (var i: u32 = 0u; i < 7u; i = i + 1u) {
        sum += (ch[i] - 0.5) * w[i];
    }
    // Soft limiting via tanh (models CA3054 characteristic)
    let limited = tanh(sum * 2.0) * 0.5;
    return clamp(limited + 0.5, 0.0, 1.0);
}

// ============================================================================
// CHANNEL EXTRACTION
// ============================================================================

/// Extract luminance from RGB
fn luminance(rgb: vec3<f32>) -> f32 {
    return dot(rgb, vec3<f32>(0.299, 0.587, 0.114));
}

/// Simple edge detection (Sobel-like)
fn edge_detect(uv: vec2<f32>, strength: f32) -> f32 {
    let tex_size = vec2<f32>(textureDimensions(input_texture));
    let pixel = 1.0 / tex_size;

    // Sample neighbors
    let c = luminance(textureSample(input_texture, input_sampler, uv).rgb);
    let l = luminance(textureSample(input_texture, input_sampler, uv - vec2<f32>(pixel.x, 0.0)).rgb);
    let r = luminance(textureSample(input_texture, input_sampler, uv + vec2<f32>(pixel.x, 0.0)).rgb);
    let u = luminance(textureSample(input_texture, input_sampler, uv - vec2<f32>(0.0, pixel.y)).rgb);
    let d = luminance(textureSample(input_texture, input_sampler, uv + vec2<f32>(0.0, pixel.y)).rgb);

    // Gradient magnitude
    let gx = r - l;
    let gy = d - u;
    let edge = sqrt(gx * gx + gy * gy);

    return clamp(edge * strength, 0.0, 1.0);
}

/// Simple colorizer (amplitude windows)
fn colorize_luma(luma: f32) -> f32 {
    // Create bands
    let band = floor(luma * 4.0) / 4.0;
    return band;
}

// ============================================================================
// MAIN SHADER
// ============================================================================

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample input
    let input_color = textureSample(input_texture, input_sampler, in.uv);
    let rgb = input_color.rgb;

    // Extract 7 channels
    var channels: array<f32, 7>;
    channels[0] = luminance(rgb);           // Ch1: Luminance
    channels[1] = rgb.r;                     // Ch2: Red
    channels[2] = rgb.g;                     // Ch3: Green
    channels[3] = rgb.b;                     // Ch4: Blue
    channels[4] = 1.0 - luminance(rgb);     // Ch5: Inverted luma
    channels[5] = edge_detect(in.uv, select(params.bias, 3.0, params.bias == 0.0)); // Ch6: Edges
    channels[6] = colorize_luma(luminance(rgb)); // Ch7: Colorized

    // Get params with defaults
    let r_int = select(params.gain, 1.0, params.gain == 0.0);
    let g_int = select(params.offset, 1.0, params.offset == 0.0);
    let b_int = select(params.mix, 1.0, params.mix == 0.0);
    let rotation = select(params.fold, 0.0, params.fold == 0.0) * 3.14159;
    let phase = select(params.frequency, 0.0, params.frequency == 0.0) * 3.14159 * 2.0;

    // Build mixing matrix with rotation
    let cos_r = cos(rotation);
    let sin_r = sin(rotation);

    // R channel weights: primarily from Ch2 (red) and Ch1 (luma)
    var r_weights: array<f32, 7>;
    r_weights[0] = 0.3 * cos_r * r_int;      // Luma
    r_weights[1] = 0.7 * r_int;               // Red
    r_weights[2] = -0.2 * sin_r * r_int;      // Green (subtract)
    r_weights[3] = 0.0;                        // Blue
    r_weights[4] = 0.1;                        // Inv luma
    r_weights[5] = 0.3;                        // Edges
    r_weights[6] = 0.2 * sin(phase);          // Colorized

    // G channel weights: primarily from Ch3 (green) and Ch1 (luma)
    var g_weights: array<f32, 7>;
    g_weights[0] = 0.3 * cos_r * g_int;
    g_weights[1] = -0.2 * sin_r * g_int;
    g_weights[2] = 0.7 * g_int;
    g_weights[3] = 0.0;
    g_weights[4] = 0.1;
    g_weights[5] = 0.2;
    g_weights[6] = 0.2 * sin(phase + 2.094);  // +120 degrees

    // B channel weights: primarily from Ch4 (blue) and Ch1 (luma)
    var b_weights: array<f32, 7>;
    b_weights[0] = 0.3 * cos_r * b_int;
    b_weights[1] = 0.0;
    b_weights[2] = -0.2 * sin_r * b_int;
    b_weights[3] = 0.7 * b_int;
    b_weights[4] = 0.1;
    b_weights[5] = 0.2;
    b_weights[6] = 0.2 * sin(phase + 4.189);  // +240 degrees

    // Apply matrix mixing
    let r_out = matrix_row(channels, r_weights);
    let g_out = matrix_row(channels, g_weights);
    let b_out = matrix_row(channels, b_weights);

    return vec4<f32>(r_out, g_out, b_out, 1.0);
}

// ============================================================================
// VERTEX SHADER - Fullscreen Triangle
// ============================================================================

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var out: VertexOutput;

    let x = f32((vi & 1u) << 2u) - 1.0;
    let y = f32((vi & 2u) << 1u) - 1.0;

    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);

    return out;
}
