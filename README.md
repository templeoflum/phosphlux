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
- **Immediate input response** - no drag threshold lag

### Device Types

**Generators**
- Test Signal Generator (7 modes: H ramp, V ramp, colorbars, checkerboard, radial, H/V oscillators)

**Paik/Abe Video Synthesizer**
- Colorizer — 6-parameter folding amplifier with per-channel gain, fold depth, saturation, slew
- Wobulator — Horizontal displacement with frequency, amplitude, phase control

**Output**
- Any device can be designated as the output device
- Preview with aspect ratio lock (Free, 1:1, 4:3, 16:9, 3:4, 9:16, 2.35:1)

### Compose-Based Shader System
Each patch topology generates a specialized WGSL shader at edit-time:
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
git clone https://github.com/yourusername/phosphlux.git
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

The application uses a 4-panel layout:

```
+------------+---------------------------+-------------+
|            |                           |   PREVIEW   |
|   NODE     |                           |   [4:3]     |
|  PALETTE   |     NODE GRAPH EDITOR     +-------------+
|            |                           |  INSPECTOR  |
|            |                           |             |
+------------+---------------------------+-------------+
```

- **Left**: Node palette - drag nodes onto canvas
- **Center**: Node graph editor with pan/zoom
- **Top Right**: Video preview with aspect ratio selector
- **Bottom Right**: Inspector showing selected node parameters

## Architecture

Phosphlux v2 uses a **compose-based** shader architecture: each patch topology gets a specialized WGSL shader with no runtime branching. When the graph changes, the shader is recomposed and the pipeline recompiled (~5-20ms). When only parameters change, the uniform buffer is updated (sub-ms).

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
│   ├── shader_composer.rs     # ShaderDevice trait, DevicePatch, compose system
│   └── renderer.rs            # wgpu + egui renderer, pipeline management
├── shaders/
│   ├── common.wgsl            # Vertex shader, timing uniform, utility functions
│   └── components.wgsl        # Op-amp, transistor, diode, waveshaping models
├── ui/
│   ├── device_registry.rs     # Device factory and palette categories
│   └── node_editor.rs         # Visual node editor, inspector, connections
└── main.rs                    # Application entry, 3-panel layout, event loop
```

### GPU Pipeline

1. **Shader Composition**: `ShaderComposer` topologically sorts the device graph and generates a single WGSL shader with straight-line code — no runtime branching
2. **Pipeline Recompile**: On topology change, the render pipeline is rebuilt with the new shader module
3. **Parameter Update**: On slider changes, device parameter values are written to the `DeviceParams` uniform buffer
4. **Render Pass**: Fullscreen triangle renders composed shader to output texture; egui overlay painted on the surface

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

MCP server exposing Ollama models for research queries:

| Tool | Model | Speed | Purpose |
|------|-------|-------|---------|
| `quick_query` | phi4-mini | ~4s | Definitions, quick facts |
| `research_query` | qwen3 | ~22s | Deep research synthesis |
| `generate_shader` | qwen3 | ~22s | WGSL drafts (Claude reviews) |
| `extract_from_image` | moondream | ~10s | Component extraction from schematics |
| `embed_document` | qwen3-embedding | ~2s | Knowledge base indexing |

### Device Workbench (`tools/device-workbench/`)

Rust/wgpu application for real-time shader testing:

| MCP Tool | Purpose |
|----------|---------|
| `workbench_load_shader` | Hot-reload WGSL shader |
| `workbench_set_params` | Adjust uniforms in real-time |
| `workbench_inject_signal` | Load test patterns (ramps, bars, etc.) |
| `workbench_screenshot` | Capture shader output |
| `workbench_record_sweep` | Create parameter sweep GIFs |
| `workbench_export_node` | Export to Phosphlux node palette |

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

See [docs/HISTORICAL_EMULATION_PLAN.md](docs/HISTORICAL_EMULATION_PLAN.md) for the research-first emulation roadmap.

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

### Infrastructure Complete
- [x] v2 compose-based shader architecture (specialized shader per patch)
- [x] Visual node editor with pan/zoom, device palette, inspector
- [x] GPU renderer with two-tier update (recompile vs param update)
- [x] Device registry and ShaderDevice trait system
- [x] Test signal generator (7 modes)
- [x] Research orchestrator MCP server
- [x] Component library (5 ICs + 2 functional blocks)
- [x] Device workbench for shader testing
- [x] 4-layer research knowledge base (31 PDFs across 4 v1 target devices)

### Paik/Abe Progress
- [x] CA3046 transistor array (differential pairs)
- [x] CA3019 diode array (PED Control colorizer)
- [x] CA3054 dual diff amp (matrix mixer, video keyer)
- [x] Folding amplifier (nonlinear waveshaper)
- [x] Wobbulator (scan displacement)
- [x] Colorizer device node (v2 compose system)
- [x] Wobulator device node (v2 compose system)

### Future
- [ ] Video input: Camera and file support
- [ ] Audio reactivity
- [ ] MIDI/OSC control
- [ ] Patch save/load

## Performance Notes

The transition from TypeScript prototype to Rust achieved a **30x performance improvement** (2 FPS to 60 FPS):

- All pixel processing in GPU shaders
- Uniform buffers minimize CPU-GPU transfer
- Ping-pong framebuffers enable stable feedback
- Graph compiler with topological sort
- Storage buffer pipeline for dynamic operation dispatch

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
