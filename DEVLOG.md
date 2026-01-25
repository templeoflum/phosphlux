# Development Log: Phosphlux

*phosphor + flux — flowing light substance*

A chronicle of building a real-time GPU analog video synthesizer and CRT emulation.

---

## Project Overview

**Goal**: Create an authentic digital emulation of classic analog video synthesis hardware, from 1960s oscilloscope art through contemporary Eurorack modular video systems.

**Key Achievement**: 2 FPS → 60 FPS — A 30x performance improvement by transitioning from TypeScript/Canvas to Rust/wgpu.

---

## The Name

**Phosphlux** combines two essential concepts:
- **Phosphor**: The luminescent coating inside CRT displays that glows when struck by electron beams — the literal source of light in analog video
- **Flux**: Continuous flow, change, transformation — capturing the dynamic nature of video synthesis

Together they describe the essence of what we're building: *light as a flowing, transforming substance*.

---

## Phase 1: Research & Foundation

### The Technical Reference Document

The project began with exhaustive research into the history and physics of analog video synthesis. The result was `VIDEO_SYNTHESIS_TECHNICAL_REFERENCE.md` — a comprehensive 600-line technical document covering:

- **CRT Physics**: Electron beam behavior, P22 phosphor persistence with multi-stage decay, beam bloom from space charge effects
- **NTSC/PAL Standards**: 525/625 line systems, color subcarrier encoding (3.579545 MHz), quadrature amplitude modulation
- **Historical Synthesizers**:
  - **Paik/Abe** (1969-1971): Seven-channel colorizer with nonlinear folding amplifiers and "wobbulator" scan modulation
  - **Sandin Image Processor** (1971-1974): Modular design with MC1445 gain-controlled amplifiers, amplitude classifier
  - **Stephen Beck DVS** (1970-1972): Constructivist synthesis with voltage-to-position converters
  - **Rutt/Etra** (1972-1974): DC-coupled raster manipulation, the famous Z-axis displacement effect
  - **Jones Colorizer** (1974): Diode/layered mixing without color encoder
- **Implementation Priorities**: Categorized techniques into Critical/High/Medium priority with specific algorithms

This document served as the "sketch" — a detailed blueprint for what needed to be built.

---

## Phase 2: TypeScript Prototype

### Initial Implementation

The first working version was built in TypeScript with a Canvas/WebGL backend:

```
__archive_typescript_reference/
├── src/
│   ├── core/           # Signal types, timing, sync generation
│   ├── modules/        # Processing modules (oscillators, waveshapers, etc.)
│   └── synth/          # Orchestration layer
├── demo/               # Browser-based interactive demo
└── scripts/            # Capture utilities for ASCII/PPM output
```

**Modules Implemented**:
- Oscillators: LFO, video-rate oscillator, ramp/shape generators, noise (white/pink/brown)
- Waveshapers: Paik/Abe folding amplifier, soft/hard clipping, Sandin-style quantizer
- Colorizers: Matrix, threshold, layered (Jones style), gradient mapper
- CRT Emulation: Phosphor persistence, beam bloom, scanlines, shadow mask, curvature
- Effects: Video feedback, frame delay, Rutt/Etra displacement, wobbulator
- Mixing: Luminance keyer, chroma keyer, four-quadrant multiplier

**The Problem**: Performance was abysmal. The TypeScript implementation ran at approximately **2 FPS** for 720x480 resolution. Video synthesis requires real-time performance — latency breaks the feedback loop that's essential to the art form.

### Lessons Learned

1. JavaScript/TypeScript isn't suitable for per-pixel processing at video rates
2. Even WebGL couldn't fully overcome the overhead of the JS runtime
3. The architecture was sound, but the platform was wrong

---

## Phase 3: Rust Rewrite

### Why Rust + wgpu?

- **Performance**: Native code with zero-cost abstractions
- **GPU Access**: wgpu provides modern, cross-platform GPU compute
- **Memory Safety**: Borrow checker prevents the kinds of bugs that plague real-time systems
- **Ecosystem**: egui for immediate-mode UI, winit for windowing

