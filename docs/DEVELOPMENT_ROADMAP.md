# Development Roadmap

This document outlines the phased development plan for **Phosphlux** — a real-time GPU analog video synthesizer.

## Guiding Principles

1. **Faithfulness over convenience** - Every effect should be rooted in real analog behavior
2. **Incremental complexity** - Start simple, validate, then expand
3. **Testability** - Each component must be independently verifiable
4. **Performance awareness** - Real-time operation is non-negotiable

---

## Phase 1: Foundation (COMPLETED)

### Core Infrastructure
- [x] TypeScript project setup with Vite/Vitest
- [x] Signal types and timing constants (NTSC/PAL)
- [x] Frame buffer operations
- [x] Color space conversions (RGB, YIQ)

### Sync Generation
- [x] Sample-accurate sync generator
- [x] Frame-based sync generator for high-level use
- [x] Horizontal/vertical ramps
- [x] Blanking interval detection

### Basic Modules
- [x] LFO with multiple waveforms
- [x] Video-rate oscillator
- [x] Ramp/shape generators
- [x] Noise generator (white, pink, brown)

### Signal Processing
- [x] Folding amplifier (Paik/Abe style)
- [x] Soft/hard clipping
- [x] Quantizer (Sandin style)
- [x] Transfer function waveshaper

### Colorization
- [x] Matrix colorizer
- [x] Threshold colorizer
- [x] Layered colorizer (Jones style)
- [x] Gradient mapper

### CRT Emulation
- [x] Phosphor persistence (per-channel)
- [x] Beam bloom
- [x] Scanlines
- [x] Shadow mask
- [x] Screen curvature

### Effects
- [x] Video feedback
- [x] Frame delay
- [x] Rutt/Etra displacement
- [x] Wobbulator

### Mixing
- [x] Video mixer
- [x] Luminance keyer
- [x] Chroma keyer
- [x] Four-quadrant multiplier

---

## Phase 2: Enhancement (IN PROGRESS)

### NTSC/PAL Signal Artifacts
- [ ] Chroma bandwidth limiting (1.5MHz vs 4.2MHz for luma)
- [ ] Color subcarrier encoding
- [ ] Dot crawl simulation
- [ ] Cross-color artifacts
- [ ] Chroma phase errors

### Component Modeling
- [ ] Op-amp bandwidth limiting
- [ ] Slew rate limiting
- [ ] Component tolerance variation
- [ ] Temperature coefficient simulation
- [ ] Aging/drift models (Arrhenius)

### Filter Implementation
- [ ] Wave Digital Filter (WDF) core
- [ ] Zero-delay feedback topology
- [ ] Lowpass/highpass/bandpass video filters
- [ ] Deflection coil inductance modeling

### Oversampling
- [ ] Polyphase filter implementation
- [ ] 2x/4x/8x oversampling options
- [ ] Anti-aliasing for nonlinear processing

### Additional Modules
- [ ] Differentiator (Sandin style)
- [ ] Integrator
- [ ] Sample and hold
- [ ] Envelope follower
- [ ] Voltage-to-position converter (Beck style)

---

## Phase 3: GPU Acceleration (COMPLETED)

### Rust + wgpu Backend
- [x] WGSL shader composition system (v2 compose architecture)
- [x] Fullscreen triangle rendering (no vertex buffer)
- [x] Specialized shader per patch topology (no runtime branching)
- [x] Two-tier update: recompile on topology change, uniform update on param change

### Performance Targets
- [x] 60fps on mid-range GPU (verified on RTX 2060)
- [ ] 4K @ 30fps on high-end GPU
- [x] Sub-frame latency for real-time use

### Hybrid Processing
- [x] CPU for control logic and timing
- [x] GPU for pixel processing via composed WGSL shaders
- [x] Efficient CPU-GPU transfer via uniform buffers

---

## Phase 4: Control Systems

### MIDI Support
- [ ] CC mapping to parameters
- [ ] Note input for triggers
- [ ] Clock sync
- [ ] MIDI learn functionality

### OSC Support
- [ ] Parameter addressing
- [ ] Bidirectional communication
- [ ] TouchOSC template

### Ableton Link
- [ ] Tempo synchronization
- [ ] Phase alignment
- [ ] Quantum awareness

### External CV (Future)
- [ ] Audio interface input
- [ ] DC-coupled support
- [ ] Calibration system

---

## Phase 5: User Interface (PARTIALLY COMPLETED)

### Node-Based Patching
- [x] Visual module representation (draggable nodes with category-colored headers)
- [x] Bezier curve cable routing between ports
- [x] Device parameter inspector panel with dynamic sliders
- [x] Device palette with categorized device buttons
- [ ] Preset management / patch save/load

### Visualization
- [ ] Oscilloscope view
- [ ] Vectorscope
- [ ] Waveform monitor
- [ ] Signal flow visualization

### Preset System
- [ ] Patch save/load (JSON)
- [ ] Preset morphing
- [ ] Randomization
- [ ] Favorites/tagging

---

## Phase 6: Advanced Features

### Neural Network Integration
- [ ] Circuit behavior learning from SPICE
- [ ] Real-time inference for complex circuits
- [ ] Adaptive feedback stability
- [ ] Aesthetic-guided parameter exploration

### Temporal Effects
- [ ] Arbitrary delay networks
- [ ] Time reversal
- [ ] Time stretching
- [ ] Frame interpolation

### Spectral Processing
- [ ] FFT-based video effects
- [ ] Spectral filtering
- [ ] Convolution effects
- [ ] Phase vocoder techniques

### Collaboration
- [ ] WebRTC video sharing
- [ ] Shared patch state
- [ ] Multi-user jam sessions
- [ ] NDI/Syphon/Spout output

---

## Research Tasks

### Documentation to Study
- [ ] LZX Industries Cadet schematics (GitHub)
- [ ] Sandin Image Processor documentation
- [ ] Jones Colorizer circuit analysis
- [ ] Rutt/Etra patent documentation
- [ ] CRT phosphor datasheets
- [ ] NTSC/PAL encoding specifications

### Technical Papers
- [ ] Kurt Werner WDF dissertation
- [ ] Zavalishin filter theory
- [ ] Analog circuit modeling literature
- [ ] Video synthesis history papers

### Reference Implementations
- [ ] Max/MSP/Jitter video objects
- [ ] TouchDesigner feedback techniques
- [ ] Hardware synthesizer demos
- [ ] Historical footage analysis

---

## Quality Metrics

### Visual Accuracy
- Compare output to actual hardware captures
- Side-by-side A/B testing
- Expert review from video synthesis community

### Performance Benchmarks
- Frame time measurements
- Memory usage profiling
- CPU/GPU utilization

### Test Coverage
- Unit tests for all modules
- Integration tests for signal paths
- Visual regression testing

---

## File Structure Evolution

```
analog/
├── src/
│   ├── core/           # Types, timing, utilities
│   ├── modules/        # Processing modules
│   ├── synth/          # Orchestration
│   ├── gpu/            # WebGL shaders (Phase 3)
│   ├── control/        # MIDI, OSC (Phase 4)
│   └── ui/             # Interface components (Phase 5)
├── demo/               # Browser demo
├── docs/               # Documentation
├── presets/            # Preset patches
├── tests/              # Test suites
└── research/           # Reference materials
```

---

## Contributing

When adding new features:

1. Start with the technical reference document
2. Research the original analog implementation
3. Create a design document explaining the approach
4. Implement with comprehensive tests
5. Document parameters and behavior
6. Add to demo presets

Each module should include:
- Type-safe interfaces
- Default parameter values
- CV modulation support
- Reset functionality
- Clear documentation
