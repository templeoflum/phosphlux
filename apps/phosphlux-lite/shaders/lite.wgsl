// Phosphlux Lite - Fixed Pipeline Shader
//
// 7-stage signal chain: INPUT -> GEOMETRY -> AMPLITUDE -> COLORIZE -> MIXER -> FEEDBACK -> OUTPUT
// Inspired by vintage analog video synthesizers: Paik/Abe, Sandin IP, Rutt/Etra, Jones, LZX

// ============================================
// UNIFORMS (matches SynthUniforms in synth.rs)
// ============================================

struct Synth {
    // Input stage (16 bytes)
    input_source_a: u32,
    input_source_b: u32,
    input_mix: f32,
    input_frequency: f32,

    // Input continued (16 bytes)
    input_phase: f32,
    input_rotation: f32,
    _pad0: f32,
    _pad1: f32,

    // Geometry stage (32 bytes)
    geo_wobbulate_h: f32,
    geo_wobbulate_v: f32,
    geo_wobble_freq: f32,
    geo_z_displacement: f32,

    geo_lissajous_x: f32,
    geo_lissajous_y: f32,
    geo_rotation: f32,
    geo_scale: f32,

    // Amplitude stage (32 bytes)
    amp_fold_gain: f32,
    amp_fold_mix: f32,
    amp_quantize_levels: f32,
    amp_quantize_mix: f32,

    amp_soft_clip: f32,
    amp_solarize: f32,
    amp_gate_threshold: f32,
    amp_invert: f32,

    // Colorize stage (32 bytes)
    color_mode: u32,
    color_hue_offset: f32,
    color_saturation: f32,
    color_levels: f32,

    color_gradient_start: vec3<f32>,
    _pad2: f32,

    // Colorize gradient end (16 bytes)
    color_gradient_end: vec3<f32>,
    _pad3: f32,

    // Mixer stage (16 bytes)
    mixer_feedback_mix: f32,
    mixer_blend_mode: u32,
    mixer_key_threshold: f32,
    mixer_key_softness: f32,

    // Mixer continued (16 bytes)
    mixer_key_invert: f32,
    mixer_layer_opacity: f32,
    _pad4: f32,
    _pad5: f32,

    // Feedback stage (32 bytes)
    fb_enabled: f32,
    fb_zoom: f32,
    fb_rotation: f32,
    fb_hue_shift: f32,

    fb_decay: f32,
    fb_offset_x: f32,
    fb_offset_y: f32,
    fb_saturation: f32,

    // Output stage (32 bytes)
    out_mode: u32,
    out_scanlines: f32,
    out_curvature: f32,
    out_bloom: f32,

    out_vignette: f32,
    out_noise: f32,
    out_tracking: f32,
    out_chroma_shift: f32,

    // Output continued (16 bytes)
    out_tape_wobble: f32,
    out_bandwidth: f32,
    out_ghosting: f32,
    _pad6: f32,

    // Timing (16 bytes)
    time: f32,
    frame: u32,
    _pad7: f32,
    _pad8: f32,
}

@group(0) @binding(0)
var<uniform> synth: Synth;

@group(1) @binding(0)
var feedback_texture: texture_2d<f32>;

@group(1) @binding(1)
var feedback_sampler: sampler;

// ============================================
// VERTEX SHADER
// ============================================

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(position, 0.0, 1.0);
    out.uv = uv;
    return out;
}

// ============================================
// UTILITY FUNCTIONS
// ============================================

const PI: f32 = 3.14159265359;
const TAU: f32 = 6.28318530718;

// Fast hue rotation using Rodrigues' rotation formula
fn rotate_hue(color: vec3<f32>, angle: f32) -> vec3<f32> {
    let k = vec3<f32>(0.57735026919); // 1/sqrt(3)
    let cos_a = cos(angle);
    let sin_a = sin(angle);
    return color * cos_a + cross(k, color) * sin_a + k * dot(k, color) * (1.0 - cos_a);
}

