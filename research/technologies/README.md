# Technologies

Specific devices, systems, and instruments in video synthesis history.

Each technology entry documents:
- What the device does
- Who built it and when
- What components it uses
- How to emulate it

## v1 Target Devices

| # | Device | Era | Figures | Status |
|---|--------|-----|---------|--------|
| 1 | [Paik/Abe Video Synthesizer](paik_abe/) | 1969 | Paik, Abe | In Progress |
| 2 | Sandin Image Processor | 1971 | Sandin | Pending |
| 3 | Rutt/Etra Video Synthesizer | 1972 | Rutt, Etra | Pending |
| 4 | Jones Colorizer | 1974 | Jones | Pending |
| 5 | LZX Cadet Series | 2012 | Larsen | Pending |

## Document Types Needed

For each technology:

| Document | Purpose | Where to Find |
|----------|---------|---------------|
| Schematics | Circuit implementation | Archives, museums, original publications |
| Patents | Legal descriptions with diagrams | Google Patents, USPTO |
| Manuals | Operation instructions | Archive.org, manufacturer sites |
| Service Docs | Repair/calibration info | Rare - try museums |
| Demo Videos | Visual reference | YouTube, Vimeo, archives |
| Interviews | Designer intent | Video Data Bank, oral histories |

## File Organization

```
technologies/
├── device_name/
│   ├── README.md           # Overview, what it does
│   ├── architecture.md     # Block diagram, signal flow
│   ├── components.md       # IC list, links to component models
│   ├── emulation_notes.md  # Specific implementation details
│   └── references.md       # Links to documents in other layers
```

## Linking Convention

Technologies link to:
- **figures/** - Who built it
- **techniques/** - What methods it uses
- **patents/** - Related IP
- **manuals/** - Operation docs
- **schematics/** - Circuit diagrams
- **components/** - IC/functional block models

Example in architecture.md:
```markdown
## Colorizer Section

Uses the [Amplitude Windowing](../../techniques/README.md#amplitude-windowing)
technique implemented with [CA3019 diode array](../../components/ics/ca3019/).

See schematic page 28 in [Paik/Abe Schematics](../../schematics/Paik-Abe%20Video%20Synthesizer%20-%20Schematics.pdf).
```
