# Analog Video Synthesizer

A real-time GPU-accelerated emulation of classic analog video synthesis hardware, built in Rust with wgpu.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)
![wgpu](https://img.shields.io/badge/wgpu-23.0-green.svg)

## Overview

This project authentically recreates the complete history of electronic image-making - from 1960s oscilloscope art through contemporary Eurorack modular video systems. Rather than simple approximations, we model the physics and circuits that defined analog video synthesis.

**Performance**: 60 FPS real-time rendering with full feedback effects.

### Key Systems Emulated

- **Paik/Abe Video Synthesizer** (1969-1971) - Nonlinear folding amplifiers, resistive color matrix
- **Sandin Image Processor** (1971-1974) - Amplitude classifier, four-quadrant multiplication
- **Rutt/Etra Video Synthesizer** (1972-1974) - DC-coupled raster manipulation, Z-axis displacement
- **Jones Colorizer** (1974) - Diode/layered mixing, multi-level keying
- **LZX Industries** (2008-present) - Modern modular video synthesis standards

## Screenshots

*Coming soon - the application renders real-time generative video with feedback effects*

## Features

### Video Effects
- **Feedback Loop**: Zoom, rotation, offset, hue shift, saturation decay
- **Luma Keying**: Adjustable black/white clip points with invert
- **Luminance Displacement**: Brightness-driven UV distortion
- **Noise Generation**: White, pink, and brown noise types
- **Shape Generators**: Circle, rectangle, diamond, star, bars, checker
- **Colorization**: Spectrum mapping, threshold posterization, grayscale

### CRT Emulation
- Scanline simulation
- Screen curvature (barrel distortion)
- Vignette effect

### User Interface
- Blender-style paned layout
- Resizable parameter panel
- Real-time parameter adjustment
- Preset system (keyboard shortcuts 1-5)
- Prepared for node-based compositor (coming soon)

## Requirements

- Rust 1.75 or later
- GPU with Vulkan, Metal, or DX12 support
- Windows, macOS, or Linux

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/analog-video-synth.git
cd analog-video-synth

# Build and run
cargo run --release
```

## Usage

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Space` | Reset all parameters to defaults |
| `1` | Preset: Spiral |
| `2` | Preset: Tunnel |
| `3` | Preset: Chaos |
| `4` | Preset: Drift |
| `5` | Preset: Kaleidoscope |
| `Esc` | Exit |

### Interface

The application opens in a maximized window with:
- **Left Panel**: Parameter controls organized by category
- **Central Panel**: Real-time video preview
- **Top Menu**: File operations, presets, help

## Architecture

```
src/
├── core/
│   └── timing.rs       # NTSC/PAL sync generation
├── gpu/
│   ├── renderer.rs     # wgpu pipeline, feedback buffers
│   └── shaders/
│       ├── composite.wgsl  # Main synthesis
│       ├── feedback.wgsl   # Recursive feedback
│       └── crt.wgsl        # Display emulation
├── modules/
│   ├── oscillators.rs  # Signal generators
│   └── waveshapers.rs  # Signal processing
├── ui/
│   ├── controls.rs     # Parameter structures
│   └── patch.rs        # Node graph (data only)
└── main.rs             # Application entry
```

### GPU Pipeline

1. **Composite Pass**: Generate base imagery (shapes, noise, oscillators)
2. **Feedback Pass**: Apply recursive transform (ping-pong buffers)
3. **CRT Pass**: Apply display emulation
4. **egui Pass**: Render UI overlay

## Development

See [DEVLOG.md](DEVLOG.md) for detailed development history and technical decisions.

See [docs/VIDEO_SYNTHESIS_TECHNICAL_REFERENCE.md](docs/VIDEO_SYNTHESIS_TECHNICAL_REFERENCE.md) for comprehensive research on analog video synthesis techniques.

### Building

```bash
# Debug build
cargo build

# Release build (recommended for performance)
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=info cargo run --release
```

## Roadmap

- [ ] **Node-Based Compositor**: Visual signal routing like TouchDesigner
- [ ] **True Rutt/Etra**: Geometry-based scan displacement (vs. UV distortion)
- [ ] **Multi-Channel Textures**: Main, Feedback, Key, Aux channels
- [ ] **Video Input**: Camera and file support
- [ ] **Audio Reactivity**: Modulate parameters from audio
- [ ] **MIDI/OSC Control**: External hardware control
- [ ] **Patch Save/Load**: Serialize and restore configurations

## Technical Reference

The project draws from extensive research into historical video synthesizers:

- **CRT Physics**: Phosphor persistence, beam bloom, electron gun behavior
- **NTSC Standard**: 525 lines @ 59.94Hz, 3.579545 MHz color subcarrier
- **Feedback Theory**: Ping-pong buffers, decay coefficients, stability
- **Circuit Modeling**: Op-amp characteristics, diode mixing, waveshaping

See the full [Technical Reference](docs/VIDEO_SYNTHESIS_TECHNICAL_REFERENCE.md) for implementation details.

## Performance Notes

The transition from TypeScript prototype to Rust achieved a **30x performance improvement** (2 FPS → 60 FPS):

- All pixel processing happens in GPU shaders
- Uniform buffers minimize CPU-GPU data transfer
- Ping-pong framebuffers enable stable feedback without read-after-write hazards
- egui's immediate mode minimizes UI overhead

## Contributing

Contributions are welcome! Please:

1. Read the technical reference document
2. Research the original analog implementation for any new effect
3. Maintain real-time performance (60 FPS target)
4. Add appropriate documentation

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

And the ongoing community of video synthesis artists and engineers.

## References

- [LZX Industries](https://lzxindustries.net/) - Modern modular video synthesis
- [Sandin Image Processor Documentation](http://www.vasulka.org/archive/Artists3/Sandin,Dan/)
- [Video Circuits](https://www.videocircuits.com/) - DIY video synthesis community
