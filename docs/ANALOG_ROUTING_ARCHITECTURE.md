# Analog Signal Routing Architecture

## Philosophy: Everything is a Module

In real analog video systems, there is no "master feedback" or "global CRT effect". Instead:
- Every piece of hardware is a module with inputs and outputs
- Feedback emerges naturally when you route an output back to an input
- Multiple independent signal paths can exist simultaneously
- The patch cables (connections) define the behavior

This document describes how to implement this properly in Phosphlux.

---

## Core Architectural Principles

### 1. No Global State
- Remove all "master" parameters (feedback, CRT, etc.)
- Every effect is a node in the graph
- The graph topology IS the configuration

### 2. Feedback as Topology
Current (wrong):
```
Signal → Processing → Output
            ↓
       [Global Feedback Buffer]
```

Correct (analog):
```
Signal → Mixer A-in
              ↓
         Processing → Monitor → [Physical Output]
              ↓
         Frame Buffer
              ↓
         Mixer B-in (creates feedback loop)
```

### 3. Typed I/O Ports
Ports have types that determine valid connections:
- `Video` - Full RGB signal (vec4)
- `Luma` - Single channel luminance (f32)
- `Key` - Matte/alpha signal (f32)
- `Control` - Parameter modulation (f32)

### 4. Multiple Render Targets
- Multiple Monitor nodes can exist
- Each Monitor renders to its own texture
- Preview monitors vs program monitors vs scopes

---

## Node Categories

### Sources (Outputs Only)
Generate signals from nothing.

| Node | Outputs | Description |
|------|---------|-------------|
| `Oscillator` | Video | Generates waveforms (H/V ramp, sine, etc.) |
| `Noise` | Video | White/pink/brown noise |
| `Shape` | Luma | Circle, rectangle, diamond shapes |
| `External Input` | Video | Camera/video file input (future) |
| `Color Bars` | Video | SMPTE/EBU test patterns |

### Frame Buffers (Read/Write)
Enable temporal effects and feedback.

| Node | Inputs | Outputs | Description |
|------|--------|---------|-------------|
| `Frame Buffer` | Video | Video | Stores frame, outputs previous frame |
| `Delay Line` | Video | Video | N-frame delay buffer |
| `Freeze Frame` | Video, Trigger | Video | Captures and holds frame |

**Key insight**: A Frame Buffer node outputs the PREVIOUS frame's content while storing the CURRENT input. This single-frame delay is what enables feedback loops.

### Processors (Inputs → Outputs)
Transform signals.

| Node | Inputs | Outputs | Description |
|------|--------|---------|-------------|
| `Folder` | Video | Video | Wavefolding distortion |
| `Quantizer` | Video | Video | Posterization |
| `Soft Clip` | Video | Video | Saturation/limiting |
| `Solarize` | Video | Video | Threshold inversion |
| `Invert` | Video | Video | Signal inversion |
| `Hue Shift` | Video | Video | Color rotation |
| `Colorizer` | Luma | Video | Maps luma to color gradient |

### Keyers (Multiple Inputs)
Combine signals based on keys.

| Node | Inputs | Outputs | Description |
|------|--------|---------|-------------|
| `Luma Key` | A: Video, B: Video | Video | Keys based on luminance |
| `Chroma Key` | A: Video, B: Video, Hue | Video | Keys based on color |
| `Matte Key` | A: Video, B: Video, Key: Luma | Video | External key input |

### Mixers
Combine multiple video signals - modeled on real hardware.

| Node | Inputs | Outputs | Description |
|------|--------|---------|-------------|
| `Two Channel Mixer` | A: Video, B: Video | Video | Simple A/B crossfade |
| `V4 Style Mixer` | A: Video, B: Video, C: Video, D: Video | Program: Video, Preview: Video | Four-input mixer with effects |

**V4 Style Mixer Details**:
- 4 input channels (A, B, C, D)
- T-bar crossfader between A/B and C/D banks
- Per-channel effects (colorize, negative, strobe)
- Program output (main mix)
- Preview output (next transition preview)
- Wipe patterns between banks

### Displays (Inputs Only)
Render signals to viewable outputs.

| Node | Inputs | Description |
|------|--------|-------------|
| `Monitor` | Video | Clean display for preview |
| `CRT Display` | Video | Full CRT emulation with all parameters |
| `Vectorscope` | Video | Color vector display |
| `Waveform` | Video | Luminance waveform display |

**CRT Display Parameters** (all on the node, not global):
- Phosphor type (P31 green, P22 RGB, amber, white)
- Scan line count and intensity
- Horizontal/vertical geometry (barrel, pincushion)
- Convergence (RGB alignment)
- Bloom and glow
- Degauss effect
- Noise and interference

### Routing Utilities
Enable complex signal routing.

| Node | Inputs | Outputs | Description |
|------|--------|---------|-------------|
| `Patch Bay` | 8× Video | 8× Video | Instant preset routing changes |
| `Matrix Router` | N× Video | M× Video | Any-to-any routing matrix |
| `Mult` | Video | 3× Video | Split signal to multiple destinations |
| `Black Burst` | - | Video | Sync/black reference signal |

---

## Example Patches

### Classic Feedback Loop
```
Oscillator → Mixer (A)
                ↓
Frame Buffer ← Folder ← Colorizer
    ↓
Mixer (B) ←──┘
    ↓
CRT Display
```

