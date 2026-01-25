# Analog Video Synthesizer

A faithful digital emulation of classic analog video synthesis techniques, from the 1960s oscilloscope art through contemporary Eurorack modular video systems.

## Demo Gallery

Visual output captured from the synthesis engine (luminance mapped to ASCII):

### Radial Gradient
*Ramp generator with spectrum colorization - concentric rings emanating from center*
```
┌──────────────────────────────────────────────────────────────────────┐
│\\(){}]?_+~>>i!lI:I!>~+--?[}{11)(((((((()11{}[??-+~>!I:Il!ii>~+_?]}{)(│
│11}]?_+<>!l;;;!>~-]}1(\//fjrxnuvvccccccvvunxrjff/\(1}]-~>!;;;l!><+_?]}│
│]]-+~>!l;Ii~~_]{)\tjnvzXXUJLQ0OOOO0000OOOO0QLJUUXzvnjt\){]__~iI;l!>~+-│
│++<i!I;!<_[11|tjncXJL0O0000QQQQLLLLLLLLLLQQQQ0000O0LJXcnjt||1[_<!;I!i<│
│>>!I;i~?}(/jjnzUC000QQQLLLCCCCJJJJJJJJJJJJCCCCLLLQQQ000CUznnj/(}?~i;I!│
│II:!~?}(trvXXC000QQLLCCJJJUUUYYYYYYXXYYYYYYUUUJJJCCLLQQ000CCXvrt(}?~!:│
│II>_[)/rvXCOO00QLLCJJUUYYYXXYUJCCLLLLLLCCJUYXXYYYUUJJCLLQ000OCXvr/)[_>│
│<<-}|fnzJ0000QLCCJUUYXXUUJLQ0OZmmwwwwwwmmZO0QLJJUXXYUUJCCLQQ000Jznf|}-│
│--{|fnzCO0QQQLCJUUYXYJLQQOZwqpdbkhhhhhhkbdpqwZOOQLJYXYUUJCLLQQ0OCznf|{│
│}}(tnzJO0QLLLCJUYYXJLQOmmqdbha**ahkkkkha**ahbdqqmOQLJXYYUJCCLLQ0OJznt(│
│11/rvUQ0QQLCCJUYYYJL0Zwppbho*kpmOQCCCCQOmpk*ohbbpwZ0LJYYYUJJCLQQ0QUvr/│
│((tnzCO0QLCCCJUYXUC0Zwpbba*admLUcnrjjrncULmda*aabpwZ0CUXYUJJCCLQ0OCznt│
│((fnzCO0QLCJJJUYXUL0Zwdkka#kqOJznf\((\fnzJOqk#aakdwZ0LUXYUJJJCLQ0OCznf│
│))txcJ00QLLCCJUYXYCQOmqbbho*kqZLJXzcczXJLZqk*ohhbqmOQCYXYUJJCLLQ00Jcxt│
│{{|juYLO0QLCCJJUYXUCQOmqqdkao*hbpwmmmmwpbh*oakddqmOQCUXYUJJJCLQ0OLYuj|│
│]])/rvUQ00QLLCJJUYXYJL0OOmqpbkhao******oahkbpqmmO0LJYXYUJJCCLQ00QUvr/)│
│++[)/rvYLO0QQLLCJUUYXYJCCQ0OmwqqpddddddpqqwmO0QQCJYXYUUJCLLLQ0OLYvr/)[│
│ii+]1\juXC0000QLCCJJUYYXXYUJCLQ00OOOOOO00QLCJUYYXYYUJJCCLQ0000CXuj\1]+│
│;;l<-}(trvXCCO00QLLCCJJUUUYYYXXYYYUUUUYYYXXYYYUUUJJCCLLQ00OOCXvrt(}-<l│
│!!I;i~?{(truuXJQO00QLLLCCCJJJUUUUUUUUUUUUUUJJJCCCLLLQ00OQJXXurt({?~i;I│
│<<i!I;i~-})\\fxvXJLO000QQQLLLLCCCCCCCCCCCCLLLLQQQ000OLJXvxff\)}-~i;I!i│
│--+<>!I;l>+??})\trncXUCQQ0O00000QQQQQQQQ00000O00QCUXcnrt\)}}?+>l;I!><+│
│}}]-_~<ilI:ll>~-]{)|/frnnuczXYUJJCCCCCCJJUYXzcuunrf/|){]-~>>l:Ili<~_-]│
└──────────────────────────────────────────────────────────────────────┘
```

