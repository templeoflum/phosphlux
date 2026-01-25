# Historical Video Synthesis Emulation Plan

## Philosophy

**"Follow the inventors, not the interpreters."**

Rather than implementing effects based on secondhand descriptions, we trace the actual historical evolution of video art hardware. Each era's innovations emerged from specific technical constraints and creative breakthroughs. By emulating the actual components, we gain:

1. **Authentic behavior** - Including the "happy accidents" that defined the aesthetic
2. **Deep understanding** - First-hand knowledge of why things work the way they do
3. **Complete palette** - Access to the full range of each device's capabilities
4. **Historical continuity** - Understanding how each innovation built on predecessors

---

## Timeline of Video Art Hardware Evolution

### Era 1: Pre-Video (1950-1965) - Oscilloscope Art
| Year | Device/Innovation | Key Figure | Priority |
|------|-------------------|------------|----------|
| 1950 | Oscilloscope Photography | Ben Laposky | HIGH |
| 1952 | "Oscillons" exhibition | Laposky | - |
| 1958 | Analog Computer Animation | John Whitney Sr. | MEDIUM |
| 1961 | Cam Machine (M-5 director) | Whitney | MEDIUM |

**Key Components to Emulate:**
- CRT phosphor persistence (vector mode)
- X-Y deflection from audio signals
- Long-exposure integration
- Lissajous figure generation

### Era 2: Birth of Video Synthesis (1965-1972)
| Year | Device/Innovation | Key Figure | Priority |
|------|-------------------|------------|----------|
| 1965 | Magnet TV (CRT manipulation) | Nam June Paik | MEDIUM |
| 1969 | Paik/Abe Video Synthesizer | Paik + Shuya Abe | **CRITICAL** |
| 1970 | Direct Video Synthesizer | Stephen Beck | HIGH |
| 1971 | Sandin Image Processor | Dan Sandin | **CRITICAL** |
| 1972 | Rutt/Etra Video Synthesizer | Steve Rutt + Bill Etra | **CRITICAL** |
| 1973 | Scanimate | Lee Harrison III | MEDIUM |

**Key Components to Emulate:**
- Paik/Abe: Folding amplifier, resistive color matrix, wobbulator
- Beck DVS: Voltage-to-position converter, geometric region processor
- Sandin IP: MC1445 gain-controlled amp, amplitude classifier, differentiator
- Rutt/Etra: DC-coupled scan manipulation, Z-axis displacement

### Era 3: Colorization & Keying (1973-1980)
| Year | Device/Innovation | Key Figure | Priority |
|------|-------------------|------------|----------|
| 1973 | Jones Colorizer | Dave Jones | HIGH |
| 1974 | Videolab | Bill Hearn | MEDIUM |
| 1975 | EMS Spectron | EMS | MEDIUM |
| 1978 | Fairlight CVI | Fairlight | MEDIUM |

**Key Components to Emulate:**
- Jones: Diode/layered mixing, threshold colorization without encoder
- Keying circuits: Luminance key, chroma key implementations
- Color encoding: NTSC/PAL encoder circuits

### Era 4: Digital Hybrid (1980-1995)
| Year | Device/Innovation | Key Figure | Priority |
|------|-------------------|------------|----------|
| 1981 | Amiga (genlock capable) | Commodore | LOW |
| 1984 | Fairlight CVI II | Fairlight | MEDIUM |
| 1990s | Video Toaster | NewTek | LOW |
| 1990s | Various frame buffers | Multiple | MEDIUM |

### Era 5: VJ Culture (1995-2010)
| Year | Device/Innovation | Key Figure | Priority |
|------|-------------------|------------|----------|
| 2000 | Edirol V4 | Roland/Edirol | HIGH |
| 2003 | Korg Entrancer | Korg | MEDIUM |
| 2005 | Edirol V8 | Roland/Edirol | MEDIUM |
| 2008 | V4EX | Roland | HIGH |

**Key Components to Emulate:**
- V4: Feedback characteristics, effects chain, T-bar behavior
- Transition types, wipe patterns
- Effects: colorize, strobe, negative, etc.

