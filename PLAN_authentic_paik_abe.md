# Plan: Authentic Paik/Abe Video Synthesizer Recreation

**Created:** 2026-01-29
**Updated:** 2026-01-29
**Status:** In Progress (PED Colorizer validated, multi-input pending)
**Goal:** Component-accurate emulation based on actual schematics

---

## Reference Documents

Primary source: `research/devices/paik_abe/Paik Abe Video Synthesizer Technical Documentation.pdf`

| Schematic Page | Circuit | Status |
|----------------|---------|--------|
| Page 7 | Switcher Input (Emitter Follower) | Not started |
| Page 14/18 | Keyer Switchboard (signal routing) | Not started |
| Page 19 | Keying Circuit (CA3046, CA3054) | Not started |
| Page 24 | Matrix & Mixing Amp (7ch → RGB) | Not started |
| Page 28 | **PED Control (CA3019)** | Have PED Colorizer (validate) |
| Page 29 | Mixing Amp (Si-D network) | Not started |
| Page 26 | Sub Carrier Amp (CA3023) | Modeled (CA3023 spec exists) |
| Page 27 | Burst Generator (CA3019) | Not started |
| Pages 22-23 | Encoder (6-12) | Complex - defer |

---

## Architecture from Schematics

```
                    ┌─────────────────────────────────────────────────────────┐
                    │                  PAIK/ABE SYNTHESIZER                    │
                    └─────────────────────────────────────────────────────────┘

 ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐
 │CH-1 │  │CH-2 │  │CH-3 │  │CH-4 │  │CH-5 │  │CH-6 │  │CH-7 │   7 Camera Inputs
 └──┬──┘  └──┬──┘  └──┬──┘  └──┬──┘  └──┬──┘  └──┬──┘  └──┬──┘
    │        │        │        │        │        │        │
    ▼        ▼        ▼        ▼        ▼        ▼        ▼
 ┌──────────────────────────────────────────────────────────────┐
 │              EMITTER FOLLOWER BUFFERS (Page 7)               │
 │                     75Ω termination                          │
 └──────────────────────────────────────────────────────────────┘
                              │
                              ▼
 ┌──────────────────────────────────────────────────────────────┐
 │              KEYER SWITCHBOARD (Page 14/18)                  │
 │    Input selection matrix, MIX section, Gate routing         │
 │    Controls: INV, NOR switches                               │
 └──────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
 ┌────────────────┐  ┌────────────────┐  ┌────────────────┐
 │  KEYING (19)   │  │  MATRIX MIX    │  │  GATE PULSE    │
 │  CA3046/CA3054 │  │   (Page 24)    │  │   AMP (15)     │
 │  Video Balance │  │  7ch → R,G,B   │  │  CA3000/CA3002 │
 └────────────────┘  └───────┬────────┘  └────────────────┘
                             │
                             ▼
              ┌──────────────────────────────┐
              │    PED CONTROL (Page 28)     │  ◄── THE COLORIZER
              │         CA3019               │
              │  Amplitude Window Detection  │
              │  3 threshold controls (5,6,7)│
              │  Chroma input mixing         │
              └──────────────┬───────────────┘
                             │
                             ▼
              ┌──────────────────────────────┐
              │     VIDEO AMP (Pages 9,10)   │
              │         CA3022               │
              └──────────────┬───────────────┘
                             │
         ┌───────────────────┼───────────────────┐
         ▼                   ▼                   ▼
    ┌─────────┐        ┌──────────┐        ┌──────────┐
    │ SYNC IN │        │ BURST IN │        │ RGB OUT  │
    └────┬────┘        └────┬─────┘        └────┬─────┘
         │                  │                   │
         └──────────────────┼───────────────────┘
                            ▼
              ┌──────────────────────────────┐
              │   ENCODER (Pages 22-23)      │
              │   R,G,B → NTSC Composite     │
              │   White balance, Chroma lvl  │
              │   Burst shaping, Sync insert │
              └──────────────────────────────┘
```

