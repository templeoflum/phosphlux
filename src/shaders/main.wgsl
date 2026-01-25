// Analog Video Synthesizer - Main Shader
//
// This shader runs on every pixel simultaneously, just like voltage
// flows through analog circuits. Each function here corresponds to
// a real analog module.

// Timing uniforms from sync generator
// Must match TimingUniforms in timing.rs exactly (64 bytes)
struct Timing {
    field: u32,           // 0-4
    frame: u32,           // 4-8
    frame_phase: f32,     // 8-12
    time: f32,            // 12-16
    h_phase: f32,         // 16-20
    v_phase: f32,         // 20-24
    lines: f32,           // 24-28
    field_rate: f32,      // 28-32
    subcarrier_phase: f32, // 32-36
    _pad0: f32,           // 36-40
    _pad1: f32,           // 40-44
    _pad2: f32,           // 44-48
    _pad3: f32,           // 48-52
    _pad4: f32,           // 52-56
    _pad5: f32,           // 56-60
    _pad6: f32,           // 60-64
}

// Feedback uniforms (32 bytes)
struct Feedback {
    mix: f32,
    zoom: f32,
    rotation: f32,
    hue_shift: f32,
    offset_x: f32,
    offset_y: f32,
    saturation: f32,
    decay: f32,
}

// Synthesis parameters (96 bytes = 24 floats)
struct Synth {
    // Oscillators
    osc1_freq: f32,
    osc1_phase: f32,
    osc2_freq: f32,
    osc2_phase: f32,
    noise_mix: f32,
    noise_type: f32,  // 0=white, 1=pink, 2=brown
    _pad0: f32,
    _pad1: f32,

    // Keyer
    key_enabled: f32,
    key_clip_black: f32,
    key_clip_white: f32,
    key_invert: f32,
    rutt_etra_enabled: f32,
    rutt_etra_amount: f32,
    colorizer_type: f32,  // 0=spectrum, 1=threshold, 2=grayscale
    colorizer_levels: f32,

    // CRT + Shape
    scanline_intensity: f32,
    curvature: f32,
    vignette_amount: f32,
    shape_type: f32,  // 0=none, 1=circle, 2=rect, 3=diamond
    shape_size: f32,
    shape_softness: f32,
    _pad2: f32,
    _pad3: f32,
}

@group(0) @binding(0)
var<uniform> timing: Timing;

@group(0) @binding(1)
var<uniform> feedback: Feedback;

@group(0) @binding(2)
var<uniform> synth: Synth;

// Feedback texture from previous frame
@group(1) @binding(0)
var feedback_texture: texture_2d<f32>;

@group(1) @binding(1)
var feedback_sampler: sampler;

// Vertex output
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// ============================================
// VERTEX SHADER
// ============================================

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
// COLOR SPACE UTILITIES
// ============================================

// RGB to HSV
fn rgb_to_hsv(rgb: vec3<f32>) -> vec3<f32> {
    let cmax = max(rgb.r, max(rgb.g, rgb.b));
    let cmin = min(rgb.r, min(rgb.g, rgb.b));
    let delta = cmax - cmin;

    var h: f32 = 0.0;
    if delta > 0.0001 {
        if cmax == rgb.r {
            h = (rgb.g - rgb.b) / delta;
        } else if cmax == rgb.g {
            h = 2.0 + (rgb.b - rgb.r) / delta;
        } else {
            h = 4.0 + (rgb.r - rgb.g) / delta;
        }
        h = fract(h / 6.0);
    }

    let s = select(0.0, delta / cmax, cmax > 0.0001);
    let v = cmax;

    return vec3<f32>(h, s, v);
}

// HSV to RGB
fn hsv_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
    let h = hsv.x * 6.0;
    let s = hsv.y;
    let v = hsv.z;

    let i = floor(h);
    let f = h - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    let hi = i32(i) % 6;

    if hi == 0 { return vec3<f32>(v, t, p); }
    if hi == 1 { return vec3<f32>(q, v, p); }
    if hi == 2 { return vec3<f32>(p, v, t); }
    if hi == 3 { return vec3<f32>(p, q, v); }
    if hi == 4 { return vec3<f32>(t, p, v); }
    return vec3<f32>(v, p, q);
}

