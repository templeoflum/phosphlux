# Research Discovery Guide

How to find primary source documents for faithful 1:1 emulation of historical video synthesis hardware.

---

## The 4-Layer Knowledge Base

```
CONTEXT                                              IMPLEMENTATION
   │                                                      │
   ▼                                                      ▼
┌────────────┐   ┌────────────┐   ┌─────────┐   ┌────────────┐
│ INNOVATORS │ → │ TECHNIQUES │ → │ DEVICES │ → │ COMPONENTS │
└────────────┘   └────────────┘   └─────────┘   └────────────┘
   Who built       How does the     How is this    How does this
   it and why?     processing       machine         part behave?
                   work?            built?
```

Each layer serves a distinct role in achieving faithful emulation:

| Layer | Purpose for Emulation | Contains |
|-------|----------------------|----------|
| **Innovators** | Design philosophy, artistic intent behind circuit choices | Bios, interviews, exhibition notes, philosophical context |
| **Techniques** | Understanding what the circuit *should do* (device-agnostic) | Papers, theory, signal processing methods |
| **Devices** | The actual circuit to recreate: topology, values, rails, calibration | Schematics, service manuals, patents, calibration data, operation guides |
| **Components** | The physics of each part: transfer functions, limits, thermal drift, power draw | Vendor datasheets, SPICE parameters, spec.json, WGSL models |

**Work flows both directions:**
- **Forward:** Understand an innovator's intent → learn their techniques → get their device docs → model the components
- **Backward:** Have a component datasheet → understand its role in a device → see which technique it implements → attribute to innovator

---

## Source Discovery by Layer

### 1. INNOVATORS
**Goal:** Understand design philosophy and artistic intent

| Source Type | Where to Find |
|-------------|---------------|
| Biographies | Wikipedia, museum archives |
| Interviews | Video Data Bank, oral histories |
| Retrospectives | Art journals, exhibition catalogs |
| Design notes | University archives, personal collections |

