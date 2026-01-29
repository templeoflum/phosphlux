# Session Handoff: v0.17 Planning

## Current State (as of 2026-01-29)

**Branch:** `research-integration`
**Version:** v0.16 complete

### Completed Work

| Version | Feature |
|---------|---------|
| v0.13 | Cable degradation (Composite, S-Video, Component, RGB) |
| v0.14 | Full Paik/Abe chain (InputMatrix, FoldingAmp, PedColorizer, OutputEncoder) |
| v0.14.1 | Authentic CA3019 diode model, clipping parameter, workbench stability |
| v0.15 | UI Overhaul - vintage oscilloscope theme, top-down node flow |
| v0.16 | Multi-input texture support - up to 8 textures per segment |

### v0.16 Multi-Input Details

**Shader Composer (`src/gpu/shader_composer.rs`):**
- `SegmentShader.input_source_segments: Vec<usize>` — maps texture slot to intermediate texture index
- `device_to_segment: HashMap<u32, usize>` — tracks which segment produces each device
- `count_cross_segment_sources()` returns source device IDs for multi-input binding

**Renderer (`src/gpu/renderer.rs`):**
- `multi_input_bg_layouts: [wgpu::BindGroupLayout; 9]` — pre-created at init (0-8 textures)
- `create_multi_input_bg_layout()` — generates layout with N textures + N samplers
- `create_segment_input_bind_group()` — builds bind group from `input_source_segments`
- `SegmentPipeline` now stores `input_texture_count` and `input_source_segments` for resize rebuild

---

## v0.17 Goal: Patch Save/Load

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

## Now Unblocked: 7-Input Matrix Mixer

With v0.16's multi-input texture support, the authentic 7-input Paik/Abe matrix mixer can be implemented:

1. Create `PaikAbeFullInputMatrix` device with 7 input ports
2. Shader samples from `seg_input_tex`, `seg_input_tex_1`, ..., `seg_input_tex_6`
3. Renderer binds each input to its corresponding upstream segment texture

---

## Key Files

| File | Purpose |
|------|---------|
| `src/ui/theme.rs` | Phosphlux theme system |
| `src/ui/node_editor.rs` | Node editor with top-down flow |
| `src/devices/paik_abe/` | All Paik/Abe device implementations |
| `src/gpu/shader_composer.rs` | Multi-input WGSL generation |
| `src/gpu/renderer.rs` | Multi-input bind group layouts (0-8 textures) |

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