// ============================================
// OSCILLATORS - Signal Sources
// ============================================

// Horizontal ramp (0 at left, 1 at right)
fn ramp_h(uv: vec2<f32>) -> f32 {
    return uv.x;
}

// Vertical ramp (0 at top, 1 at bottom)
fn ramp_v(uv: vec2<f32>) -> f32 {
    return uv.y;
}

// Sine oscillator locked to horizontal position
fn osc_h(uv: vec2<f32>, frequency: f32, phase: f32) -> f32 {
    return sin((uv.x * frequency + phase) * 6.283185) * 0.5 + 0.5;
}

// Sine oscillator locked to vertical position
fn osc_v(uv: vec2<f32>, frequency: f32, phase: f32) -> f32 {
    return sin((uv.y * frequency + phase) * 6.283185) * 0.5 + 0.5;
}

// Radial ramp (distance from center)
fn ramp_radial(uv: vec2<f32>) -> f32 {
    let centered = uv - vec2<f32>(0.5);
    return length(centered) * 2.0;
}

// ============================================
// NOISE GENERATORS
// ============================================

// Hash function for pseudo-random numbers
fn hash(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    p3 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

// 2D hash returning vec2
fn hash2(p: vec2<f32>) -> vec2<f32> {
    let k = vec2<f32>(0.3183099, 0.3678794);
    var n = p * k + k.yx;
    return fract(16.0 * k * fract(n.x * n.y * (n.x + n.y)));
}

// White noise - flat frequency spectrum, totally random
fn noise_white(uv: vec2<f32>, time: f32) -> f32 {
    return hash(uv * 1000.0 + time * 100.0);
}

// Value noise (smoother, interpolated noise)
fn noise_value(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);

    // Cubic interpolation
    let u = f * f * (3.0 - 2.0 * f);

    // Sample corners
    let a = hash(i + vec2<f32>(0.0, 0.0));
    let b = hash(i + vec2<f32>(1.0, 0.0));
    let c = hash(i + vec2<f32>(0.0, 1.0));
    let d = hash(i + vec2<f32>(1.0, 1.0));

    // Bilinear interpolation
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// Pink noise approximation - 1/f spectrum, "natural" sounding
// Summed octaves of noise with decreasing amplitude
fn noise_pink(uv: vec2<f32>, time: f32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    let offset = time * 0.5;

    // Sum multiple octaves
    for (var i = 0; i < 5; i++) {
        value += noise_value(uv * frequency + offset) * amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    return value;
}

// Brown/Brownian noise - 1/f^2 spectrum, very smooth/bassy
// More weighted toward low frequencies than pink noise
fn noise_brown(uv: vec2<f32>, time: f32) -> f32 {
    var value = 0.0;
    var amplitude = 0.6;
    var frequency = 0.5;
    let offset = time * 0.2;

    // Fewer octaves, more emphasis on low frequencies
    for (var i = 0; i < 3; i++) {
        value += noise_value(uv * frequency + offset) * amplitude;
        amplitude *= 0.4;  // Faster falloff than pink
        frequency *= 2.0;
    }

    return value;
}

// Animated noise field with controllable scale and speed
fn noise_animated(uv: vec2<f32>, scale: f32, speed: f32, time: f32) -> f32 {
    return noise_value(uv * scale + time * speed);
}

// ============================================
// SHAPE GENERATORS
// ============================================

// Rectangle shape (1 inside, 0 outside)
fn shape_rect(uv: vec2<f32>, center: vec2<f32>, size: vec2<f32>) -> f32 {
    let d = abs(uv - center) - size * 0.5;
    let outside = max(d.x, d.y);
    return select(1.0, 0.0, outside > 0.0);
}

// Soft rectangle with adjustable edge softness
fn shape_rect_soft(uv: vec2<f32>, center: vec2<f32>, size: vec2<f32>, softness: f32) -> f32 {
    let d = abs(uv - center) - size * 0.5;
    let outside = max(d.x, d.y);
    return 1.0 - smoothstep(0.0, softness, outside);
}

// Circle shape (1 inside, 0 outside)
fn shape_circle(uv: vec2<f32>, center: vec2<f32>, radius: f32) -> f32 {
    let d = length(uv - center);
    return select(1.0, 0.0, d > radius);
}

// Soft circle with adjustable edge softness
fn shape_circle_soft(uv: vec2<f32>, center: vec2<f32>, radius: f32, softness: f32) -> f32 {
    let d = length(uv - center);
    return 1.0 - smoothstep(radius - softness, radius + softness, d);
}

// Ring/annulus shape
fn shape_ring(uv: vec2<f32>, center: vec2<f32>, inner_radius: f32, outer_radius: f32) -> f32 {
    let d = length(uv - center);
    return select(0.0, 1.0, d > inner_radius && d < outer_radius);
}

// Diamond shape (rotated square)
fn shape_diamond(uv: vec2<f32>, center: vec2<f32>, size: f32) -> f32 {
    let d = abs(uv - center);
    let manhattan = d.x + d.y;
    return select(1.0, 0.0, manhattan > size);
}

// Soft diamond
fn shape_diamond_soft(uv: vec2<f32>, center: vec2<f32>, size: f32, softness: f32) -> f32 {
    let d = abs(uv - center);
    let manhattan = d.x + d.y;
    return 1.0 - smoothstep(size - softness, size + softness, manhattan);
}

// Star shape with configurable points
fn shape_star(uv: vec2<f32>, center: vec2<f32>, outer_radius: f32, inner_radius: f32, points: f32) -> f32 {
    let centered = uv - center;
    let angle = atan2(centered.y, centered.x);
    let r = length(centered);

    // Create star pattern
    let a = 3.14159 / points;
    let star_r = mix(outer_radius, inner_radius, abs(fract(angle / a / 2.0) - 0.5) * 2.0);

    return select(1.0, 0.0, r > star_r);
}

// Horizontal bars pattern
fn shape_bars_h(uv: vec2<f32>, count: f32, duty_cycle: f32) -> f32 {
    let y = fract(uv.y * count);
    return select(1.0, 0.0, y > duty_cycle);
}

// Vertical bars pattern
fn shape_bars_v(uv: vec2<f32>, count: f32, duty_cycle: f32) -> f32 {
    let x = fract(uv.x * count);
    return select(1.0, 0.0, x > duty_cycle);
}

// Checkerboard pattern
fn shape_checker(uv: vec2<f32>, scale: f32) -> f32 {
    let check = floor(uv.x * scale) + floor(uv.y * scale);
    return fract(check * 0.5) * 2.0;
}

// Grid pattern (returns 1 on grid lines, 0 elsewhere)
fn shape_grid(uv: vec2<f32>, scale: f32, line_width: f32) -> f32 {
    let grid = fract(uv * scale);
    let lines = min(min(grid.x, 1.0 - grid.x), min(grid.y, 1.0 - grid.y));
    return select(0.0, 1.0, lines < line_width);
}

// ============================================
// KEYERS - Signal Mixing/Switching
// ============================================

// Luma key - classic analog video effect
// Replaces pixels based on luminance threshold
// clip_black/clip_white define the key range
// Returns key signal (0-1) that can be used for mixing
fn luma_key(color: vec3<f32>, clip_black: f32, clip_white: f32) -> f32 {
    let luma = dot(color, vec3<f32>(0.299, 0.587, 0.114));
    return smoothstep(clip_black, clip_white, luma);
}

// Inverted luma key
fn luma_key_invert(color: vec3<f32>, clip_black: f32, clip_white: f32) -> f32 {
    return 1.0 - luma_key(color, clip_black, clip_white);
}

// Hard luma key (no softness)
fn luma_key_hard(color: vec3<f32>, threshold: f32) -> f32 {
    let luma = dot(color, vec3<f32>(0.299, 0.587, 0.114));
    return select(0.0, 1.0, luma > threshold);
}

// Chroma key - key based on color distance from target
// target_color: the color to key out (e.g., green screen)
// tolerance: how close colors need to be to be keyed
// softness: edge softness
fn chroma_key(color: vec3<f32>, target_color: vec3<f32>, tolerance: f32, softness: f32) -> f32 {
    let dist = distance(color, target_color);
    return smoothstep(tolerance, tolerance + softness, dist);
}

// Difference key - key based on difference between two signals
fn difference_key(color_a: vec3<f32>, color_b: vec3<f32>, threshold: f32, softness: f32) -> f32 {
    let diff = length(color_a - color_b);
    return smoothstep(threshold, threshold + softness, diff);
}

// Mix two colors using a key signal
fn key_mix(color_a: vec3<f32>, color_b: vec3<f32>, key: f32) -> vec3<f32> {
    return mix(color_a, color_b, key);
}

// Multiply blend (darkens)
fn blend_multiply(color_a: vec3<f32>, color_b: vec3<f32>) -> vec3<f32> {
    return color_a * color_b;
}

// Screen blend (lightens)
fn blend_screen(color_a: vec3<f32>, color_b: vec3<f32>) -> vec3<f32> {
    return 1.0 - (1.0 - color_a) * (1.0 - color_b);
}

// Additive blend
fn blend_add(color_a: vec3<f32>, color_b: vec3<f32>) -> vec3<f32> {
    return min(color_a + color_b, vec3<f32>(1.0));
}

// Difference blend
fn blend_difference(color_a: vec3<f32>, color_b: vec3<f32>) -> vec3<f32> {
    return abs(color_a - color_b);
}

// Overlay blend
fn blend_overlay(color_a: vec3<f32>, color_b: vec3<f32>) -> vec3<f32> {
    let r = select(
        1.0 - 2.0 * (1.0 - color_a.r) * (1.0 - color_b.r),
        2.0 * color_a.r * color_b.r,
        color_a.r < 0.5
    );
    let g = select(
        1.0 - 2.0 * (1.0 - color_a.g) * (1.0 - color_b.g),
        2.0 * color_a.g * color_b.g,
        color_a.g < 0.5
    );
    let b = select(
        1.0 - 2.0 * (1.0 - color_a.b) * (1.0 - color_b.b),
        2.0 * color_a.b * color_b.b,
        color_a.b < 0.5
    );
    return vec3<f32>(r, g, b);
}

// ============================================
// WAVESHAPERS - Signal Processors
// ============================================

// Soft clipping (tube-style saturation)
fn soft_clip(x: f32) -> f32 {
    return x / (1.0 + abs(x));
}

// Wave folding (Paik/Abe style)
fn fold(x: f32, gain: f32) -> f32 {
    var v = x * gain;
    v = abs(fract(v * 0.5 + 0.25) * 2.0 - 1.0);
    return v;
}

// Quantizer (Sandin Amplitude Classifier)
fn quantize(x: f32, levels: f32) -> f32 {
    return floor(x * levels) / (levels - 1.0);
}

// Solarize effect
fn solarize(x: f32, threshold: f32) -> f32 {
    if x > threshold {
        return 1.0 - x;
    }
    return x;
}

// ============================================
// COLORIZERS - Luminance to RGB
// ============================================

// Spectrum colorizer (rainbow gradient)
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

// Threshold colorizer (Jones style)
fn colorize_threshold(luma: f32, levels: f32) -> vec3<f32> {
    let level = floor(luma * levels);
    let t = level / levels;
    return colorize_spectrum(t);
}

// ============================================
// VIDEO FEEDBACK
// ============================================

// Sample feedback with transform (zoom, rotate, offset)
fn sample_feedback(uv: vec2<f32>) -> vec3<f32> {
    // Center coordinates
    var centered = uv - 0.5;

    // Apply zoom
    centered = centered / feedback.zoom;

    // Apply rotation
    let cos_r = cos(feedback.rotation);
    let sin_r = sin(feedback.rotation);
    let rotated = vec2<f32>(
        centered.x * cos_r - centered.y * sin_r,
        centered.x * sin_r + centered.y * cos_r
    );

    // Apply offset
    var fb_uv = rotated + 0.5 + vec2<f32>(feedback.offset_x, feedback.offset_y);

    // Sample feedback texture
    var fb_color = textureSample(feedback_texture, feedback_sampler, fb_uv).rgb;

    // Apply hue shift
    var hsv = rgb_to_hsv(fb_color);
    hsv.x = fract(hsv.x + feedback.hue_shift);
    hsv.y = hsv.y * feedback.saturation;
    fb_color = hsv_to_rgb(hsv);

    // Apply decay
    fb_color = fb_color * feedback.decay;

    return fb_color;
}

// ============================================
// RUTT/ETRA Z-AXIS DISPLACEMENT
// ============================================

// The signature Rutt/Etra effect: luminance displaces vertical position
// Creates the iconic topographic 3D terrain appearance
// z_amount controls the displacement strength (0 = off, 0.1-0.3 typical)
fn rutt_etra_displacement(uv: vec2<f32>, z_amount: f32) -> vec2<f32> {
    // Sample the signal at this position to get luminance for displacement
    // We use the feedback texture as the displacement source
    let sample_color = textureSample(feedback_texture, feedback_sampler, uv).rgb;
    let luma = dot(sample_color, vec3<f32>(0.299, 0.587, 0.114));

    // Displace Y based on luminance (bright = up, dark = down)
    // Center around 0.5 so mid-gray = no displacement
    let displacement = (luma - 0.5) * z_amount;

    return vec2<f32>(uv.x, uv.y + displacement);
}

// Alternative: displace based on generated signal rather than feedback
fn rutt_etra_signal_displacement(uv: vec2<f32>, signal: f32, z_amount: f32) -> vec2<f32> {
    let displacement = (signal - 0.5) * z_amount;
    return vec2<f32>(uv.x, uv.y + displacement);
}

// ============================================
// CRT EMULATION
// ============================================

// Scanline effect
fn scanlines(uv: vec2<f32>, intensity: f32) -> f32 {
    let line = sin(uv.y * timing.lines * 3.14159) * 0.5 + 0.5;
    return mix(1.0, line, intensity);
}

// Phosphor glow
fn phosphor_glow(color: vec3<f32>, intensity: f32) -> vec3<f32> {
    let bloom = color * intensity;
    return color + bloom * 0.2;
}

// Screen curvature (barrel distortion)
fn crt_curve(uv: vec2<f32>, amount: f32) -> vec2<f32> {
    let centered = uv - 0.5;
    let dist = dot(centered, centered);
    let curved = centered * (1.0 + dist * amount);
    return curved + 0.5;
}

// Vignette (darkening at edges)
fn vignette(uv: vec2<f32>, amount: f32) -> f32 {
    let centered = uv - 0.5;
    let dist = dot(centered, centered);
    return 1.0 - dist * amount;
}

// ============================================
// FRAGMENT SHADER - Main synthesis
// ============================================

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;
    let time = timing.time;

    // Apply CRT screen curvature (from UI)
    uv = crt_curve(uv, synth.curvature);

    // Check if we're outside the curved screen
    if uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    // ========== RUTT/ETRA Z-AXIS DISPLACEMENT ==========

    var safe_uv = uv;
    if synth.rutt_etra_enabled > 0.5 {
        // Animate displacement slightly
        let z_amount = synth.rutt_etra_amount * (1.0 + sin(time * 0.3) * 0.3);
        let displaced_uv = rutt_etra_displacement(uv, z_amount);
        safe_uv = clamp(displaced_uv, vec2<f32>(0.0), vec2<f32>(1.0));
    }

    // ========== VIDEO FEEDBACK ==========

    let fb_color = sample_feedback(safe_uv);

    // ========== SIGNAL GENERATION ==========

    // Base signal from radial ramp
    let radial = ramp_radial(safe_uv);

    // Oscillators with UI-controlled parameters
    let osc1 = osc_h(safe_uv, synth.osc1_freq, time * 0.5 + synth.osc1_phase);
    let osc2 = osc_v(safe_uv, synth.osc2_freq, time * 0.3 + synth.osc2_phase);

    // Noise based on selected type
    var noise_val = 0.0;
    if synth.noise_type < 0.5 {
        noise_val = noise_white(safe_uv, time);
    } else if synth.noise_type < 1.5 {
        noise_val = noise_pink(safe_uv, time);
    } else {
        noise_val = noise_brown(safe_uv, time);
    }

    // Shape generation based on UI selection
    let shape_center = vec2<f32>(0.5 + sin(time * 0.4) * 0.1, 0.5 + cos(time * 0.3) * 0.1);
    var shape_val = 1.0;
    if synth.shape_type > 0.5 && synth.shape_type < 1.5 {
        // Circle
        shape_val = shape_circle_soft(safe_uv, shape_center, synth.shape_size, synth.shape_softness);
    } else if synth.shape_type > 1.5 && synth.shape_type < 2.5 {
        // Rectangle
        shape_val = shape_rect_soft(safe_uv, shape_center, vec2<f32>(synth.shape_size), synth.shape_softness);
    } else if synth.shape_type > 2.5 && synth.shape_type < 3.5 {
        // Diamond
        shape_val = shape_diamond_soft(safe_uv, shape_center, synth.shape_size, synth.shape_softness);
    } else if synth.shape_type > 3.5 && synth.shape_type < 4.5 {
        // Star
        shape_val = f32(shape_star(safe_uv, shape_center, synth.shape_size, synth.shape_size * 0.4, 5.0));
    } else if synth.shape_type > 4.5 && synth.shape_type < 5.5 {
        // Bars
        shape_val = shape_bars_h(safe_uv, 8.0, 0.5);
    } else if synth.shape_type > 5.5 {
        // Checker
        shape_val = shape_checker(safe_uv, 8.0);
    }

    // Mix signals
    var signal = radial;
    signal = mix(signal, osc1, 0.3);
    signal = mix(signal, osc2, 0.2);
    signal = mix(signal, noise_val * 0.5 + 0.5, synth.noise_mix);
    signal = signal * mix(0.8, 1.0, shape_val);

    // ========== SIGNAL PROCESSING ==========

    // Apply wave folding
    let folded = fold(signal, 2.0 + sin(time * 0.7));

    // Quantize with UI-controlled levels
    let quantized = quantize(folded, synth.colorizer_levels);

    // Mix processed signals
    var processed = mix(folded, quantized, 0.5);

    // ========== COLORIZATION ==========

    var new_color: vec3<f32>;
    if synth.colorizer_type < 0.5 {
        // Spectrum
        new_color = colorize_spectrum(processed);
    } else if synth.colorizer_type < 1.5 {
        // Threshold
        new_color = colorize_threshold(processed, synth.colorizer_levels);
    } else {
        // Grayscale
        new_color = vec3<f32>(processed);
    }

    // ========== LUMA KEY ==========

    if synth.key_enabled > 0.5 {
        var key = luma_key(new_color, synth.key_clip_black, synth.key_clip_white);
        if synth.key_invert > 0.5 {
            key = 1.0 - key;
        }
        // Use key to mix with feedback
        new_color = key_mix(fb_color, new_color, key);
    }

    // ========== MIX WITH FEEDBACK ==========

    var color = mix(new_color, fb_color, feedback.mix);

    // ========== CRT EFFECTS ==========

    // Apply scanlines (from UI)
    let scan = scanlines(uv, synth.scanline_intensity);
    color = color * scan;

    // Apply vignette (from UI)
    let vig = vignette(uv, synth.vignette_amount);
    color = color * vig;

    // Phosphor glow
    color = phosphor_glow(color, 0.2);

    return vec4<f32>(color, 1.0);
}