**Key Archives:**
- [Vasulka Archive](http://vasulka.org) - Comprehensive video art history
- [Experimental Television Center](http://experimentaltvcenter.org) - Equipment and artists
- [Video Data Bank](http://www.vdb.org) - Artist interviews and works
- [Fondation Langlois](http://fondation-langlois.org) - Daniel Langlois collection

**Why this matters for emulation:** Understanding *why* Paik chose to fold video signals tells you what the circuit should feel like, not just what it does mathematically. Intent guides parameter tuning.

### 2. TECHNIQUES
**Goal:** Understand abstract signal processing methods (device-agnostic)

| Source Type | Where to Find |
|-------------|---------------|
| Academic papers | Google Scholar, ResearchGate, IEEE Xplore |
| Textbooks | Archive.org, library databases |
| Tutorial articles | Electronotes, synth-diy archives |
| Patent method claims | Google Patents (claims section) |

**Search terms:**
- "video synthesis [technique]"
- "analog video [effect]"
- "CRT [manipulation]"
- "[effect] circuit design"
- "wavefolding amplifier"
- "amplitude windowing video"

**Why this matters for emulation:** Techniques like wavefolding, amplitude keying, and scan displacement are shared across devices. Understanding the theory means you can validate any device's implementation.

### 3. DEVICES
**Goal:** Get ALL documentation for a specific machine in one place

This is the most critical layer for 1:1 emulation. Each device directory collects:

| Document Type | What It Provides | Where to Find |
|---------------|-----------------|---------------|
| **Schematics** | Circuit topology, component values, power rails | Archives, patents, service manuals |
| **Service manuals** | Schematics + calibration voltages + test points | Archive.org, eBay, HiFi Engine |
| **Patents** | Detailed circuit descriptions, design rationale | Google Patents, USPTO, Espacenet |
| **Operation guides** | Parameter ranges, expected behavior | Museum archives, user communities |
| **Calibration data** | Voltage test points, alignment procedures | Service manuals |

**Search Strategy (for any device):**

1. **Google Scholar:**
   ```
   "[device name]" OR "[inventor name]" video synthesizer
   ```

2. **Patent search:**
   ```
   site:patents.google.com "[inventor name]" video
   site:patents.google.com "video synthesizer" "[device name]"
   ```

3. **Archive.org:**
   ```
   site:archive.org "[device name]" (schematic OR manual OR documentation)
   ```

4. **GitHub (for modern/open-source devices):**
   ```
   site:github.com "[device name]" schematic
   ```

5. **Known archives:** Vasulka, Experimental TV Center, Fondation Langlois

**Patent Sections to Extract:**
- Claims → describes function
- Detailed Description → how it works
- Drawings → circuit diagrams
- References Cited → related patents

**For Open-Source Devices:**
- [LZX Industries GitHub](https://github.com/lzxindustries) - Cadet, Castle, etc.
- [Syntonie DIY](https://github.com/syntonie) - Video synth circuits

### 4. COMPONENTS
**Goal:** Get vendor datasheets and transfer functions for every IC/transistor/part

Each component directory should contain:
- **Vendor datasheet PDF** - the authoritative source
- **spec.json** - extracted SPICE parameters, transfer functions, limits
- **model.wgsl** - WGSL shader implementation
- **provenance.json** - where the data came from

**Primary Datasheet Sources:**
- [AllDatasheet](https://www.alldatasheet.com)
- [DatasheetCatalog](https://www.datasheetcatalog.com)
- [Datasheet Archive](https://www.datasheetarchive.com)
- [BITSAVERS](http://bitsavers.org) - Historical datasheets

**For Obsolete ICs:**
- [1978 RCA Linear IC Databook](https://www.smcelectronics.com/DOWNLOADS/1978-RCA%20LINEAR%20IC.PDF) - CA-30xx series
- Archive.org databook scans
- eBay "NOS" (new old stock) listings often have datasheets

**Key IC Families for Video:**

| Prefix | Manufacturer | Era | Notes |
|--------|-------------|-----|-------|
| CA30xx | RCA | 1960s-1980s | Video amps, arrays |
| LM3xx | National | 1970s-present | Comparators, op-amps |
| MC14xx | Motorola | 1970s | Analog switches, multipliers |
| AD6xx | Analog Devices | 1980s-present | High-speed video |

**Critical for power-starve emulation:** Datasheets specify operating ranges AND behavior at supply limits. The "Absolute Maximum Ratings" and "Electrical Characteristics" tables show how parts degrade when starved.

---

## Directory Structure

```
research/
├── innovators/         # Per-person directories
│   ├── paik_abe/
│   └── beck_stephen/
├── techniques/         # Technique papers (PDFs)
├── devices/            # Per-device: ALL docs together
│   ├── paik_abe/       # Schematics, patents, manuals, calibration
│   ├── sandin_ip/
│   ├── rutt_etra/
│   ├── jones_colorizer/
│   └── lzx_cadet/
├── components/         # Per-part: datasheets + models
│   ├── RCA_1975_Linear_ICs*.pdf   # Comprehensive databook (splits)
│   ├── RCA_1978_Linear_IC_Databook*.pdf
│   ├── ics/
│   │   ├── ca3046/               # Component root
│   │   │   ├── CA3046_*.pdf      # Vendor datasheet (permanent ref)
│   │   │   ├── PROVENANCE.md     # Source URLs, retrieval notes
│   │   │   └── v1/               # WGSL model iteration
│   │   │       ├── spec.json     # SPICE params, transfer functions
│   │   │       └── model.wgsl    # Shader implementation
│   │   └── ...
│   └── functional/
│       ├── folding_amp/v1/
│       └── wobbulator/v1/
├── RESEARCH_INDEX.md   # Master status tracker
└── DISCOVERY_GUIDE.md  # This file
```

**Key structural rules:**
- **Datasheets** go at component root (permanent reference across model versions)
- **WGSL models** go in versioned subdirs (`v1/`, `v2/`, etc.)
- **PROVENANCE.md** at component root documents source URLs and retrieval status

---

## PDF Handling Rules

Claude can only read PDFs under **20MB**. Large documents must be split.

### Split Workflow

```bash
# Check if a PDF needs splitting
python tools/pdf-utils/split_pdf.py info <path.pdf>

# Auto-split to ~15MB chunks
python tools/pdf-utils/split_pdf.py split <path.pdf>

# Extract specific pages
python tools/pdf-utils/split_pdf.py extract <path.pdf> 10 25
```

### Git Rules for PDFs

- **Originals >20MB**: Listed in `.gitignore`, kept locally only
- **Splits (_pages_N-M.pdf)**: Tracked in git, all under 20MB
- To regenerate splits from originals: `python tools/pdf-utils/split_pdf.py split <original.pdf>`

### Current Large PDFs (local only, splits tracked)

| Original | Size | Splits |
|----------|------|--------|
| ETC_SandinIP_Manual.pdf | 106MB | 8 parts |
| Paik Abe Video Synthesizer Technical Documentation.pdf | 51MB | 3 parts |
| RCA_1975_Linear_ICs.pdf | 46MB | 4 parts |
| Sandin_Distribution_Religion.pdf | 33MB | 3 parts |
| RCA_1978_Linear_IC_Databook.pdf | 33MB | 3 parts |

---

## Creative Synthesis Flags

Documents can be flagged for creative exploration beyond faithful emulation:

| Flag | Meaning |
|------|---------|
| `glitch_potential` | Circuit behavior under non-standard conditions |
| `happy_accident` | Unexpected visual results worth exploring |
| `cross_device` | Technique transferable between devices |
| `power_starve` | Behavior when power rails are degraded |
| `component_abuse` | Operating outside spec for creative effect |
| `signal_feedback` | Feedback loop possibilities |
| `novel_topology` | Unusual circuit connections |

Flag documents as you discover them. These feed future creative synthesis work after faithful emulation is complete.

---

## Document Quality Checklist

Before relying on a document, verify:

| Check | Why |
|-------|-----|
| Original source? | Avoid transcription errors |
| Complete? | Partial schematics miss crucial details |
| Legible? | Can read all component values? |
| Dated? | Know which revision you have |
| Verified? | Cross-reference with other sources |

---

## Discovered Sources Log

See [RESEARCH_INDEX.md](RESEARCH_INDEX.md) for the full inventory of retrieved documents (31 PDFs across all 4 layers).

### Paik/Abe Video Synthesizer
- [x] Schematics (26 pages) - ETC3311 + pre-existing
- [x] Technical Documentation (87 pages) - Pre-existing (split to 3 parts)
- [x] Restoration docs, budget, Vasulka Eigenwelt excerpt
- **Not patented** — was an art tool, never commercialized

### Sandin Image Processor
- [x] Distribution Religion (117 pages, split to 3 parts) - criticalartware.net
- [x] ETC Manual with schematics (63 pages, split to 8 parts) - archive.org/details/ETC1365
- [x] Connolly reformatted guide (8MB) - jameshconnolly.com

### Rutt/Etra Video Synthesizer
- [x] Vasulka general docs + synth continuation + Eigenwelt + ETC panel (4 PDFs)
- [ ] Service/calibration data - NOT FOUND

### Jones Colorizer
- [x] MVIP Instructions - jonesvideo.com
- [ ] Full schematics - NOT publicly available (contact Dave Jones Design)

### LZX Cadet Series
- [x] Schematics, BOMs, firmware — GitHub (github.com/lzxindustries/lzxcadet)

---

## Retrieval Tools

### MCP Tools (via Claude)

| Tool | Description |
|------|-------------|
| `retrieve_patent` | Search/download patents → store in device dir |
| `retrieve_datasheet` | Search/download datasheets → store in component dir |
| `retrieve_schematic` | Search/download schematics → store in device dir |
| `retrieve_manual` | Search/download manuals → store in device dir |
| `list_components` | List WGSL component models |
| `component_history` | Get version history for a component |

### PDF Utilities

```bash
python tools/pdf-utils/split_pdf.py info <pdf>      # Check size and page count
python tools/pdf-utils/split_pdf.py split <pdf>      # Auto-split to <15MB chunks
python tools/pdf-utils/split_pdf.py extract <pdf> N M # Extract page range
python tools/pdf-utils/split_pdf.py images <pdf>     # Convert pages to PNGs
```

---

*This guide should be updated as new sources are discovered.*
