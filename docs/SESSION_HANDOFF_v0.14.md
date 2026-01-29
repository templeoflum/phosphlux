# Session Handoff: v0.14 Planning

## Current State (as of 2026-01-29)

**Branch:** `research-integration`
**Last commit:** `cbaeed3` - Mark v0.13 cable degradation complete in roadmap

### Completed Work
- v0.13 Cable Degradation fully working
  - `CableType` enum: Ideal, Composite, S-Video, Component, RGB
  - `src/shaders/cables.wgsl` with YIQ/YPbPr color space conversions
  - Cable selector UI in inspector (click cable wire to select)
  - Colored cables in node editor (yellow=Composite, etc.)
  - Shader composer wraps signals with degradation functions
  - Visually verified: composite degradation visible on color bars

### v0.14 Goal: Full Paik/Abe Chain

Refactor the monolithic `PaikAbeColorizer` into 5 separate authentic devices matching the real hardware signal path:

| Device | ProcessingMode | Description |
|--------|---------------|-------------|
| `PaikAbeInputMatrix` | PixelProcessor | 7-channel weighted mixing (CA3054 diff amps) |
| `PaikAbePedColorizer` | PixelProcessor | Amplitude window colorization (CA3019 diode array) |
| `PaikAbeFoldingAmp` | PixelProcessor | Triangle wave folding per channel |
| `PaikAbeWobulator` | ScanModulator | Already exists - UV displacement |
| `PaikAbeOutputEncoder` | PixelProcessor | CA3022/CA3023 video amplifier output stage |

### Key Files to Understand

```
src/devices/
├── mod.rs                    # Device exports
├── generators.rs             # TestSignalGenerator
└── paik_abe/
    ├── mod.rs                # PaikAbeColorizer, PaikAbeWobulator
    ├── colorizer.wgsl        # Current monolithic colorizer shader
    └── wobulator.wgsl        # Wobulator shader (keep as-is)

src/gpu/
├── shader_composer.rs        # ShaderDevice trait, compose_plan()
└── renderer.rs               # Multi-pass pipeline

research/
├── devices/paik-abe/         # Research documents
└── components/               # IC models (CA3046, CA3019, etc.)
```

### Current Colorizer Implementation

The existing `PaikAbeColorizer` in `src/devices/paik_abe/colorizer.wgsl` has:
- 6-stage folding amplifier (R, G, B channels)
- Gain, fold depth, saturation, slew parameters
- This needs to be split into InputMatrix + PedColorizer + FoldingAmp + OutputEncoder

### Research Resources Available

- `research/components/ics/` - IC datasheets and models
- `research/devices/paik-abe/` - Paik/Abe specific docs
- MCP research tools: `mcp__research__research_query`, `mcp__research__retrieve_patent`

### Suggested Approach for v0.14

1. **Research Phase**: Use MCP tools to understand each stage's behavior
   - InputMatrix: How does the 7-channel mixing work?
   - PedColorizer: What is "amplitude window colorization"?
   - FoldingAmp: Already partially implemented - extract and refine
   - OutputEncoder: What does the output stage do?

2. **Implementation Phase**: Create each device one at a time
   - Start with FoldingAmp (already have code to extract)
   - Then InputMatrix (relatively simple mixing)
   - Then PedColorizer (CA3019 diode array behavior)
   - Finally OutputEncoder

3. **Integration Phase**:
   - Create default Paik/Abe preset patch
   - Add patch save/load (JSON)

### Additional Context

- The app runs at 60fps with real-time shader compilation
- Each device implements `ShaderDevice` trait
- `ProcessingMode::PixelProcessor` = same UV, `ScanModulator` = displaced UV
- Shader composer automatically chains devices based on connections
