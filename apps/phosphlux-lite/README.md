# Phosphlux Lite

A simple, unified video synthesizer with a fixed 7-stage signal chain. Captures the aesthetic of vintage analog video synthesis without requiring hardware-accurate emulation.

## Inspiration

- **Paik/Abe Video Synthesizer** - Wobbulation, colorization
- **Sandin Image Processor** - Amplitude classification, keying
- **Rutt/Etra Scan Processor** - Z-axis displacement, 3D terrain effects
- **Jones Colorizer** - Threshold-based colorization bands
- **LZX Cadet** - Modular signal flow concepts
- **Beck Direct Video Synthesizer** - Oscillator-based pattern generation
- **VHS/CRT aesthetics** - Tape degradation, scanlines, phosphor glow

## Signal Chain

```
[INPUT] → [GEOMETRY] → [AMPLITUDE] → [COLORIZE] → [MIXER] → [FEEDBACK] → [OUTPUT]
```

### Stage 1: Input Matrix
Generate and mix signal sources:
- **Sources**: Horizontal/vertical ramps, sine oscillators, white/pink/brown noise, geometric shapes, checkerboard
- **Controls**: Source A/B selection, mix blend, frequency, phase, rotation

### Stage 2: Geometry
Spatial distortions inspired by analog video processors:
- **Wobbulation**: Horizontal/vertical wave distortion (Paik/Abe style)
- **Z-Displacement**: Luminance-based vertical displacement (Rutt/Etra style)
- **Lissajous**: X/Y modulation patterns
- **Transform**: Rotation and scale

### Stage 3: Amplitude
Waveform shaping and signal processing:
- **Folding**: Wave folding with adjustable gain (creates harmonic complexity)
- **Quantization**: Reduce to N levels (Sandin Amplitude Classifier style)
- **Soft Clip**: Gentle saturation
- **Solarize**: Threshold-based inversion
- **Gate**: Hard threshold cutoff
- **Invert**: Signal inversion

### Stage 4: Colorize
Map luminance to color:
- **Spectrum**: Rainbow gradient mapping
- **Threshold**: Quantized color bands (Jones Colorizer style)
- **Gradient**: Custom two-color gradient
- **Monochrome**: Grayscale output
- **Controls**: Hue offset, saturation adjustment

### Stage 5: Mixer
Blend current frame with feedback:
- **Blend Modes**: Mix, Add, Multiply, Screen, Overlay, Difference
- **Luma Key**: Key based on luminance with threshold/softness
- **Controls**: Feedback mix amount, layer opacity

### Stage 6: Feedback
Temporal effects using previous frame:
- **Zoom**: Expand/contract feedback
- **Rotation**: Rotate feedback each frame
- **Hue Shift**: Color rotation over time
- **Decay**: Brightness falloff
- **Offset**: X/Y drift

### Stage 7: Output
Display emulation modes:
- **Clean**: No processing
- **CRT**: Scanlines, curvature, bloom, vignette
- **VHS**: Tracking errors, tape wobble, chroma shift
- **Cable**: Bandwidth limiting, RF ghosting

## Built-in Presets

| Preset | Description |
|--------|-------------|
| Feedback Spiral | Classic video feedback with zoom and rotation |
| Wobbulator | Paik/Abe style wobbulation distortion |
| Rutt/Etra Terrain | Z-axis displacement creating 3D terrain |
| VHS Memory | Degraded VHS tape aesthetic |
| Colorizer Bands | Jones-style banded colorization |
| Noise Meditation | Slow-moving noise patterns |

## Usage

```bash
# Run from workspace root
cargo run -p phosphlux-lite --release

# Or build and run separately
cargo build -p phosphlux-lite --release
./target/release/phosphlux-lite
```

## Controls

- **Top Panel**: Preset selection, Randomize button
- **Bottom Panel**: Stage tabs (INPUT, GEOM, AMP, COLOR, MIX, FEEDBACK, OUTPUT)
- **Center**: Video preview (4:3 aspect ratio)

Click a stage tab to reveal its controls. All parameters update in real-time.

## Technical Details

- **Resolution**: 640x480 internal rendering
- **Window**: 1024x768 default
- **GPU**: wgpu (Vulkan/Metal/DX12)
- **UI**: egui immediate-mode GUI
- **Feedback**: Ping-pong texture buffers for temporal effects

## Differences from Main Phosphlux

| Feature | Phosphlux | Phosphlux Lite |
|---------|-----------|----------------|
| Architecture | Node graph, dynamic routing | Fixed 7-stage pipeline |
| Complexity | Full device emulation | Visual approximation |
| Target | Accuracy, flexibility | Simplicity, playability |
| Shader | Graph compiler | Single fixed shader |

## Future Plans (v1.1+)

- External video input (webcam, capture card)
- MIDI CC mapping for all parameters
- Audio reactivity (envelope follower)
- Preset export/import
- Recording to video file
