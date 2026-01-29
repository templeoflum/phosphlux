# Paik/Abe Device Audit

**Date:** 2026-01-29
**Auditor:** Claude
**Reference:** Paik Abe Video Synthesizer Technical Documentation.pdf (Pages 1-30)

---

## Summary

| Device | Schematic Ref | Inputs | Authentic? | Action |
|--------|---------------|--------|------------|--------|
| Folding Amp | NONE | 1 | **NO** | Reclassify as prototype |
| Input Matrix | Page 14/18/24 | 1 (needs 7) | **PARTIAL** | Needs multi-input |
| PED Colorizer | Page 28 | 1 (needs 2) | **PARTIAL** | Missing chroma input |
| Output Encoder | Page 22-23 | 1 | **PARTIAL** | Uses wrong IC model |
| Colorizer (legacy) | NONE | 1 | **NO** | Already reclassified |

---

## 1. Folding Amplifier

### Current Implementation
- File: `src/devices/paik_abe/folding_amp.wgsl`
- Triangle wave folding with gain/depth/bias controls
- Claims to be "heart of Paik/Abe color generation"

### Schematic Evidence
**NONE.** There is no "folding amplifier" circuit in the Paik/Abe schematics.

The schematics show:
- Page 28: PED Control (amplitude windows via CA3019)
- Page 24: Matrix Mixing Amp (weighted summing)
- Page 19: Keying Circuit (CA3046/CA3054)

### Verdict: **PROTOTYPE - NOT AUTHENTIC**

### Recommendation
Reclassify as prototype alongside the legacy colorizer. The "folding" effect is creative but not based on actual Paik/Abe circuits.

---

## 2. Input Matrix

### Current Implementation
- File: `src/devices/paik_abe/input_matrix.wgsl`
- **ONE** video input, derives 7 "virtual channels" from it
- CA3054-style soft limiting

### Schematic Evidence
**Page 14/18 (Keyer Switchboard):** Shows 7 actual camera inputs going through a switching matrix.

**Page 24 (Matrix & Mixing Amp):** Shows 7 channels (CH-1 through CH-7) with:
- Individual switches per channel
- Routing to R-out, G-out, B-out
- This is how 7 B&W cameras become color

### Current Problems
1. **Only 1 input** - Real hardware has 7
2. **"Virtual channels"** - Creative workaround, not authentic
3. **Weight matrix** - Values are speculative, not from schematic

### Verdict: **PARTIAL - NEEDS MULTI-INPUT**

### Recommendation
- Keep current as "simplified mode" for single-input use
- Add proper 7-input version for authentic recreation
- Or add 3-input version as minimum (R, G, B cameras)

---

## 3. PED Colorizer

### Current Implementation
- File: `src/devices/paik_abe/ped_colorizer.wgsl`
- CA3019-based amplitude window detection
- 3 thresholds (matches PED CTL pins 5, 6, 7)
- 4 color windows: blue, magenta, yellow, cyan
- Softness parameter for edge transitions

### Schematic Evidence (Page 28 - PED.CONT.)

**Inputs documented:**
| Pin | Signal | Our Implementation |
|-----|--------|-------------------|
| 3,4 | VID IN | YES (video_in) |
| 5,6,7 | PED CTL | YES (threshold_1/2/3) |
| 8 | CLP | **MISSING** (clipping control) |
| 10 | Croma IN | **MISSING** (chroma input!) |
| 19 | BLK IN | **MISSING** (blanking) |

**Outputs documented:**
| Pin | Signal | Our Implementation |
|-----|--------|-------------------|
| 14 | C.OUT | Not explicit |
| 16 | V.OUT | Not explicit |

### Current Problems
1. **Missing Chroma Input (pin 10)** - The real PED Control can mix in chroma signal!
2. **Missing Clipping Control (pin 8)** - Affects signal limiting
3. **Missing Blanking (pin 19)** - Sync handling
4. **Color palette** - Blue/magenta/yellow/cyan is plausible but not verified against original

### What's Correct
- CA3019 diode array for amplitude detection ✓
- 3 threshold controls matching PED CTL ✓
- Soft edges modeling diode behavior ✓
- Basic 4-window architecture ✓

### Verdict: **PARTIAL - MOSTLY CORRECT**

### Recommendation
1. Add second input for Chroma (video_in, chroma_in)
2. Add CLP (clipping) parameter
3. Document that exact color palette is unverified
4. Consider adding output selection (C.OUT vs V.OUT)

---

## 4. Output Encoder

### Current Implementation
- File: `src/devices/paik_abe/output_encoder.wgsl`
- Claims "LM741 op-amp saturation"
- Saturation, slew, warmth parameters

### Schematic Evidence (Pages 22-23, 9-10)

**ICs Actually Used:**
- CA3022 - Video amplifier (Page 10, 20)
- CA3023 - Subcarrier amplifier (Page 26, 30)
- Various 2SC transistors
- **NO LM741**

### Current Problems
1. **Wrong IC model** - Uses LM741 characteristics, should use CA3022
2. **Parameters** - "Slew (741 Char)" explicitly wrong
3. **Missing components** - Real encoder is much more complex

### Verdict: **PARTIAL - WRONG IC MODEL**

### Recommendation
1. Replace LM741 model with CA3022 characteristics
2. Rename parameters to match actual circuit
3. For full authenticity, would need much more complex encoder

---

## 5. Module Header Documentation Issues

### mod.rs Header (Lines 1-12)
```rust
//! Key characteristics:
//! - Built around LM741 op-amps (slow slew rate = character)  // WRONG
//! - Triangle wave folding for colorization                    // WRONG
```

**NEEDS UPDATE** - These claims are not supported by schematics.

---

## Priority Actions

### Immediate
1. Reclassify Folding Amp as prototype
2. Update mod.rs documentation to remove false claims
3. Add audit notes to device_validation.json

### Short Term
1. Add chroma input to PED Colorizer (makes it 2-input device)
2. Update Output Encoder to use CA3022 model (we have the spec)
3. Create Input Matrix variant with at least 3 inputs (R, G, B)

### Medium Term
1. Full 7-input matrix mixer
2. Keying circuit (CA3046/CA3054)
3. Gate pulse amplifier

---

## Appendix: Schematic Page Reference

| Page | Circuit | Notes |
|------|---------|-------|
| 3 | 6-12 Encoder (partial) | Complex encoder board |
| 5 | Control Panel | Shows T-GAIN, F-GAIN, PED-CTL controls |
| 6 | Mixer-Fader Output Amp | 2N4400/2N4402 |
| 7 | Switcher Input E.F. | Emitter follower buffers |
| 10 | Video Out Amp for Chroma Key | CA3022 |
| 12 | Power Supply | CA-3055 regulator |
| 14/18 | Keyer Switchboard | 7 inputs, switching matrix |
| 15 | Gate Pulse Amp | CA3000, CA3002 |
| 19 | Keying | CA3046, CA-3054 |
| 22-23 | Encoder (6-12) | Full encoder schematic |
| 24 | Matrix & Mixing Amp | 7ch→RGB, Si-D network |
| 26 | Sub Carrier Amp | CA3023 |
| 27 | Burst Generator | CA3019 |
| 28 | **PED Control** | CA3019 - THE colorizer |
| 29 | Mixing Amp | Si-D diode mixer |
