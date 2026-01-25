# Phosphlux v1 Emulation Roadmap

Target: Component-level emulation of five foundational video synthesis devices.

---

## Device Order

| # | Device | Era | Complexity | Status |
|---|--------|-----|------------|--------|
| 1 | **Paik/Abe Video Synthesizer** | 1969-1971 | High | 🔄 In Progress |
| 2 | **Sandin Image Processor** | 1971-1974 | High | ⏳ Pending |
| 3 | **Rutt/Etra Video Synthesizer** | 1972-1974 | Medium | ⏳ Pending |
| 4 | **Jones Colorizer** | 1974 | Medium | ⏳ Pending |
| 5 | **LZX Cadet Series** | 2012 | Low | ⏳ Pending |

---

## 1. Paik/Abe Video Synthesizer (1969-1971)

*Nam June Paik & Shuya Abe — The first dedicated video synthesizer*

### What It Does
- 7-channel video input with matrix mixing
- Amplitude window colorization (luminance → color bands)
- "Wobbulator" scan modulation
- Nonlinear folding amplifiers

### Documents Collected
- [x] Schematics (26 pages)
- [x] Technical Documentation (87 pages)
- [ ] Patents (if any exist)

### Components Needed

| IC | Type | Function | Status |
|----|------|----------|--------|
| CA3046 | Transistor Array | Diff pairs, current mirrors | ✅ Modeled |
| CA3019 | Diode Array | Amplitude window detection | ❌ Need datasheet |
| CA3022 | Video Amp | Signal conditioning | ❌ Need datasheet |
| CA3023 | Video Amp | Signal conditioning | ❌ Need datasheet |
| CA3054 | Diff Amp | Matrix mixing | ❌ Need datasheet |
| CA3055 | Diff Amp | Matrix mixing | ❌ Need datasheet |

### Functional Blocks to Build
1. [ ] **7-Channel Input Matrix** — Weighted mixing of video sources
2. [x] **Amplitude Window Colorizer** — CA3046 diff pairs (colorizer.wgsl exists)
3. [ ] **Folding Amplifier** — Nonlinear waveshaping
4. [ ] **Wobbulator** — Horizontal/vertical scan displacement
5. [ ] **Output Encoder** — RGB to composite (optional)

### Current Progress
- CA3046 colorizer shader tested in workbench
- Need remaining CA-30xx datasheets from RCA Linear IC Databook

---

## 2. Sandin Image Processor (1971-1974)

*Dan Sandin — Open-source modular video processor*

### What It Does
- Modular "Distribution Religion" — schematics freely shared
- Amplitude classifier (quantized color levels)
- Four-quadrant multiplication
- Differencing amplifier
- Analog logic gates for video

### Documents Needed
- [ ] Full schematics from Archive.org
- [ ] Distribution Religion documentation
- [ ] IP module specifications

### Components Needed

| IC | Type | Function | Status |
|----|------|----------|--------|
| CA3080 | OTA | Voltage controlled gain | ✅ Modeled |
| MC1445 | Gain Block | Signal amplification | ❌ Need datasheet |
| CA3046 | Transistor Array | Various | ✅ Modeled |
| LM311 | Comparator | Level detection | ✅ Modeled |

### Functional Blocks to Build
1. [x] **VCA (Voltage Controlled Amplifier)** — CA3080 OTA (vca.wgsl exists)
2. [ ] **Amplitude Classifier** — Quantized luminance levels
3. [ ] **Differencing Amplifier** — A - B operation
4. [ ] **Four-Quadrant Multiplier** — A × B with sign
5. [ ] **Analog Logic Gates** — AND, OR, XOR for video

---

## 3. Rutt/Etra Video Synthesizer (1972-1974)

*Steve Rutt & Bill Etra — Raster manipulation*

### What It Does
- **Z-axis displacement** — Luminance controls scan line deflection
- DC-coupled throughout (preserves DC levels)
- Horizontal and vertical raster bending
- The iconic "terrain" effect

### Documents Needed
- [ ] Schematics (search Archive.org, Vasulka)
- [ ] Patents
- [ ] Service manual (if exists)

### Components Needed

| IC | Type | Function | Status |
|----|------|----------|--------|
| TBD | High-speed op-amp | DC-coupled video | ❌ Research needed |
| TBD | Multiplier | Deflection modulation | ❌ Research needed |

### Functional Blocks to Build
1. [ ] **Z-Axis Displacement** — Geometry-based (not UV distortion!)
2. [ ] **Horizontal Deflection Mod** — X-axis raster bend
3. [ ] **Vertical Deflection Mod** — Y-axis raster bend
4. [ ] **DC-Coupled Signal Path** — Preserve absolute levels

