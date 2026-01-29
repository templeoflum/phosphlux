# Session Handoff: v0.15 UI Overhaul

## Current State (as of 2026-01-29)

**Branch:** `research-integration`
**Last commit:** `e26cc25` - Fix PED Colorizer CA3019 model and workbench stability

### Completed Work (v0.14.1)

1. **PED Colorizer CA3019 Fixes**
   - Proper exponential diode detection via `ped_ca3019_diode_detect()`
   - `PED_DIODE_SHARPNESS = 40.0` (~1/Vt at room temperature)
   - Added `clipping` parameter matching CLP pin 8 from schematic
   - All visual tests passed in workbench

2. **Workbench Stability**
   - Added naga WGSL validation before GPU compilation
   - No more crashes from invalid shader code
   - Clean error messages returned to MCP client

3. **Removed Ollama Integration**
   - Workbench MCP server no longer depends on Ollama
   - Vision analysis removed (Claude handles verification directly)
   - Simplified tool surface area

4. **Multi-Input Foundation**
   - `InputBinding` struct in shader_composer.rs
   - WGSL generation ready for multiple input textures
   - Runtime binding still pending (v0.16+)

---

## v0.15 Goal: UI Overhaul

Transform Phosphlux with vintage oscilloscope aesthetic and top-down node flow.

### Plan File

Full specification: `PLAN_ui_overhaul.md`

### Implementation Phases

| Phase | Description | Status |
|-------|-------------|--------|
| 1 | Theme System | Not started |
| 2 | Node Geometry | Not started |
| 3 | Node Visuals | Not started |
| 4 | Default Placement | Not started |
| 5 | UI Polish | Not started |

### Key Files to Modify

| File | Changes |
|------|---------|
| `src/ui/theme.rs` | **NEW** — PhosphluxTheme, apply_theme() |
| `src/ui/mod.rs` | Add `pub mod theme;` |
| `src/main.rs` | Apply theme on startup |
| `src/ui/node_editor.rs` | socket_pos, bezier, draw_node, grid |

### Color Palette Reference

```rust
// Backgrounds
bg_dark: #0A0E0D       // CRT background
bg_panel: #1C1C1C      // Bakelite panels
bg_widget: #2A2A2A     // Widget backgrounds

// Text
text_primary: #F5F5DC  // Cream labels
text_secondary: #A8A9AD // Aluminum gray

// Phosphor
phosphor_bright: #39FF14  // P31 green (active)
phosphor_glow: #7FFF00    // Standard glow
phosphor_fade: #4A7729    // Persistence

// Semantic
accent_selected: #FFD700  // Dial yellow
accent_error: #C41E3A     // Warning red

// Node categories
generators: #7FFF00   // Green
paik_abe: #FFD700     // Yellow
sandin: #4169E1       // Blue
jones: #FF6600        // Orange
```

### Visual Target

```
┌────────────────────────────────────────────────────────────────┐
│ ▸ File   Help                                    [●] OK        │
├──────────────────┬─────────────────────────────┬───────────────┤
│ DEVICES          │                             │               │
│ ─────────────────│  ┌──────────┐ ┌──────────┐ │   PREVIEW     │
│ ▾ Generators     │  │▀▀▀▀▀▀▀▀▀▀│ │▀▀▀▀▀▀▀▀▀▀│ │               │
│   [Test Signal]  │  │ Test Sig │ │Oscillator│ │  ┌─────────┐  │
│   [Oscillator]   │  │    ●     │ │    ●     │ │  │         │  │
│ ▾ Paik/Abe      │  └────┬─────┘ └────┬─────┘ │  │  Video   │  │
│   [Colorizer]    │       │            │       │  │  Output  │  │
│                  │       └─────┬──────┘       │  │         │  │
│ ─────────────────│             │              │  └─────────┘  │
│ INSPECTOR        │       ┌─────┴─────┐        │               │
│ ─────────────────│       │●        ●│        │               │
│ Selected device  │       │▀▀▀▀▀▀▀▀▀▀│        │               │
│                  │       │ Colorizer│        │               │
│ Param ═══════●   │       │    ● [●]│        │               │
└──────────────────┴─────────────────────────────┴───────────────┘
```

Key elements:
- Nodes have colored top bars matching category
- Inputs (●) on TOP, outputs (●) on BOTTOM
- Cables flow DOWN from outputs to inputs
- Selected items glow yellow
- Output device has red LED indicator [●]

---

## Blocked Tasks

### 7-Input Matrix Mixer

**Blocked by:** Runtime multi-input texture binding in renderer.rs

The shader composer can generate WGSL for multiple inputs, but the renderer still uses single-input bind groups. Requires:
1. Variable texture count bind group layouts
2. Per-segment input texture array binding
3. Graph-based rendering instead of linear segments

**Workaround (current):** Single-input devices derive multiple virtual channels internally.

---

## Quick Reference

### Build & Run
```bash
cargo build --release
cargo run --release
```

### Workbench
```bash
cd tools/device-workbench
python mcp_server.py  # Starts MCP server on port 8765
cd workbench-app && cargo run --release  # Starts wgpu renderer
```

### Key Shortcuts
| Key | Action |
|-----|--------|
| Del | Delete selected node |
| F | Frame on selected |
| Home | Reset view |
| Ctrl+Click | Quick-connect nodes |

---

## Research Resources

- `research/devices/paik_abe/` — Paik/Abe technical documentation
- `research/components/ics/` — IC datasheets and WGSL models
- `research/audits/` — Verification documents
- `PLAN_authentic_paik_abe.md` — Component-accuracy plan
