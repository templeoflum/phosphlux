# Research Index

Component-level emulation of historical video synthesis hardware.

---

## 7-Layer Research Architecture

```
research/
├── techniques/      # Abstract methods (amplitude windowing, wavefolding, etc.)
├── figures/         # Key inventors (Paik, Abe, Sandin, etc.)
├── technologies/    # Specific devices (Paik/Abe, Sandin IP, etc.)
├── patents/         # Legal documents with implementation details
├── manuals/         # Operation and service documentation
├── schematics/      # Circuit diagrams
├── datasheets/      # IC specifications
└── components/      # WGSL shader models
    ├── ics/         # Integrated circuit models
    └── functional/  # Higher-level functional blocks
```

**See [DISCOVERY_GUIDE.md](DISCOVERY_GUIDE.md) for how to find primary sources.**

### Provenance Database

All artifacts are tracked in `provenance.db`:
- **artifacts** - Downloaded documents with SHA256 hashes
- **versions** - Component model versions
- **rd_passes** - R&D session tracking
- **relationships** - Links between artifacts (derived_from, references, etc.)
- **sources** - Knowledge source quality ratings

---

## v1 Target Devices

| # | Device | Era | Status | Documents |
|---|--------|-----|--------|-----------|
| 1 | **Paik/Abe Video Synthesizer** | 1969 | Core Complete | Schematics ✓, Tech Docs ✓ |
| 2 | **Sandin Image Processor** | 1971 | Pending | Need schematics |
| 3 | **Rutt/Etra Video Synthesizer** | 1972 | Pending | Need schematics |
| 4 | **Jones Colorizer** | 1974 | Pending | Need schematics |
| 5 | **LZX Cadet Series** | 2012 | Pending | GitHub ✓ |

---

## Component Library Status

### ICs Modeled

| Component | Type | Used In | Shaders |
|-----------|------|---------|---------|
| CA3046 | Transistor Array | Paik/Abe, Sandin IP | model.wgsl, colorizer.wgsl |
| CA3019 | Diode Array | Paik/Abe PED Control | model.wgsl, ped_control.wgsl, colorizer.wgsl |
| CA3054 | Dual Diff Amp | Paik/Abe Matrix, Keyer | model.wgsl, matrix_mixer.wgsl, video_keyer.wgsl |
| CA3080 | OTA | Sandin IP | model.wgsl, vca.wgsl |
| LM311 | Comparator | Generic Keying | model.wgsl, keyer.wgsl |

### Functional Blocks Modeled

| Block | Based On | Used In | Shaders |
|-------|----------|---------|---------|
| Folding Amplifier | Discrete transistors | Paik/Abe | model.wgsl, folder.wgsl |
| Wobbulator | CRT deflection mod | Paik/Abe | model.wgsl, wobbulator.wgsl |

### ICs Needed

| Component | Type | Used In | Datasheet |
|-----------|------|---------|-----------|
| CA3022 | Video Amp | Paik/Abe | RCA Databook |
| CA3023 | Video Amp | Paik/Abe | RCA Databook |
| MC1445 | Gain Block | Sandin IP | Motorola |

---

## Collected Documents

### Schematics
| Device | File | Pages |
|--------|------|-------|
| Paik/Abe | `schematics/Paik-Abe Video Synthesizer - Schematics.pdf` | 26 |
| Paik/Abe | `schematics/Paik Abe Video Synthesizer Technical Documentation.pdf` | 87 |

### Patents
*None collected yet - see [DISCOVERY_GUIDE.md](DISCOVERY_GUIDE.md) for search strategies*

### Manuals
*None collected yet*

### Datasheets
*Embedded in component spec.json files*

---

## Source Links

### Primary Archives
- [Vasulka Archive](http://vasulka.org) - Historical documentation
- [Experimental TV Center](http://experimentaltvcenter.org) - Equipment docs
- [LZX Industries GitHub](https://github.com/lzxindustries) - Cadet schematics

### Datasheet Sources
- [1978 RCA Linear IC Databook](https://www.smcelectronics.com/DOWNLOADS/1978-RCA%20LINEAR%20IC.PDF) - CA-30xx series
- [AllDatasheet](https://www.alldatasheet.com) - General search
- [Datasheet Archive](https://www.datasheetarchive.com) - Obsolete parts

### Patent Search
- [Google Patents](https://patents.google.com) - Free searchable database

### Community
- [Scanlines.xyz](https://scanlines.xyz) - Video synthesis forum
- [ModWiggler Video Forum](https://modwiggler.com/forum) - Eurorack video

---

## Research Notes

### 2026-01-25: Paik/Abe Core Complete
All major functional blocks modeled and tested:
- CA3046 transistor array (differential pairs)
- CA3019 diode array (PED Control colorizer)
- CA3054 dual diff amp (matrix mixer, video keyer)
- Folding amplifier (nonlinear waveshaper)
- Wobbulator (scan displacement)

Parameter sweep documentation captured for each component.

### 2026-01-25: 7-Layer Research Architecture
Implemented structured approach for all future devices:
1. Techniques - Abstract methods
2. Figures - Key inventors
3. Technologies - Specific devices
4. Patents - Legal documents
5. Manuals - Operation docs
6. Schematics - Circuit diagrams
7. Components - IC models

See [DISCOVERY_GUIDE.md](DISCOVERY_GUIDE.md) for source discovery workflows.

### 2026-01-25: Component Agent Workflow
Automated pipeline:
- Extract IC part numbers from schematics
- Look up in internal database or datasheet sources
- Generate spec.json with SPICE parameters
- Create WGSL model with transfer functions
- Test in workbench with parameter sweeps

---

*Update this file as research progresses.*
