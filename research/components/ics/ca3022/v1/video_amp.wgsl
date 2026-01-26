// CA3022 Wideband Video Amplifier - Workbench Application Shader
// Demonstrates the CA3022 as video output amplifier in Paik/Abe context
// Schematic pages 10, 20: Chroma key video output amplifier
//
// From research/components/ics/ca3022/v1/model.wgsl
//
// Workbench Parameter Mapping:
//   gain      -> Amplifier gain (R2/R1 ratio), range 0.5x to 4x
//   offset    -> DC operating point bias, shifts black level
//   mix       -> Bandwidth cutoff (lowpass), 0=full BW, 1=heavy filter
//   fold      -> Slew rate limit, 0=unlimited, 1=heavy limiting
//   bias      -> Saturation headroom, 0=hard clip, 1=lots of room
//   frequency -> (unused in this shader)
//   phase     -> (unused in this shader)
//   threshold -> Noise floor amount, 0=clean, 1=noisy

struct Params {
    gain: f32,      // Amplifier gain
    offset: f32,    // DC bias offset
    mix: f32,       // Bandwidth cutoff
    fold: f32,      // Slew rate limit
    bias: f32,      // Saturation headroom
    frequency: f32, // (unused)
    phase: f32,     // (unused)
    threshold: f32, // Noise floor
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

// === CA3022 Component Model ===
// From research/components/ics/ca3022/model.wgsl

fn ca3022_gain_stage(input: f32, gain: f32, bias: f32) -> f32 {
    return input * gain + bias;
}

fn ca3022_saturate(input: f32, headroom: f32) -> f32 {
    let h = max(headroom, 0.05);
    return tanh(input / h) * h;
}

fn ca3022_slew_limit(current: f32, previous: f32, max_rate: f32) -> f32 {
    let delta = current - previous;
    let limited_delta = clamp(delta, -max_rate, max_rate);
    return previous + limited_delta;
}

fn ca3022_noise(signal: f32, seed: f32, amount: f32) -> f32 {
    let n = fract(sin(seed * 12.9898 + signal * 78.233) * 43758.5453);
    let thermal = (n - 0.5) * 2.0;
    let shot = thermal * sqrt(max(abs(signal), 0.001));
    return signal + (thermal * 0.6 + shot * 0.4) * amount;
}

// === Shader ===

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((vi & 1u) << 2u) - 1.0;
    let y = f32((vi & 2u) << 1u) - 1.0;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(input_texture, input_sampler, in.uv);

    // Map parameters to physical controls
    let gain = 0.5 + params.gain * 3.5;          // 0.5x to 4.0x gain
    let bias = (params.offset - 0.5) * 0.4;      // +/-0.2 DC offset
    let bw_cutoff = params.mix;                   // 0=full BW, 1=heavy filter
    let slew_limit = params.fold;                 // 0=unlimited, 1=heavy
    let headroom = 0.2 + params.bias * 0.8;       // 0.2 to 1.0 headroom
    let noise_amt = params.threshold * 0.08;       // 0 to 0.08 noise

    // Bandwidth limiting: spatial blur using neighbor samples
    // Approximates RC lowpass rolloff in the video bandwidth domain
    let texel_size = 1.0 / vec2<f32>(textureDimensions(input_texture));
    let left = textureSample(input_texture, input_sampler, in.uv - vec2<f32>(texel_size.x, 0.0));
    let right = textureSample(input_texture, input_sampler, in.uv + vec2<f32>(texel_size.x, 0.0));
    let filtered = mix(color, (left + color + right) / 3.0, bw_cutoff * 0.8);

    // Process each channel through CA3022 pipeline
    // Center signal around 0 for proper amplifier behavior
    var r = filtered.r - 0.5;
    var g = filtered.g - 0.5;
    var b = filtered.b - 0.5;

    // Stage 1: Gain + DC bias
    r = ca3022_gain_stage(r, gain, bias);
    g = ca3022_gain_stage(g, gain, bias);
    b = ca3022_gain_stage(b, gain, bias);

    // Stage 2: Slew rate limiting (horizontal neighbor as "previous" sample)
    if (slew_limit > 0.01) {
        let max_rate = 0.02 + (1.0 - slew_limit) * 0.98;
        let prev_r = left.r - 0.5;
        let prev_g = left.g - 0.5;
        let prev_b = left.b - 0.5;
        r = ca3022_slew_limit(r, ca3022_gain_stage(prev_r, gain, bias), max_rate);
        g = ca3022_slew_limit(g, ca3022_gain_stage(prev_g, gain, bias), max_rate);
        b = ca3022_slew_limit(b, ca3022_gain_stage(prev_b, gain, bias), max_rate);
    }

    // Stage 3: Soft saturation at supply rails
    r = ca3022_saturate(r, headroom);
    g = ca3022_saturate(g, headroom);
    b = ca3022_saturate(b, headroom);

    // Stage 4: Analog noise floor
    if (noise_amt > 0.001) {
        let seed_r = in.uv.x * 1000.0 + in.uv.y * 7919.0;
        let seed_g = in.uv.x * 1000.0 + in.uv.y * 7919.0 + 0.333;
        let seed_b = in.uv.x * 1000.0 + in.uv.y * 7919.0 + 0.667;
        r = ca3022_noise(r, seed_r, noise_amt);
        g = ca3022_noise(g, seed_g, noise_amt);
        b = ca3022_noise(b, seed_b, noise_amt);
    }

    // Re-center to 0-1 range and clamp
    let output = vec3<f32>(
        clamp(r + 0.5, 0.0, 1.0),
        clamp(g + 0.5, 0.0, 1.0),
        clamp(b + 0.5, 0.0, 1.0)
    );

    return vec4<f32>(output, 1.0);
}
