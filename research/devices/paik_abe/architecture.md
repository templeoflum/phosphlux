# Paik/Abe Video Synthesizer Architecture

Analyzed from original schematics (26 pages).

## System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                     PAIK/ABE VIDEO SYNTHESIZER                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────┐   ┌─────┐   ┌─────────────┐   ┌───────────┐   ┌─────────┐ │
│  │CH-1 │──▶│ E.F │──▶│             │   │           │   │         │ │
│  └─────┘   └─────┘   │             │   │           │   │         │ │
│  ┌─────┐   ┌─────┐   │   KEYER     │   │  MATRIX   │   │         │ │
│  │CH-2 │──▶│ E.F │──▶│ SWITCHBOARD │──▶│  MIXING   │──▶│  VIDEO  │ │
│  └─────┘   └─────┘   │             │   │   AMP     │   │   AMP   │ │
│    ...       ...     │   (7 ch)    │   │           │   │         │ │
│  ┌─────┐   ┌─────┐   │             │   │  R,G,B    │   │         │ │
│  │CH-7 │──▶│ E.F │──▶│             │   │  outputs  │   │         │ │
│  └─────┘   └─────┘   └─────────────┘   └───────────┘   └────┬────┘ │
│                             │                                │      │
│                             ▼                                ▼      │
│                      ┌─────────────┐                  ┌──────────┐  │
│                      │ PED CONTROL │                  │ ENCODER  │  │
│                      │  (CA3019)   │                  │  (NTSC)  │  │
│                      │  Amplitude  │                  │          │  │
│                      │  Windows    │                  │ VID OUT  │  │
│                      └─────────────┘                  └──────────┘  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Key Components

### ICs Used

| IC | Type | Function | Schematic Page | Model Status |
|----|------|----------|----------------|--------------|
| CA3019 | Diode Array | Amplitude window detection, burst generator | 27, 28 | ✅ Modeled |
| CA3022 | Video Amp | Chroma key video output | 10, 20 | ❌ Need |
| CA3023 | Video Amp | Subcarrier amplifier | 26, 30 | ❌ Need |
| CA3046 | Transistor Array | Keying circuits, diff pairs | 19 | ✅ Modeled |
| CA3054 | Diff Amp | Keying signal processing | 19 | ✅ Modeled |
| CA3055 | Diff Amp | Power supply regulation | 12, 25 | ❌ Need |

### Discrete Components

- **2N2102** - Output transistors (video amps)
- **2N4400/2N4402** - Complementary pair (mixer-fader)
- **MPS5172** - Signal transistors
- **1N914/1N4148** - Signal diodes

## Functional Blocks

### 1. Input Stage (E.F. Buffers)
- 7 camera inputs
- Emitter follower buffers for impedance matching
- 75Ω termination

### 2. Keyer Switchboard (Page 15)
- Input selection switching
- MIX section for combining signals
- Gate pulse routing
- Connections to PED CTL and VID AMP

### 3. Matrix Mixing Amp (Page 8)
- Weighted summing of channels
- Silicon diode (Si-D) mixing network
- Separate R, G, B output paths
- ±12V, -6V supplies

### 4. PED Control - Amplitude Windows (Page 7)
**This is the COLORIZER core!**
- Uses CA3019 diode array
- Creates amplitude windows from luminance
- Multiple threshold levels
- Chroma input mixing
- Controls pedestal (black level)

### 5. Keying Circuit (Page 16)
- CA3054 differential amplifier
- CA3046 transistor array for precise switching
- Video balance control
- Gate signal for hard/soft keying
- External control panel input

### 6. Video Amplifiers (Pages 9, 10, 12, 17)
- CA3022 and CA3023 ICs
- Wide bandwidth for video
- Output buffering
- Sync insertion point

### 7. Subcarrier Amp (Pages 3, 4)
- 3.58 MHz color subcarrier processing
- CA3023 amplifier
- Clamp pulse generation
- Phase control

### 8. NTSC Encoder (Pages 22-26)
- Complex PCB with many transistors
- White balance controls
- Chrominance level
- Burst flag shaping
- Composite video output

## Signal Levels

- Video: 1V p-p (standard NTSC)
- Supplies: +15V, +12V, -6V
- Impedance: 75Ω

## Key Transfer Functions

### CA3019 Amplitude Window
The CA3019 creates sharp threshold windows using matched diodes:
```
window(signal, low, high) = (signal > low) AND (signal < high)
```
Multiple windows create the color banding effect.

### CA3046 Differential Pair
Used for gain control and keying:
```
output = tanh((v1 - v2) / (2 * Vt)) * tail_current
```
Where Vt ≈ 26mV (thermal voltage).

### Matrix Mixing
Weighted sum of channels:
```
R_out = Σ(channel[i] * weight_R[i])
G_out = Σ(channel[i] * weight_G[i])
B_out = Σ(channel[i] * weight_B[i])
```

## Emulation Priority

1. **PED Control (Amplitude Windows)** - Core colorizer effect
2. **Matrix Mixer** - Channel combining
3. **Keying Circuit** - Hard/soft key switching
4. **Video Amps** - Gain and offset control

## Notes from Schematics

- Hand-drawn original schematics with component values
- Some pages show PCB layouts (encoder)
- Test points labeled (TP-1 through TP-17)
- Multiple identical amplifier stages ("SAME AS Above" noted)
- Careful attention to grounding and decoupling
