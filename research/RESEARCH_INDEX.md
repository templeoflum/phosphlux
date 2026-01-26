# Research Index

Component-level emulation of historical video synthesis hardware.

---

## 4-Layer Knowledge Base

```
research/
├── innovators/     # Key inventors, artists, engineers
├── techniques/     # Abstract signal processing methods (device-agnostic)
├── devices/        # Per-device: schematics, patents, manuals, calibration
│   ├── paik_abe/
│   ├── sandin_ip/
│   ├── rutt_etra/
│   └── jones_colorizer/
├── components/     # Per-part: vendor datasheets, spec.json, model.wgsl
│   ├── ics/        # Integrated circuit models
│   └── functional/ # Higher-level functional blocks
└── notes/          # Working research notes
```

**See [DISCOVERY_GUIDE.md](DISCOVERY_GUIDE.md) for how to find primary sources.**

---

## v1 Target Devices

| # | Device | Era | Status | Documents |
|---|--------|-----|--------|-----------|
| 1 | **Paik/Abe Video Synthesizer** | 1969 | Comprehensive | 8 PDFs (schematics, tech docs, budget, restoration, Vasulka) |
| 2 | **Sandin Image Processor** | 1971 | Comprehensive | 3 PDFs (Distribution Religion 34.7MB, ETC Manual 111MB, Connolly Guide 8MB) |
| 3 | **Rutt/Etra Video Synthesizer** | 1972 | Good Coverage | 4 PDFs (Vasulka general, synth cont, Eigenwelt, ETC panel) |
| 4 | **Jones Colorizer** | 1974 | Partial | 1 PDF (MVIP instructions); full schematics not publicly available |
| 5 | **LZX Cadet Series** | 2012 | Available | Open source on GitHub (github.com/lzxindustries/lzxcadet) |

---

## Devices Layer

### Paik/Abe Video Synthesizer (1969)

| Document | File | Size | Source |
|----------|------|------|--------|
| Technical Documentation (87 pages) | `devices/paik_abe/Paik Abe Video Synthesizer Technical Documentation.pdf` | 53MB | Pre-existing |
| Schematics (26 pages) | `devices/paik_abe/Paik-Abe Video Synthesizer - Schematics.pdf` | 1.1MB | Pre-existing |
| ETC Schematics | `devices/paik_abe/ETC_PaikAbe_Schematics.pdf` | 1.1MB | archive.org/details/ETC3311 |
| Restoration Project (2011) | `devices/paik_abe/ETC_PaikAbe_Restoration_2011.pdf` | 793KB | archive.org/details/ETC3310 |
| Budget Document (1972) | `devices/paik_abe/ETC_PaikAbe_Budget_1972.pdf` | 9.5MB | archive.org/details/ETC3316 |
| Vasulka Eigenwelt pp.126-129 | `devices/paik_abe/Vasulka_Eigenwelt_PaikAbe_126-129.pdf` | 582KB | vasulka.org |
| Architecture analysis | `devices/paik_abe/architecture.md` | - | Local |

**Patent status:** The Paik/Abe Video Synthesizer was never patented. It was an art tool, not a commercial product. The US4011557 reference previously in this index was incorrect (that patent covers an unrelated Swiss watch display).

### Sandin Image Processor (1971)

| Document | File | Size | Source |
|----------|------|------|--------|
| Distribution Religion Manual | `devices/sandin_ip/Sandin_Distribution_Religion.pdf` | 34.7MB | criticalartware.net |
| ETC Manual (full schematics) | `devices/sandin_ip/ETC_SandinIP_Manual.pdf` | 111MB | archive.org/details/ETC1365 |
| Connolly Guide (reformatted) | `devices/sandin_ip/Connolly_Guide_SandinIP_screen.pdf` | 8MB | jameshconnolly.com |

### Rutt/Etra Video Synthesizer (1972)

| Document | File | Size | Source |
|----------|------|------|--------|
| General Documentation | `devices/rutt_etra/Vasulka_RuttEtra_General.pdf` | 2.6MB | vasulka.org |
| Video Synth Continuation | `devices/rutt_etra/Vasulka_RuttEtra_VidSynthCont.pdf` | 2.8MB | vasulka.org |
| Eigenwelt pp.136-139 | `devices/rutt_etra/Vasulka_Eigenwelt_RuttEtra_136-139.pdf` | 394KB | vasulka.org |
| ETC Panel Documentation | `devices/rutt_etra/ETC_RuttEtra_Panel.pdf` | 242KB | archive.org/details/ETC2530 |

### Jones Colorizer (1974)

| Document | File | Size | Source |
|----------|------|------|--------|
| MVIP Instructions | `devices/jones_colorizer/Jones_MVIP_Instructions.pdf` | 235KB | jonesvideo.com |

**Note:** Full colorizer schematics are not publicly available. Contact Dave Jones Design (djdesign.com) or check ETC archives. Video History Project has detailed design history: videohistoryproject.org/jones-colorizer-history-design

