// ============================================
// PHOSPHLUX v2 - Common Shader Definitions
// ============================================
//
// This file is included in all composed shaders.
// Contains timing uniforms, vertex shader, and shared utilities.

// === TIMING UNIFORMS ===
// Matches TimingUniforms in timing.rs (64 bytes)
struct Timing {
    field: u32,              // Current field (0=even, 1=odd)
    frame: u32,              // Frame counter
    frame_phase: f32,        // Phase within frame [0, 1]
    time: f32,               // Seconds since start
    h_phase: f32,            // Horizontal phase (derived from UV)
    v_phase: f32,            // Vertical phase (derived from UV)
    lines: f32,              // Visible scanlines (480 NTSC, 576 PAL)
    field_rate: f32,         // Field rate Hz (59.94 NTSC, 50.0 PAL)
    subcarrier_phase: f32,   // Color subcarrier phase
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
    _pad3: f32,
    _pad4: f32,
    _pad5: f32,
    _pad6: f32,
}

// === BIND GROUPS ===
// Group 0: Global timing
@group(0) @binding(0)
var<uniform> timing: Timing;

// Group 1: Device textures (feedback, inputs)
// Bound per-device by the shader composer

// Group 2: Device parameters
// Bound per-device by the shader composer

// === VERTEX SHADER ===
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;
    return out;
}

// === SIGNAL UTILITIES ===

// NTSC luma coefficients
const LUMA_R: f32 = 0.299;
const LUMA_G: f32 = 0.587;
const LUMA_B: f32 = 0.114;

// Convert RGB to luminance (NTSC standard)
fn to_luma(rgb: vec3<f32>) -> f32 {
    return dot(rgb, vec3<f32>(LUMA_R, LUMA_G, LUMA_B));
}

// Convert luminance to RGB (grayscale)
fn from_luma(luma: f32) -> vec3<f32> {
    return vec3<f32>(luma);
}

// Clamp signal to video range [0, 1]
fn video_clamp(v: f32) -> f32 {
    return clamp(v, 0.0, 1.0);
}

fn video_clamp3(rgb: vec3<f32>) -> vec3<f32> {
    return clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0));
}

// Soft clip (tube-style saturation)
fn soft_clip(x: f32) -> f32 {
    return x / (1.0 + abs(x));
}

fn soft_clip3(rgb: vec3<f32>) -> vec3<f32> {
    return rgb / (vec3<f32>(1.0) + abs(rgb));
}

// === COLOR SPACE ===

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
    return vec3<f32>(h, s, cmax);
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

// Fast hue rotation (Rodrigues formula around gray axis)
fn rotate_hue(color: vec3<f32>, angle: f32) -> vec3<f32> {
    let k = vec3<f32>(0.57735026919); // 1/sqrt(3) - normalized (1,1,1) axis
    let cos_a = cos(angle);
    let sin_a = sin(angle);
    return color * cos_a + cross(k, color) * sin_a + k * dot(k, color) * (1.0 - cos_a);
}

// RGB to YIQ (NTSC native)
fn rgb_to_yiq(rgb: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        0.299 * rgb.r + 0.587 * rgb.g + 0.114 * rgb.b,
        0.5959 * rgb.r - 0.2746 * rgb.g - 0.3213 * rgb.b,
        0.2115 * rgb.r - 0.5227 * rgb.g + 0.3112 * rgb.b
    );
}

// YIQ to RGB
fn yiq_to_rgb(yiq: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        yiq.x + 0.956 * yiq.y + 0.619 * yiq.z,
        yiq.x - 0.272 * yiq.y - 0.647 * yiq.z,
        yiq.x - 1.106 * yiq.y + 1.703 * yiq.z
    );
}

// === NOISE FUNCTIONS ===

// Hash function for pseudo-random
fn hash(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    p3 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

// Value noise (smooth)
fn noise_value(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let a = hash(i);
    let b = hash(i + vec2<f32>(1.0, 0.0));
    let c = hash(i + vec2<f32>(0.0, 1.0));
    let d = hash(i + vec2<f32>(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// === GEOMETRIC UTILITIES ===

// Center coordinates around 0.5
fn center_uv(uv: vec2<f32>) -> vec2<f32> {
    return uv - 0.5;
}

// Apply zoom (scale around center)
fn apply_zoom(uv: vec2<f32>, zoom: f32) -> vec2<f32> {
    return center_uv(uv) / zoom + 0.5;
}

// Apply rotation (around center)
fn apply_rotation(uv: vec2<f32>, angle: f32) -> vec2<f32> {
    let centered = center_uv(uv);
    let cos_a = cos(angle);
    let sin_a = sin(angle);
    let rotated = vec2<f32>(
        centered.x * cos_a - centered.y * sin_a,
        centered.x * sin_a + centered.y * cos_a
    );
    return rotated + 0.5;
}

// Barrel distortion (CRT curvature)
fn barrel_distort(uv: vec2<f32>, amount: f32) -> vec2<f32> {
    let centered = center_uv(uv);
    let dist_sq = dot(centered, centered);
    return centered * (1.0 + dist_sq * amount) + 0.5;
}

// Check if UV is in bounds
fn in_bounds(uv: vec2<f32>) -> bool {
    return uv.x >= 0.0 && uv.x <= 1.0 && uv.y >= 0.0 && uv.y <= 1.0;
}