### Color Bars
*Quantized horizontal ramp with threshold colorization - classic test pattern*
```
┌──────────────────────────────────────────────────────────────────────┐
│||||||ZZZZZZZZZZZZZZZZZZZZUUUUUUUUU00000000000))))))))))))))))))))ffff│
│||||||ZZZZZZZZZZZZZZZZZZZZUUUUUUUUU00000000000))))))))))))))))))))ffff│
│||||||ZZZZZZZZZZZZZZZZZZZZUUUUUUUUU00000000000))))))))))))))))))))ffff│
│||||||ZZZZZZZZZZZZZZZZZZZZUUUUUUUUU00000000000))))))))))))))))))))ffff│
│||||||ZZZZZZZZZZZZZZZZZZZZUUUUUUUUU00000000000))))))))))))))))))))ffff│
│||||||ZZZZZZZZZZZZZZZZZZZZUUUUUUUUU00000000000))))))))))))))))))))ffff│
│||||||ZZZZZZZZZZZZZZZZZZZZUUUUUUUUU00000000000))))))))))))))))))))ffff│
│||||||ZZZZZZZZZZZZZZZZZZZZUUUUUUUUU00000000000))))))))))))))))))))ffff│
└──────────────────────────────────────────────────────────────────────┘
```

### Horizontal Oscillator
*Video oscillator at 8 cycles/screen with electric gradient - sine wave pattern*
```
┌──────────────────────────────────────────────────────────────────────┐
│]]*@*]"."]*@@*]"."]*@*]""."]*@*]"."]]*@*]"."]*@@*]"."]*@*]""."]*@*]"."│
│]]*@*]"."]*@@*]"."]*@*]""."]*@*]"."]]*@*]"."]*@@*]"."]*@*]""."]*@*]"."│
│]]*@*]"."]*@@*]"."]*@*]""."]*@*]"."]]*@*]"."]*@@*]"."]*@*]""."]*@*]"."│
│]]*@*]"."]*@@*]"."]*@*]""."]*@*]"."]]*@*]"."]*@@*]"."]*@*]""."]*@*]"."│
│]]*@*]"."]*@@*]"."]*@*]""."]*@*]"."]]*@*]"."]*@@*]"."]*@*]""."]*@*]"."│
│]]*@*]"."]*@@*]"."]*@*]""."]*@*]"."]]*@*]"."]*@@*]"."]*@*]""."]*@*]"."│
│]]*@*]"."]*@@*]"."]*@*]""."]*@*]"."]]*@*]"."]*@@*]"."]*@*]""."]*@*]"."│
│]]*@*]"."]*@@*]"."]*@*]""."]*@*]"."]]*@*]"."]*@@*]"."]*@*]""."]*@*]"."│
└──────────────────────────────────────────────────────────────────────┘
```

