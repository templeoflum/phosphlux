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

### Node Types

**Sources**
- Ramps (Horizontal, Vertical, Radial)
- Oscillators
- Noise (White, Pink, Brown)
- Shape generators
- Frame Buffer (per-node feedback with independent parameters)

**Processing**
- Waveshapers: Folder, Quantizer, Soft Clip, Solarize, Inverter
- Mixer: 8 blend modes (Mix, Add, Multiply, Screen, Overlay, Difference, Luma Key A/B)
- Color: Hue Shift
- Colorizers: Spectrum, Threshold, Gradient

**Outputs**
- Display node with aspect ratio lock (Free, 1:1, 4:3, 16:9, 3:4, 9:16, 2.35:1)

### Per-Node Frame Buffers
Each Frame Buffer node has its own ping-pong texture pair with independent parameters:
- Zoom, rotation, offset
- Hue shift, saturation, decay
- Multiple independent feedback loops can coexist

### CRT Emulation
- Scanline simulation
- Screen curvature (barrel distortion)
- Vignette effect

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

```
src/
├── core/
│   ├── timing.rs          # NTSC/PAL sync generation
│   └── signal.rs          # Signal types, voltage standards
├── gpu/
│   ├── renderer.rs        # wgpu pipeline, per-node Frame Buffers
│   └── graph_compiler.rs  # Topological sort, GPU operation dispatch
├── shaders/
│   └── main.wgsl          # Combined synthesis + graph evaluation
├── ui/
│   ├── patch.rs           # Node graph data structures
│   └── node_editor.rs     # Visual editor, inspector
└── main.rs                # Application entry, 4-panel layout
```

### GPU Pipeline

1. **Graph Compilation**: Topological sort with cycle-aware handling
2. **Operation Dispatch**: Nodes compiled to GPU storage buffer operations
3. **Per-Node Textures**: Frame Buffers get independent ping-pong pairs
4. **Display Pass**: Final output with aspect ratio handling

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

### Swarm Agent System (`tools/swarm-agent/`)

Multi-agent architecture for parallel research and development:

```
swarm/
├── orchestrator.py      # Coordinates agents, handles events
├── component_agent.py   # IC lookup → spec.json → model.wgsl
├── research_agent.py    # Ollama-powered research with token tracking
├── token_tracker.py     # Cloud vs local cost analysis
└── model_benchmark.py   # A/B testing for prompt optimization
```

**Token Tracking**: Monitors cloud (Claude) vs local (Ollama) usage with cost analysis.

**Model Benchmarking**: A/B tests prompt variants to find optimal model configurations.

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

Reusable analog IC models:

```
components/ics/
├── ca3046/           # NPN Transistor Array
│   ├── spec.json     # SPICE parameters, pinout
│   ├── model.wgsl    # differential_pair(), current_mirror()
│   └── colorizer.wgsl # Paik/Abe amplitude window effect
├── ca3080/           # OTA (Operational Transconductance Amplifier)
│   ├── model.wgsl    # ota_transfer(), ota_vca()
│   └── vca.wgsl      # Voltage controlled amplifier
└── lm311/            # Voltage Comparator
    ├── model.wgsl    # lm311_compare(), lm311_window()
    └── keyer.wgsl    # Video threshold keyer
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

## Roadmap

- [x] Visual node editor with pan/zoom
- [x] Per-node Frame Buffers for independent feedback
- [x] Mixer with blend modes and luma keying
- [x] Research orchestrator MCP server
- [x] Swarm agent system with token tracking
- [x] Component library (CA3046, CA3080, LM311)
- [x] Device workbench for shader testing
- [ ] Complete Paik/Abe component set (CA3019, CA3022, CA3054)
- [ ] True Rutt/Etra: Geometry-based scan displacement
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
