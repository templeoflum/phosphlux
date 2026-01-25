// CA3054 Video Keyer - Paik/Abe Keying Circuit
// Based on schematic page 16 - Keying Circuit
//
// The CA3054 provides hard and soft keying between video sources.
// Uses differential amplifier for smooth or sharp transitions.
//
// Demo mode: Creates two virtual sources from input:
//   Source A: Original image
//   Source B: Colorized/inverted version
//   Key: Luminance-derived key signal
//
// Workbench Parameters:
//   gain     → Key clip level (black level threshold)
//   offset   → Key gain (transition sharpness)
//   mix      → Soft/hard key blend (0=hard, 1=soft)
//   fold     → Key invert (0=normal, 1=inverted)
//   bias     → Source B colorization amount

struct Params {
    gain: f32,      // Key clip
    offset: f32,    // Key gain
    mix: f32,       // Soft/hard
    fold: f32,      // Invert
    bias: f32,      // Colorization
    frequency: f32,
    phase: f32,
    threshold: f32,
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
// CA3054 KEYING FUNCTIONS
// ============================================================================

const VT: f32 = 0.026;

/// Soft key - smooth crossfade using diff amp characteristic
fn soft_key(source_a: vec3<f32>, source_b: vec3<f32>, key: f32, softness: f32) -> vec3<f32> {
    // Softness controls transition width
    let sharpness = mix(50.0, 5.0, softness);
    let key_centered = key - 0.5;

    // Tanh gives S-curve transition
    let blend = tanh(key_centered * sharpness) * 0.5 + 0.5;

    return mix(source_a, source_b, blend);
}

/// Hard key - binary switching with slight hysteresis
fn hard_key(source_a: vec3<f32>, source_b: vec3<f32>, key: f32, threshold: f32) -> vec3<f32> {
    // Sharp threshold comparison
    let above = step(threshold, key);
    return mix(source_a, source_b, above);
}

/// Luminance key generator
fn generate_key(rgb: vec3<f32>, clip: f32, key_gain: f32) -> f32 {
    let luma = dot(rgb, vec3<f32>(0.299, 0.587, 0.114));

    // Clip removes values below threshold
    let clipped = max(luma - clip, 0.0);

    // Gain amplifies the key
    let gained = clipped * key_gain;

    return clamp(gained, 0.0, 1.0);
}

// ============================================================================
// SOURCE GENERATION (for demo)
// ============================================================================

/// Paik/Abe style colorization
fn paik_colorize(rgb: vec3<f32>, amount: f32) -> vec3<f32> {
    let luma = dot(rgb, vec3<f32>(0.299, 0.587, 0.114));

    // Create color bands
    let band = floor(luma * 4.0) / 3.0;

    // Psychedelic palette
    let palette_r = sin(band * 3.14159 * 2.0) * 0.5 + 0.5;
    let palette_g = sin(band * 3.14159 * 2.0 + 2.094) * 0.5 + 0.5;
    let palette_b = sin(band * 3.14159 * 2.0 + 4.189) * 0.5 + 0.5;
    let colorized = vec3<f32>(palette_r, palette_g, palette_b);

    return mix(rgb, colorized, amount);
}

/// Inverted/solarized source
fn solarize(rgb: vec3<f32>) -> vec3<f32> {
    let luma = dot(rgb, vec3<f32>(0.299, 0.587, 0.114));

    // Solarize: invert values above 0.5
    if (luma > 0.5) {
        return vec3<f32>(1.0) - rgb;
    }
    return rgb;
}

// ============================================================================
// MAIN SHADER
// ============================================================================

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let input_color = textureSample(input_texture, input_sampler, in.uv);
    let rgb = input_color.rgb;

    // Get params with defaults
    let clip = select(params.gain, 0.2, params.gain == 0.0);
    let key_gain = select(params.offset, 2.0, params.offset == 0.0);
    let softness = select(params.mix, 0.5, params.mix == 0.0);
    let invert = select(params.fold, 0.0, params.fold == 0.0);
    let colorize_amt = select(params.bias, 0.8, params.bias == 0.0);

    // Generate two sources
    let source_a = rgb;
    let source_b = paik_colorize(solarize(rgb), colorize_amt);

    // Generate key from luminance
    var key = generate_key(rgb, clip, key_gain);

    // Optionally invert key
    key = mix(key, 1.0 - key, invert);

    // Blend between hard and soft keying
    let hard_result = hard_key(source_a, source_b, key, 0.5);
    let soft_result = soft_key(source_a, source_b, key, softness);

    // Mix between hard and soft based on softness param
    let final_color = mix(hard_result, soft_result, softness);

    return vec4<f32>(final_color, 1.0);
}

// ============================================================================
// VERTEX SHADER
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
