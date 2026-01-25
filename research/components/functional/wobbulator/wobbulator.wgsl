// Wobbulator - Paik/Abe Scan Displacement Effect
// CRT raster modulation creating warping and wobbling
//
// Workbench Parameters:
//   gain      → Horizontal oscillator frequency
//   offset    → Horizontal amplitude
//   mix       → Vertical oscillator frequency
//   fold      → Vertical amplitude
//   bias      → Video-driven displacement amount
//   frequency → Animation speed
//   phase     → Phase offset
//   threshold → Edge mode (0=clamp, 0.5=wrap, 1=mirror)

struct Params {
    gain: f32,      // H frequency
    offset: f32,    // H amplitude
    mix: f32,       // V frequency
    fold: f32,      // V amplitude
    bias: f32,      // Video mod
    frequency: f32, // Speed
    phase: f32,     // Phase
    threshold: f32, // Edge mode
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
// OSCILLATORS
// ============================================================================

fn osc_sine(phase: f32) -> f32 {
    return sin(phase * TAU);
}

fn osc_triangle(phase: f32) -> f32 {
    let p = fract(phase);
    return abs(p * 4.0 - 2.0) - 1.0;
}

// ============================================================================
// DISPLACEMENT
// ============================================================================

fn displace_oscillator(uv: vec2<f32>, h_freq: f32, h_amp: f32, v_freq: f32, v_amp: f32, time: f32, phase: f32) -> vec2<f32> {
    // Horizontal displacement based on Y (creates vertical ripples)
    let h_phase = uv.y * h_freq + time + phase;
    let h_disp = osc_sine(h_phase) * h_amp;

    // Vertical displacement based on X (creates horizontal ripples)
    let v_phase = uv.x * v_freq + time * 0.7 + phase * 1.3;
    let v_disp = osc_sine(v_phase) * v_amp;

    return vec2<f32>(h_disp, v_disp);
}

fn displace_video(uv: vec2<f32>, amount: f32) -> vec2<f32> {
    // Sample luminance at current position
    let color = textureSample(input_texture, input_sampler, uv);
    let luma = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));

    // Center around 0.5, so mid-gray = no displacement
    let luma_centered = luma - 0.5;

    // Radial displacement based on luminance
    let center = vec2<f32>(0.5, 0.5);
    let to_center = normalize(uv - center);

    return to_center * luma_centered * amount;
}

fn apply_edge_mode(uv: vec2<f32>, mode: f32) -> vec2<f32> {
    if (mode < 0.33) {
        // Clamp
        return clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0));
    } else if (mode < 0.66) {
        // Wrap
        return fract(uv);
    } else {
        // Mirror
        let wrapped = fract(uv);
        let flip_x = u32(floor(uv.x)) & 1u;
        let flip_y = u32(floor(uv.y)) & 1u;
        return vec2<f32>(
            select(wrapped.x, 1.0 - wrapped.x, flip_x == 1u),
            select(wrapped.y, 1.0 - wrapped.y, flip_y == 1u)
        );
    }
}

// ============================================================================
// MAIN SHADER
// ============================================================================

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Get parameters with defaults
    let h_freq = select(params.gain, 3.0, params.gain == 0.0);
    let h_amp = select(params.offset, 0.05, params.offset == 0.0);
    let v_freq = select(params.mix, 2.0, params.mix == 0.0);
    let v_amp = select(params.fold, 0.04, params.fold == 0.0);
    let video_mod = select(params.bias, 0.0, params.bias == 0.0);
    let speed = select(params.frequency, 0.5, params.frequency == 0.0);
    let phase = select(params.phase, 0.0, params.phase == 0.0);
    let edge_mode = select(params.threshold, 0.0, params.threshold == 0.0);

    // Simulate time progression using phase parameter
    // In real implementation, time would be a uniform
    let time = phase * 10.0;

    // Calculate oscillator displacement
    let osc_disp = displace_oscillator(in.uv, h_freq, h_amp, v_freq, v_amp, time * speed, phase);

    // Calculate video-driven displacement
    let video_disp = displace_video(in.uv, video_mod);

    // Combine displacements
    let total_disp = osc_disp + video_disp;

    // Apply displacement to UV
    let displaced_uv = in.uv + total_disp;

    // Handle edges
    let final_uv = apply_edge_mode(displaced_uv, edge_mode);

    // Sample at displaced position
    let output_color = textureSample(input_texture, input_sampler, final_uv);

    return output_color;
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
