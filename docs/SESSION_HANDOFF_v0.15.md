# Session Handoff: v0.16 Planning

## Current State (as of 2026-01-29)

**Branch:** `research-integration`
**Version:** v0.15 complete

### Completed Work

| Version | Feature |
|---------|---------|
| v0.13 | Cable degradation (Composite, S-Video, Component, RGB) |
| v0.14 | Full Paik/Abe chain (InputMatrix, FoldingAmp, PedColorizer, OutputEncoder) |
| v0.14.1 | Authentic CA3019 diode model, clipping parameter, workbench stability |
| v0.15 | UI Overhaul - vintage oscilloscope theme, top-down node flow |

### v0.15 UI Overhaul Details

**Theme System (`src/ui/theme.rs`):**
- `PhosphluxTheme` struct with full color palette
- CRT background (#0A0E0D), Bakelite panels (#1C1C1C)
- Cream text (#F5F5DC), phosphor accents (#39FF14, #7FFF00)
- Category colors applied to nodes: generators=green, Paik/Abe=yellow

**Node Flow:**
- Inputs on TOP edge, outputs on BOTTOM edge
- Vertical Bezier curves (top-down signal flow)
- 3px category color bar on node top edge

---

## v0.16 Goal: Patch Save/Load

Enable saving and loading patch configurations.

### Tasks

- [ ] Define JSON patch file format
- [ ] Implement save patch (File menu)
- [ ] Implement load patch
- [ ] Create default Paik/Abe preset patch
- [ ] Recent files list

### Suggested JSON Format

```json
{
  "version": "1.0",
  "name": "My Patch",
  "devices": [
    {
      "instance_id": 1,
      "device_id": "test_signal_generator",
      "position": [100, 100],
      "parameters": {
        "mode": 0.0,
        "frequency": 1.0
      }
    }
  ],
  "connections": [
    {
      "from_device": 1,
      "from_port": "video_out",
      "to_device": 2,
      "to_port": "video_in",
      "cable_type": "Ideal"
    }
  ],
  "output_device": 2
}
```

---

## Blocked Tasks

### 7-Input Matrix Mixer

**Blocked by:** Runtime multi-input texture binding in renderer.rs

The shader composer generates multi-input WGSL, but renderer needs:
1. Variable texture count bind group layouts
2. Per-segment input texture array binding

**Workaround:** Single-input devices derive virtual channels internally.

---

## Key Files

| File | Purpose |
|------|---------|
| `src/ui/theme.rs` | Phosphlux theme system |
| `src/ui/node_editor.rs` | Node editor with top-down flow |
| `src/devices/paik_abe/` | All Paik/Abe device implementations |
| `src/gpu/shader_composer.rs` | Multi-input WGSL generation (ready) |
| `src/gpu/renderer.rs` | Single-input binding (needs multi-input) |

---

## Quick Reference

### Build & Run
```bash
cargo build --release
cargo run --release
```

### Key Shortcuts
| Key | Action |
|-----|--------|
| Del | Delete selected node |
| F | Frame on selected |
| Home | Reset view |
| Ctrl+Click | Quick-connect nodes |
