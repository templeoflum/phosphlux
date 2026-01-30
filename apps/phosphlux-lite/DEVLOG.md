# Phosphlux Lite Development Log

## v0.1.0 - Initial Release (2026-01-30)

### Overview

First release of Phosphlux Lite - a simplified, fixed-pipeline video synthesizer designed for immediate playability while capturing the aesthetic of vintage analog video synthesis.

### Architecture Decisions

**Why a separate app?**
- The main Phosphlux app uses a node-graph architecture with dynamic shader compilation
- This is powerful but complex - users need to understand signal flow and patching
- Phosphlux Lite provides a "just works" experience with a fixed signal chain
- Both apps share the same workspace and GPU infrastructure

**Why a fixed pipeline?**
- Predictable signal flow: INPUT → GEOMETRY → AMPLITUDE → COLORIZE → MIXER → FEEDBACK → OUTPUT
- Every stage has a clear purpose inspired by classic hardware
- No routing decisions - just adjust parameters and explore
- Single shader file (`lite.wgsl`) is easier to understand and modify

**Stage design rationale:**
1. **Input**: Multiple generator types cover the basics (ramps, oscillators, noise, shapes)
2. **Geometry**: Combines wobbulation + z-displacement + lissajous - the spatial transformations
3. **Amplitude**: Wave shaping (fold, quantize, clip) - the signal processors
4. **Colorize**: Luminance-to-color mapping - the colorizers
5. **Mixer**: Feedback integration with blend modes - the compositing
6. **Feedback**: Temporal recursion - the time domain
7. **Output**: Display emulation - the final presentation

### Implementation Notes

**Uniform buffer packing:**
- 256-byte uniform struct covering all 7 stages
- Carefully aligned to 16-byte boundaries for GPU compatibility
- Timing (time, frame) included for animation

**Feedback system:**
- Ping-pong texture pair for temporal effects
- Reads from texture A, writes to texture B, then swaps
- Transforms (zoom, rotation, offset) applied during sampling
- Hue shift and decay for evolving colors

**Output emulation modes:**
- CRT: Scanlines, barrel distortion, vignette, phosphor bloom
- VHS: Tracking errors (per-line horizontal jitter), tape wobble, chroma/luma separation
- Cable: Bandwidth limiting (blur), RF ghosting (delayed echo)

**Preset system:**
- Presets stored as `SynthState` structs with all 7 stages
- JSON serialization ready for save/load (not yet exposed in UI)
- 6 built-in presets demonstrating different synthesis styles

### Known Limitations

**v0.1 scope (intentional):**
- Internal generators only - no webcam/video input
- Mouse/keyboard only - no MIDI
- Visual synthesis only - no audio reactivity
- No preset save/load UI (infrastructure exists)

**Technical:**
- Fixed 640x480 render resolution
- No texture resize on window resize (planned)
- Some dead code warnings (unused resize methods)

### File Structure

```
apps/phosphlux-lite/
├── Cargo.toml          # Package manifest
├── README.md           # User documentation
├── DEVLOG.md           # This file
├── src/
│   ├── main.rs         # Entry point, window, event loop
│   ├── app.rs          # Application state, presets, randomize
│   ├── synth.rs        # Stage parameter structs, GPU uniforms
│   ├── renderer.rs     # wgpu renderer, feedback textures
│   ├── ui.rs           # egui UI, stage panels
│   └── presets.rs      # Built-in presets, save/load
└── shaders/
    └── lite.wgsl       # Fixed-pipeline synthesis shader
```

### Shader Organization

The `lite.wgsl` shader is organized by stage:

1. **Uniforms**: Single `Synth` struct with all parameters
2. **Utilities**: Hash, noise, hue rotation
3. **Stage 1 - Input**: `generate_source()`, `stage_input()`
4. **Stage 2 - Geometry**: `stage_geometry()` with wobbulation, z-displacement
5. **Stage 3 - Amplitude**: `fold()`, `quantize()`, `soft_clip()`, `solarize()`, `stage_amplitude()`
6. **Stage 4 - Colorize**: `colorize_spectrum()`, `colorize_threshold()`, `stage_colorize()`
7. **Stage 5 - Mixer**: Blend modes, luma key, `stage_mixer()`
8. **Stage 6 - Feedback**: Transform sampling, `stage_feedback()`
9. **Stage 7 - Output**: CRT/VHS/Cable emulation, `stage_output()`
10. **Fragment main**: Orchestrates all stages in order

### Performance

- Release build runs at 60fps easily on integrated graphics
- Single draw call per frame (fullscreen quad)
- Feedback texture copy adds minimal overhead
- No graph compilation or dynamic dispatch

### Future Development

**v0.1.1 (bug fixes):**
- Window resize handling
- Remove dead code warnings

**v0.2 (UI polish):**
- Preset save/load UI
- Parameter tooltips
- Keyboard shortcuts (randomize, preset switching)

**v0.3 (external input):**
- Webcam input as source option
- Video file playback

**v0.4 (control):**
- MIDI learn for any parameter
- OSC input support

**v0.5 (audio):**
- Audio input for reactivity
- Envelope follower → parameter modulation
- Beat detection

---

## Development History

### 2026-01-30: Project Creation

- Created `apps/phosphlux-lite/` directory structure
- Implemented all 7 stage parameter structs in `synth.rs`
- Built fixed-pipeline shader `lite.wgsl` with ~600 lines
- Created egui UI with stage tabs and sliders
- Added 6 built-in presets covering different synthesis styles
- Integrated with workspace (shares dependencies with main app)
- Both `phosphlux` and `phosphlux-lite` build and run successfully
