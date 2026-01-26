// Paik/Abe Complete Signal Chain - Composed 5-Stage Processor
//
// Single-pass shader composing the full Paik/Abe signal path:
//   Input -> [Wobbulator UV] -> Matrix Mixer -> Folding Amp -> PED Colorizer -> Output
//                                                           \-> Video Keyer -/
//
// Wobbulator displaces the input UV (spatial modulation on source).
// Matrix Mixer extracts 7 channels and creates weighted RGB.
// Folding Amp applies nonlinear wavefolding distortion.
// PED Colorizer maps luminance to color bands via amplitude windows.
// Video Keyer blends between folded and colorized output by luminance.
//
// Workbench Parameters:
//   gain      -> Fold amount (wavefolder intensity)
//   offset    -> PED threshold 1
//   mix       -> PED threshold 2
//   fold      -> PED threshold 3
//   bias      -> Wobbulator displacement amount
//   frequency -> Wobbulator oscillator speed
//   phase     -> Matrix rotation angle
//   threshold -> Key clip level (keyer blend point)

struct Params {
    gain: f32,      // Fold amount
    offset: f32,    // PED threshold 1
    mix: f32,       // PED threshold 2
    fold: f32,      // PED threshold 3
    bias: f32,      // Wobbulator amount
    frequency: f32, // Wobbulator speed
    phase: f32,     // Matrix rotation
    threshold: f32, // Key clip
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

const PI: f32 = 3.14159265359;
const TAU: f32 = 6.28318530718;

// ============================================================================
// STAGE 1: WOBBULATOR (UV displacement)
// ============================================================================

fn wobbulate_uv(uv: vec2<f32>, amount: f32, speed: f32) -> vec2<f32> {
    if (amount < 0.001) {
        return uv;
    }

    // Fixed oscillator frequencies for the chain
    let h_freq = 3.0;
    let v_freq = 2.0;
    let time = speed * 10.0;

    let h_phase = uv.y * h_freq + time;
    let h_disp = sin(h_phase * TAU) * amount * 0.1;

    let v_phase = uv.x * v_freq + time * 0.7;
    let v_disp = sin(v_phase * TAU) * amount * 0.08;

    let displaced = uv + vec2<f32>(h_disp, v_disp);

    // Mirror edge mode
    let wrapped = fract(displaced);
    let flip_x = u32(floor(displaced.x)) & 1u;
    let flip_y = u32(floor(displaced.y)) & 1u;
    return vec2<f32>(
        select(wrapped.x, 1.0 - wrapped.x, flip_x == 1u),
        select(wrapped.y, 1.0 - wrapped.y, flip_y == 1u)
    );
}

// ============================================================================
// STAGE 2: MATRIX MIXER (7-channel to RGB)
// ============================================================================

fn luminance(rgb: vec3<f32>) -> f32 {
    return dot(rgb, vec3<f32>(0.299, 0.587, 0.114));
}

fn matrix_row(ch: array<f32, 7>, w: array<f32, 7>) -> f32 {
    var sum: f32 = 0.0;
    for (var i: u32 = 0u; i < 7u; i = i + 1u) {
        sum += (ch[i] - 0.5) * w[i];
    }
    let limited = tanh(sum * 2.0) * 0.5;
    return clamp(limited + 0.5, 0.0, 1.0);
}

fn matrix_mix(rgb: vec3<f32>, uv: vec2<f32>, rotation: f32) -> vec3<f32> {
    let luma = luminance(rgb);

    // Extract 7 channels from input
    var ch: array<f32, 7>;
    ch[0] = luma;                    // Luminance
    ch[1] = rgb.r;                   // Red
    ch[2] = rgb.g;                   // Green
    ch[3] = rgb.b;                   // Blue
    ch[4] = 1.0 - luma;             // Inverted luma
    ch[5] = 0.5;                     // Placeholder (no edge detect in chain)
    ch[6] = floor(luma * 4.0) / 4.0; // Quantized luma

    let cos_r = cos(rotation * PI);
    let sin_r = sin(rotation * PI);

    // R weights
    var rw: array<f32, 7>;
    rw[0] = 0.3 * cos_r; rw[1] = 0.7; rw[2] = -0.2 * sin_r;
    rw[3] = 0.0; rw[4] = 0.1; rw[5] = 0.0; rw[6] = 0.15;

    // G weights
    var gw: array<f32, 7>;
    gw[0] = 0.3 * cos_r; gw[1] = -0.2 * sin_r; gw[2] = 0.7;
    gw[3] = 0.0; gw[4] = 0.1; gw[5] = 0.0; gw[6] = 0.15;

    // B weights
    var bw: array<f32, 7>;
    bw[0] = 0.3 * cos_r; bw[1] = 0.0; bw[2] = -0.2 * sin_r;
    bw[3] = 0.7; bw[4] = 0.1; bw[5] = 0.0; bw[6] = 0.15;

    return vec3<f32>(
        matrix_row(ch, rw),
        matrix_row(ch, gw),
        matrix_row(ch, bw)
    );
}

// ============================================================================
// STAGE 3: FOLDING AMPLIFIER
// ============================================================================

fn fold_triangle(x: f32) -> f32 {
    let saw = fract(x);
    return abs(saw * 2.0 - 1.0);
}

fn fold_paik_abe(x: f32, gain: f32) -> f32 {
    let amplified = x * gain;
    var sig = fold_triangle(amplified);
    // Soft saturation with transistor-like warmth
    sig = tanh(sig * 1.7) / tanh(1.7);
    // Second fold for harmonic richness at high gain
    if (gain > 2.0) {
        sig = mix(sig, fold_triangle(sig * 1.5), (gain - 2.0) * 0.2);
    }
    return sig;
}

fn fold_rgb(rgb: vec3<f32>, gain: f32) -> vec3<f32> {
    return vec3<f32>(
        fold_paik_abe(rgb.r, gain),
        fold_paik_abe(rgb.g, gain * 1.02),   // Slight detuning for color fringing
        fold_paik_abe(rgb.b, gain * 1.04)
    );
}

// ============================================================================
// STAGE 4: PED COLORIZER (amplitude window colorization)
// ============================================================================

fn amplitude_window_soft(signal: f32, low: f32, high: f32, softness: f32) -> f32 {
    let edge_width = max(softness * 0.1, 0.001);
    let low_edge = smoothstep(low - edge_width, low + edge_width, signal);
    let high_edge = smoothstep(high - edge_width, high + edge_width, signal);
    return low_edge * (1.0 - high_edge);
}

fn ped_colorize(luma: f32, t1: f32, t2: f32, t3: f32) -> vec3<f32> {
    // Sort thresholds
    var s1 = min(min(t1, t2), t3);
    var s3 = max(max(t1, t2), t3);
    var s2 = t1 + t2 + t3 - s1 - s3;

    let softness = 0.3;

    let w0 = amplitude_window_soft(luma, 0.0, s1, softness);
    let w1 = amplitude_window_soft(luma, s1, s2, softness);
    let w2 = amplitude_window_soft(luma, s2, s3, softness);
    let w3 = amplitude_window_soft(luma, s3, 1.0, softness);

    // Classic Paik/Abe palette
    let c0 = vec3<f32>(0.1, 0.0, 0.8);   // Deep blue
    let c1 = vec3<f32>(0.9, 0.0, 0.6);   // Magenta
    let c2 = vec3<f32>(1.0, 0.9, 0.0);   // Yellow
    let c3 = vec3<f32>(0.0, 0.9, 0.8);   // Cyan

    return c0 * w0 + c1 * w1 + c2 * w2 + c3 * w3;
}

// ============================================================================
// STAGE 5: VIDEO KEYER (blend folded + colorized)
// ============================================================================

fn video_key(source_a: vec3<f32>, source_b: vec3<f32>, luma: f32, clip: f32) -> vec3<f32> {
    // Generate key from luminance with clip threshold
    let key_signal = clamp((luma - clip) * 3.0, 0.0, 1.0);

    // Soft S-curve blend via tanh
    let blend = tanh((key_signal - 0.5) * 8.0) * 0.5 + 0.5;

    return mix(source_a, source_b, blend);
}

// ============================================================================
// MAIN SHADER - FULL SIGNAL CHAIN
// ============================================================================

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Get parameters with defaults
    let fold_amount = select(params.gain, 1.5, params.gain == 0.0);
    let ped_t1 = select(params.offset, 0.25, params.offset == 0.0);
    let ped_t2 = select(params.mix, 0.5, params.mix == 0.0);
    let ped_t3 = select(params.fold, 0.75, params.fold == 0.0);
    let wob_amount = select(params.bias, 0.3, params.bias == 0.0);
    let wob_speed = select(params.frequency, 0.2, params.frequency == 0.0);
    let mat_rotation = select(params.phase, 0.0, params.phase == 0.0);
    let key_clip = select(params.threshold, 0.3, params.threshold == 0.0);

    // --- Stage 1: Wobbulator (UV displacement) ---
    let displaced_uv = wobbulate_uv(in.uv, wob_amount, wob_speed);

    // Sample input at displaced position
    let input_color = textureSample(input_texture, input_sampler, displaced_uv);
    let rgb = input_color.rgb;

    // --- Stage 2: Matrix Mixer ---
    let mixed = matrix_mix(rgb, displaced_uv, mat_rotation);

    // --- Stage 3: Folding Amplifier ---
    let folded = fold_rgb(mixed, fold_amount);

    // --- Stage 4: PED Colorizer ---
    let folded_luma = luminance(folded);
    let colorized = ped_colorize(folded_luma, ped_t1, ped_t2, ped_t3);

    // --- Stage 5: Video Keyer ---
    // Blend between folded (warm analog) and colorized (PED bands)
    let input_luma = luminance(rgb);
    let output_color = video_key(colorized, folded, input_luma, key_clip);

    return vec4<f32>(output_color, 1.0);
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