### Architecture

```
src/
├── core/
│   └── timing.rs       # SyncGenerator - NTSC/PAL master timing
├── gpu/
│   ├── mod.rs
│   └── renderer.rs     # wgpu renderer with ping-pong feedback buffers
├── modules/
│   ├── mod.rs
│   ├── oscillators.rs  # CPU-side oscillator logic
│   └── waveshapers.rs  # Waveshaping functions
├── ui/
│   ├── mod.rs
│   ├── controls.rs     # SynthParams - all UI parameters
│   └── patch.rs        # SignalPatch/SignalNode - node graph data structures
└── main.rs             # Application entry point, egui integration
```

### The Shader Pipeline

All video processing happens in WGSL shaders:

1. **Composite Shader** (`composite.wgsl`): Generates base imagery
   - Shape generators (circle, rectangle, diamond, star, bars, checker)
   - Noise generation (white, pink, brown — GPU random with filtering)
   - Oscillator patterns
   - Colorization (spectrum, threshold, grayscale)
   - Luma keying
   - Luminance displacement

2. **Feedback Shader** (`feedback.wgsl`): Recursive processing
   - Ping-pong buffer architecture (read A → write B → swap)
   - Configurable: zoom, rotation, offset, hue shift, saturation, decay
   - Soft clipping prevents value runaway

3. **CRT Shader** (`crt.wgsl`): Display emulation
   - Scanline simulation
   - Screen curvature (barrel distortion)
   - Vignette

### Performance Result

**60 FPS** at full resolution with all effects active. This represents a **30x improvement** over the TypeScript prototype.

The key insight: GPUs are massively parallel. Every pixel can be processed simultaneously. The feedback loop that defines analog video synthesis maps perfectly to ping-pong framebuffer rendering.

---

## Phase 4: Control Panel UI

### egui Integration

Added real-time parameter control using egui (immediate-mode GUI):

```rust
pub struct SynthParams {
    pub feedback: FeedbackUniforms,
    pub key_enabled: bool,
    pub key_clip_black: f32,
    pub key_clip_white: f32,
    pub rutt_etra_enabled: bool,
    pub rutt_etra_amount: f32,
    pub noise_mix: f32,
    pub noise_type: NoiseType,
    // ... etc
}
```

**Control Categories**:
- **Feedback**: Mix, zoom, rotation, hue shift, decay, saturation, offset
- **Luma Key**: Enable, clip black/white, invert
- **Luminance Displacement**: Enable, amount
- **Oscillators**: Frequency, phase for 2 oscillators
- **Noise**: Mix amount, type (white/pink/brown)
- **Colorizer**: Mode (spectrum/threshold/grayscale), levels
- **Shapes**: Type, size, softness
- **CRT Effects**: Scanlines, curvature, vignette

### Preset System

Keyboard shortcuts and menu for quick feedback presets:
- **1/Spiral**: Rotating spiral tunnel
- **2/Tunnel**: Zooming tunnel effect
- **3/Chaos**: High-energy chaotic feedback
- **4/Drift**: Subtle drifting patterns
- **5/Kaleidoscope**: Colorful kaleidoscopic rotation

---

## Phase 5: Blender-Style Paned Layout

### The Problem with Floating Panels

Initial UI had a floating control panel over the video output. User feedback indicated this wasn't ideal:
- Covered the video preview
- Felt disconnected from the content
- Not scalable for future node graph editor

### Solution: Maximized Window with Resizable Panes

Redesigned to match professional creative tools (Blender, TouchDesigner, Nuke):

```
┌─────────────────────────────────────────────────────────┐
│ File  Presets  Help                           [60 FPS]  │
├──────────────────┬──────────────────────────────────────┤
│ [Parameters]     │                                      │
│ [Node Graph]     │                                      │
│                  │                                      │
│ ▼ Feedback       │         VIDEO PREVIEW                │
│   Mix: ═══●═══   │                                      │
│   Zoom: ══●════  │    (GPU output rendered to          │
│   ...            │     egui texture widget)            │
│                  │                                      │
│ ▼ Luma Key       │                                      │
│   ...            │                                      │
│                  │                                      │
└──────────────────┴──────────────────────────────────────┘
```