### Era 6: Modular Renaissance (2010-Present)
| Year | Device/Innovation | Key Figure | Priority |
|------|-------------------|------------|----------|
| 2010 | LZX Visionary | LZX Industries | **CRITICAL** |
| 2012 | LZX Cadet (DIY) | LZX Industries | HIGH |
| 2015 | Brownshoesonly modules | Brownshoesonly | MEDIUM |
| 2018 | LZX Gen3 | LZX Industries | HIGH |
| 2020 | Chromagnon | LZX Industries | HIGH |

**Key Components to Emulate:**
- LZX signal standards (0-1V, 1V/octave)
- Cadet series (schematics publicly available)
- Gen3 architecture patterns

### Display Technology
| Device | Type | Priority |
|--------|------|----------|
| Consumer CRT | Display | HIGH |
| Sony PVM/BVM | Display | **CRITICAL** |
| Vectorscope | Display | MEDIUM |
| RGB Monitor | Display | HIGH |

---

## Research Agent Architecture

### Data Sources (Priority Order)

**Primary Sources (Most Reliable):**
1. Original patents and patent applications
2. Manufacturer service manuals
3. Official schematics and blueprints
4. Academic papers by the inventors
5. Datasheet archives (component specs)

**Secondary Sources (Verify Against Primary):**
6. Video synthesis forums (Scanlines.xyz, Modwiggler)
7. DIY builder documentation
8. Museum archives (EAI, Ars Electronica)
9. Interview transcripts with inventors
10. Restoration project documentation

**Tertiary Sources (Context Only):**
11. Wikipedia / general articles
12. YouTube teardown videos
13. Facebook groups (Video Circuits, etc.)
14. Reddit communities

### Source Repositories to Monitor

```
OFFICIAL/MANUFACTURER:
- lzxindustries.net (schematics, manuals)
- analoguehaven.com (archived manuals)
- roland.com/support (V4/V8 service docs)

ARCHIVES:
- archive.org (Wayback Machine for defunct sites)
- experimentaltvcenter.org
- vasulka.org (Steina & Woody Vasulka archive)
- fondation-langlois.org (Daniel Langlois archive)

DIY/COMMUNITY:
- github.com/lzxindustries (Cadet schematics)
- scanlines.xyz (video synthesis forum)
- modwiggler.com/forum (eurorack video section)
- muffwiggler archives

ACADEMIC:
- Google Scholar (search: video synthesis, CRT, analog video)
- IEEE Xplore (broadcast engineering papers)
- AES papers (audio/video engineering)

COMPONENT DATA:
- alldatasheet.com
- datasheetcatalog.com
- archive.org semiconductor archives
```

### Research Agent Workflow

```
┌─────────────────────────────────────────────────────────────┐
│                    RESEARCH PIPELINE                         │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  1. DISCOVERY                                               │
│     ├─ Web search (targeted queries)                        │
│     ├─ Forum crawl (keywords + device names)                │
│     ├─ Archive.org historical snapshots                     │
│     └─ Patent database search                               │
│                                                             │
│  2. COLLECTION                                              │
│     ├─ Download PDFs, images, schematics                    │
│     ├─ Extract text from scanned documents (OCR)            │
│     ├─ Capture forum threads with context                   │
│     └─ Store with source attribution + date                 │
│                                                             │
│  3. EXTRACTION                                              │
│     ├─ Identify component values (resistors, caps, ICs)     │
│     ├─ Map signal flow from schematics                      │
│     ├─ Note operating voltages and impedances               │
│     └─ Extract behavioral descriptions                      │
│                                                             │
│  4. CROSS-REFERENCE                                         │
│     ├─ Compare multiple sources for same device             │
│     ├─ Flag contradictions for manual review                │
│     ├─ Weight by source reliability                         │
│     └─ Build confidence scores                              │
│                                                             │
│  5. VALIDATION                                              │
│     ├─ Compare against known reference footage              │
│     ├─ Check component behavior against datasheets          │
│     ├─ Expert review (community feedback)                   │
│     └─ A/B testing with real hardware (when possible)       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Validation Framework

### Levels of Verification

**Level 1: Schematic Accuracy**
- Component values match original documentation
- Signal flow matches block diagrams
- Operating points within specified ranges

**Level 2: Behavioral Accuracy**
- Transfer functions match measured data
- Frequency response within tolerance
- Nonlinearities match (clipping, saturation)

**Level 3: Visual Accuracy**
- Output matches reference footage
- Expert blind testing (can't distinguish from hardware)
- Side-by-side A/B comparison passes

**Level 4: Experiential Accuracy**
- "Feels" right to experienced users
- Unexpected behaviors match hardware quirks
- Edge cases behave authentically

### Verification Checklist Per Device

```markdown
## [Device Name] Verification

