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

// CRT emulation parameters (16 bytes)
// Signal generation is now handled by the node graph
struct Synth {
    scanline_intensity: f32,
    curvature: f32,
    vignette_amount: f32,
    _pad0: f32,
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

// ============================================
// NODE GRAPH EXECUTION
// ============================================

// Graph header - controls execution
struct GraphHeader {
    op_count: u32,
    main_out_slot: u32,
    feedback_out_slot: u32,
    _pad: u32,
}

// Single graph operation
struct GraphOp {
    opcode: u32,
    input_a: u32,
    input_b: u32,
    output_slot: u32,
    param0: f32,
    param1: f32,
    param2: f32,
    param3: f32,
}

// Operation codes (must match graph_compiler.rs OpCode enum)
const OP_RAMP_H: u32 = 0u;
const OP_RAMP_V: u32 = 1u;
const OP_RAMP_RADIAL: u32 = 2u;
const OP_OSC_H: u32 = 3u;
const OP_OSC_V: u32 = 4u;
const OP_NOISE_WHITE: u32 = 5u;
const OP_NOISE_PINK: u32 = 6u;
const OP_NOISE_BROWN: u32 = 7u;
const OP_SHAPE_CIRCLE: u32 = 8u;
const OP_SHAPE_RECT: u32 = 9u;
const OP_SHAPE_DIAMOND: u32 = 10u;
const OP_FEEDBACK_IN: u32 = 11u;
const OP_CHANNEL_KEY_IN: u32 = 12u;
const OP_CHANNEL_AUX1_IN: u32 = 13u;
const OP_CHANNEL_AUX2_IN: u32 = 14u;
const OP_FOLDER: u32 = 20u;
const OP_QUANTIZER: u32 = 21u;
const OP_SOFT_CLIP: u32 = 22u;
const OP_SOLARIZE: u32 = 23u;
const OP_INVERTER: u32 = 24u;
const OP_HUE_SHIFT: u32 = 25u;
const OP_COLOR_SPECTRUM: u32 = 30u;
const OP_COLOR_THRESHOLD: u32 = 31u;
const OP_COLOR_GRADIENT: u32 = 32u;
const OP_LUMA_KEY: u32 = 40u;
const OP_MIXER: u32 = 41u;
const OP_MULTIPLIER: u32 = 42u;
const OP_CHROMA_KEY: u32 = 43u;
const OP_MAIN_OUT: u32 = 50u;
const OP_FEEDBACK_OUT: u32 = 51u;
const OP_CHANNEL_KEY_OUT: u32 = 52u;
const OP_CHANNEL_AUX1_OUT: u32 = 53u;
const OP_CHANNEL_AUX2_OUT: u32 = 54u;
const OP_NOP: u32 = 255u;

// Graph bindings (group 2)
@group(2) @binding(0)
var<uniform> graph_header: GraphHeader;

@group(2) @binding(1)
var<storage, read> graph_ops: array<GraphOp>;

// Signal slots for intermediate values (16 vec4 slots - reduced for performance)
// Each slot stores RGBA where A is typically 1.0 for RGB signals
// or the luminance is stored in R for grayscale signals
var<private> slots: array<vec4<f32>, 16>;

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

// Fast hue rotation using Rodrigues' rotation formula
// This is MUCH faster than RGB->HSV->rotate->RGB conversion
// Rotates the color around the (1,1,1) axis (gray line) in RGB space
fn rotate_hue_fast(color: vec3<f32>, angle: f32) -> vec3<f32> {
    let k = vec3<f32>(0.57735026919); // 1/sqrt(3) - normalized (1,1,1) axis
    let cos_a = cos(angle);
    let sin_a = sin(angle);

    // Rodrigues rotation: v' = v*cos(a) + (k×v)*sin(a) + k*(k·v)*(1-cos(a))
    return color * cos_a + cross(k, color) * sin_a + k * dot(k, color) * (1.0 - cos_a);
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

    // Apply hue shift using fast rotation (much faster than HSV conversion)
    // Convert hue_shift (0-1) to radians (0-2π)
    let hue_angle = feedback.hue_shift * 6.283185307;
    fb_color = rotate_hue_fast(fb_color, hue_angle);

    // Apply saturation adjustment
    // Mix with luminance to desaturate
    let luma = dot(fb_color, vec3<f32>(0.299, 0.587, 0.114));
    fb_color = mix(vec3<f32>(luma), fb_color, feedback.saturation);

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
// NODE GRAPH EVALUATION
// ============================================

// Execute a single graph operation
fn execute_op(op: GraphOp, uv: vec2<f32>, time: f32) {
    let opcode = op.opcode;
    var result = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    // Get input values (clamped to 16 slots for performance)
    var input_a = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    var input_b = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    if op.input_a < 16u {
        input_a = slots[op.input_a];
    }
    if op.input_b < 16u {
        input_b = slots[op.input_b];
    }

    // Source operations (no inputs)
    if opcode == OP_RAMP_H {
        result = vec4<f32>(vec3<f32>(ramp_h(uv)), 1.0);
    } else if opcode == OP_RAMP_V {
        result = vec4<f32>(vec3<f32>(ramp_v(uv)), 1.0);
    } else if opcode == OP_RAMP_RADIAL {
        result = vec4<f32>(vec3<f32>(ramp_radial(uv)), 1.0);
    } else if opcode == OP_OSC_H {
        let freq = select(4.0, op.param0, op.param0 > 0.0);
        result = vec4<f32>(vec3<f32>(osc_h(uv, freq, time * 0.5 + op.param1)), 1.0);
    } else if opcode == OP_OSC_V {
        let freq = select(4.0, op.param0, op.param0 > 0.0);
        result = vec4<f32>(vec3<f32>(osc_v(uv, freq, time * 0.5 + op.param1)), 1.0);
    } else if opcode == OP_NOISE_WHITE {
        result = vec4<f32>(vec3<f32>(noise_white(uv, time)), 1.0);
    } else if opcode == OP_NOISE_PINK {
        result = vec4<f32>(vec3<f32>(noise_pink(uv, time)), 1.0);
    } else if opcode == OP_NOISE_BROWN {
        result = vec4<f32>(vec3<f32>(noise_brown(uv, time)), 1.0);
    } else if opcode == OP_SHAPE_CIRCLE {
        let center = vec2<f32>(0.5 + sin(time * 0.4) * 0.1, 0.5 + cos(time * 0.3) * 0.1);
        let size = select(0.3, op.param0, op.param0 > 0.0);
        let soft = select(0.05, op.param1, op.param1 > 0.0);
        result = vec4<f32>(vec3<f32>(shape_circle_soft(uv, center, size, soft)), 1.0);
    } else if opcode == OP_SHAPE_RECT {
        let center = vec2<f32>(0.5 + sin(time * 0.4) * 0.1, 0.5 + cos(time * 0.3) * 0.1);
        let size = select(0.3, op.param0, op.param0 > 0.0);
        let soft = select(0.05, op.param1, op.param1 > 0.0);
        result = vec4<f32>(vec3<f32>(shape_rect_soft(uv, center, vec2<f32>(size), soft)), 1.0);
    } else if opcode == OP_SHAPE_DIAMOND {
        let center = vec2<f32>(0.5 + sin(time * 0.4) * 0.1, 0.5 + cos(time * 0.3) * 0.1);
        let size = select(0.3, op.param0, op.param0 > 0.0);
        let soft = select(0.05, op.param1, op.param1 > 0.0);
        result = vec4<f32>(vec3<f32>(shape_diamond_soft(uv, center, size, soft)), 1.0);
    } else if opcode == OP_FEEDBACK_IN {
        // Sample feedback with transform
        result = vec4<f32>(sample_feedback(uv), 1.0);
    }
    // Single-input processors
    else if opcode == OP_FOLDER {
        let gain = select(2.0, op.param0, op.param0 > 0.0);
        let luma = dot(input_a.rgb, vec3<f32>(0.299, 0.587, 0.114));
        result = vec4<f32>(vec3<f32>(fold(luma, gain)), 1.0);
    } else if opcode == OP_QUANTIZER {
        let levels = select(8.0, op.param0, op.param0 > 0.0);
        let luma = dot(input_a.rgb, vec3<f32>(0.299, 0.587, 0.114));
        result = vec4<f32>(vec3<f32>(quantize(luma, levels)), 1.0);
    } else if opcode == OP_SOFT_CLIP {
        let luma = dot(input_a.rgb, vec3<f32>(0.299, 0.587, 0.114));
        result = vec4<f32>(vec3<f32>(soft_clip(luma * 2.0 - 1.0) * 0.5 + 0.5), 1.0);
    } else if opcode == OP_SOLARIZE {
        let threshold = select(0.5, op.param0, op.param0 > 0.0);
        let luma = dot(input_a.rgb, vec3<f32>(0.299, 0.587, 0.114));
        result = vec4<f32>(vec3<f32>(solarize(luma, threshold)), 1.0);
    } else if opcode == OP_INVERTER {
        result = vec4<f32>(vec3<f32>(1.0) - input_a.rgb, 1.0);
    } else if opcode == OP_HUE_SHIFT {
        let amount = op.param0 * 6.283185307;
        result = vec4<f32>(rotate_hue_fast(input_a.rgb, amount), 1.0);
    }
    // Colorizers
    else if opcode == OP_COLOR_SPECTRUM {
        let luma = dot(input_a.rgb, vec3<f32>(0.299, 0.587, 0.114));
        result = vec4<f32>(colorize_spectrum(luma), 1.0);
    } else if opcode == OP_COLOR_THRESHOLD {
        let levels = select(8.0, op.param0, op.param0 > 0.0);
        let luma = dot(input_a.rgb, vec3<f32>(0.299, 0.587, 0.114));
        result = vec4<f32>(colorize_threshold(luma, levels), 1.0);
    } else if opcode == OP_COLOR_GRADIENT {
        // Simple gradient from black to white through color
        let luma = dot(input_a.rgb, vec3<f32>(0.299, 0.587, 0.114));
        result = vec4<f32>(vec3<f32>(luma), 1.0);
    }
    // Two-input processors
    else if opcode == OP_LUMA_KEY {
        let clip_black = select(0.2, op.param0, op.param0 > 0.0);
        let clip_white = select(0.8, op.param1, op.param1 > 0.0);
        let key = luma_key(input_a.rgb, clip_black, clip_white);
        result = vec4<f32>(key_mix(input_b.rgb, input_a.rgb, key), 1.0);
    } else if opcode == OP_MIXER {
        let mix_amount = select(0.5, op.param0, op.param0 > 0.0);
        result = vec4<f32>(mix(input_a.rgb, input_b.rgb, mix_amount), 1.0);
    } else if opcode == OP_MULTIPLIER {
        result = vec4<f32>(input_a.rgb * input_b.rgb, 1.0);
    } else if opcode == OP_CHROMA_KEY {
        // Chroma key with input_a as foreground, input_b as background
        let key_color = vec3<f32>(0.0, 1.0, 0.0); // Green screen
        let key = chroma_key(input_a.rgb, key_color, 0.3, 0.1);
        result = vec4<f32>(key_mix(input_b.rgb, input_a.rgb, key), 1.0);
    }
    // Output nodes
    else if opcode == OP_MAIN_OUT || opcode == OP_FEEDBACK_OUT {
        result = input_a;
    }

    // Store result (clamped to 16 slots for performance)
    if op.output_slot < 16u {
        slots[op.output_slot] = result;
    }
}

// Evaluate the entire node graph and return the main output color
fn evaluate_graph(uv: vec2<f32>, time: f32) -> vec4<f32> {
    // Note: slots are private and zero-initialized by default in WGSL
    // We don't need to explicitly initialize them each frame

    // Execute operations - use early exit pattern for better GPU performance
    // The compiler can optimize this better than a dynamic loop
    let op_count = graph_header.op_count;

    if op_count > 0u { execute_op(graph_ops[0], uv, time); }
    if op_count > 1u { execute_op(graph_ops[1], uv, time); }
    if op_count > 2u { execute_op(graph_ops[2], uv, time); }
    if op_count > 3u { execute_op(graph_ops[3], uv, time); }
    if op_count > 4u { execute_op(graph_ops[4], uv, time); }
    if op_count > 5u { execute_op(graph_ops[5], uv, time); }
    if op_count > 6u { execute_op(graph_ops[6], uv, time); }
    if op_count > 7u { execute_op(graph_ops[7], uv, time); }
    if op_count > 8u { execute_op(graph_ops[8], uv, time); }
    if op_count > 9u { execute_op(graph_ops[9], uv, time); }
    if op_count > 10u { execute_op(graph_ops[10], uv, time); }
    if op_count > 11u { execute_op(graph_ops[11], uv, time); }
    if op_count > 12u { execute_op(graph_ops[12], uv, time); }
    if op_count > 13u { execute_op(graph_ops[13], uv, time); }
    if op_count > 14u { execute_op(graph_ops[14], uv, time); }
    if op_count > 15u { execute_op(graph_ops[15], uv, time); }
    // Support up to 16 ops for now - can extend if needed

    // Return main output (clamped to 16 slots)
    if graph_header.main_out_slot < 16u {
        return slots[graph_header.main_out_slot];
    }

    // Fallback: return first slot
    return slots[0];
}

// ============================================
// FRAGMENT SHADER - Main synthesis
// ============================================

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;
    let time = timing.time;

    // Apply CRT screen curvature
    uv = crt_curve(uv, synth.curvature);

    // Check if we're outside the curved screen
    if uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    // ========== NODE GRAPH EVALUATION ==========
    // All signal generation and processing is handled by the node graph
    let graph_result = evaluate_graph(uv, time);
    var color = graph_result.rgb;

    // ========== CRT EFFECTS ==========
    // These are global post-processing effects applied after the node graph

    // Apply scanlines
    let scan = scanlines(uv, synth.scanline_intensity);
    color = color * scan;

    // Apply vignette
    let vig = vignette(uv, synth.vignette_amount);
    color = color * vig;

    // Phosphor glow
    color = phosphor_glow(color, 0.2);

    return vec4<f32>(color, 1.0);
}
