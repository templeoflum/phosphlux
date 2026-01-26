# Devices

Specific hardware devices, systems, and instruments in video synthesis history.

Each device directory is the **single source of truth** for all documentation needed
to faithfully emulate that machine: schematics, patents, service manuals, calibration
data, and operation guides all live together under the device they describe.

## v1 Target Devices

| # | Device | Era | Innovators | Status |
|---|--------|-----|------------|--------|
| 1 | [Paik/Abe Video Synthesizer](paik_abe/) | 1969 | Paik, Abe | In Progress |
| 2 | Sandin Image Processor | 1971 | Sandin | Pending |
| 3 | Rutt/Etra Video Synthesizer | 1972 | Rutt, Etra | Pending |
| 4 | Jones Colorizer | 1974 | Jones | Pending |
| 5 | LZX Cadet Series | 2012 | Larsen | Pending |

## Document Types Per Device

| Document | Purpose for Emulation |
|----------|-----------------------|
| Schematics | Circuit topology, component values, power rails |
| Patents | Detailed circuit descriptions, design rationale |
| Service manuals | Schematics + calibration voltages + test points |
| Operation guides | Parameter ranges, expected behavior |
| Calibration data | Voltage test points, alignment procedures |
| Reference material | Analysis notes, architecture docs |

## File Organization

```
devices/
├── device_name/
│   ├── manifest.json       # Document inventory with creative synthesis flags
│   ├── architecture.md     # Block diagram, signal flow
│   ├── *.pdf               # All device-specific documents
│   └── emulation_notes.md  # Implementation-specific details
```

## Linking Convention

Devices link to:
- **innovators/** - Who built it
- **techniques/** - What signal processing methods it uses
- **components/** - IC/functional block models used in the device

Example in architecture.md:
```markdown
## Colorizer Section

Uses the [Amplitude Windowing](../../techniques/README.md#amplitude-windowing)
technique implemented with [CA3019 diode array](../../components/ics/ca3019/).
```
