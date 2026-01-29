# Development Roadmap

This document outlines the phased development plan for **Phosphlux** — a real-time GPU analog video synthesizer built in Rust with wgpu.

## Guiding Principles

1. **Faithfulness over convenience** - Every effect should be rooted in real analog behavior
2. **Incremental complexity** - Start simple, validate, then expand
3. **Testability** - Each component must be independently verifiable
4. **Performance awareness** - Real-time operation is non-negotiable

---

## Completed Work

### TypeScript Prototype (v0.1, archived)

Initial implementation validated the architecture but ran at ~2 FPS. Archived to `__archive_typescript_reference/`. The prototype proved the concept; the Rust rewrite made it performant.

### Rust + wgpu Backend (v0.2-v0.7)

- [x] 60 FPS real-time rendering with wgpu
- [x] NTSC/PAL sync generation
- [x] egui control panel integration
- [x] Paned layout redesign
- [x] Visual node editor with pan/zoom, draggable nodes, Bezier connections
- [x] Per-node feedback textures (ping-pong buffers)
- [x] Mixer with 8 blend modes

### Research Infrastructure (v0.8-v0.10)

- [x] Research orchestrator MCP server (Ollama-powered tiered models)
- [x] Device workbench for real-time shader testing
- [x] Component library: 5 ICs (CA3046, CA3019, CA3054, CA3080, LM311) + 2 functional blocks
- [x] 4-layer knowledge base with 31 PDFs across 4 target devices
- [x] Patent/datasheet/schematic/manual retrieval system

### v2 Compose Architecture (v0.11)

- [x] Shader composition — specialized WGSL per patch topology, no runtime branching
- [x] Device shader deduplication by type
- [x] Instance-ID based connections (stable across node removal)
- [x] Two-tier update: topology change → recompile, param change → buffer write
- [x] ShaderDevice trait and device registry
- [x] TestSignalGenerator (7 modes)
- [x] PaikAbeColorizer and PaikAbeWobulator device nodes

### Multi-Pass Renderer (v0.12)

- [x] ProcessingMode enum: PixelProcessor vs ScanModulator
- [x] Render plan segmentation at scan modulator boundaries
- [x] Intermediate textures for spatial effects (wobulator samples at displaced UV)
- [x] Reachable device filtering (only upstream devices included in render plan)
- [x] Per-parameter LFO automation (sine wave, 3 speed presets, adjustable lo/hi range)
- [x] Inspector moved to left panel for full-width preview
- [x] Wobulator: H/V displacement with 7 parameters

---

## Versioned Roadmap to v1.0

### v0.13 — Cable Degradation ("Signal Has Character") ✓

Different cable/connector types introduce distinct signal artifacts:

- [x] Add `CableType` to connections: Ideal, Composite, S-Video, Component, RGB
- [x] Create `src/shaders/cables.wgsl` with per-cable-type degradation functions
- [x] Composite: chroma/luma crosstalk, ~4MHz bandwidth, dot crawl (YIQ color space)
- [x] S-Video: separate Y/C (no crosstalk), bandwidth-limited
- [x] Component/RGB: wider bandwidth, minimal degradation, cable-length HF rolloff (YPbPr)
- [x] Cable type selector in inspector, colored cables in node editor

### v0.14 — Full Paik/Abe Chain ("Authentic Signal Path") ✓

The real Paik/Abe has 5 stages. Refactored the existing colorizer into separate devices:

| Device | ProcessingMode | Description |
|--------|---------------|-------------|
| `PaikAbeInputMatrix` | PixelProcessor | 7-channel weighted mixing (CA3054 diff amps) |
| `PaikAbePedColorizer` | PixelProcessor | Amplitude window colorization (CA3019 diode array) |
| `PaikAbeFoldingAmp` | PixelProcessor | Triangle wave folding per channel |
| `PaikAbeWobulator` | ScanModulator | UV displacement from oscillators + video modulation |
| `PaikAbeOutputEncoder` | PixelProcessor | LM741 op-amp output stage |

- [x] Implement InputMatrix, PedColorizer, FoldingAmp, OutputEncoder devices
- [x] Refactor existing colorizer into separate devices
- [x] Ctrl+click quick-connect for rapid patch building
- [x] LFO enhancements: global BPM, per-param phase offset and subdivide
- [ ] Create default Paik/Abe preset patch
- [ ] Add patch save/load system (JSON)