### Documentation Status
- [ ] Service manual obtained
- [ ] Schematic verified against multiple sources
- [ ] Component datasheets collected
- [ ] Block diagram created

### Implementation Status
- [ ] Signal flow implemented
- [ ] Component values accurate
- [ ] Nonlinearities modeled
- [ ] Edge cases handled

### Validation Status
- [ ] Level 1: Schematic review passed
- [ ] Level 2: Behavioral tests passed
- [ ] Level 3: Visual comparison passed
- [ ] Level 4: Expert review passed

### Reference Materials
- Links to sources
- Notes on discrepancies
- Confidence score: X/10
```

---

## Component Emulator Application

### Purpose
Isolated development environment for creating and validating hardware emulation nodes before integration into main Phosphlux application.

### Features

**Input Section:**
- Load reference images/video
- Generate test patterns (ramps, bars, circles)
- Input from main Phosphlux (for integration testing)

**Emulation Section:**
- Load component schematic view
- Adjust component values in real-time
- See signal flow visualization
- Probe points throughout circuit

**Comparison Section:**
- Side-by-side view (emulation vs reference)
- Difference visualization
- Waveform comparison (oscilloscope view)
- Histogram comparison

**Validation Section:**
- Run automated test suites
- Generate accuracy reports
- Export validated node to Phosphlux

### Architecture

```
phosphlux-emulator/
├── src/
│   ├── main.rs              # App entry
│   ├── devices/             # Device emulations
│   │   ├── paik_abe/
│   │   ├── sandin_ip/
│   │   ├── rutt_etra/
│   │   ├── jones_colorizer/
│   │   ├── v4/
│   │   └── lzx/
│   ├── components/          # Low-level component models
│   │   ├── opamp.rs
│   │   ├── comparator.rs
│   │   ├── resistor_network.rs
│   │   └── capacitor.rs
│   ├── validation/          # Testing framework
│   └── ui/                  # Comparison/analysis UI
├── reference/               # Reference footage/images
├── schematics/              # Collected schematics
└── tests/                   # Validation test suites
```

---

## Immediate Next Steps

### Phase 1: Research Infrastructure (Week 1-2)
1. Set up research document repository structure
2. Create research agent prompts/workflows
3. Begin systematic collection for Paik/Abe (first target)
4. Identify component datasheet sources

### Phase 2: Component Emulator Foundation (Week 2-3)
1. Create separate Rust project for emulator
2. Implement basic test pattern generation
3. Implement comparison/diff visualization
4. Create component model framework

### Phase 3: First Device - Oscilloscope/Vector Mode (Week 3-4)
1. Research CRT phosphor characteristics
2. Implement vector drawing mode
3. Implement phosphor persistence model
4. Validate against Laposky reference images

### Phase 4: Paik/Abe Synthesizer (Week 4-6)
1. Complete research collection
2. Implement folding amplifier
3. Implement resistive color matrix
4. Implement wobbulator (scan modulation)
5. Validate against known Paik footage

---

## Questions to Resolve Together

1. **Research Agent Implementation**: Should this be a separate tool, or integrated into Claude Code workflow with web search?

2. **Reference Material Storage**: Local folder structure? Git LFS for large files? Separate repository?

3. **Validation Footage Sources**: Do you have access to hardware for capture? Known good reference footage?

4. **Development Priority**: Start with oscilloscope mode (simplest) or jump to Paik/Abe (most iconic)?

5. **Emulator App**: Separate application or integrated "dev mode" in main Phosphlux?

---

*This is a living document. Update as research progresses.*