### Wave Folding
*Paik/Abe style folding amplifier - creates complex harmonics from simple ramps*
```
┌──────────────────────────────────────────────────────────────────────┐
│  .'`^"l_{tuuU0paW@Wap0UUut{_l"^`'.  .'`^"l_{tuuU0paW@Wap0UUut{_l"^`'.│
│  .'`^"l_{tuuU0paW@Wap0UUut{_l"^`'.  .'`^"l_{tuuU0paW@Wap0UUut{_l"^`'.│
│  .'`^"l_{tuuU0paW@Wap0UUut{_l"^`'.  .'`^"l_{tuuU0paW@Wap0UUut{_l"^`'.│
│  .'`^"l_{tuuU0paW@Wap0UUut{_l"^`'.  .'`^"l_{tuuU0paW@Wap0UUut{_l"^`'.│
│  .'`^"l_{tuuU0paW@Wap0UUut{_l"^`'.  .'`^"l_{tuuU0paW@Wap0UUut{_l"^`'.│
│  .'`^"l_{tuuU0paW@Wap0UUut{_l"^`'.  .'`^"l_{tuuU0paW@Wap0UUut{_l"^`'.│
│  .'`^"l_{tuuU0paW@Wap0UUut{_l"^`'.  .'`^"l_{tuuU0paW@Wap0UUut{_l"^`'.│
│  .'`^"l_{tuuU0paW@Wap0UUut{_l"^`'.  .'`^"l_{tuuU0paW@Wap0UUut{_l"^`'.│
└──────────────────────────────────────────────────────────────────────┘
```

> **Note**: Characters map from dark (` .'"`) to bright (`*#@$`). Run `npm run dev` to see full-color interactive output, or use `npm run capture` for terminal visualization.

## Overview

This project aims to authentically recreate the complete history of electronic image-making by deeply understanding and modeling the physics, circuits, and techniques that defined analog video synthesis. Rather than simple approximations, we faithfully recreate each component down to the circuit level.

### Key Systems Emulated

- **Paik/Abe Video Synthesizer** (1969-1971) - Nonlinear folding amplifiers, resistive color matrix
- **Sandin Image Processor** (1971-1974) - Amplitude classifier, four-quadrant multiplication
- **Stephen Beck Direct Video Synthesizer** (1970-1972) - Voltage-to-position converters, geometric region processor
- **Rutt/Etra Video Synthesizer** (1972-1974) - DC-coupled raster manipulation, Z-axis displacement
- **Jones Colorizer** (1974) - Diode/layered mixing, multi-level keying
- **LZX Industries** (2008-present) - Modern modular video synthesis standards

### CRT Display Emulation

Authentic cathode ray tube characteristics:
- **P22 phosphor persistence** - Per-channel decay with different time constants for R, G, B
- **Beam bloom** - Brightness-dependent spot size expansion
- **Scanlines** - Physical line structure
- **Shadow mask / aperture grille** - RGB phosphor stripe patterns
- **Screen curvature** - Barrel distortion and vignetting

## Installation

```bash
npm install
```

## Usage

### Development

```bash
# Run the interactive demo
npm run dev

# Run tests
npm test

# Build for production
npm run build

# Visual capture (ASCII art / PPM export)
npm run capture -- gradient           # Radial gradient pattern
npm run capture -- bars               # Color bars test pattern
npm run capture -- oscillator         # Horizontal sine waves
npm run capture -- folded             # Wave folding pattern
npm run capture -- gradient --output frame.ppm  # Export as image
```

### As a Library

```typescript
import { VideoSynthesizer, createDefaultSynth } from 'analog-video-synth';

// Create a synthesizer
const synth = new VideoSynthesizer({
  width: 720,
  height: 480,
  crtEmulation: true,
});

// Add modules
synth.addModule('osc', 'video-osc', { frequency: 4 });
synth.addModule('colorizer', 'threshold-colorizer', { levels: 8 });

// Connect modules
synth.connect('osc', 'out', 'colorizer', 'in');

// Process a frame
const frameBuffer = synth.processFrame();
```

## Architecture

### Core Components

```
src/
├── core/
│   ├── types.ts          # Signal types, timing constants, utilities
│   └── sync-generator.ts # Master timing generation (NTSC/PAL)
│
├── modules/
│   ├── oscillators.ts    # LFO, video oscillators, ramp generators
│   ├── waveshapers.ts    # Folding, clipping, quantization
│   ├── colorizers.ts     # Matrix, threshold, layered colorization
│   ├── crt-emulation.ts  # Phosphor, bloom, scanlines, curvature
│   ├── video-feedback.ts # Feedback loops, frame delay
│   ├── raster-manipulation.ts # Rutt/Etra, wobbulator effects
│   └── mixing.ts         # Keying, compositing, multiplication
│
└── synth/
    └── synthesizer.ts    # Main synthesizer orchestration
```

