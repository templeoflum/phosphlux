// CA3023 High-Gain Subcarrier Amplifier - Workbench Application Shader
// Demonstrates the CA3023 as NTSC 3.58 MHz color subcarrier amplifier
// Schematic pages 3, 4, 26, 30: Subcarrier amplifier and phase control
//
// From research/components/ics/ca3023/v1/model.wgsl
//
// Workbench Parameter Mapping:
//   gain      -> Total gain (both stages combined), 0=unity, 1=max overdrive
//   offset    -> DC clamp level (black level restore)
//   mix       -> Bandwidth / Q factor (bandpass sharpness)
//   fold      -> Saturation drive (overdrive amount)
//   bias      -> Phase offset (color rotation angle)
//   frequency -> Center frequency tuning (spatial freq of bandpass)
//   phase     -> Subcarrier phase angle (hue rotation)
//   threshold -> Clamp restore level (black level target)

struct Params {
    gain: f32,      // Total gain
    offset: f32,    // DC clamp level
    mix: f32,       // Bandwidth / Q
    fold: f32,      // Saturation drive
    bias: f32,      // Phase offset
    frequency: f32, // Center frequency
    phase: f32,     // Subcarrier phase
    threshold: f32, // Clamp restore
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

// === CA3023 Component Model ===
// From research/components/ics/ca3023/model.wgsl

fn ca3023_two_stage_amp(input: f32, gain1: f32, gain2: f32) -> f32 {
    let stage1 = tanh(input * gain1 / 1.5) * 1.5;
    let stage2 = tanh(stage1 * gain2 / 2.0) * 2.0;
    return stage2;
}

fn ca3023_bandpass(input: f32, neighbor_avg: f32, q: f32) -> f32 {
    let highpass = input - neighbor_avg;
    let emphasis = clamp(q / 10.0, 0.1, 2.0);
    return input + highpass * emphasis;
}

fn ca3023_phase_shift_iq(i_in: f32, q_in: f32, angle: f32) -> vec2<f32> {
    let cos_a = cos(angle);
    let sin_a = sin(angle);
    return vec2<f32>(
        i_in * cos_a - q_in * sin_a,
        i_in * sin_a + q_in * cos_a
    );
}

fn ca3023_clamp(input: f32, level: f32) -> f32 {
    return max(input, level);
}

fn ca3023_drive_saturate(input: f32, drive: f32) -> f32 {
    let d = max(drive, 0.1);
    return tanh(input * d) / tanh(d);
}

// === Color space utilities ===

fn rgb_to_yiq(rgb: vec3<f32>) -> vec3<f32> {
    // NTSC YIQ color space
    let y = dot(rgb, vec3<f32>(0.299, 0.587, 0.114));
    let i = dot(rgb, vec3<f32>(0.596, -0.274, -0.322));
    let q = dot(rgb, vec3<f32>(0.211, -0.523, 0.312));
    return vec3<f32>(y, i, q);
}

fn yiq_to_rgb(yiq: vec3<f32>) -> vec3<f32> {
    let r = yiq.x + 0.956 * yiq.y + 0.621 * yiq.z;
    let g = yiq.x - 0.272 * yiq.y - 0.647 * yiq.z;
    let b = yiq.x - 1.106 * yiq.y + 1.703 * yiq.z;
    return vec3<f32>(r, g, b);
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
    let total_gain = 1.0 + params.gain * 8.0;       // 1x to 9x total gain
    let gain1 = sqrt(total_gain) * 1.5;              // Stage 1: sqrt distribution
    let gain2 = sqrt(total_gain);                    // Stage 2: sqrt distribution
    let clamp_level = params.offset * 0.3;           // 0.0 to 0.3 clamp level
    let q_factor = 1.0 + params.mix * 15.0;          // Q from 1 to 16
    let drive = 0.5 + params.fold * 3.0;             // Drive: 0.5 to 3.5
    let phase_offset = params.bias * TAU;            // 0 to 2*PI phase offset
    let freq_tune = 0.5 + params.frequency * 2.0;   // Spatial frequency multiplier
    let phase_angle = params.phase * TAU;            // 0 to 2*PI subcarrier phase
    let restore_level = params.threshold * 0.2;      // 0.0 to 0.2 restore level

    // Convert to YIQ color space (NTSC subcarrier domain)
    let yiq = rgb_to_yiq(color.rgb);
    var y = yiq.x;
    var i = yiq.y;
    var q = yiq.z;

    // Bandpass filtering on chrominance (I and Q channels)
    // Sample neighbors for spatial frequency content
    let texel = 1.0 / vec2<f32>(textureDimensions(input_texture));
    let left = textureSample(input_texture, input_sampler, in.uv - vec2<f32>(texel.x * freq_tune, 0.0));
    let right = textureSample(input_texture, input_sampler, in.uv + vec2<f32>(texel.x * freq_tune, 0.0));
    let up = textureSample(input_texture, input_sampler, in.uv - vec2<f32>(0.0, texel.y * freq_tune));
    let down = textureSample(input_texture, input_sampler, in.uv + vec2<f32>(0.0, texel.y * freq_tune));

    let avg_yiq = (rgb_to_yiq(left.rgb) + rgb_to_yiq(right.rgb) +
                   rgb_to_yiq(up.rgb) + rgb_to_yiq(down.rgb)) / 4.0;

    // Apply bandpass to chroma channels
    i = ca3023_bandpass(i, avg_yiq.y, q_factor);
    q = ca3023_bandpass(q, avg_yiq.z, q_factor);

    // Two-stage amplification on chroma
    i = ca3023_two_stage_amp(i, gain1, gain2);
    q = ca3023_two_stage_amp(q, gain1, gain2);

    // Drive saturation (overdriven subcarrier amp distortion)
    i = ca3023_drive_saturate(i, drive);
    q = ca3023_drive_saturate(q, drive);

    // Phase rotation (hue control via subcarrier phase)
    let combined_phase = phase_offset + phase_angle;
    let rotated = ca3023_phase_shift_iq(i, q, combined_phase);
    i = rotated.x;
    q = rotated.y;

    // Black level clamp on luminance
    y = ca3023_clamp(y, restore_level);

    // DC clamp on luminance
    y = ca3023_clamp(y - clamp_level, 0.0) + clamp_level;

    // Convert back to RGB
    let output_rgb = yiq_to_rgb(vec3<f32>(y, i, q));

    return vec4<f32>(clamp(output_rgb, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
