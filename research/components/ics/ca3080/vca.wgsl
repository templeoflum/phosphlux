// CA3080 OTA-based Voltage Controlled Amplifier
// Uses OTA transfer function for CV-controlled gain
// Classic Sandin Image Processor technique
//
// The CA3080 OTA's transconductance (gm) is controlled by bias current,
// making it ideal for voltage-controlled amplifiers. The tanh()
// characteristic provides natural soft clipping.

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Params {
    gain: f32,      // Base gain (0-2)
    offset: f32,    // Output offset
    mix: f32,       // Wet/dry mix
    fold: f32,      // CV/LFO amount
    bias: f32,      // OTA bias (affects softness)
    frequency: f32, // LFO frequency
    phase: f32,     // LFO phase
    threshold: f32, // Soft clip amount
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
    _pad3: f32,
    _pad4: f32,
    _pad5: f32,
    _pad6: f32,
    _pad7: f32,
};

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;
@group(0) @binding(2) var<uniform> params: Params;

// === CA3080 OTA Model ===
// From research/components/ics/ca3080/model.wgsl
//
// OTA transfer function: Iout = gm * (V+ - V-)
// gm is proportional to bias current (Iabc)
// The tanh gives soft saturation like real OTAs

// VCA using OTA characteristic
// CV directly controls gain, softness controls clipping hardness
fn ota_vca(signal: f32, cv: f32, softness: f32) -> f32 {
    let gain = cv;
    // Softness controls how hard the clipping is
    let scaled = signal * gain / max(softness, 0.1);
    return tanh(scaled) * softness;
}

// === Shader ===

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0)
    );
    var uvs = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(2.0, 1.0),
        vec2<f32>(0.0, -1.0)
    );

    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    output.uv = uvs[vertex_index];
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(input_texture, input_sampler, input.uv);

    // Generate spatial LFO (creates horizontal stripes of varying gain)
    let lfo = sin(input.uv.x * params.frequency * 6.28318 + params.phase) * 0.5 + 0.5;

    // Mix between static gain and LFO-modulated gain
    let cv = mix(params.gain, lfo * params.gain * 2.0, params.fold);

    // Softness from bias (higher = softer clipping)
    let softness = 0.3 + params.bias * 0.7;

    // Process each channel through OTA VCA
    // Center around 0 for proper VCA behavior
    var r = ota_vca(color.r - 0.5, cv, softness) + 0.5;
    var g = ota_vca(color.g - 0.5, cv, softness) + 0.5;
    var b = ota_vca(color.b - 0.5, cv, softness) + 0.5;

    // Add offset
    let processed = vec3<f32>(r, g, b) + params.offset;

    // Mix with original
    let output = mix(color.rgb, processed, params.mix);

    return vec4<f32>(clamp(output, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
