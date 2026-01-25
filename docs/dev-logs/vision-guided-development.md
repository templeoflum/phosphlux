# Vision-Guided Development Workflow

**Date:** 2026-01-25
**Discovery Context:** Paik/Abe Colorizer development in Phosphlux workbench

## Summary

During device emulation development, we discovered a highly effective workflow where Claude directly observes visual output, evaluates correctness, and iterates without human visual confirmation. This eliminates the traditional "write code → human looks → human describes problem → AI fixes" loop.

## The Traditional Problem

When developing visual systems (shaders, UI, graphics), the typical AI-assisted workflow is:

```
1. Human describes desired output
2. AI writes code
3. Human runs code, looks at result
4. Human describes what's wrong: "the colors are too bright" / "it's shifted left"
5. AI guesses at fix based on description
6. Repeat until human says it looks right
```

**Problems:**
- Human becomes the visual feedback loop (slow, fatiguing)
- Natural language is imprecise for visual descriptions
- AI can't verify its own output
- Multiple round-trips for simple visual issues
- Human must context-switch between coding and visual evaluation

## The Vision-Guided Workflow

```
1. Human describes desired output (or provides reference image)
2. AI writes code
3. AI loads code into workbench via MCP
4. AI injects test signal
5. AI takes screenshot
6. AI VIEWS screenshot directly
7. AI evaluates: "the folding is creating 4 color bands, need more gain for 6 bands"
8. AI adjusts parameters or code
9. Repeat until AI confirms visual match
10. AI exports final result
```

**Advantages:**
- AI is the visual feedback loop (fast, tireless)
- Direct pixel-level evaluation, not natural language approximation
- AI can verify its own output against reference
- Single-turn fixes for visual issues
- Human only needs to approve final result or provide direction

## Infrastructure Required

### 0. Tiered Agent Architecture (Token Efficiency + Parallelism)

The workflow leverages multiple AI tiers to optimize both token usage and execution speed:

```
┌─────────────────────────────────────────────────────────────┐
│  Claude (Orchestrator)                                       │
│  - High-level reasoning and decision making                  │
│  - Reviews outputs from local agents                         │
│  - Makes implementation decisions                            │
│  - Views screenshots and evaluates results                   │
└──────────────────────┬──────────────────────────────────────┘
                       │ Dispatches parallel tasks
        ┌──────────────┼──────────────────┐
        ▼              ▼                  ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ Fast Tier    │ │ Deep Tier    │ │ Vision Tier  │
│ (phi4-mini)  │ │ (llama3.1)   │ │ (moondream)  │
│ ~3-5s        │ │ ~10-30s      │ │ ~5-10s       │
│              │ │              │ │              │
│ Quick facts  │ │ Research     │ │ Image OCR    │
│ Definitions  │ │ Code drafts  │ │ Schematics   │
│ Conversions  │ │ Analysis     │ │ Extraction   │
└──────────────┘ └──────────────┘ └──────────────┘
        │              │                  │
        └──────────────┴──────────────────┘
                       │
                       ▼
              ┌──────────────┐
              │ Embed Tier   │
              │ (qwen3)      │
              │ ~0.1s warm   │
              │              │
              │ Knowledge    │
              │ base search  │
              └──────────────┘
```

**Token Efficiency:**
- Claude tokens are expensive; local model tokens are free
- Offload routine tasks (research, drafts, extraction) to local models
- Claude reviews and decides; doesn't do grunt work

**Parallel Execution:**
- Multiple research queries can run simultaneously
- Vision extraction while research runs
- Embedding happens in background
- Claude continues working while agents complete

**Example: Researching a New Device**
```
Claude dispatches IN PARALLEL:
  - research_query("Sandin IP differencing amp", focus="components")
  - research_query("Sandin IP", focus="behavior")
  - extract_from_image("sandin_schematic.png", "components")
  - quick_query("What is a differencing amplifier?")

All 4 run simultaneously (~10s total, not ~40s sequential)
Claude reviews all results, synthesizes, implements
```

### 1. Workbench Application
A lightweight rendering environment with:
- GPU-accelerated rendering (wgpu)
- Hot-reload for code changes
- Test signal injection (ramps, colorbars, patterns)
- Screenshot capture to accessible path
- Minimal overhead (not a full application)