### Technical Implementation

**Render-to-Texture Pipeline**:
1. Render synthesizer output to an offscreen texture
2. Register texture with egui renderer
3. Display texture in egui `Image` widget in central panel
4. Handle dynamic resize when pane boundaries change

```rust
// In renderer.rs
pub fn render_to_texture(&mut self, timing: TimingInfo) {
    // Render synth to output_texture instead of surface
    // ...
}

pub fn register_output_texture(&mut self, egui_renderer: &mut egui_wgpu::Renderer) {
    self.output_texture_id = Some(egui_renderer.register_native_texture(
        self.device(),
        &self.output_view,
        wgpu::FilterMode::Linear,
    ));
}
```

**Borrow Checker Challenge**:
Rust's borrow checker initially blocked the egui integration. The solution was extracting render logic into free functions that take specific mutable references rather than `&mut self`:

```rust
fn render_paned_layout_inner(
    ctx: &egui::Context,
    control_panel_params: &mut SynthParams,
    patch: &mut SignalPatch,
    left_panel_mode: &mut LeftPanelMode,
    // ... individual borrows instead of &mut App
)
```

---

## Phase 6: Visual Node Editor

### The Vision
Transform from parameter-based control to true node-based signal routing — like TouchDesigner, Nuke, or Blender's shader editor.

### Implementation

**New Layout (4-panel design):**
```
┌──────────┬────────────────────────────────────┬─────────────┐
│          │                                    │  PREVIEW    │
│  NODE    │                                    │   [4:3 ▼]   │
│ PALETTE  │       NODE GRAPH EDITOR            ├─────────────┤
│          │                                    │  INSPECTOR  │
│          │                                    │             │
└──────────┴────────────────────────────────────┴─────────────┘
```

**Node Editor Features:**
- Pan/zoom canvas with grid background
- Visual nodes with category-colored headers
- Bezier curve connections between sockets
- Immediate input response (no drag threshold lag)

**Connection Management (major UX focus):**
- Drag from output socket → input socket to connect
- Drag from connected input to **unplug and reconnect** elsewhere
- Right-click on cables to delete connections
- Hover highlighting on nodes, sockets, and cables
- Inspector panel shows connections with disconnect buttons

**Keyboard Shortcuts:**
- `Del`/`Backspace` — Delete selected node
- `F` — Frame/center on selected node
- `Home` — Reset view (pan and zoom)

**Aspect Ratio Lock:**
- Preview supports locked aspect ratios: Free, 1:1, 4:3, 16:9, 3:4, 9:16, 2.35:1
- Ensures graphics render faithfully without warping
- Dropdown selector in preview header

**Node Categories:**
- **Sources**: Ramps (H/V/Radial), Oscillators, Noise (White/Pink/Brown), Shapes, Feedback Input
- **Waveshapers**: Folder, Quantizer, Soft Clip, Solarize, Inverter
- **Mixing**: Luma Key, Chroma Key, Mixer, Multiplier
- **Color**: Hue Shift
- **Colorizers**: Spectrum, Threshold, Gradient
- **Outputs**: Main Out, Feedback Out

### Technical Challenges Solved

**Immediate Node Interaction:**
Original implementation used egui's `drag_started()` which has a minimum drag threshold — users had to "click and hold" before nodes responded. Solution: switched to raw pointer state (`primary_pressed`/`primary_down`/`primary_released`) for immediate response.

**Connection Unplugging:**
Implemented `ConnectionDragMode` enum to distinguish between:
- `FromOutput` — Creating new connection
- `FromInput` — Unplugging existing connection (stores original source for reconnection)

**Borrow Checker Conflicts:**
Multiple operations required careful two-pass approaches:
1. First pass: collect information (immutable borrow)
2. Second pass: perform mutation (mutable borrow)

---

## Current State

