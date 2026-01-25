# Research Discovery Guide

How to find primary source documents for historical video synthesis hardware.

---

## The 7-Layer Research Model

```
ABSTRACT                                              CONCRETE
   │                                                      │
   ▼                                                      ▼
┌──────────┐  ┌──────────┐  ┌─────────────┐  ┌─────────┐  ┌─────────┐  ┌───────────┐  ┌────────────┐
│TECHNIQUES│→ │ FIGURES  │→ │TECHNOLOGIES │→ │ PATENTS │→ │ MANUALS │→ │SCHEMATICS │→ │ COMPONENTS │
└──────────┘  └──────────┘  └─────────────┘  └─────────┘  └─────────┘  └───────────┘  └────────────┘
   What?         Who?          Which?          Legal       How to       How it's        IC models
                                              docs         use it       built           & shaders
```

Work flows both directions:
- **Forward:** Research a technique → find who invented it → find their devices → get docs → model components
- **Backward:** Have a schematic → identify components → understand techniques → attribute to figures

---

## Source Discovery by Layer

### 1. TECHNIQUES
**Goal:** Understand abstract methods before implementing

| Source Type | Where to Find |
|-------------|---------------|
| Academic papers | Google Scholar, ResearchGate, IEEE Xplore |
| Textbooks | Archive.org, library databases |
| Tutorial articles | Electronotes, synth-diy archives |
| Patent descriptions | Google Patents (method claims) |

**Search terms:**
- "video synthesis [technique]"
- "analog video [effect]"
- "CRT [manipulation]"
- "[effect] circuit design"

### 2. FIGURES
**Goal:** Find the inventors and their work

| Source Type | Where to Find |
|-------------|---------------|
| Biographies | Wikipedia, museum archives |
| Interviews | Video Data Bank, oral histories |
| Retrospectives | Art journals, exhibition catalogs |
| Obituaries | Newspapers, art publications |