// Hash function for noise
fn hash(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    p3 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

// Value noise
fn noise_value(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let a = hash(i + vec2<f32>(0.0, 0.0));
    let b = hash(i + vec2<f32>(1.0, 0.0));
    let c = hash(i + vec2<f32>(0.0, 1.0));
    let d = hash(i + vec2<f32>(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// ============================================
// STAGE 1: INPUT SOURCES
// ============================================

const SRC_RAMP_H: u32 = 0u;
const SRC_RAMP_V: u32 = 1u;
const SRC_OSC_H: u32 = 2u;
const SRC_OSC_V: u32 = 3u;
const SRC_NOISE_WHITE: u32 = 4u;
const SRC_NOISE_PINK: u32 = 5u;
const SRC_NOISE_BROWN: u32 = 6u;
const SRC_SHAPE_CIRCLE: u32 = 7u;
const SRC_SHAPE_RECT: u32 = 8u;
const SRC_SHAPE_DIAMOND: u32 = 9u;
const SRC_CHECKERBOARD: u32 = 10u;

fn generate_source(source: u32, uv: vec2<f32>, time: f32, frequency: f32, phase: f32) -> f32 {
    if source == SRC_RAMP_H {
        return uv.x;
    } else if source == SRC_RAMP_V {
        return uv.y;
    } else if source == SRC_OSC_H {
        return sin((uv.x * frequency + phase + time * 0.5) * TAU) * 0.5 + 0.5;
    } else if source == SRC_OSC_V {
        return sin((uv.y * frequency + phase + time * 0.5) * TAU) * 0.5 + 0.5;
    } else if source == SRC_NOISE_WHITE {
        return hash(uv * 1000.0 + time * 100.0);
    } else if source == SRC_NOISE_PINK {
        var value = 0.0;
        var amplitude = 0.5;
        var freq = 1.0;
        let offset = time * 0.5;
        for (var i = 0; i < 5; i++) {
            value += noise_value(uv * freq + offset) * amplitude;
            amplitude *= 0.5;
            freq *= 2.0;
        }
        return value;
    } else if source == SRC_NOISE_BROWN {
        var value = 0.0;
        var amplitude = 0.6;
        var freq = 0.5;
        let offset = time * 0.2;
        for (var i = 0; i < 3; i++) {
            value += noise_value(uv * freq + offset) * amplitude;
            amplitude *= 0.4;
            freq *= 2.0;
        }
        return value;
    } else if source == SRC_SHAPE_CIRCLE {
        let center = vec2<f32>(0.5);
        let d = length(uv - center);
        return 1.0 - smoothstep(0.25, 0.3, d);
    } else if source == SRC_SHAPE_RECT {
        let center = vec2<f32>(0.5);
        let d = abs(uv - center);
        let outside = max(d.x - 0.2, d.y - 0.15);
        return 1.0 - smoothstep(0.0, 0.02, outside);
    } else if source == SRC_SHAPE_DIAMOND {
        let center = vec2<f32>(0.5);
        let d = abs(uv - center);
        let manhattan = d.x + d.y;
        return 1.0 - smoothstep(0.25, 0.3, manhattan);
    } else if source == SRC_CHECKERBOARD {
        let scale = frequency;
        let check = floor(uv.x * scale) + floor(uv.y * scale);
        return fract(check * 0.5) * 2.0;
    }
    return 0.5;
}

fn apply_input_rotation(uv: vec2<f32>, rotation: f32) -> vec2<f32> {
    let centered = uv - 0.5;
    let angle = rotation * TAU;
    let cos_a = cos(angle);
    let sin_a = sin(angle);
    let rotated = vec2<f32>(
        centered.x * cos_a - centered.y * sin_a,
        centered.x * sin_a + centered.y * cos_a
    );
    return rotated + 0.5;
}

fn stage_input(uv: vec2<f32>, time: f32) -> f32 {
    let rotated_uv = apply_input_rotation(uv, synth.input_rotation);
    let source_a = generate_source(synth.input_source_a, rotated_uv, time, synth.input_frequency, synth.input_phase);
    let source_b = generate_source(synth.input_source_b, rotated_uv, time, synth.input_frequency, synth.input_phase);
    return mix(source_a, source_b, synth.input_mix);
}

// ============================================
// STAGE 2: GEOMETRY
// ============================================

fn stage_geometry(uv: vec2<f32>, signal: f32, time: f32) -> vec2<f32> {
    var modified_uv = uv;

    // Center for transformations
    let centered = uv - 0.5;

    // Apply scale
    var transformed = centered / synth.geo_scale;

    // Apply rotation
    let rot_angle = synth.geo_rotation * TAU;
    let cos_r = cos(rot_angle);
    let sin_r = sin(rot_angle);
    transformed = vec2<f32>(
        transformed.x * cos_r - transformed.y * sin_r,
        transformed.x * sin_r + transformed.y * cos_r
    );

    // Apply Lissajous modulation
    let liss_phase = time * 0.5;
    transformed.x += sin(liss_phase * 3.0 + uv.y * TAU) * synth.geo_lissajous_x * 0.1;
    transformed.y += sin(liss_phase * 2.0 + uv.x * TAU) * synth.geo_lissajous_y * 0.1;

    // Apply wobbulation (Paik/Abe style)
    let wobble_phase = time * synth.geo_wobble_freq;
    transformed.x += sin(uv.y * 10.0 + wobble_phase) * synth.geo_wobbulate_h * 0.1;
    transformed.y += sin(uv.x * 10.0 + wobble_phase * 1.3) * synth.geo_wobbulate_v * 0.1;

    // Apply Z-axis displacement (Rutt/Etra style)
    // Luminance displaces vertical position
    let displacement = (signal - 0.5) * synth.geo_z_displacement;
    transformed.y += displacement;

    return transformed + 0.5;
}

// ============================================
// STAGE 3: AMPLITUDE
// ============================================

fn fold(x: f32, gain: f32) -> f32 {
    var v = x * gain;
    v = abs(fract(v * 0.5 + 0.25) * 2.0 - 1.0);
    return v;
}

fn quantize(x: f32, levels: f32) -> f32 {
    return floor(x * levels) / (levels - 1.0);
}

fn soft_clip(x: f32) -> f32 {
    return x / (1.0 + abs(x));
}

fn solarize(x: f32, threshold: f32) -> f32 {
    if x > threshold {
        return 1.0 - x;
    }
    return x;
}

fn stage_amplitude(signal: f32) -> f32 {
    var value = signal;

    // Apply folding
    if synth.amp_fold_mix > 0.001 {
        let folded = fold(value, synth.amp_fold_gain);
        value = mix(value, folded, synth.amp_fold_mix);
    }

    // Apply quantization
    if synth.amp_quantize_mix > 0.001 {
        let quantized = quantize(value, synth.amp_quantize_levels);
        value = mix(value, quantized, synth.amp_quantize_mix);
    }

    // Apply soft clipping
    if synth.amp_soft_clip > 0.001 {
        let clipped = soft_clip((value - 0.5) * (1.0 + synth.amp_soft_clip * 4.0)) * 0.5 + 0.5;
        value = mix(value, clipped, synth.amp_soft_clip);
    }

    // Apply solarize
    if synth.amp_solarize < 0.999 {
        value = solarize(value, synth.amp_solarize);
    }

    // Apply gate
    if synth.amp_gate_threshold > 0.001 {
        value = select(0.0, value, value > synth.amp_gate_threshold);
    }

    // Apply invert
    if synth.amp_invert > 0.5 {
        value = 1.0 - value;
    }

    return clamp(value, 0.0, 1.0);
}

// ============================================
// STAGE 4: COLORIZE
// ============================================

const COLOR_SPECTRUM: u32 = 0u;
const COLOR_THRESHOLD: u32 = 1u;
const COLOR_GRADIENT: u32 = 2u;
const COLOR_MONOCHROME: u32 = 3u;

fn colorize_spectrum(luma: f32) -> vec3<f32> {
    let h = luma * 6.0;
    let i = floor(h);
    let f = h - i;
    let q = 1.0 - f;

    let hi = i32(i) % 6;

    if hi == 0 { return vec3<f32>(1.0, f, 0.0); }
    if hi == 1 { return vec3<f32>(q, 1.0, 0.0); }
    if hi == 2 { return vec3<f32>(0.0, 1.0, f); }
    if hi == 3 { return vec3<f32>(0.0, q, 1.0); }
    if hi == 4 { return vec3<f32>(f, 0.0, 1.0); }
    return vec3<f32>(1.0, 0.0, q);
}

fn colorize_threshold(luma: f32, levels: f32) -> vec3<f32> {
    let level = floor(luma * levels);
    let t = level / levels;
    return colorize_spectrum(t);
}

fn stage_colorize(signal: f32) -> vec3<f32> {
    var color: vec3<f32>;

    if synth.color_mode == COLOR_SPECTRUM {
        color = colorize_spectrum(signal);
    } else if synth.color_mode == COLOR_THRESHOLD {
        color = colorize_threshold(signal, synth.color_levels);
    } else if synth.color_mode == COLOR_GRADIENT {
        color = mix(synth.color_gradient_start, synth.color_gradient_end, signal);
    } else {
        // Monochrome
        color = vec3<f32>(signal);
    }

    // Apply hue offset
    if synth.color_hue_offset > 0.001 {
        color = rotate_hue(color, synth.color_hue_offset * TAU);
    }

    // Apply saturation
    let luma = dot(color, vec3<f32>(0.299, 0.587, 0.114));
    color = mix(vec3<f32>(luma), color, synth.color_saturation);

    return color;
}

// ============================================
// STAGE 5: MIXER (blend with feedback)
// ============================================

const BLEND_MIX: u32 = 0u;
const BLEND_ADD: u32 = 1u;
const BLEND_MULTIPLY: u32 = 2u;
const BLEND_SCREEN: u32 = 3u;
const BLEND_OVERLAY: u32 = 4u;
const BLEND_DIFFERENCE: u32 = 5u;
const BLEND_LUMA_KEY_A: u32 = 6u;
const BLEND_LUMA_KEY_B: u32 = 7u;

fn blend_screen(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return 1.0 - (1.0 - a) * (1.0 - b);
}

fn blend_overlay(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    let r = select(1.0 - 2.0 * (1.0 - a.r) * (1.0 - b.r), 2.0 * a.r * b.r, a.r < 0.5);
    let g = select(1.0 - 2.0 * (1.0 - a.g) * (1.0 - b.g), 2.0 * a.g * b.g, a.g < 0.5);
    let bb = select(1.0 - 2.0 * (1.0 - a.b) * (1.0 - b.b), 2.0 * a.b * b.b, a.b < 0.5);
    return vec3<f32>(r, g, bb);
}

fn luma_key(color: vec3<f32>, threshold: f32, softness: f32) -> f32 {
    let luma = dot(color, vec3<f32>(0.299, 0.587, 0.114));
    let half_soft = softness * 0.5;
    return smoothstep(threshold - half_soft, threshold + half_soft, luma);
}

fn stage_mixer(color: vec3<f32>, feedback: vec3<f32>) -> vec3<f32> {
    let mix_amount = synth.mixer_feedback_mix;

    if mix_amount < 0.001 {
        return color;
    }

    var result: vec3<f32>;
    let mode = synth.mixer_blend_mode;

    if mode == BLEND_MIX {
        result = mix(color, feedback, mix_amount);
    } else if mode == BLEND_ADD {
        result = mix(color, min(color + feedback, vec3<f32>(1.0)), mix_amount);
    } else if mode == BLEND_MULTIPLY {
        result = mix(color, color * feedback, mix_amount);
    } else if mode == BLEND_SCREEN {
        result = mix(color, blend_screen(color, feedback), mix_amount);
    } else if mode == BLEND_OVERLAY {
        result = mix(color, blend_overlay(color, feedback), mix_amount);
    } else if mode == BLEND_DIFFERENCE {
        result = mix(color, abs(color - feedback), mix_amount);
    } else if mode == BLEND_LUMA_KEY_A {
        var key = luma_key(color, synth.mixer_key_threshold, synth.mixer_key_softness);
        key = select(key, 1.0 - key, synth.mixer_key_invert > 0.5);
        result = mix(color, mix(color, feedback, key), mix_amount);
    } else if mode == BLEND_LUMA_KEY_B {
        var key = luma_key(feedback, synth.mixer_key_threshold, synth.mixer_key_softness);
        key = select(key, 1.0 - key, synth.mixer_key_invert > 0.5);
        result = mix(color, mix(color, feedback, key), mix_amount);
    } else {
        result = mix(color, feedback, mix_amount);
    }

    return result * synth.mixer_layer_opacity;
}

// ============================================
// STAGE 6: FEEDBACK (sample previous frame)
// ============================================

fn stage_feedback(uv: vec2<f32>) -> vec3<f32> {
    if synth.fb_enabled < 0.5 {
        return vec3<f32>(0.0);
    }

    // Transform UV for feedback sampling
    var centered = uv - 0.5;

    // Apply zoom
    centered = centered / synth.fb_zoom;

    // Apply rotation
    let rot_angle = synth.fb_rotation * PI * 0.25; // Map to +/- pi/8
    let cos_r = cos(rot_angle);
    let sin_r = sin(rot_angle);
    centered = vec2<f32>(
        centered.x * cos_r - centered.y * sin_r,
        centered.x * sin_r + centered.y * cos_r
    );

    // Apply offset
    var fb_uv = centered + 0.5 + vec2<f32>(synth.fb_offset_x, synth.fb_offset_y);

    // Sample feedback
    var fb_color = textureSample(feedback_texture, feedback_sampler, fb_uv).rgb;

    // Apply hue shift
    if synth.fb_hue_shift > 0.001 {
        fb_color = rotate_hue(fb_color, synth.fb_hue_shift * TAU);
    }

    // Apply saturation
    let luma = dot(fb_color, vec3<f32>(0.299, 0.587, 0.114));
    fb_color = mix(vec3<f32>(luma), fb_color, synth.fb_saturation);

    // Apply decay
    fb_color = fb_color * synth.fb_decay;

    return fb_color;
}

// ============================================
// STAGE 7: OUTPUT (display emulation)
// ============================================

const OUT_CLEAN: u32 = 0u;
const OUT_CRT: u32 = 1u;
const OUT_VHS: u32 = 2u;
const OUT_CABLE: u32 = 3u;

fn scanlines(uv: vec2<f32>, intensity: f32) -> f32 {
    let line = sin(uv.y * 480.0 * PI) * 0.5 + 0.5;
    return mix(1.0, line, intensity);
}

fn crt_curve(uv: vec2<f32>, amount: f32) -> vec2<f32> {
    let centered = uv - 0.5;
    let dist = dot(centered, centered);
    let curved = centered * (1.0 + dist * amount);
    return curved + 0.5;
}

fn vignette(uv: vec2<f32>, amount: f32) -> f32 {
    let centered = uv - 0.5;
    let dist = dot(centered, centered);
    return 1.0 - dist * amount;
}

fn phosphor_bloom(color: vec3<f32>, amount: f32) -> vec3<f32> {
    let bloom = color * amount;
    return color + bloom * 0.3;
}

fn vhs_tracking(uv: vec2<f32>, time: f32, amount: f32) -> vec2<f32> {
    var modified = uv;

    // Horizontal tracking errors (random per-line offsets)
    let line = floor(uv.y * 240.0);
    let track_noise = hash(vec2<f32>(line, floor(time * 2.0)));

    // Only affect some lines
    if track_noise > 0.9 - amount * 0.3 {
        modified.x += (track_noise - 0.5) * amount * 0.1;
    }

    return modified;
}

fn vhs_wobble(uv: vec2<f32>, time: f32, amount: f32) -> vec2<f32> {
    var modified = uv;

    // Slow tape wobble
    let wobble = sin(time * 0.5 + uv.y * 10.0) * amount * 0.005;
    modified.x += wobble;

    // Faster flutter
    let flutter = sin(time * 30.0 + uv.y * 100.0) * amount * 0.001;
    modified.x += flutter;

    return modified;
}

fn vhs_chroma_shift(uv: vec2<f32>, color: vec3<f32>, amount: f32) -> vec3<f32> {
    // Sample chroma from offset positions
    let offset = amount;
    let r = textureSample(feedback_texture, feedback_sampler, uv + vec2<f32>(offset, 0.0)).r;
    let b = textureSample(feedback_texture, feedback_sampler, uv - vec2<f32>(offset, 0.0)).b;
    return vec3<f32>(r, color.g, b);
}

fn cable_bandwidth_limit(uv: vec2<f32>, color: vec3<f32>, bandwidth: f32) -> vec3<f32> {
    // Simple blur to simulate bandwidth limiting
    let blur_amount = (1.0 - bandwidth) * 0.01;
    var blurred = color;

    if blur_amount > 0.001 {
        blurred += textureSample(feedback_texture, feedback_sampler, uv + vec2<f32>(blur_amount, 0.0)).rgb;
        blurred += textureSample(feedback_texture, feedback_sampler, uv - vec2<f32>(blur_amount, 0.0)).rgb;
        blurred /= 3.0;
    }

    return blurred;
}

fn cable_ghosting(uv: vec2<f32>, color: vec3<f32>, amount: f32) -> vec3<f32> {
    // RF ghosting - faint delayed echo
    let ghost_uv = uv + vec2<f32>(amount * 0.1, 0.0);
    let ghost = textureSample(feedback_texture, feedback_sampler, ghost_uv).rgb;
    return mix(color, color + ghost * 0.3, amount);
}

fn stage_output(uv: vec2<f32>, color: vec3<f32>, time: f32) -> vec3<f32> {
    var modified_uv = uv;
    var output_color = color;

    let mode = synth.out_mode;

    // Apply curvature (CRT and VHS)
    if synth.out_curvature > 0.001 && mode != OUT_CLEAN {
        modified_uv = crt_curve(modified_uv, synth.out_curvature);

        // Black outside curved area
        if modified_uv.x < 0.0 || modified_uv.x > 1.0 || modified_uv.y < 0.0 || modified_uv.y > 1.0 {
            return vec3<f32>(0.0);
        }
    }

    // Mode-specific processing
    if mode == OUT_VHS {
        // VHS tracking errors
        if synth.out_tracking > 0.001 {
            modified_uv = vhs_tracking(modified_uv, time, synth.out_tracking);
        }

        // VHS tape wobble
        if synth.out_tape_wobble > 0.001 {
            modified_uv = vhs_wobble(modified_uv, time, synth.out_tape_wobble);
        }

        // VHS chroma/luma separation
        if synth.out_chroma_shift > 0.0001 {
            output_color = vhs_chroma_shift(modified_uv, output_color, synth.out_chroma_shift);
        }
    } else if mode == OUT_CABLE {
        // Bandwidth limiting
        if synth.out_bandwidth < 0.999 {
            output_color = cable_bandwidth_limit(modified_uv, output_color, synth.out_bandwidth);
        }

        // RF ghosting
        if synth.out_ghosting > 0.001 {
            output_color = cable_ghosting(modified_uv, output_color, synth.out_ghosting);
        }
    }

    // Common effects for CRT/VHS/Cable
    if mode != OUT_CLEAN {
        // Scanlines
        if synth.out_scanlines > 0.001 {
            output_color = output_color * scanlines(modified_uv, synth.out_scanlines);
        }

        // Vignette
        if synth.out_vignette > 0.001 {
            output_color = output_color * vignette(modified_uv, synth.out_vignette);
        }

        // Bloom
        if synth.out_bloom > 0.001 {
            output_color = phosphor_bloom(output_color, synth.out_bloom);
        }
    }

    // Noise (all modes)
    if synth.out_noise > 0.001 {
        let noise = hash(modified_uv * 500.0 + time * 100.0) * 2.0 - 1.0;
        output_color = output_color + vec3<f32>(noise * synth.out_noise);
    }

    return clamp(output_color, vec3<f32>(0.0), vec3<f32>(1.0));
}

// ============================================
// FRAGMENT SHADER - MAIN PIPELINE
// ============================================

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let time = synth.time;

    // Stage 1: Generate input signal
    let input_signal = stage_input(uv, time);

    // Stage 2: Apply geometry transformations
    let geo_uv = stage_geometry(uv, input_signal, time);

    // Re-sample input at transformed coordinates
    let geo_signal = stage_input(geo_uv, time);

    // Stage 3: Apply amplitude processing
    let amp_signal = stage_amplitude(geo_signal);

    // Stage 4: Colorize
    let color = stage_colorize(amp_signal);

    // Stage 6: Get feedback (needs to happen before mixer)
    let feedback = stage_feedback(uv);

    // Stage 5: Mix with feedback
    let mixed = stage_mixer(color, feedback);

    // Stage 7: Output emulation
    let output = stage_output(uv, mixed, time);

    return vec4<f32>(output, 1.0);
}