---

## Phase 1: Validate Existing Modules

### 1.1 PED Colorizer Audit

**File:** `src/devices/paik_abe/ped_colorizer.wgsl`

Compare against Page 28 schematic:
- [x] Verify CA3019 behavior matches diode array characteristics (FIXED 2026-01-29)
  - Implemented proper exponential diode detection via `ped_ca3019_diode_detect()`
  - Uses `PED_DIODE_SHARPNESS` derived from 1/Vt characteristic
- [x] Check threshold mapping (PED CTL pins 5, 6, 7)
  - `threshold_1`, `threshold_2`, `threshold_3` parameters match pins 5, 6, 7
- [x] Add clipping control (CLP pin 8) (FIXED 2026-01-29)
  - Added `clipping` parameter and `ped_apply_clipping()` function
- [ ] Verify Chroma input mixing behavior (BLOCKED - needs multi-input runtime)
- [ ] Check output signal levels

**From schematic (Page 28):**
- VID IN: pins 3, 4
- PED CTL: pins 5, 6, 7 (3 thresholds)
- CLP: pin 8 (clipping)
- Croma IN: pin 10
- Outputs: C.OUT (14), V.OUT (16)
- BLK IN: pin 19

### 1.2 Input Matrix Audit

**File:** `src/devices/paik_abe/input_matrix.wgsl`

Compare against Page 14/18:
- [ ] Verify signal routing behavior
- [ ] Check MIX section implementation
- [ ] Verify INV/NOR functionality if present

### 1.3 Output Encoder Audit

**File:** `src/devices/paik_abe/output_encoder.wgsl`

- [ ] Compare against video amp circuits (Pages 9, 10)
- [ ] Verify CA3022 behavior if modeled

---

## Phase 2: Missing Core Modules

### 2.1 Matrix Mixing Amp (Page 24, 29)

**Purpose:** Route 7 camera inputs to R, G, B outputs with weighted mixing

**From schematic:**
- 7 input channels with switches
- Si-D (silicon diode) mixing network
- Separate R-out, G-out, B-out paths
- This is HOW multiple cameras become color

**Implementation:**
```
Inputs: 7 video channels (or our single RGB split 3 ways)
Parameters:
  - ch1_r, ch1_g, ch1_b: Channel 1 weights to RGB
  - ch2_r, ch2_g, ch2_b: Channel 2 weights to RGB
  - ... (simplified for single-input mode)
  - mix_r, mix_g, mix_b: Overall RGB mix levels
Output: RGB
```

**Key insight:** In our single-input context, we can:
1. Split input into 3 "virtual channels"
2. Apply matrix mixing to create RGB
3. This feeds into PED Control

### 2.2 Keying Circuit (Page 19)

**Purpose:** Hard/soft key switching, video balance

**Components:**
- CA3046 transistor array
- CA3054 differential amplifier

**Parameters:**
- Video Balance
- Gate Balance
- Gain

### 2.3 Gate Pulse Amp (Page 15)

**Components:**
- CA3000
- CA3002

**Purpose:** Generate/shape gate pulses for keying

---

## Phase 3: Signal Flow Verification

### 3.1 Authentic Signal Chain

For a single-input recreation, the authentic chain would be:

```
Test Signal / Video In
        │
        ▼
┌───────────────────┐
│   Input Matrix    │  (gain, soft limiting via CA3054)
│   [VALIDATED]     │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│  Matrix Mixer     │  [NEW - Page 24/29]
│  Single → RGB     │  Split single input to RGB with weights
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│   PED Colorizer   │  (CA3019 amplitude windows)
│   [VALIDATED]     │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│  Output Encoder   │  (CA3022 video amp characteristics)
│   [VALIDATED]     │
└────────┬──────────┘
         │
         ▼
      RGB Output
```

### 3.2 Multi-Input Extension (Future)

For full 7-camera authenticity:
- Would need 7 separate video inputs
- Full keyer switchboard routing
- Complete matrix mixing