### 2. MCP Server Integration
Tools that allow Claude to:
- `load_shader` - Push code to workbench
- `set_params` - Adjust parameters in real-time
- `inject_signal` - Load test inputs
- `screenshot` - Capture current output
- `analyze` - Vision model describes what it sees
- `compare` - Diff against reference image
- `record_sweep` - Capture parameter animations

### 3. Vision Capability
Claude must be able to:
- Read image files directly (screenshot paths)
- Interpret visual content accurately
- Compare against reference images
- Identify specific visual artifacts

## Example Session: Paik/Abe Colorizer

### Initial State
- Goal: Emulate 1969 Paik/Abe Video Synthesizer colorizer
- Reference: Historical documentation of wavefolding behavior

### Iteration 1
```
Claude: Loads initial shader with triangle wave folding
Claude: Injects horizontal ramp test signal
Claude: Takes screenshot, views it
Claude: "I see 4 color bands transitioning gray→green→red→blue.
        The folding gain is too low for the characteristic Paik/Abe look.
        Increasing gain parameter."
```

### Iteration 2
```
Claude: Sets gain from 0.5 to 0.8
Claude: Takes screenshot, views it
Claude: "Now showing 6+ color bands with tighter transitions.
        The op-amp saturation isn't visible yet - colors are too sharp.
        Adding slew rate limiting."
```

### Iteration 3
```
Claude: Modifies shader to add LM741 slew rate emulation
Claude: Takes screenshot, views it
Claude: "Edges are now softer, warm bias visible in highlights.
        This matches the historical Paik/Abe aesthetic."
```

### Validation
```
Claude: Loads portrait photograph as input
Claude: Takes screenshot, views it
Claude: "Skin tones mapping to cyan/orange bands, shadows showing
        colored fringing, white shirt creating green/red folding.
        This is the authentic psychedelic video art effect."
Claude: Exports node for integration
```

**Total time:** ~5 minutes
**Human visual confirmations required:** 0 during iteration, 1 at end

## Key Principles

### 1. Test Signals Are Critical
Standard test signals (ramps, colorbars) reveal behavior systematically:
- **Horizontal ramp**: Shows transfer function across full luminance range
- **Colorbars**: Shows color channel separation and interaction
- **Real images**: Validates natural content behavior

### 2. Reference Comparison Enables Accuracy
When emulating existing hardware:
- Load reference image of original hardware output
- Use `diff` tool to measure pixel-level similarity
- Iterate until metrics meet threshold

### 3. Parameter Sweeps Document Behavior
Recording animated parameter sweeps:
- Creates visual documentation automatically
- Reveals edge cases and artifacts
- Useful for both development and end-user documentation

### 4. Keep Workbench Minimal
The workbench should be:
- Fast to start (< 2 seconds)
- Single-purpose (test ONE device)
- Low memory footprint
- No distracting UI elements

The full application (Phosphlux) is separate. Mixing them dilutes both tools.

## Generalizing to Other Domains

This workflow applies wherever visual output must be tuned:

### Shader Development
- Fragment shaders, post-processing effects
- Material authoring
- Procedural textures

### UI Development
- Component styling
- Layout tuning
- Animation timing
- Responsive breakpoints

### Data Visualization
- Chart styling
- Color scales
- Label placement
- Axis formatting

### Image Processing
- Filter parameter tuning
- Color correction
- Compression quality

### Game Development
- Particle effects
- Lighting parameters
- Camera settings

### Generative Art
- Procedural generation parameters
- Style transfer tuning
- GAN output evaluation

## Implementation Checklist

To replicate this workflow for a new domain:

- [ ] Build minimal workbench app with hot-reload
- [ ] Implement screenshot capture to file
- [ ] Create MCP server with load/param/screenshot tools
- [ ] Define standard test inputs for the domain
- [ ] Enable reference image comparison
- [ ] Document test signal meanings

## Metrics

For the Paik/Abe colorizer session:

| Metric | Value |
|--------|-------|
| Iterations to working shader | 3 |
| Human visual checks during dev | 0 |
| Time from start to export | ~5 min |
| Lines of shader code | ~80 |
| Parameter sweeps recorded | 3 |

## Conclusion

Vision-guided development inverts the traditional AI coding workflow. Instead of humans being the visual feedback loop, the AI observes its own output and self-corrects. This is faster, more precise, and less fatiguing for humans.

The key enabler is **infrastructure**: a workbench with MCP integration that lets the AI load code, run it, and see the result. Without this tooling, we're back to the "describe what you see" loop.

This workflow should be considered for any project involving visual output generation.