### Implementation Note
True Rutt/Etra requires **geometry-based rendering** — drawing displaced scan lines as line primitives, not UV-based pixel displacement. This is fundamentally different from most "Rutt/Etra" digital effects.

---

## 4. Jones Colorizer (1974)

*Dave Jones — Layered keying colorizer*

### What It Does
- Multi-level luminance keying
- Diode mixing (no active color encoder)
- Layered color assignment
- Simple, buildable design

### Documents Needed
- [ ] Schematics (Jones published openly)
- [ ] Build documentation

### Components Needed

| IC | Type | Function | Status |
|----|------|----------|--------|
| LM311 | Comparator | Level detection | ✅ Modeled |
| 1N914 | Signal Diode | Diode mixing | Simple passive |
| Op-amps | Various | Buffering | Common parts |

### Functional Blocks to Build
1. [x] **Window Comparator** — LM311 (keyer.wgsl exists)
2. [ ] **Multi-Level Keyer** — Stack of comparators
3. [ ] **Diode Mixer** — OR-style color combination
4. [ ] **Color Assignment Matrix** — Key → RGB mapping

---

## 5. LZX Cadet Series (2012)

*Lars Larsen / LZX Industries — Modern Eurorack video*

### What It Does
- Eurorack modular video synthesis
- Open-source schematics (GitHub)
- Modern implementation of classic techniques
- 1V video standard

### Documents Available
- [x] Full schematics on GitHub (lzxindustries/lzxcadet)
- [x] BOMs and build docs
- [x] Firmware (for digital modules)

### Modules to Emulate

| Module | Function | Complexity |
|--------|----------|------------|
| C1 Sync Generator | Video timing | Low |
| C2 Ramp Generator | H/V ramps | Low |
| C3 Video Processor | Gain/offset/invert | Low |
| C4 Waveshaper | Folding/clipping | Medium |
| C5 Keyer | Luma key | Low |
| C6 Multiplier | A × B | Medium |
| C7 Fader | Crossfade | Low |
| C8 Mixer | 4-input sum | Low |
| C9 Scaler | Level adjust | Low |
| C10 Output | Video encoder | Medium |

### Why Last?
LZX Cadet is well-documented and uses modern components, making it the easiest to implement. Starting with historical devices ensures we develop robust techniques that work on harder problems first.

---

## Component Library Progress

### Modeled (Ready to Use)
| Component | Type | Shaders |
|-----------|------|---------|
| CA3046 | Transistor Array | model.wgsl, colorizer.wgsl |
| CA3080 | OTA | model.wgsl, vca.wgsl |
| LM311 | Comparator | model.wgsl, keyer.wgsl |

### Priority Queue (Need Datasheets)
1. **CA3019** — Paik/Abe amplitude windows (CRITICAL)
2. **CA3054/CA3055** — Paik/Abe matrix mixing
3. **CA3022/CA3023** — Paik/Abe video amps
4. **MC1445** — Sandin IP gain blocks

### Known Sources
- [1978 RCA Linear IC Databook](https://www.smcelectronics.com/DOWNLOADS/1978-RCA%20LINEAR%20IC.PDF) — All CA-30xx parts
- [LZX Cadet GitHub](https://github.com/lzxindustries/lzxcadet) — Modern schematics + BOMs
- [Archive.org](https://archive.org) — Sandin IP documentation

---

## Milestone Definitions

### M1: Paik/Abe Core
- [ ] All CA-30xx components modeled
- [ ] Matrix mixer functional
- [ ] Colorizer with adjustable bands
- [ ] Basic wobbulator effect

### M2: Sandin IP Core
- [ ] Amplitude classifier
- [ ] Differencing amplifier
- [ ] Four-quadrant multiplier
- [ ] VCA chains

### M3: Rutt/Etra
- [ ] Geometry-based scan displacement
- [ ] Z-axis luminance mapping
- [ ] Horizontal/vertical deflection

### M4: Jones Colorizer
- [ ] Multi-level keyer stack
- [ ] Diode mixing logic
- [ ] Full colorizer patch

### M5: LZX Cadet
- [ ] All 10 Cadet modules
- [ ] Eurorack-style patching
- [ ] 1V signal standard

---

## Next Actions

1. **Download CA-30xx datasheets** from RCA Linear IC Databook
2. **Model CA3019** diode array for amplitude windows
3. **Build Paik/Abe matrix mixer** using CA3046 diff pairs
4. **Test in workbench** with real video input
5. **Document transfer functions** for each component

---

*Last updated: January 2026*
