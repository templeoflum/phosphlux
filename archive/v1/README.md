# Phosphlux v1 Archive

Archived: 2026-01-25

## Why Archived

This version used a **runtime dispatch** architecture where all node operations were handled by a single monolithic shader with if-else branching. While functional, this approach had fundamental limitations for circuit-level hardware emulation:

1. **GPU divergence** - Every pixel evaluated all branches
2. **Effects-based thinking** - Nodes were "effects" not circuit simulations
3. **Parameter packing hacks** - Fragility from cramming params into limited slots
4. **32-operation limit** - Hard ceiling on patch complexity
5. **Single frame buffer** - Only first FB node actually worked

## What Was Valuable (Preserved in v2)

- **Timing system** (core/timing.rs) - NTSC/PAL sync generation
- **Ping-pong feedback concept** - Correct approach, re-implemented properly
- **MCP server infrastructure** - Both servers kept in tools/
- **Documentation** - All research preserved in docs/
- **egui/wgpu foundation** - Same stack, better architecture

## Key Files Reference

- `src/gpu/graph_compiler.rs` - Slot-based VM, replaced by shader composition
- `src/shaders/main.wgsl` - 1068 lines, monolithic dispatch
- `src/gpu/renderer.rs` - Multi-FB texture allocation (broken routing)
- `src/ui/node_editor.rs` - Effects-centric UI

## Lessons Learned

1. Design for circuits, not effects
2. Compile-time shader composition beats runtime dispatch
3. Each device needs its own shader module
4. State persistence is essential for capacitor/integrator behavior
5. Component tolerances are what give analog its character