### LZX Cadet Series (2012)

Full open-source hardware at: https://github.com/lzxindustries/lzxcadet
Includes KiCad schematics, PCB layouts, BOMs, and owner's manuals.

---

## Components Layer

### RCA Databooks (Comprehensive Reference)

| Document | File | Size | Covers |
|----------|------|------|--------|
| 1975 RCA Linear ICs | `components/RCA_1975_Linear_ICs.pdf` | 48MB | All CA30xx series |
| 1978 RCA Linear IC Databook | `components/RCA_1978_Linear_IC_Databook.pdf` | 34MB | All CA30xx series |

These are the authoritative vendor references for every CA30xx component used in the Paik/Abe synthesizer.

### Individual IC Datasheets Retrieved

| Component | Type | File | Size | Source |
|-----------|------|------|------|--------|
| CA3046 | Transistor Array | `components/ics/ca3046/CA3046_Intersil_datasheet.pdf` | 121KB | ivytechengineering.com |
| CA3080 | OTA | `components/ics/ca3080/CA3080_Intersil_datasheet.pdf` | 216KB | octopart.com |
| CA3080 | OTA App Note | `components/ics/ca3080/CA3080_Application_Note_AN6668.pdf` | 1.9MB | intersil.com |
| LM311 | Comparator | `components/ics/lm311/LM311_TI_datasheet.pdf` | 2.4MB | ti.com |

### Individual IC Datasheets Pending (Browser Download Required)

These datasheets could not be retrieved programmatically due to aggregator site JavaScript gating. Full specs are available in the RCA databooks above. Provenance notes with known download URLs are in each component directory.

| Component | Type | Known Browser Sources |
|-----------|------|----------------------|
| CA3019 | Diode Array | semiee.com, alldatasheet.com |
| CA3054 | Dual Diff Amp | mouser.com (fn388.pdf), ibselectronics.com |
| CA3022 | Video Amp | datasheet4u.com |
| CA3023 | Video Amp | datasheet4u.com, datasheetspdf.com |

### ICs Modeled (WGSL)

| Component | Type | Used In | Files |
|-----------|------|---------|-------|
| CA3046 | Transistor Array | Paik/Abe, Sandin IP | model.wgsl, colorizer.wgsl |
| CA3019 | Diode Array | Paik/Abe PED Control | model.wgsl, ped_control.wgsl, colorizer.wgsl |
| CA3054 | Dual Diff Amp | Paik/Abe Matrix, Keyer | model.wgsl, matrix_mixer.wgsl, video_keyer.wgsl |
| CA3080 | OTA | Sandin IP | model.wgsl, vca.wgsl |
| LM311 | Comparator | Generic Keying | model.wgsl, keyer.wgsl |

### Functional Blocks Modeled

| Block | Based On | Used In | Files |
|-------|----------|---------|-------|
| Folding Amplifier | Discrete transistors | Paik/Abe | model.wgsl, folder.wgsl |
| Wobbulator | CRT deflection mod | Paik/Abe | model.wgsl, wobbulator.wgsl |

---

## Innovators Layer

| Person | Role | Device(s) | Documents Retrieved |
|--------|------|-----------|-------------------|
| Nam June Paik & Shuya Abe | Artist/Engineer | Paik/Abe | 3 PDFs |
| Stephen Beck | Artist/Engineer | Beck Direct Video | 1 PDF |
| Dan Sandin | Designer | Sandin IP | (docs under devices/) |
| Bill Etra & Steve Rutt | Engineers | Rutt/Etra | (docs under devices/) |
| Dave Jones | Engineer | Jones Colorizer | (docs under devices/) |

### Innovator Documents

| Document | File | Size | Source |
|----------|------|------|--------|
| Meigh-Andrews: Development, Functions & Legacy of Paik-Abe | `innovators/paik_abe/Meigh-Andrews_Development_Functions_Legacy_PaikAbe.pdf` | 198KB | NJP Art Center |
| Vasulka Eigenwelt: Ars Electronica catalogue | `innovators/paik_abe/Vasulka_Eigenwelt_ArsElectronica_AE043.pdf` | 724KB | vasulka.org |
| Vasulka Eigenwelt: Introduction pp.1-9 | `innovators/paik_abe/Vasulka_Eigenwelt_Intro_001-009.pdf` | 290KB | vasulka.org |
| Beck: Image Processing and Video Synthesis | `innovators/beck_stephen/Beck_Image_Processing_Video_Synthesis.pdf` | 1.4MB | vasulka.org |

---

## Techniques Layer