**Key Archives:**
- [Vasulka Archive](http://vasulka.org) - Comprehensive video art history
- [Experimental Television Center](http://experimentaltvcenter.org) - Equipment and artists
- [Video Data Bank](http://www.vdb.org) - Artist interviews and works
- [Fondation Langlois](http://fondation-langlois.org) - Daniel Langlois collection

### 3. TECHNOLOGIES
**Goal:** Document specific devices

| Source Type | Where to Find |
|-------------|---------------|
| Device histories | Museum collections, Wikipedia |
| Technical descriptions | Journal articles, theses |
| Demo videos | YouTube, Vimeo, archive footage |
| User testimonials | Forums, interviews |

**Search strategy:**
1. Start with device name + "video synthesizer"
2. Add inventor names for more specific results
3. Check museum collections (Smithsonian, ZKM, etc.)

### 4. PATENTS
**Goal:** Get legal descriptions with detailed diagrams

**Primary Sources:**
- [Google Patents](https://patents.google.com) - Free, searchable, includes PDFs
- [USPTO](https://www.uspto.gov) - US patents
- [Espacenet](https://espacenet.com) - European patents

**Search Strategy:**
```
site:patents.google.com "video synthesizer" [inventor name]
site:patents.google.com [device name]
site:patents.google.com "video signal" "colorizer" 1970..1980
```

**Example Searches:**
- `site:patents.google.com "Shuya Abe"`
- `site:patents.google.com "video colorizer" 1970..1975`
- `site:patents.google.com "raster manipulation"`

**Patent Sections to Extract:**
- Claims (what's protected - describes function)
- Detailed Description (how it works)
- Drawings (circuit diagrams!)
- References Cited (find related patents)

### 5. MANUALS
**Goal:** Find operation and service documentation

**Primary Sources:**
- [Archive.org](https://archive.org) - Massive document archive
- [Manuals Library](https://www.manualslib.com) - User manuals
- [HiFi Engine](https://www.hifiengine.com) - Audio/video equipment
- eBay listings - Sometimes include manual scans
- Museum archives - May have unpublished docs

**Search Strategy:**
```
site:archive.org "[device name]" manual
site:archive.org "[device name]" service
"[device name]" filetype:pdf manual
```

### 6. SCHEMATICS
**Goal:** Get circuit diagrams for component identification

**Primary Sources:**
- [Archive.org](https://archive.org) - Historical documents
- [Elektrotanya](https://elektrotanya.com) - Service manuals with schematics
- [Vasulka Archive](http://vasulka.org) - Video synthesis schematics
- GitHub repositories - Modern open-source designs
- University archives - Research equipment docs
- eBay listings - May include service docs

**Search Strategy:**
```
site:archive.org "[device name]" schematic
site:github.com "[device name]" schematic
"[device name]" filetype:pdf schematic
```

**For Open-Source Devices:**
- [LZX Industries GitHub](https://github.com/lzxindustries) - Cadet, Castle, etc.
- [Syntonie DIY](https://github.com/syntonie) - Video synth circuits

### 7. COMPONENTS (Datasheets)
**Goal:** Get IC specifications and transfer functions

**Primary Sources:**
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

---

## Search Workflow

### When Starting a New Device

1. **Google Scholar search:**
   ```
   "[device name]" OR "[inventor name]" video synthesizer
   ```

2. **Patent search:**
   ```
   site:patents.google.com "[inventor name]" video
   ```

3. **Archive.org search:**
   ```
   site:archive.org "[device name]" (schematic OR manual OR documentation)
   ```

4. **GitHub search:**
   ```
   site:github.com "[device name]"
   ```

5. **Check known archives:**
   - Vasulka Archive
   - Experimental TV Center
   - Fondation Langlois

### When You Have a Schematic

1. **Extract IC part numbers** (e.g., CA3046, LM311)
2. **Search datasheets** for each IC
3. **Identify transfer functions** from datasheet graphs
4. **Cross-reference** with technique documentation
5. **Build component models** in WGSL

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

Track what's been found for each device:

### Paik/Abe Video Synthesizer
- [x] Schematics (26 pages) - From archive
- [x] Technical Documentation (87 pages) - From archive
- [ ] Patents - NOT FOUND (may not exist)
- [ ] Service Manual - NOT FOUND

### Sandin Image Processor
- [ ] Schematics - Check Archive.org, Vasulka
- [ ] Distribution Religion docs - Archive.org
- [ ] Patents - Search Google Patents

### Rutt/Etra Video Synthesizer
- [ ] Schematics - Check Vasulka archive
- [ ] Patents - Search Google Patents
- [ ] Service docs - Rare

### Jones Colorizer
- [ ] Schematics - Jones published openly
- [ ] Build docs - Community sources

### LZX Cadet Series
- [x] Schematics - GitHub (complete)
- [x] BOMs - GitHub
- [x] Firmware - GitHub

---

## Automation Opportunities

### What Can Be Automated
- Datasheet search for known part numbers
- Patent search by inventor name
- Archive.org queries
- GitHub repository discovery

### What Requires Human Judgment
- Evaluating document authenticity
- Interpreting unclear schematics
- Choosing between conflicting sources
- Museum/archive access requests

---

## Retrieval Tools (Implemented)

The following tools are now available for automated document retrieval:

### Command Line Interface

```bash
# Patents
python -m tools.retrieval.patents search "video synthesizer" --inventor="Abe"
python -m tools.retrieval.patents download US4011557

# Datasheets
python -m tools.retrieval.datasheets search CA3046
python -m tools.retrieval.datasheets download CA3046

# Schematics
python -m tools.retrieval.schematics search paik-abe
python -m tools.retrieval.schematics download paik-abe-synthesizer

# Manuals
python -m tools.retrieval.manuals search fairlight --type=service
python -m tools.retrieval.manuals download fairlight-cvi

# Components
python -m tools.retrieval.components list
python -m tools.retrieval.components history ics/ca3046
```

### MCP Tools (via Claude)

| Tool | Description |
|------|-------------|
| `retrieve_patent` | Search/download patents |
| `retrieve_datasheet` | Search/download datasheets |
| `retrieve_schematic` | Search/download schematics |
| `retrieve_manual` | Search/download manuals |
| `get_provenance` | Get artifact provenance |
| `list_components` | List WGSL components |
| `component_history` | Get version history |

### Provenance Database

All retrieved artifacts are tracked in `research/provenance.db`:

```sql
-- List all patents
SELECT * FROM artifacts WHERE type = 'patent';

-- Get version history for a component
SELECT * FROM versions WHERE artifact_id = 'component:ics/ca3046';

-- Find relationships
SELECT * FROM relationships WHERE from_id = 'datasheet:CA3046';
```

---

*This guide should be updated as new sources are discovered.*
