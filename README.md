# Phosphlux

*phosphor + flux — flowing light substance*

Real-time GPU analog video synthesizer and CRT emulation, built in Rust with wgpu.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)
![wgpu](https://img.shields.io/badge/wgpu-23.0-green.svg)

## Overview

Phosphlux authentically recreates the complete history of electronic image-making — from 1960s oscilloscope art through contemporary Eurorack modular video systems. Rather than simple approximations, we model the physics and circuits that defined analog video synthesis.

**Performance**: 60 FPS real-time rendering with full feedback effects.

### Key Systems Emulated

- **Paik/Abe Video Synthesizer** (1969-1971) - Nonlinear folding amplifiers, resistive color matrix
- **Sandin Image Processor** (1971-1974) - Amplitude classifier, four-quadrant multiplication
- **Rutt/Etra Video Synthesizer** (1972-1974) - DC-coupled raster manipulation, Z-axis displacement
- **Jones Colorizer** (1974) - Diode/layered mixing, multi-level keying
- **LZX Industries** (2008-present) - Modern modular video synthesis standards

## Features

### Visual Node Editor
- **Pan/zoom canvas** with grid background
- **Draggable nodes** with category-colored headers
- **Bezier curve connections** between sockets
- **Real-time parameter adjustment** via inspector panel
- **LFO automation** per parameter (sine wave, 3 speed presets, adjustable lo/hi range)
- **Immediate input response** - no drag threshold lag

### Device Types

**Generators**
- Test Signal Generator (7 modes: H ramp, V ramp, colorbars, checkerboard, radial, H/V oscillators)

**Paik/Abe Video Synthesizer**
- Colorizer — 6-parameter folding amplifier with per-channel gain, fold depth, saturation, slew
- Wobulator — H/V scan displacement with frequency, amplitude, phase, video modulation, speed control (7 parameters)

**Output**
- Any device can be designated as the output device
- Preview with aspect ratio lock (Free, 1:1, 4:3, 16:9, 3:4, 9:16, 2.35:1)

### Multi-Pass Compose Shader System
Each patch topology generates specialized WGSL shaders at edit-time:
- **Multi-pass rendering** — pixel processors compose into single-pass segments; scan modulators (wobulator) get their own render pass with intermediate textures
- **Reachable device filtering** — only devices upstream of the output are included in the render plan
- No runtime branching — only active devices are in the shader
- Device shader code is deduplicated by type (multiple instances share one function)
- Parameter uniform buffer updated per-frame without recompilation
- Reusable `ShaderDevice` trait for adding new devices

## Requirements

- Rust 1.75 or later
- GPU with Vulkan, Metal, or DX12 support
- Windows, macOS, or Linux

## Installation

```bash
# Clone the repository
git clone https://github.com/templeoflum/phosphlux.git
cd phosphlux

# Build and run
cargo run --release
```

## Usage

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Del` / `Backspace` | Delete selected node |
| `F` | Frame/center on selected node |
| `Home` | Reset view (pan and zoom) |
| `Esc` | Exit |

### Interface

The application uses a 3-panel layout:

```
+--------------------+---------------------------+-----------+
|   NODE PALETTE     |                           |           |
+--------------------+                           |  PREVIEW  |
|                    |     NODE GRAPH EDITOR     |  [4:3]    |
|   INSPECTOR        |                           |           |
|   (selected node)  |                           |           |
+--------------------+---------------------------+-----------+
```

- **Left**: Node palette (top) + inspector with parameter sliders and LFO controls (bottom)
- **Center**: Node graph editor with pan/zoom
- **Right**: Video preview with aspect ratio selector, full available height

## Architecture

Phosphlux v2 uses a **multi-pass compose** shader architecture. The device graph is segmented into render passes: consecutive pixel processors compose into a single WGSL shader, while scan modulators (wobulator, displacement effects) get their own render pass with intermediate textures for spatial sampling. Only devices upstream of the output device are included (reachable filtering). When the graph changes, the render plan is rebuilt and pipelines recompiled. When only parameters change, uniform buffers are updated (sub-ms).

```
src/
├── core/
│   ├── timing.rs              # NTSC/PAL sync generation
│   ├── signal.rs              # Signal types, voltage standards
│   └── component.rs           # Analog IC models (op-amp, transistor, diode, RC)
├── devices/
│   ├── generators.rs          # TestSignalGenerator (7 pattern modes)
│   └── paik_abe/
│       ├── mod.rs             # PaikAbeColorizer (PixelProcessor), PaikAbeWobulator (ScanModulator)
│       └── colorizer.wgsl     # Colorizer shader (6-stage folding amp)
├── gpu/
│   ├── shader_composer.rs     # ShaderDevice trait, DevicePatch, multi-pass compose, render plan
│   └── renderer.rs            # wgpu + egui renderer, multi-pass pipeline, intermediate textures
├── shaders/
│   ├── common.wgsl            # Vertex shader, timing uniform, utility functions
│   └── components.wgsl        # Op-amp, transistor, diode, waveshaping models
├── ui/
│   ├── device_registry.rs     # Device factory and palette categories
│   └── node_editor.rs         # Visual node editor, inspector, LFO automation
└── main.rs                    # Application entry, 3-panel layout, event loop
```

### GPU Pipeline

1. **Graph Analysis**: `ShaderComposer` topologically sorts the device graph, filters to reachable devices, and segments into render passes at scan modulator boundaries
2. **Shader Composition**: Each segment gets a specialized WGSL shader with straight-line code — no runtime branching. Pixel processor segments compose multiple devices into one pass; scan modulators get their own pass with texture sampling.
3. **Pipeline Recompile**: On topology change, per-segment pipelines are rebuilt with new shader modules and intermediate textures allocated
4. **Parameter Update**: On slider changes, per-segment uniform buffers are updated (sub-ms)
5. **Multi-Pass Render**: Segments execute in order — each fullscreen triangle pass renders to its output texture; the final segment renders to the preview; egui overlay painted on the surface

## Research & Development Tools

Phosphlux includes a comprehensive toolchain for AI-assisted historical research and component-level emulation.

### Philosophy
**"Follow the inventors, not the interpreters."**

Rather than implementing effects from secondhand descriptions, we model at the **component level**:
1. Extract IC part numbers from schematics
2. Find datasheets with transfer functions
3. Generate WGSL shader models from circuit physics
4. Compose components into device emulations

### Research Orchestrator (`tools/research-orchestrator/`)

MCP server providing research capabilities:

| Tool | Purpose |
|------|---------|
| `quick_query` | Definitions, quick facts |
| `research_query` | Deep video synthesis research |
| `generate_shader` | WGSL shader code drafts (Claude reviews) |
| `embed_document` | Knowledge base indexing |
| `search_knowledge` | Semantic search over embedded documents |
| `retrieve_patent` | Patent search and download |
| `retrieve_datasheet` | IC datasheet retrieval |
| `retrieve_schematic` | Circuit schematic archives |

### Device Workbench (`tools/device-workbench/`)

Rust/wgpu application for real-time shader testing with WGSL validation:

| MCP Tool | Purpose |
|----------|---------|
| `workbench_load_shader` | Hot-reload WGSL shader (naga-validated) |
| `workbench_set_params` | Adjust uniforms in real-time |
| `workbench_get_params` | Get current parameter values |
| `workbench_inject_signal` | Load test patterns (ramps, bars, checkers) |
| `workbench_screenshot` | Capture shader output |
| `workbench_record_sweep` | Create parameter sweep GIFs |
| `workbench_doc_capture` | Documentation screenshots with metadata |
| `workbench_export_node` | Export to Phosphlux node palette |

**Stability**: Workbench uses naga WGSL validation before GPU compilation, preventing crashes from invalid shader code.

### Component Library (`research/components/`)

Reusable analog IC and functional block models. Vendor datasheets live at the component root; WGSL model iterations are versioned in `v1/`, `v2/`, etc.

```
components/
├── RCA_1975_Linear_ICs.pdf          # Comprehensive CA30xx reference
├── RCA_1978_Linear_IC_Databook.pdf  # Comprehensive CA30xx reference
├── ics/
│   ├── ca3046/                      # NPN Transistor Array
│   │   ├── CA3046_Intersil_datasheet.pdf
│   │   └── v1/
│   │       ├── spec.json            # SPICE parameters, pinout
│   │       ├── model.wgsl           # differential_pair(), current_mirror()
│   │       └── colorizer.wgsl
│   ├── ca3019/                      # Ultra-Fast Diode Array
│   │   ├── PROVENANCE.md            # Datasheet sources, known URLs
│   │   └── v1/
│   │       ├── model.wgsl           # diode_transfer(), amplitude_window()
│   │       └── ped_control.wgsl     # Paik/Abe colorizer
│   ├── ca3054/                      # Dual Differential Amplifier
│   │   ├── PROVENANCE.md
│   │   └── v1/
│   │       ├── model.wgsl           # diff_amp(), soft_clip()
│   │       ├── matrix_mixer.wgsl    # 7-channel matrix
│   │       └── video_keyer.wgsl     # Soft/hard keying
│   ├── ca3080/                      # OTA (Transconductance Amplifier)
│   │   ├── CA3080_Intersil_datasheet.pdf
│   │   ├── CA3080_Application_Note_AN6668.pdf
│   │   └── v1/
│   │       ├── model.wgsl           # ota_transfer(), ota_vca()
│   │       └── vca.wgsl
│   └── lm311/                       # Voltage Comparator
│       ├── LM311_TI_datasheet.pdf
│       └── v1/
│           ├── model.wgsl           # lm311_compare(), lm311_window()
│           └── keyer.wgsl
└── functional/
    ├── folding_amp/                 # Nonlinear Waveshaper
    └── wobbulator/                  # CRT Scan Displacement
```

Each IC is modeled once and reused across any device that uses it.

## Development

See [DEVLOG.md](DEVLOG.md) for detailed development history and technical decisions.

See [docs/VIDEO_SYNTHESIS_TECHNICAL_REFERENCE.md](docs/VIDEO_SYNTHESIS_TECHNICAL_REFERENCE.md) for comprehensive research on analog video synthesis techniques.

See [docs/DEVELOPMENT_ROADMAP.md](docs/DEVELOPMENT_ROADMAP.md) for the v0.13-v1.0 development roadmap.

See [docs/HISTORICAL_EMULATION_PLAN.md](docs/HISTORICAL_EMULATION_PLAN.md) for the research-first emulation plan.

### Building

```bash
# Debug build
cargo build

# Release build (recommended)
cargo build --release

# Run with logging
RUST_LOG=info cargo run --release
```

## v1 Emulation Roadmap

Five foundational video synthesis devices, in order:

| # | Device | Era | Status |
|---|--------|-----|--------|
| 1 | **Paik/Abe Video Synthesizer** | 1969 | 🔄 In Progress |
| 2 | **Sandin Image Processor** | 1971 | ⏳ Pending |
| 3 | **Rutt/Etra Video Synthesizer** | 1972 | ⏳ Pending |
| 4 | **Jones Colorizer** | 1974 | ⏳ Pending |
| 5 | **LZX Cadet Series** | 2012 | ⏳ Pending |

See [docs/V1_EMULATION_ROADMAP.md](docs/V1_EMULATION_ROADMAP.md) for detailed component requirements and milestones.

### Infrastructure Complete (v0.12-v0.13)
- [x] Multi-pass compose shader architecture (segments at scan modulator boundaries)
- [x] Reachable device filtering (only upstream devices rendered)
- [x] Visual node editor with pan/zoom, device palette, inspector
- [x] Per-parameter LFO automation (sine wave, 3 speeds, adjustable range)
- [x] GPU renderer with two-tier update (recompile vs param update)
- [x] Device registry and ShaderDevice trait system
- [x] Test signal generator (7 modes)
- [x] Research orchestrator MCP server
- [x] Component library (5 ICs + 2 functional blocks)
- [x] Device workbench for shader testing
- [x] 4-layer research knowledge base (31 PDFs across 4 v1 target devices)
- [x] Cable degradation modeling (Composite, S-Video, Component, RGB)

### Paik/Abe Progress
- [x] CA3046 transistor array (differential pairs)
- [x] CA3019 diode array (PED Control colorizer) — **v0.14.1: proper exponential diode model**
- [x] CA3054 dual diff amp (matrix mixer, video keyer)
- [x] Folding amplifier (nonlinear waveshaper)
- [x] Wobbulator (scan displacement with texture sampling)
- [x] **4 separate device nodes** matching real hardware signal path:
  - InputMatrix (CA3054 diff amps, 7-channel weighted mixing)
  - FoldingAmp (triangle wave folding per channel)
  - PedColorizer (CA3019 amplitude windows, clipping control)
  - OutputEncoder (LM741 op-amp saturation and warmth)
- [x] Wobulator device node (scan modulator, own render pass)
- [x] Ctrl+click quick-connect for rapid patch building
- [x] LFO automation with global BPM, phase offset, and subdivide

### Next: v0.16-v1.0 Roadmap
- [x] Cable/connector signal degradation — **v0.13 complete**
- [x] Full Paik/Abe 5-stage chain — **v0.14 complete**
- [x] Authentic CA3019 diode model with clipping parameter — **v0.14.1 complete**
- [x] UI Overhaul — Vintage oscilloscope theme, top-down node flow — **v0.15 complete**
- [ ] CRT monitor device (phosphor persistence, scanlines, curvature, bloom)
- [ ] Camcorder device (AGC, rolling shutter) + CRT→camera feedback loop
- [ ] Patch save/load (JSON)
- [ ] Additional devices: Sandin IP, Rutt/Etra, Jones Colorizer
- [ ] MIDI/OSC control
- [ ] Video input: camera and file support

## Performance Notes

- All pixel processing in GPU shaders via composed WGSL
- Multi-pass render segments with intermediate textures for spatial effects
- Per-segment uniform buffers minimize CPU-GPU transfer
- Reachable device filtering prevents unnecessary render passes
- Two-tier update: topology change → recompile, param change → buffer write

## The Name

**Phosphlux** combines *phosphor* (the luminescent coating inside CRT displays) with *flux* (continuous flow and change). Together they capture the essence of analog video synthesis: light as a flowing, transforming substance.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

This project draws inspiration from the pioneering work of:

- Nam June Paik & Shuya Abe
- Dan Sandin
- Stephen Beck
- Steve Rutt & Bill Etra
- Dave Jones
- Lars Larsen (LZX Industries)

## References

- [LZX Industries](https://lzxindustries.net/) - Modern modular video synthesis
- [Sandin Image Processor Documentation](http://www.vasulka.org/archive/Artists3/Sandin,Dan/)
- [Video Circuits](https://www.videocircuits.com/) - DIY video synthesis community