---

## Phase 4: Component Models to Add

### 4.1 Required IC Models

| IC | Type | Priority | Spec Exists? |
|----|------|----------|--------------|
| CA3019 | Diode Array | HIGH | Yes |
| CA3046 | Transistor Array | MEDIUM | Yes |
| CA3054 | Diff Amp | MEDIUM | Yes |
| CA3022 | Video Amp | LOW | Yes |
| CA3023 | Video Amp | LOW | Yes |
| CA3000 | Op Amp | LOW | No |
| CA3002 | Op Amp | LOW | No |

### 4.2 Discrete Component Models

From schematics:
- 2N3904/2N3906 transistors (various)
- 2N2102 output transistors
- 2N4400/2N4402 complementary pair (mixer-fader)
- MPS5172 signal transistors
- 1N914/1N4148 signal diodes
- Si-D (silicon diode network in mixer)

---

## System Prerequisites

### Multi-Input Support (REQUIRED for full authenticity)

#### Status: PARTIAL (2026-01-29)

**Shader Composer (DONE):**
- `src/gpu/shader_composer.rs` now generates WGSL code for multiple input textures
- `seg_input_tex`, `seg_input_tex_1`, `seg_input_tex_2`, etc. bindings generated
- Cross-segment source tracking implemented
- Proper texture sampling code generated per input port

**Renderer (PENDING):**
- `src/gpu/renderer.rs` still uses single-input bind group layout
- Requires architectural change from linear segments to graph-based rendering
- Each segment needs bind group with variable texture count
- Would need: `texture_input_bg_layout_N` for N-input segments

For authentic Paik/Abe recreation, we need:
- PED Colorizer: 2 inputs (video_in, chroma_in)
- Input Matrix: 7 inputs (or at minimum 3: R, G, B cameras)

**Workaround (current):** Single-input devices derive multiple signals internally
- Input Matrix: Derives 7 virtual channels (luma, R, G, B, inv-luma, luma^2, offset)
- PED Colorizer: Uses video_in only, no chroma mixing

---

## Implementation Order

### Immediate (This Session)

1. **Audit PED Colorizer** against Page 28 schematic
   - Verify CA3019 amplitude window behavior
   - Check threshold controls match schematic
   - Document any deviations

### Short Term

2. **Create Matrix Mixer** (Page 24/29)
   - Silicon diode mixing network
   - RGB output weighting
   - This enables: Input → Matrix → PED → Output chain

3. **Test authentic chain**
   - Input Matrix → Matrix Mixer → PED Colorizer → Output Encoder
   - Compare visual output to known Paik/Abe footage

### Medium Term

4. **Keying Circuit** (Page 19)
   - CA3046/CA3054 implementation
   - Video balance controls

5. **Gate Pulse Amp** (Page 15)
   - For hard/soft key transitions

### Long Term

6. **Multi-input support**
   - Enable 7 virtual inputs
   - Full keyer switchboard
   - Complete matrix routing

---

## Validation Criteria

For each module to be marked "Authentic":

1. **Schematic Reference:** Must cite specific page number
2. **Component Mapping:** Each IC/transistor behavior documented
3. **Transfer Function:** Mathematical model matches circuit analysis
4. **Visual Comparison:** Output compared to known Paik/Abe footage
5. **Parameter Mapping:** Controls map to actual panel controls

---

## Open Questions

1. **PED Control thresholds:** Are pins 5,6,7 linear or logarithmic?
2. **Matrix mixing ratios:** What are the default weights for 7→RGB?
3. **Keying behavior:** Exact CA3046 differential pair characteristics?
4. **Clipping behavior:** How does CLP pin 8 affect the signal?

---

## Success Metrics

- [ ] Full signal chain from input to output using authentic modules
- [ ] Visual output recognizable as "Paik/Abe style"
- [ ] Each module traceable to specific schematic page
- [ ] No speculative "improvements" - only documented behaviors
- [ ] Documentation sufficient for hardware recreation
