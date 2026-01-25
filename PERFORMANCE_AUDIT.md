# Performance Audit: Phosphlux

*Analysis of current performance characteristics and optimization opportunities*

---

## Current Architecture Summary

### Render Pipeline
```
CPU: Timing → Uniforms → Queue.write_buffer()
                           ↓
GPU: Pass 1 (Feedback) → Pass 2 (Output) → Present
          ↓                    ↓
      ping-pong           egui texture
      buffers             for preview
```

### Per-Frame Operations
1. **CPU Side**
   - Timing calculation (SyncGenerator)
   - Uniform buffer updates (3x `queue.write_buffer`)
   - Command encoder creation
   - egui UI layout/rendering

2. **GPU Side**
   - 2 render passes per frame
   - Full-screen fragment shader (~730 lines WGSL)
   - Texture sampling (feedback)
   - HSV↔RGB conversions

---

## Performance Bottlenecks Identified

### 1. **Double Rendering** (MODERATE)
```rust
// render_to_texture() runs the full shader TWICE:
// Pass 1: Render to feedback texture
// Pass 2: Render to output texture (for egui)
```
**Impact**: 2x GPU fragment shader invocations
**Why it exists**: Ping-pong buffers need to update AND display

**Optimization**: Use `encoder.copy_texture_to_texture()` instead of re-rendering:
```rust
// After Pass 1, just copy:
encoder.copy_texture_to_texture(
    feedback_textures[write_index],
    output_texture,
    ...
);
```
**Expected gain**: ~40% reduction in GPU work

### 2. **HSV Conversions in Feedback Loop** (MODERATE)
```wgsl
// Every pixel does this EVERY FRAME:
var hsv = rgb_to_hsv(fb_color);     // 15+ operations
hsv.x = fract(hsv.x + feedback.hue_shift);
fb_color = hsv_to_rgb(hsv);         // 20+ operations
```
**Impact**: ~35 ALU operations per pixel just for hue shift

**Optimization**: Use direct RGB hue rotation matrix:
```wgsl
fn rotate_hue(color: vec3<f32>, angle: f32) -> vec3<f32> {
    let k = vec3<f32>(0.57735);  // 1/sqrt(3)
    let cos_a = cos(angle * 6.283185);
    let sin_a = sin(angle * 6.283185);
    return color * cos_a + cross(k, color) * sin_a + k * dot(k, color) * (1.0 - cos_a);
}
```
**Expected gain**: ~60% faster hue shifting

### 3. **Noise Generation** (LOW-MODERATE)
```wgsl
// Pink noise does 5 octaves:
for (var i = 0; i < 5; i++) {
    value += noise_value(uv * frequency + offset) * amplitude;
    // noise_value() samples 4 corners + interpolation
}
```
**Impact**: ~20 texture-like operations per pixel

**Optimization options**:
- Pre-baked noise texture (massive speedup, slightly less dynamic)
- Reduce octaves based on zoom level
- Use GPU texture LOD for automatic octave selection

### 4. **Branching in Fragment Shader** (LOW)
```wgsl
if synth.noise_type < 0.5 { ... }
else if synth.noise_type < 1.5 { ... }
else { ... }
```
**Impact**: GPU branch divergence (warps wait for each other)

**Optimization**: Pre-compute noise selection on CPU, or use mix():
```wgsl
let white = noise_white(...);
let pink = noise_pink(...);
let brown = noise_brown(...);
// blend based on type (wastes some compute but no branches)
```

### 5. **Uniform Buffer Updates** (LOW)
```rust
// 3 separate buffer writes per frame:
self.queue.write_buffer(&self.timing_buffer, ...);
self.queue.write_buffer(&self.feedback_buffer, ...);
self.queue.write_buffer(&self.synth_buffer, ...);
```
**Impact**: 3 CPU→GPU transfers vs 1

**Optimization**: Single combined uniform struct:
```rust
struct CombinedUniforms {
    timing: TimingUniforms,
    feedback: FeedbackUniforms,
    synth: SynthUniforms,
}
// One write_buffer() call
```

---

## "Exponential Gain" Opportunities

### A. **Compute Shader Pre-pass** (HIGH POTENTIAL)
Move synthesis to compute shader, run at lower resolution, upsample:
```
Compute (256x256) → Feedback → Upscale (via fragment)
```
**Potential**: 9x fewer pixel shader invocations at 768x576

### B. **Temporal Reprojection** (HIGH POTENTIAL)
Don't re-render every frame — reproject previous frame when parameters haven't changed:
```rust
if params_unchanged && time_delta < threshold {
    // Just display previous frame with time offset
}
```
**Potential**: Near-zero GPU cost for static scenes

### C. **Feedback Resolution Independence** (MEDIUM)
Run feedback loop at fixed 512x512, only final composite at full res:
```
Feedback (512x512) → Sample in full-res shader
```
**Potential**: 4x reduction in feedback texture bandwidth

### D. **Subgroup Operations** (MEDIUM)
WGSL subgroup operations for parallel reductions:
```wgsl
// Share noise values within wave/subgroup
let shared_noise = subgroupBroadcastFirst(noise_val);
```
**Requires**: Feature detection, not all GPUs support

### E. **Async Compute** (MEDIUM)
Overlap compute work with UI rendering:
```rust
// Start GPU work early
encoder.submit();
// Do CPU work (egui layout) while GPU runs
// Present when both done
```

---

## Quick Wins (Implement First)

1. **Copy instead of re-render** - Easy, big impact
2. **Hue rotation matrix** - Pure shader change
3. **Single uniform buffer** - Clean refactor
4. **Lower feedback resolution** - Configuration option

---

## Measurement Approach

Before optimizing, instrument:
```rust
// Frame timing
let start = std::time::Instant::now();
renderer.render_to_texture(timing);
queue.submit(...);
device.poll(wgpu::Maintain::Wait);  // Force GPU sync for accurate timing
let gpu_time = start.elapsed();
```

Or use wgpu timestamp queries for precise GPU timing.

---

## Priority Ranking

| Optimization | Effort | Impact | Priority |
|--------------|--------|--------|----------|
| Copy vs re-render | Low | High | **1** |
| Hue rotation matrix | Low | Medium | **2** |
| Single uniform buffer | Low | Low | 3 |
| Lower feedback res | Medium | High | **4** |
| Compute pre-pass | High | Very High | 5 |
| Temporal reprojection | High | Very High | 6 |

---

## Next Steps

1. Implement Priority 1-2 (quick wins)
2. Add frame time instrumentation
3. Profile with specific GPU (behavior varies by vendor)
4. Consider making feedback resolution a user setting
5. Evaluate compute shader approach for Phase 3 (GPU signal routing)

---

*Audit Date: January 2026*