| Document | File | Size | Focus |
|----------|------|------|-------|
| Kane: Synthetic Color & Video Synthesis Circa 1969 | `techniques/Kane_Synthetic_Color_Video_Synthesis_1969.pdf` | 1MB | Historical survey, Paik/Abe colorizer |
| DAFx-17: Virtual Analog Buchla 259 Wavefolder | `techniques/DAFx17_Buchla259_Wavefolder.pdf` | 1.5MB | Wavefolding theory, parallel folding stages |
| SMC-17: Virtual Analog Lockhart Wavefolder | `techniques/SMC17_Lockhart_Wavefolder.pdf` | 347KB | Lockhart/Serge wavefolder, Lambert-W function |
| TI: Demystifying the OTA | `techniques/TI_Demystifying_OTA_sboa117a.pdf` | 266KB | OTA theory, video-rate applications |
| Stromeko: OTA Overview for Synth DIY | `techniques/Stromeko_OTA_Overview.pdf` | 419KB | Practical OTA circuits for synthesis |

---

## Creative Synthesis Flags

Tracked in `creative_synthesis_flags.json`. See [DISCOVERY_GUIDE.md](DISCOVERY_GUIDE.md#creative-synthesis-flags) for flag definitions.

---

## Source Archives

### Primary Archives
- [Vasulka Archive](http://vasulka.org) - Historical documentation (multiple PDFs retrieved)
- [Internet Archive / ETC](https://archive.org) - Experimental TV Center collection (ETC3310, ETC3311, ETC3316, ETC1365, ETC2530)
- [LZX Industries GitHub](https://github.com/lzxindustries) - Cadet schematics (open source)
- [NJP Art Center](https://njpart.ggcf.kr) - Paik/Abe academic papers
- [criticalartware.net](http://criticalartware.net) - Sandin Distribution Religion

### Datasheet Sources
- 1975 & 1978 RCA Linear IC Databooks (downloaded, comprehensive)
- [AllDatasheet](https://www.alldatasheet.com) - General search (browser only)
- [Datasheet Archive](https://www.datasheetarchive.com) - Obsolete parts (browser only)

### Community
- [Scanlines.xyz](https://scanlines.xyz) - Video synthesis forum
- [ModWiggler Video Forum](https://modwiggler.com/forum) - Eurorack video
- [Video History Project](https://www.videohistoryproject.org) - Historical documentation

---

## Research Log

### 2026-01-26: Research Cleanup & Component Restructure
- Removed legacy `patents/` and `notes/` directories (patents collected under devices/)
- Removed `provenance.db` (old 7-layer system artifact)
- Moved datasheets and PROVENANCE.md from `v1/` to component root
  - Reference docs (datasheets) are permanent, shared across model versions
  - WGSL models (model.wgsl, spec.json) stay versioned in `v1/`, `v2/`, etc.
- Archived swarm agent system to `archive/swarm-agent-concept/`
- Component structure now: `ics/part_name/{datasheet.pdf, PROVENANCE.md, v1/{model.wgsl, spec.json}}`

### 2026-01-26: Full Document Retrieval Run
Retrieved 31 valid PDFs across all 4 knowledge layers:
- **Devices**: 16 PDFs covering 4 of 5 v1 target devices
  - Paik/Abe: 7 PDFs (schematics, tech docs, budget, restoration, Vasulka)
  - Sandin IP: 3 PDFs (Distribution Religion, ETC Manual, Connolly Guide) - 154MB total
  - Rutt/Etra: 4 PDFs (Vasulka general, synth continuation, Eigenwelt, ETC panel)
  - Jones Colorizer: 1 PDF (MVIP instructions)
  - LZX Cadet: Available on GitHub (not PDF)
- **Components**: 6 PDFs (2 RCA databooks + 4 individual datasheets)
  - CA3046, CA3080, CA3080 App Note, LM311 - individual datasheets
  - RCA 1975 & 1978 databooks cover ALL CA30xx parts
  - CA3019, CA3054, CA3022, CA3023 - blocked by aggregator JavaScript (provenance notes created)
- **Innovators**: 4 PDFs (Meigh-Andrews, Beck, 2x Vasulka Eigenwelt)
- **Techniques**: 5 PDFs (Kane, DAFx17, SMC17, TI OTA, Stromeko OTA)
- **Patent discovery**: Paik/Abe synthesizer was never patented (US4011557 reference was wrong)

### 2026-01-26: Knowledge Base Restructured to 4-Layer Model
Replaced 7-layer research pyramid with focused 4-layer taxonomy:
- Innovators, Techniques, Devices, Components
- Patents/manuals/schematics consolidated under Devices (per-device)
- Vendor datasheets consolidated under Components (per-part)
- Added creative synthesis flags for artistic exploration

### 2026-01-25: Paik/Abe Core Complete
All major functional blocks modeled and tested:
- CA3046, CA3019, CA3054, CA3080, LM311
- Folding amplifier, Wobbulator
- Parameter sweep documentation captured

### 2026-01-25: Component Agent Workflow
Automated pipeline: Extract ICs -> lookup datasheets -> spec.json -> model.wgsl -> test

---

*Update this file as research progresses.*
