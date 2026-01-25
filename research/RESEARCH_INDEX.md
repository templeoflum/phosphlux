# Research Index

Collected research materials for historical video synthesis hardware emulation.

## Priority Devices (from HISTORICAL_EMULATION_PLAN.md)

### CRITICAL Priority
- [x] Paik/Abe Video Synthesizer (1969) - schematics collected
- [ ] Sandin Image Processor (1971)
- [ ] Rutt/Etra Video Synthesizer (1972)
- [ ] LZX Visionary (2010)
- [ ] Sony PVM/BVM CRT Display

### HIGH Priority
- [ ] Ben Laposky Oscilloscope Art (1950)
- [ ] Stephen Beck Direct Video Synthesizer (1970)
- [ ] Jones Colorizer (1973)
- [ ] LZX Cadet Series (2012)
- [ ] Edirol V4/V4EX (2000/2008)

---

## Collected Materials

### Schematics
| Device | File | Source | Status |
|--------|------|--------|--------|
| Paik/Abe Video Synthesizer | `schematics/Paik-Abe Video Synthesizer - Schematics.pdf` | Archive | Complete |
| Paik/Abe Video Synthesizer | `schematics/Paik Abe Video Synthesizer Technical Documentation.pdf` | Archive | Complete (87 pages) |

### Datasheets / Component Models
| Component | Type | File | Used In | Status |
|-----------|------|------|---------|--------|
| CA3046 | Transistor Array | `components/ics/ca3046/` | Paik/Abe, Sandin IP | **Modeled** |
| CA3080 | OTA | `components/ics/ca3080/` | Sandin IP | **Modeled** |
| LM311 | Comparator | `components/ics/lm311/` | Generic | **Modeled** |
| CA3019 | Diode Array | *need datasheet* | Paik/Abe (amplitude windows) | **PRIORITY** |
| CA3022 | Video Amp | *need datasheet* | Paik/Abe | Needed |
| CA3023 | Video Amp | *need datasheet* | Paik/Abe | Needed |
| CA3054 | Diff Amp | *need datasheet* | Paik/Abe | Needed |
| CA3055 | Diff Amp | *need datasheet* | Paik/Abe | Needed |

### Component Library
See `components/README.md` for the component modeling system.

### Patents
| Device | Patent # | File | Status |
|--------|----------|------|--------|
| *none yet* | | | |

### Manuals
| Device | File | Source | Status |
|--------|------|--------|--------|
| *none yet* | | | |

---

## Source Links

### Primary Sources
- [LZX Industries GitHub](https://github.com/lzxindustries) - Cadet schematics
- [Vasulka Archive](http://vasulka.org) - Historical documentation
- [Experimental TV Center](http://experimentaltvcenter.org) - Historical equipment
- [Fondation Langlois](http://fondation-langlois.org) - Daniel Langlois archive

### Community Sources
- [Scanlines.xyz](https://scanlines.xyz) - Video synthesis forum
- [ModWiggler Video Section](https://modwiggler.com/forum) - Eurorack video

### Datasheet Archives
- [AllDatasheet](https://www.alldatasheet.com)
- [DatasheetCatalog](https://www.datasheetcatalog.com)

---

## Research Notes

### 2026-01-25: Component Agent Workflow Implemented
Built automated component research workflow:
- ComponentAgent looks up part numbers, checks internal database
- If found: creates spec.json and generates WGSL model automatically
- If not found: queues datasheet download request for user
- Models generated: CA3046 (transistor array), CA3080 (OTA), LM311 (comparator)

**WGSL model features:**
- Transfer functions normalized for 0-1V video signals
- Key functions for each component type (diff pair, current mirror, etc.)
- Soft limiting for realistic analog behavior

### 2026-01-25: Paik/Abe Schematics Analyzed
Collected full technical documentation (87 pages) and schematics (26 pages).

**Key findings from schematics:**
- 7 camera input channels (CH-1 through CH-7)
- Matrix & Mixing Amp combines channels → R, G, B outputs
- Color banding uses **amplitude windows**, NOT wavefolding
- Key ICs: CA-3019, CA-3022, CA-3023, CA-3046, CA-3054, CA-3055
- Final output goes to encoder for NTSC signal

**Next steps:**
- Find datasheets for CA-30xx series ICs
- Create notes on amplitude window circuit behavior
- Search for related patents

### 2024-XX-XX: Project Started
Initial research infrastructure created. Next steps:
1. Download LZX Cadet schematics from GitHub
2. Search archive.org for Sandin IP documentation
3. Find Paik/Abe patent (if exists)

---

*Update this file as materials are collected.*
