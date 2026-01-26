# Device Workbench Development Workflow

The device workbench (`tools/device-workbench/workbench-app/`) is a Rust/wgpu application that hot-loads WGSL shaders via an HTTP API (port 3030). It exposes MCP tools that allow Claude to load shaders, adjust parameters, capture screenshots, and compare output against reference imagery.

## Starting the Workbench

```
cd tools/device-workbench/workbench-app
cargo run --release
```

The workbench opens a GPU-accelerated window and starts an HTTP server on `http://localhost:3030`.

## WGSL Shader Format

All workbench shaders must follow this structure:

### Params Struct (64 bytes, 16 x f32)

```wgsl
struct Params {
    gain: f32,      // Slot 0
    offset: f32,    // Slot 1
    mix: f32,       // Slot 2
    fold: f32,      // Slot 3
    bias: f32,      // Slot 4
    frequency: f32, // Slot 5
    phase: f32,     // Slot 6
    threshold: f32, // Slot 7
    _pad0: f32, _pad1: f32, _pad2: f32, _pad3: f32,
    _pad4: f32, _pad5: f32, _pad6: f32, _pad7: f32,
}
```

Each shader documents which named slot maps to what via a header comment block:

```wgsl
// Workbench Parameters:
//   gain      -> fold amount
//   offset    -> DC bias
//   mix       -> dry/wet blend
//   ...
```

### Required Bindings

```wgsl
@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;
@group(0) @binding(2) var<uniform> params: Params;
```

### Required Entry Points

- `vs_main` -- Vertex shader (fullscreen triangle):
  ```wgsl
  @vertex
  fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
      var out: VertexOutput;
      let x = f32((vi & 1u) << 2u) - 1.0;
      let y = f32((vi & 2u) << 1u) - 1.0;
      out.position = vec4<f32>(x, y, 0.0, 1.0);
      out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
      return out;
  }
  ```

- `fs_main` -- Fragment shader (processing logic):
  ```wgsl
  @fragment
  fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
      let input_color = textureSample(input_texture, input_sampler, in.uv);
      // ... processing ...
      return vec4<f32>(output, 1.0);
  }
  ```

### VertexOutput Struct

```wgsl
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}
```

## MCP Tool Reference

### Shader Loading

| Tool | Purpose |
|------|---------|
| `workbench_load_shader` | Hot-reload WGSL code into the workbench |
| `workbench_get_state` | Get current workbench state (shader, params, errors) |

### Parameter Control

| Tool | Purpose |
|------|---------|
| `workbench_set_params` | Set parameter values (key-value pairs, 0.0-1.0 unless specified) |
| `workbench_get_params` | Read current parameter values and ranges |

### Input Signals

| Tool | Purpose |
|------|---------|
| `workbench_inject_signal` | Load a test pattern as shader input |

Available signal types:
- `ramp_h` -- Horizontal luminance ramp (black left, white right)
- `ramp_v` -- Vertical luminance ramp (black top, white bottom)
- `colorbars` -- SMPTE-style color bars
- `sweep` -- Frequency sweep pattern
- `gray50` -- 50% gray field
- `white` -- White field
- `black` -- Black field
- `checker` -- Checkerboard pattern

### Capture and Analysis

| Tool | Purpose |
|------|---------|
| `workbench_screenshot` | Capture current output as image |
| `workbench_analyze` | Vision model analysis of current output |
| `workbench_doc_capture` | Capture screenshot with metadata for documentation |

### Reference Comparison

| Tool | Purpose |
|------|---------|
| `workbench_load_reference` | Load a reference image for comparison |
| `workbench_diff` | Pixel-level comparison vs reference (similarity, MAE, peak_error) |
| `workbench_compare_visual` | Vision model comparison (focus: colors, edges, contrast, overall, artifacts) |

### Sweep and Export

| Tool | Purpose |
|------|---------|
| `workbench_record_sweep` | Animate a parameter sweep as GIF |
| `workbench_export_node` | Export shader as a Phosphlux node |

## Development Loop

The core workflow is iterative:

```
Write WGSL -> load_shader -> screenshot -> vision analysis -> adjust -> iterate
```

### Step-by-Step Calibration Process

1. **Load shader** into the workbench via `workbench_load_shader`
2. **Inject test signal** appropriate for the effect:
   - Colorizers: `ramp_v` (shows amplitude windows as horizontal bands)
   - Mixers: `colorbars` or `ramp_h` (tests mixing behavior)
   - Wavefolders: `ramp_h` (shows fold curves across luminance range)
   - Keyers: `colorbars` (shows key threshold behavior)
   - Displacement: `checker` or `colorbars` (shows spatial modulation)
3. **Set default params** via `workbench_set_params` to establish a baseline
4. **Screenshot + analyze** -- verify output matches expected behavior
5. **Compare against reference** if available:
   - Load reference via `workbench_load_reference`
   - Run `workbench_diff` for quantitative metrics
   - Run `workbench_compare_visual` for qualitative assessment
6. **Sweep key parameters** via `workbench_record_sweep` to verify smooth, continuous response
7. **Adjust shader code** if vision analysis reveals issues, then repeat from step 1
8. **Document** final results via `workbench_doc_capture`

### Expected Visual Behaviors

| Component | Expected Output |
|-----------|----------------|
| Matrix Mixer | Weighted RGB mixing with soft limiting (tanh saturation) |
| PED Colorizer | Distinct color bands at threshold boundaries, smooth transitions |
| Folding Amp | Wrap-around distortion, increasing folds with gain parameter |
| Video Keyer | Clean transitions between two virtual sources at key threshold |
| Wobbulator | Oscillating spatial displacement/modulation of input |

## Parameter Conventions

The 8 named parameter slots have conventional meanings:

| Slot | Name | Convention |
|------|------|------------|
| 0 | `gain` | Primary intensity / amount control |
| 1 | `offset` | Secondary control / DC offset |
| 2 | `mix` | Dry/wet blend or tertiary control |
| 3 | `fold` | Shape / softness / mode control |
| 4 | `bias` | Bias / additional modifier |
| 5 | `frequency` | Frequency / speed / rate |
| 6 | `phase` | Phase offset |
| 7 | `threshold` | Threshold / clip level |

Parameters default to 0.0 when not set. Shaders use `select()` to provide sensible defaults when a parameter is 0.0:

```wgsl
let fold_gain = select(params.gain, 2.0, params.gain == 0.0);
```

This means "use params.gain, but if it's 0.0, use 2.0 instead."

## File Organization

Component shaders live under `research/components/`:

```
research/components/
  ics/
    ca3054/v1/
      matrix_mixer.wgsl    -- CA3054 matrix mixer
      video_keyer.wgsl     -- CA3054 video keyer
    ca3019/v1/
      ped_control.wgsl     -- CA3019 PED colorizer
  functional/
    folding_amp/v1/
      folder.wgsl          -- Folding amplifier
    wobbulator/v1/
      wobbulator.wgsl      -- Scan displacement
    paik_abe_chain/v1/
      signal_chain.wgsl    -- Full composed chain
```

## GPU Packing

The Rust side packs parameters into a 16-element `f32` array (`state.rs:get_params_for_gpu()`):

```
[gain, offset, mix, fold, bias, frequency, phase, threshold, 0, 0, 0, 0, 0, 0, 0, 0]
```

This maps directly to the WGSL `Params` struct fields. The 8 padding slots are reserved for future use.