### What Works
- Real-time 60 FPS rendering
- Full feedback effect pipeline with 8 parameters
- Luma keying with adjustable thresholds
- Luminance displacement effect
- Multiple noise types (white, pink, brown)
- Shape generators (7 types)
- Colorization modes (spectrum, threshold, grayscale)
- CRT emulation (scanlines, curvature, vignette)
- Professional paned layout UI
- **Visual node editor with full connection management**
- **Aspect ratio locked preview**
- **GPU-compiled node graph execution**
- **Real-time parameter adjustment**
- **Per-node Frame Buffer textures for feedback**
- **Mixer with blend modes and luma keying**

### Architecture Strengths
- Clean separation: timing (CPU) → uniforms → shaders (GPU)
- Ping-pong feedback buffers for stable recursion
- Type-safe parameter binding via uniform structs
- **Graph compiler with topological sort and cycle-aware compilation**
- **Storage buffer pipeline for dynamic operation dispatch**
- **Per-node temporal textures for independent feedback loops**

---

## Roadmap

### ✅ COMPLETE: GPU Signal Routing (Phase 3)
The node graph now controls real-time GPU rendering:
- ✅ Visual node editor with pan/zoom canvas
- ✅ Draggable nodes with input/output sockets
- ✅ Bezier curve connections
- ✅ **Graph compiler with topological sort**
- ✅ **GPU storage buffer for operations**
- ✅ **Real-time parameter updates**
- ⏳ Multiple signal channels (Main, Feedback, Key, Aux)