### Dual Independent Loops
```
Oscillator 1 → Mixer 1 → Frame Buffer 1 → Mixer 1
                              ↓
                         CRT Display 1

Oscillator 2 → Mixer 2 → Frame Buffer 2 → Mixer 2
                              ↓
                         CRT Display 2
```

### V4-Style Live Performance Setup
```
Camera 1 ────→ V4 Mixer (A) ───→ Program Out → CRT Display
Camera 2 ────→ V4 Mixer (B)         ↓
                                Frame Buffer
Video Synth → V4 Mixer (C)          ↓
                    ↑           V4 Mixer (D)
                    └───────────────┘

Preview Out → Monitor (clean preview)
```

---

## Implementation Details

### Graph Compilation with Cycles

The current topological sort fails on cyclic graphs. New approach:

1. **Identify Frame Buffer nodes** - These are the "cut points" for cycles
2. **Break cycles at Frame Buffers** - Frame Buffer reads happen BEFORE writes
3. **Two-pass execution**:
   - Pass 1: Read all Frame Buffer outputs (previous frame data)
   - Pass 2: Process graph, write to Frame Buffers for next frame

```rust
pub struct CompiledGraph {
    // Pre-execution: Read frame buffer outputs
    frame_buffer_reads: Vec<FrameBufferRead>,

    // Main execution: Process in dependency order
    operations: Vec<GraphOp>,

    // Post-execution: Write frame buffer inputs
    frame_buffer_writes: Vec<FrameBufferWrite>,

    // Render targets
    display_outputs: Vec<DisplayOutput>,
}
```

### Frame Buffer Implementation

```rust
pub struct FrameBufferNode {
    // Two textures for ping-pong
    texture_a: wgpu::Texture,
    texture_b: wgpu::Texture,

    // Which texture is "previous frame"
    read_index: usize,
}

impl FrameBufferNode {
    fn get_read_texture(&self) -> &wgpu::Texture {
        if self.read_index == 0 { &self.texture_a } else { &self.texture_b }
    }

    fn get_write_texture(&self) -> &wgpu::Texture {
        if self.read_index == 0 { &self.texture_b } else { &self.texture_a }
    }

    fn swap(&mut self) {
        self.read_index = 1 - self.read_index;
    }
}
```

### Monitor Node Implementation

Each Monitor node needs its own render target:

```rust
pub struct MonitorNode {
    output_texture: wgpu::Texture,
    output_view: wgpu::TextureView,

    // CRT parameters (if CRT type)
    crt_params: Option<CrtParams>,
}

pub struct CrtParams {
    pub phosphor_type: PhosphorType,
    pub scanline_count: u32,
    pub scanline_intensity: f32,
    pub curvature: f32,
    pub bloom: f32,
    pub convergence: [f32; 2],
    // ... etc
}
```

### Port Type System

```rust
#[derive(Clone, Copy, PartialEq)]
pub enum PortType {
    Video,   // vec4 RGB + alpha
    Luma,    // f32 luminance
    Key,     // f32 matte
    Control, // f32 0-1 modulation
}

pub struct InputPort {
    pub port_type: PortType,
    pub name: &'static str,
    pub connected_to: Option<(NodeId, OutputIndex)>,
}

pub struct OutputPort {
    pub port_type: PortType,
    pub name: &'static str,
}
```

Type compatibility rules:
- `Video → Video`: Direct connection
- `Luma → Video`: Expands to grayscale RGB
- `Video → Luma`: Extracts luminance
- `Key → Luma`: Direct (same underlying type)
- `Control → any`: Modulates a parameter

---

## Migration Path

### Phase 1: Frame Buffer Node
1. Add `FrameBuffer` node type
2. Implement ping-pong texture swap
3. Update compiler to handle cycles via frame buffers
4. Remove global feedback - users create feedback with Frame Buffer nodes

### Phase 2: Monitor Nodes
1. Add `Monitor` and `CRTDisplay` node types
2. Move CRT parameters to node params
3. Each Monitor gets its own render target
4. Remove global CRT post-processing

### Phase 3: Enhanced Mixers
1. Add `TwoChannelMixer` with proper A/B/Mix
2. Add `V4Mixer` with banks, effects, preview
3. Remove simple `Mixer` node

### Phase 4: Routing Utilities
1. Add `Mult` for signal splitting
2. Add `PatchBay` for preset routing
3. Add `MatrixRouter` for complex setups

---

## UI Considerations

### Node Graph Enhancements
- Color-coded ports by type
- Visual feedback loop indicators
- Texture preview on Frame Buffer nodes
- Waveform preview on Monitor nodes

### Performance Monitoring
- Per-node GPU time display
- Frame buffer memory usage
- Texture resolution indicators

### Preset System
- Save/load entire patches
- "Patch bay" quick-swap presets
- Parameter automation recording

---

## Future Possibilities

With this architecture, we can add:
- **External video input** - Just another source node
- **Video file playback** - Source node with transport controls
- **Recording** - Monitor variant that writes to disk
- **Network streaming** - Source/destination nodes for NDI/Syphon
- **Hardware control** - MIDI/OSC controlling any node parameter
- **Multiple windows** - Each Monitor can be a separate OS window

The key insight is that **the architecture doesn't need to change** to add these features - they're just new node types that plug into the existing routing system.
