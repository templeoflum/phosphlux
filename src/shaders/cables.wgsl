// ============================================
// CABLE DEGRADATION FUNCTIONS
// ============================================
//
// Models signal degradation introduced by different cable/connector types.
// Each cable type has distinct characteristics based on physical properties.
//
// Usage: Shader composer inserts appropriate cable function between
// device outputs and inputs based on Connection.cable_type
//
// v0.13 - Partial implementation (stress test)

// ─── Composite Video ─────────────────────────────────────────
// Single RCA cable carrying combined luma + chroma
// Characteristics:
// - ~4MHz bandwidth (NTSC subcarrier at 3.58MHz)
// - Chroma/luma crosstalk (dot crawl, rainbow artifacts)
// - Color bleeding on sharp transitions

fn composite_degrade(color: vec4<f32>, uv: vec2<f32>, time: f32) -> vec4<f32> {
    // Convert to YIQ for NTSC-style processing
    let y = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    let i = dot(color.rgb, vec3<f32>(0.596, -0.274, -0.322));
    let q = dot(color.rgb, vec3<f32>(0.211, -0.523, 0.312));

    // Simulate chroma bandwidth limiting (~1.3MHz for I, ~0.4MHz for Q)
    // This is a simplified model - real implementation would use convolution
    let chroma_blur = 0.003;
    let i_degraded = i * 0.95;  // Slight I channel loss
    let q_degraded = q * 0.85;  // More Q channel loss (narrower bandwidth)

    // Dot crawl artifact (chroma/luma interference at subcarrier frequency)
    let subcarrier_phase = uv.x * 227.5 + time * 60.0;  // ~3.58MHz relative to line
    let dot_crawl = sin(subcarrier_phase * 6.283185) * 0.02;

    // Convert back to RGB with artifacts
    var r = y + 0.956 * i_degraded + 0.621 * q_degraded;
    var g = y - 0.272 * i_degraded - 0.647 * q_degraded;
    var b = y - 1.106 * i_degraded + 1.703 * q_degraded;

    // Add dot crawl to luma
    r = r + dot_crawl;
    g = g + dot_crawl;
    b = b + dot_crawl;

    return vec4<f32>(clamp(vec3<f32>(r, g, b), vec3<f32>(0.0), vec3<f32>(1.0)), color.a);
}

// ─── S-Video ─────────────────────────────────────────────────
// Separate Y (luma) and C (chroma) signals
// TODO: Implement in v0.13 completion
// Characteristics:
// - No chroma/luma crosstalk
// - ~4.2MHz luma bandwidth
// - ~1.3MHz chroma bandwidth

fn svideo_degrade(color: vec4<f32>, uv: vec2<f32>, time: f32) -> vec4<f32> {
    // STUB - not yet implemented
    return color;
}

// ─── Component Video ─────────────────────────────────────────
// Three separate cables for Y, Pb, Pr
// TODO: Implement in v0.13 completion
// Characteristics:
// - Wide bandwidth per channel (~5-6MHz)
// - Minimal crosstalk
// - Cable length affects HF rolloff

fn component_degrade(color: vec4<f32>, uv: vec2<f32>, cable_length_ft: f32) -> vec4<f32> {
    // STUB - not yet implemented
    return color;
}

// ─── RGB ─────────────────────────────────────────────────────
// Direct RGB signals (VGA, SCART RGB mode)
// TODO: Implement in v0.13 completion
// Characteristics:
// - Best quality, no color space conversion
// - HF rolloff proportional to cable length
// - Minor crosstalk on very long cables

fn rgb_degrade(color: vec4<f32>, uv: vec2<f32>, cable_length_ft: f32) -> vec4<f32> {
    // STUB - not yet implemented
    return color;
}