### v0.15 — CRT Monitor ("Phosphors Glow")

A `DisplayOutput` device with its own render pass and feedback pair for temporal persistence:

- [ ] P22 phosphor persistence per-channel (R=0.85, G=0.75, B=0.65 decay)
- [ ] Beam width, bloom amount/radius
- [ ] Scanline count and weight
- [ ] Screen curvature, corner radius
- [ ] Shadow mask type (none/shadow mask/aperture grille) and strength
- [ ] Ping-pong feedback pair for persistence across frames

### v0.16 — Camcorder + Feedback ("The Loop Closes")

An `OpticalInput` device that reads the CRT's screen texture (previous frame):

- [ ] AGC target and speed (auto gain control)
- [ ] White balance, gamma curve
- [ ] Sensor bloom, noise floor
- [ ] Rolling shutter (scan-line temporal offset)
- [ ] Optical connection type (dashed line, lens icon in UI)
- [ ] CRT → Camera → Signal Chain → CRT feedback loop

### v0.17-v0.19 — Additional Devices + Polish

- [ ] Rutt/Etra displacement (ScanModulator)
- [ ] Sandin Image Processor devices (amplitude classifier, four-quadrant multiplier)
- [ ] Jones Colorizer devices (diode/layered mixing, multi-level keying)
- [ ] Multiple CRT + camera support
- [ ] Preset library with classic video art patches

### v1.0 — Release

- [ ] Complete documentation
- [ ] Cross-GPU testing (Vulkan, Metal, DX12)
- [ ] Performance benchmarks at various resolutions
- [ ] Stable patch file format
- [ ] User guide

---

## Future Phases (Post v1.0)

### Control Systems

- [ ] MIDI CC mapping with learn functionality
- [ ] OSC parameter addressing
- [ ] Audio reactivity
- [ ] External CV via DC-coupled audio interface

### Signal Artifacts

- [ ] NTSC/PAL color subcarrier encoding artifacts
- [ ] Chroma bandwidth limiting, dot crawl, cross-color
- [ ] Op-amp bandwidth and slew rate limiting
- [ ] Component tolerance variation and temperature drift

### Advanced Features

- [ ] Video input (camera, file)
- [ ] Recording/export
- [ ] NDI/Syphon/Spout output
- [ ] Oscilloscope/vectorscope/waveform monitor views
- [ ] Patch preset morphing and randomization

---

## Project Structure

```
src/
├── core/
│   ├── timing.rs              # NTSC/PAL sync generation
│   ├── signal.rs              # Signal types, voltage standards
│   └── component.rs           # Analog IC models (op-amp, transistor, diode, RC)
├── devices/
│   ├── generators.rs          # TestSignalGenerator (7 pattern modes)
│   └── paik_abe/
│       ├── mod.rs             # PaikAbeColorizer, PaikAbeWobulator
│       └── colorizer.wgsl     # Colorizer shader (6-stage folding amp)
├── gpu/
│   ├── shader_composer.rs     # ShaderDevice trait, DevicePatch, multi-pass compose
│   └── renderer.rs            # wgpu + egui renderer, multi-pass pipeline
├── shaders/
│   ├── common.wgsl            # Vertex shader, timing uniform, utility functions
│   └── components.wgsl        # Op-amp, transistor, diode, waveshaping models
├── ui/
│   ├── device_registry.rs     # Device factory and palette categories
│   └── node_editor.rs         # Visual node editor, inspector, LFO automation
└── main.rs                    # Application entry, 3-panel layout, event loop

tools/
├── research-orchestrator/     # MCP server for AI-assisted research
└── device-workbench/          # Real-time WGSL shader testing

research/
├── components/                # IC models and functional blocks
│   ├── ics/                   # CA3046, CA3019, CA3054, CA3080, LM311
│   └── functional/            # Folding amplifier, wobbulator
├── devices/                   # Per-device documentation
└── provenance.db              # Document provenance tracking
```

---

## Contributing

When adding new devices:

1. Research the original analog implementation (patents, schematics, service manuals)
2. Identify the ICs and extract transfer functions from datasheets
3. Implement the `ShaderDevice` trait with appropriate `ProcessingMode`
4. Test in the device workbench before integrating
5. Register in `device_registry.rs` with the correct category
6. Document parameters and behavior