### Signal Standards

Following LZX Industries conventions:
- **Video signals**: 0-1V range, DC coupled
- **Control voltages**: 0-1V (unipolar) or -1V to +1V (bipolar)
- **All timing**: Derived from master sync generator

## Module Reference

### Oscillators

| Module | Description |
|--------|-------------|
| `lfo` | Low-frequency oscillator for modulation |
| `video-osc` | Sync-locked pattern generator |
| `ramp-gen` | Position ramps and geometric signals |
| `shape-gen` | Rectangle, circle, diamond shapes |
| `noise-gen` | White, pink, brown noise |

### Waveshapers

| Module | Description |
|--------|-------------|
| `folder` | Paik/Abe style folding amplifier |
| `soft-clip` | Tube-style saturation |
| `hard-clip` | Digital clipping |
| `quantizer` | Amplitude classifier (Sandin style) |
| `solarize` | Threshold-based inversion |

### Colorizers

| Module | Description |
|--------|-------------|
| `matrix-colorizer` | Paik/Abe resistive matrix |
| `threshold-colorizer` | Sandin amplitude classifier |
| `layered-colorizer` | Jones diode mixing |
| `gradient-mapper` | Luminance to color gradient |

### Effects

| Module | Description |
|--------|-------------|
| `rutt-etra` | Z-axis displacement |
| `wobbulator` | Scan modulation |
| `video-feedback` | Recursive feedback loop |
| `frame-delay` | Temporal delay line |

### Mixing

| Module | Description |
|--------|-------------|
| `video-mixer` | Two-input crossfader with blend modes |
| `luma-keyer` | Luminance-based compositing |
| `chroma-keyer` | Color-based compositing |
| `four-quadrant` | MC1445 style multiplication |

## Design Philosophy

1. **Voltage control as universal modulation** - Any parameter accepts CV input
2. **Modularity enables experimentation** - Separate blocks with standardized connections
3. **Imperfection creates character** - Noise, drift, non-linearity are features
4. **Real-time performance** - Latency breaks the feedback loop
5. **Open documentation** - Following Sandin's "Distribution Religion"

## Technical Documentation

See [docs/VIDEO_SYNTHESIS_TECHNICAL_REFERENCE.md](docs/VIDEO_SYNTHESIS_TECHNICAL_REFERENCE.md) for comprehensive technical details on:

- CRT physics and phosphor characteristics
- Historical synthesizer circuit analysis
- Implementation approaches for each effect
- Performance optimization techniques

## Roadmap

### Phase 1: Foundation (Current)
- [x] Core signal types and timing
- [x] Basic oscillators and ramp generators
- [x] Fundamental waveshapers
- [x] Colorization modules
- [x] CRT display emulation
- [x] Video feedback
- [x] Rutt/Etra displacement

### Phase 2: Enhancement
- [ ] NTSC/PAL encoding artifacts
- [ ] Component drift and aging simulation
- [ ] Zero-delay feedback filters (WDF)
- [ ] GPU shader acceleration
- [ ] WebGL rendering backend

### Phase 3: Extended Features
- [ ] Audio input/reactivity
- [ ] MIDI/OSC control
- [ ] Patch preset library
- [ ] Node-based visual patching UI
- [ ] Neural network circuit modeling

## Contributing

Contributions are welcome! Please see the technical reference document for implementation guidelines.

## License

MIT

## Acknowledgments

This project draws inspiration from the pioneering work of:
- Nam June Paik & Shuya Abe
- Dan Sandin
- Stephen Beck
- Steve Rutt & Bill Etra
- Dave Jones
- Lars Larsen (LZX Industries)

And the ongoing community of video synthesis artists and engineers.
