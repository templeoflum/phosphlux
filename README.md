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

## Research Orchestrator

Phosphlux includes an MCP server for AI-assisted historical research:

```
tools/research-orchestrator/
├── mcp_server.py      # Exposes Ollama models as tools
├── requirements.txt   # Python dependencies
└── README.md          # Setup and usage
```

### Philosophy
**"Follow the inventors, not the interpreters."**

Local Ollama models handle research and code drafts. Claude reviews all outputs, catches errors, and makes final implementation decisions.

### Tools Available
| Tool | Model | Purpose |
|------|-------|---------|
| `quick_query` | phi4-mini | Fast lookups (~4s) |
| `research_query` | qwen3 | Deep research (~22s) |
| `generate_shader` | qwen3 | WGSL drafts for review |
| `extract_from_image` | moondream | Component extraction |
| `embed_document` | qwen3-embedding | Knowledge base |

See [tools/research-orchestrator/README.md](tools/research-orchestrator/README.md) for setup.

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