**Implementation Details:**
- `GraphCompiler` performs topological sort (Kahn's algorithm)
- Operations packed into `GraphOp` structs (32 bytes each)
- Header uniform + ops storage buffer (bind group 2)
- Shader uses unrolled execution (16 ops max) for GPU performance
- Slots reduced from 64→16 to minimize per-fragment memory
- Legacy hardcoded path preserved when no graph present

### Future: True Rutt/Etra Implementation
The current "Luminance Displacement" effect is UV-based distortion. True Rutt/Etra:
- Geometry-based rendering (draw scan lines as displaced line geometry)
- Z-axis displacement in 3D space where brightness = depth
- Proper scan-line aesthetics

### Long-term Vision
- Video input support (camera, file)
- Audio reactivity
- MIDI/OSC control
- Patch save/load
- Recording/export

---

## Technical Decisions Log

### Why wgpu over raw Vulkan/DirectX?
- Cross-platform (Windows, Mac, Linux, WebGPU)
- Higher-level API reduces boilerplate
- Active development, good Rust integration
- WebGPU compatibility for potential web port

### Why egui over other GUI frameworks?
- Immediate mode fits real-time rendering
- Built-in wgpu integration
- Simple API, rapid iteration
- No callback hell

### Why Ping-Pong Buffers for Feedback?
- GPU can't read and write same texture simultaneously
- Two textures: read from A, write to B, then swap
- Matches how analog feedback actually works (delay through processing chain)

### Uniform Buffer Design
Shader parameters packed into structs matching WGSL layout:
```rust
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SynthUniforms {
    pub time: f32,
    pub resolution: [f32; 2],
    // ... 40+ parameters
}
```
This allows single `queue.write_buffer()` call per frame.

---

## Lessons Learned

1. **Prototype in high-level, ship in low-level**: TypeScript prototype validated the architecture; Rust made it performant

2. **GPUs are parallel, use them that way**: Per-pixel operations map perfectly to fragment shaders

3. **Borrow checker is your friend**: The solutions it forces (extracting logic to free functions) often improve architecture

4. **UI architecture matters**: Floating panels → paned layout was a significant UX improvement

5. **Document the "why"**: Understanding original analog circuits informed correct digital implementation

---

## Files Reference

| File | Purpose |
|------|---------|
| `src/main.rs` | Application entry, event loop, egui integration |
| `src/gpu/renderer.rs` | wgpu setup, shader pipeline, feedback buffers |
| `src/gpu/shaders/composite.wgsl` | Main synthesis shader |
| `src/gpu/shaders/feedback.wgsl` | Feedback processing shader |
| `src/gpu/shaders/crt.wgsl` | CRT emulation shader |
| `src/core/timing.rs` | NTSC/PAL sync generation |
| `src/ui/controls.rs` | Parameter structures |
| `src/ui/patch.rs` | Node graph data structures |
| `docs/VIDEO_SYNTHESIS_TECHNICAL_REFERENCE.md` | Research document |

---

## Files Reference (Updated)

| File | Purpose |
|------|---------|
| `src/main.rs` | Application entry, event loop, 4-panel layout |
| `src/gpu/renderer.rs` | wgpu setup, per-node Frame Buffer textures, graph upload |
| `src/gpu/graph_compiler.rs` | Cycle-aware topological sort, node→GPU operation compilation |
| `src/shaders/main.wgsl` | Combined synthesis shader + graph evaluation + blend modes |
| `src/core/timing.rs` | NTSC/PAL sync generation |
| `src/core/signal.rs` | Signal types and voltage standards (foundational) |
| `src/ui/patch.rs` | Node graph data (SignalPatch, SignalNode, Connection) |
| `src/ui/node_editor.rs` | Visual node editor (NodeEditor, AspectRatio, Inspector) |
| `docs/VIDEO_SYNTHESIS_TECHNICAL_REFERENCE.md` | Research document |
| `docs/ANALOG_ROUTING_ARCHITECTURE.md` | Per-node feedback architecture design |

---

## Phase 7: Analog Routing Architecture

### The Problem with Global Effects
Previous architecture had global feedback and CRT parameters that applied uniformly. Real analog synthesizers have:
- Multiple independent feedback paths
- Per-channel mixing with various blend modes
- Structure for feedback to "grab onto" (pure feedback fades to nothing)

### Solution: Per-Node Temporal Textures

**Frame Buffer Nodes:**
- Each Frame Buffer node gets its own ping-pong texture pair
- Parameters (zoom, rotation, hue shift, decay, etc.) are per-node
- Multiple independent feedback loops can coexist
- Graph compiler handles cycles by treating Frame Buffers as temporal nodes

**Mixer Node with Blend Modes:**
- 8 blend modes: Mix, Add, Multiply, Screen, Overlay, Difference, Luma Key (A), Luma Key (B)
- Per-channel fader with fine drag control (0.002 speed)
- Built-in luma keying with threshold and softness
- Key invert toggle

**Code Cleanup:**
- Removed ~500 lines of dead code
- Deleted: `channels.rs`, `modules/mod.rs`, `controls.rs`
- Removed global feedback/CRT parameter sync
- Added `#[allow(dead_code)]` for foundational types kept for future use

---

## Phase 8: Research Orchestration Infrastructure

### Historical Emulation Philosophy
**"Follow the inventors, not the interpreters."**

Rather than implementing effects based on secondhand descriptions, we trace the actual historical evolution of video art hardware. This provides:
- Authentic behavior including "happy accidents"
- Deep first-hand understanding
- Complete palette of each device's capabilities
- Historical continuity

### Research Agent Architecture

Created `tools/research-orchestrator/` — an MCP server that exposes Ollama models as tools for Claude CLI.

**Architecture: Claude as Orchestrator + Quality Gate**

Local Ollama models handle heavy lifting (research, code drafts, component extraction). Claude reviews all outputs, catches errors/inconsistencies, and makes final implementation decisions. Nothing gets committed without Claude's verification.

**Tiered Model System:**

| Tier | Model | Speed | Purpose |
|------|-------|-------|---------|
| Fast | phi4-mini | ~4s | Quick lookups, simple questions |
| Deep | qwen3 | ~22s | Research synthesis, shader drafts |
| Vision | moondream | ~5-10s | Component extraction, OCR |
| Embed | qwen3-embedding | ~2s | Knowledge base search |

**Tools Available:**

| Tool | Tier | Purpose |
|------|------|---------|
| `quick_query` | Fast | Simple questions, definitions |
| `research_query` | Deep | Video synthesis research |
| `generate_shader` | Deep | WGSL drafts (Claude reviews) |
| `extract_from_image` | Vision | Component lists from schematics |
| `embed_document` | Embed | Add to knowledge base |
| `search_knowledge` | Embed | Search knowledge base |

**Design Decisions:**
- Removed `verify_emulation` tool — Claude handles all verification
- Removed slow vision models (qwen3-vl took 11+ minutes) — Claude handles deep schematic analysis directly
- Added moondream for quick component extraction only
- Added `quick_query` for fast lookups via phi4-mini

### Priority Devices (from HISTORICAL_EMULATION_PLAN.md)
1. **CRITICAL**: Paik/Abe, Sandin IP, Rutt/Etra, LZX Visionary, Sony PVM
2. **HIGH**: Oscilloscope Art, Beck DVS, Jones Colorizer, LZX Cadet, Edirol V4

---

## Version History

- **v0.1**: TypeScript prototype (2 FPS)
- **v0.2**: Rust + wgpu rewrite (60 FPS)
- **v0.3**: egui control panel integration
- **v0.4**: Paned layout redesign + renamed to **Phosphlux**
- **v0.5**: Visual node editor with full UX
- **v0.6**: GPU signal routing — node graph compiles to GPU operations
- **v0.7**: Analog routing architecture — per-node Frame Buffers, blend modes, codebase cleanup
- **v0.8**: Research orchestrator — tiered MCP server with Claude as quality gate
- **v0.9**: Swarm agent system — multi-agent component research and WGSL model generation

---

## Phase 9: Swarm Agent System & Component-Level Emulation

### Philosophy: Component-Level Analog Hardware Emulation

The key insight driving this phase: **authentic emulation requires modeling at the component level, not the effect level**.

Instead of asking "what does a Paik/Abe colorizer look like?", we ask:
1. What ICs are in the circuit? (CA3046, CA3080, LM311, etc.)
2. What are their transfer functions? (BJT exponential, OTA gm control, comparator threshold)
3. How are they connected? (differential pairs, current mirrors, amplitude windows)

This approach ensures:
- **Authenticity**: Behavior emerges from real circuit physics
- **Reusability**: Model CA3046 once, use it in Paik/Abe, Sandin IP, and any circuit that uses it
- **Happy accidents**: Analog quirks (soft clipping, thermal voltage effects) come naturally

### Document Priority Hierarchy

Research materials ordered by information density:
```
Patents > Service Manuals > Schematics > Component Datasheets > Academic Papers
```

### Swarm Agent Architecture

Built a multi-agent system in `tools/swarm-agent/` for parallel research and development:

```
tools/swarm-agent/
├── swarm/
│   ├── orchestrator.py      # Coordinates agents, handles events
│   ├── shared_state.py      # Filesystem-based state (research/ folder)
│   ├── event_bus.py         # Inter-agent communication
│   ├── base_agent.py        # Agent base class
│   ├── research_agent.py    # Ollama-powered research
│   ├── development_agent.py # Code generation
│   ├── documentation_agent.py # Docs and commits
│   ├── component_agent.py   # IC lookup and WGSL generation
│   ├── token_tracker.py     # Cloud vs local cost tracking
│   └── model_benchmark.py   # A/B testing and self-improvement
└── tests/
    └── test_swarm.py        # Comprehensive test suite
```

**Agent Responsibilities:**

| Agent | Role |
|-------|------|
| **Orchestrator** | Coordinates workflow, handles pauses, routes PDF downloads |
| **ResearchAgent** | Uses Ollama for research queries, tracks token usage |
| **ComponentAgent** | Looks up ICs, generates spec.json and model.wgsl |
| **DevelopmentAgent** | Generates shader code, requests component models |
| **DocumentationAgent** | Tracks progress, prepares commits |

### Token Tracking & Cost Analysis

The system tracks token usage across cloud (Claude) and local (Ollama) models:

```python
class TokenTracker:
    def record_cloud(self, operation, input_tokens, output_tokens, latency_ms, agent)
    def record_local(self, model_name, operation, input_tokens, output_tokens, latency_ms, agent)
    def get_session_stats() -> Dict  # Returns cost analysis and savings
```

**Cost Model:**
- Claude input: $0.003/1K tokens
- Claude output: $0.015/1K tokens
- Local models: $0 (compute cost amortized)

By offloading research queries to local Ollama models, we achieve significant cost savings while maintaining quality through Claude's review.

### Model Benchmarking & Self-Improvement

The `ModelBenchmark` class enables A/B testing of prompt variants:

```python
class ModelBenchmark:
    def add_ground_truth(topic, facts, source, confidence)
    def record_response(model, variant, topic, response, latency_ms)
    def get_best_model() -> (model, variant)  # Returns optimal configuration
```

**Benchmark Results (Paik/Abe Folding Amplifier Query):**

| Model | Time | Accuracy | Notes |
|-------|------|----------|-------|
| tinyllama | 2.6s | ✗ | Wrong (generic amp) |
| smollm2 | 4.7s | ✗ | Wrong (Sony product) |
| phi4-mini | 5.4s | ~ | Partial (correct inventors) |
| llama3.1 | 7.0s | ✓ | Correct (video synthesis context) |
| granite3.3 | 12.2s | ✓ | **Best** (correct + detailed) |
| qwen3 | 18.8s | ~ | Partial (audio focus) |

**Finding**: Only 2 of 15 models correctly identified the Paik/Abe folding amplifier as a video synthesis device. This validates our approach of using Claude for quality review.

### Tier Switching Performance

```
┌─────────┬─────────────┬────────────┬─────────────┐
│ Tier    │ Cold Start  │ Warm Mean  │ Best For    │
├─────────┼─────────────┼────────────┼─────────────┤
│ Fast    │ 8.7s        │ 3.4s       │ Definitions │
│ Deep    │ 9.3s        │ 10.2s      │ Research    │
│ Embed   │ 4.4s        │ 0.1s       │ Search      │
└─────────┴─────────────┴────────────┴─────────────┘

Tier switching overhead: ~3.7s average
```

### Component Library

Created `research/components/` — a reusable library of analog IC models:

```
research/components/ics/
├── ca3046/
│   ├── spec.json        # Full BJT SPICE parameters
│   ├── model.wgsl       # Core: differential_pair(), current_mirror()
│   └── colorizer.wgsl   # App: Paik/Abe amplitude window colorizer
├── ca3080/
│   ├── spec.json        # OTA specifications
│   ├── model.wgsl       # Core: ota_transfer(), ota_vca()
│   └── vca.wgsl         # App: Voltage controlled amplifier
└── lm311/
    ├── spec.json        # Comparator specifications
    ├── model.wgsl       # Core: lm311_compare(), lm311_window()
    └── keyer.wgsl       # App: Video threshold keyer
```

**Component Model Structure:**

Each IC has:
1. **spec.json** - Electrical specs, pinout, transfer function parameters, SPICE model
2. **model.wgsl** - Core transfer functions (building blocks)
3. **Application shaders** - Complete effects demonstrating the component

**Example: CA3046 Differential Pair**

```wgsl
const CA3046_VT: f32 = 0.026;  // Thermal voltage ~26mV

fn differential_pair(v1: f32, v2: f32, tail_current: f32) -> f32 {
    let vdiff = v1 - v2;
    return tanh(vdiff / (2.0 * CA3046_VT)) * tail_current;
}
```

The `tanh()` response emerges naturally from BJT physics — this is why modeled colorizers have soft transitions between bands, just like the real hardware.

### Device Workbench

Built `tools/device-workbench/` — a Rust/wgpu application for testing component shaders:

```
tools/device-workbench/
├── workbench-app/       # Rust wgpu renderer
├── mcp_server.py        # MCP tools for Claude integration
├── test_signals/        # Ramps, colorbars, checkers, etc.
└── API_SPEC.md          # HTTP API documentation
```

**MCP Tools:**

| Tool | Purpose |
|------|---------|
| `workbench_load_shader` | Hot-reload WGSL shader |
| `workbench_set_params` | Adjust uniforms in real-time |
| `workbench_inject_signal` | Load test patterns |
| `workbench_screenshot` | Capture output |
| `workbench_record_sweep` | Create parameter sweep GIFs |
| `workbench_analyze` | Vision model analysis |
| `workbench_export_node` | Export to Phosphlux node |

### Workbench Test Results

**CA3046 Colorizer on Horizontal Ramp:**

![CA3046 Colorizer](docs/screenshots/ca3046_colorizer_ramp_20260125_050110.png)

The horizontal ramp (black→white) is converted to color bands:
- Blue (low luma) → Cyan → Green → Yellow → Red (high luma)

The smooth transitions come from the differential pair's `tanh()` response.

**LM311 Window Comparator:**

![LM311 Keyer](docs/screenshots/lm311_keyer_window_20260125_050459.png)

Hard threshold keying with adjustable window width.

### Workflow: Device Emulation Pipeline

```
1. IDENTIFY    → Find device documentation (patents, service manuals, schematics)
2. STUB        → Create device stub (inputs, outputs, what it does, not how)
3. DECOMPOSE   → Extract component list (ICs, transistors, key passives)
4. COLLECT     → Download datasheets for each component
5. MODEL       → Generate spec.json and model.wgsl for each IC
6. ASSEMBLE    → Compose component models into device shader
7. VALIDATE    → Test in workbench against reference materials
```

### Current Component Status

| Component | Type | Status | Used In |
|-----------|------|--------|---------|
| CA3046 | Transistor Array | ✓ Modeled | Paik/Abe, Sandin IP |
| CA3080 | OTA | ✓ Modeled | Sandin IP |
| LM311 | Comparator | ✓ Modeled | Various |
| CA3019 | Diode Array | Need datasheet | Paik/Abe (amplitude windows) |
| CA3022 | Video Amp | Need datasheet | Paik/Abe |
| CA3054 | Diff Amp | Need datasheet | Paik/Abe |

### Key Findings

1. **Most LLMs don't know video synthesis history** — Only 2/15 models correctly identified the Paik/Abe folding amplifier. This validates using specialized research with Claude review.

2. **Component-level modeling enables reuse** — The CA3046 model works for any circuit using that IC, not just one specific device.

3. **Transfer functions capture analog behavior** — The `tanh()` in differential pairs, soft limiting in OTAs — these emerge from physics, not artistic choice.

4. **Hot-reload workbench accelerates iteration** — Test shaders in real-time without recompiling the main application.

---

## Files Reference (Phase 9)

| File | Purpose |
|------|---------|
| `tools/swarm-agent/swarm/orchestrator.py` | Multi-agent coordination |
| `tools/swarm-agent/swarm/component_agent.py` | IC lookup and WGSL generation |
| `tools/swarm-agent/swarm/token_tracker.py` | Cloud vs local cost tracking |
| `tools/swarm-agent/swarm/model_benchmark.py` | A/B testing infrastructure |
| `tools/device-workbench/mcp_server.py` | Workbench MCP tools |
| `tools/device-workbench/workbench-app/` | Rust/wgpu test renderer |
| `research/components/README.md` | Component library documentation |
| `research/components/ics/*/model.wgsl` | Core IC transfer functions |
| `research/components/ics/*/*.wgsl` | Application shaders |
| `research/RESEARCH_INDEX.md` | Master research tracking |

---

## Version History

- **v0.1**: TypeScript prototype (2 FPS)
- **v0.2**: Rust + wgpu rewrite (60 FPS)
- **v0.3**: egui control panel integration
- **v0.4**: Paned layout redesign + renamed to **Phosphlux**
- **v0.5**: Visual node editor with full UX
- **v0.6**: GPU signal routing — node graph compiles to GPU operations
- **v0.7**: Analog routing architecture — per-node Frame Buffers, blend modes, codebase cleanup
- **v0.8**: Research orchestrator — tiered MCP server with Claude as quality gate
- **v0.9**: Swarm agent system — multi-agent component research, token tracking, model benchmarking, component library with CA3046/CA3080/LM311 models, device workbench for shader testing

---

*Last updated: January 2026*
