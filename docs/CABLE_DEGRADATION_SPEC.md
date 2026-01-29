# Cable Degradation Specification v0.13

**Version:** 0.13
**Created:** 2026-01-28
**Purpose:** Define accurate cable degradation models based on actual NTSC specifications

---

## 1. Source References

This specification is derived from the following sources:

| Source | Used For |
|--------|----------|
| [NTSC Wikipedia](https://en.wikipedia.org/wiki/NTSC) | Subcarrier frequency, signal structure |
| [YIQ Wikipedia](https://en.wikipedia.org/wiki/YIQ) | Color space conversion matrices |
| [Dot Crawl Wikipedia](https://en.wikipedia.org/wiki/Dot_crawl) | Artifact characteristics |
| [S-Video Wikipedia](https://en.wikipedia.org/wiki/S-Video) | Y/C separation benefits |
| [RG-59 Wikipedia](https://en.wikipedia.org/wiki/RG-59) | Cable attenuation values |
| [MATLAB rgb2ntsc](https://www.mathworks.com/help/images/ref/rgb2ntsc.html) | YIQ matrix coefficients |
| `docs/VIDEO_SYNTHESIS_TECHNICAL_REFERENCE.md` | Local project reference |

---

## 2. NTSC Fundamentals

### 2.1 Color Subcarrier

- **Frequency:** 3.579545 MHz (exactly 315/88 MHz, ±10 Hz)
- **Purpose:** Carries chrominance via quadrature amplitude modulation (QAM)
- **Backward compatibility:** Monochrome receivers ignore the subcarrier

### 2.2 YIQ Color Space

**RGB to YIQ (1953 NTSC):**
```
Y = 0.299·R + 0.587·G + 0.114·B
I = 0.596·R - 0.274·G - 0.322·B
Q = 0.211·R - 0.523·G + 0.312·B
```

**YIQ to RGB:**
```
R = Y + 0.956·I + 0.621·Q
G = Y - 0.272·I - 0.647·Q
B = Y - 1.106·I + 1.703·Q
```

**Valid ranges:**
- Y: [0, 1]
- I: [-0.5957, +0.5957]
- Q: [-0.5226, +0.5226]

### 2.3 Bandwidth Specifications

| Channel | Bandwidth | Notes |
|---------|-----------|-------|
| Luminance (Y) | 4.2 MHz | Full baseband video bandwidth |
| I (orange-cyan axis) | 1.3 MHz | Vestigial sideband: 0.5 MHz DSB + 0.8 MHz SSB |
| Q (green-magenta axis) | 0.4 MHz | Double sideband only |

**Practical receivers:** Many consumer devices limit both I and Q to ~0.5 MHz (equiband mode).

---

## 3. Cable Type Specifications

### 3.1 Composite Video (CVBS)

**Signal architecture:** Single cable carries combined Y + C (modulated at 3.58 MHz)

**Characteristics:**
- Luma and chroma share same conductor
- Inevitable Y/C crosstalk due to frequency overlap
- Dot crawl artifact at color transitions
- Rainbow artifact on high-frequency luma patterns

**Bandwidth (effective):**
- Luminance: ~4.2 MHz (but contaminated by chroma crosstalk above ~2 MHz)
- Chroma I: 1.3 MHz (spec), ~0.5-1.0 MHz (consumer equipment)
- Chroma Q: 0.4 MHz (spec), ~0.3-0.5 MHz (consumer equipment)

**Artifacts:**
- **Dot crawl:** Moving checkerboard pattern at horizontal color transitions
  - Frequency: 3.579545 MHz (subcarrier)
  - Pattern: Alternates polarity every frame
  - Appears as "crawling dots" moving up or down edges
- **Rainbow:** False color on high-frequency luma (fine diagonal lines)
- **Color bleeding:** Chroma spreads into adjacent areas due to low bandwidth

**Transfer function (simplified model):**
```
// Input: linear RGB [0,1]
// Output: degraded RGB with composite artifacts

Y = 0.299·R + 0.587·G + 0.114·B
I = 0.596·R - 0.274·G - 0.322·B
Q = 0.211·R - 0.523·G + 0.312·B

// Bandwidth limiting (spatial blur in shader)
// I channel: ~1.3 MHz → blur radius ≈ 0.002 normalized (at 720px)
// Q channel: ~0.4 MHz → blur radius ≈ 0.006 normalized

I_limited = lowpass(I, radius_I)
Q_limited = lowpass(Q, radius_Q)

// Dot crawl injection
phase = uv.x × 227.5 + time × 59.94  // 227.5 = 3.58MHz / 15.734kHz line rate
dot_crawl = sin(phase × 2π) × dot_crawl_amplitude

// Amplitude scales with chroma magnitude (only visible at Y/C boundaries)
dot_crawl_amplitude = 0.03 × sqrt(I² + Q²)

// Reconstruct RGB
R = Y + dot_crawl + 0.956·I_limited + 0.621·Q_limited
G = Y + dot_crawl - 0.272·I_limited - 0.647·Q_limited
B = Y + dot_crawl - 1.106·I_limited + 1.703·Q_limited
```

**Expected visual characteristics (colorbars test):**
- Visible softening/bleeding on vertical color transitions
- Crawling dot pattern on saturated color edges
- Overall slightly softer than S-Video

**Acceptance criteria:**
- [ ] Chroma visibly bleeds beyond luma edges
- [ ] Dot crawl pattern visible at high zoom
- [ ] Color bars show transition smearing

---

### 3.2 S-Video (Y/C)

**Signal architecture:** Two cables - Y (luminance + sync) and C (chrominance)

**Characteristics:**
- No Y/C crosstalk (signals physically separated)
- No dot crawl artifact
- Still has chroma bandwidth limitation
- Significantly cleaner than composite

**Bandwidth:**
- Luminance: ~5-7 MHz (no crosstalk contamination)
- Chroma: ~1.5 MHz (both channels)

**Transfer function:**
```
Y = 0.299·R + 0.587·G + 0.114·B
I = 0.596·R - 0.274·G - 0.322·B
Q = 0.211·R - 0.523·G + 0.312·B

// Chroma bandwidth limiting only (luma preserved)
// I channel: ~1.5 MHz → blur radius ≈ 0.0015 normalized
// Q channel: ~1.5 MHz → blur radius ≈ 0.0015 normalized (equiband)

I_limited = lowpass(I, radius_chroma)
Q_limited = lowpass(Q, radius_chroma)

// NO dot crawl - clean Y/C separation

// Reconstruct RGB
R = Y + 0.956·I_limited + 0.621·Q_limited
G = Y - 0.272·I_limited - 0.647·Q_limited
B = Y - 1.106·I_limited + 1.703·Q_limited
```

**Expected visual characteristics:**
- Cleaner than composite - no crawling artifacts
- Still shows slight chroma blur on sharp edges
- Luma detail preserved

**Acceptance criteria:**
- [ ] No visible dot crawl
- [ ] Slight chroma softening on color edges
- [ ] Sharper than composite overall

---

### 3.3 Component Video (YPbPr)

**Signal architecture:** Three cables - Y, Pb (B-Y), Pr (R-Y)

**Characteristics:**
- Wide bandwidth per channel
- No color subcarrier encoding (no QAM artifacts)
- Supports HD resolutions (480p, 720p, 1080i)
- Minimal crosstalk

**Bandwidth:**
- Y: ~18 MHz (unfiltered), ~6 MHz (typical SD equipment)
- Pb: ~4-6 MHz
- Pr: ~4-6 MHz

**Transfer function:**
```
// YPbPr uses different matrix than YIQ
Y  = 0.299·R + 0.587·G + 0.114·B
Pb = -0.169·R - 0.331·G + 0.500·B  // (B-Y) scaled
Pr = 0.500·R - 0.419·G - 0.081·B   // (R-Y) scaled

// Cable-length dependent HF rolloff
// At SD frequencies, minimal degradation
// Model as very slight lowpass on all channels

bandwidth_factor = 1.0 - cable_length_ft × 0.001  // Minimal loss
Y_out = lowpass(Y, radius_component)   // radius_component ≈ 0.0005
Pb_out = lowpass(Pb, radius_component)
Pr_out = lowpass(Pr, radius_component)

// Reconstruct RGB
R = Y_out + 1.402·Pr_out
G = Y_out - 0.344·Pb_out - 0.714·Pr_out
B = Y_out + 1.772·Pb_out
```

**Expected visual characteristics:**
- Nearly transparent
- Minimal visible degradation
- Only long cables show slight softening

**Acceptance criteria:**
- [ ] Nearly indistinguishable from source on short cables
- [ ] Slight softening only visible on fine patterns with long cables

---

### 3.4 RGB (Direct)

**Signal architecture:** Three cables for R, G, B (plus sync)

**Characteristics:**
- Maximum bandwidth (no color space conversion losses)
- No chroma subcarrier, no QAM
- Best possible analog quality
- Cable length affects HF rolloff equally on all channels

**Bandwidth:**
- Per-channel: Limited only by cable characteristics
- RG-59 at 5 MHz: ~0.15 dB/100ft attenuation
- Effective bandwidth: ~10-20 MHz depending on cable quality

**Cable attenuation (RG-59):**
- 1 MHz: 0.6 dB/100ft
- 5 MHz: ~1.0 dB/100ft
- 10 MHz: 1.1 dB/100ft
- 50 MHz: 2.4 dB/100ft

**Transfer function:**
```
// Simple HF rolloff based on cable length
// Model as single-pole lowpass

rolloff_freq = base_freq / (1.0 + cable_length_ft × 0.005)

// Apply identical lowpass to R, G, B
// radius_rgb ≈ 0.0003 × cable_length_ft for visible effect

R_out = lowpass(R, radius_rgb)
G_out = lowpass(G, radius_rgb)
B_out = lowpass(B, radius_rgb)

// Optional: slight crosstalk on very long cables (>50ft)
// crosstalk ≈ -40dB = 0.01 amplitude
```

**Expected visual characteristics:**
- Cleanest of all cable types
- Only long cables show visible softening
- No color artifacts

**Acceptance criteria:**
- [ ] Cleanest output of all cable types
- [ ] Degradation only visible with very long cable settings
- [ ] No color-specific artifacts

---

## 4. Implementation Constants

### 4.1 YIQ Conversion (use these exact values)

```wgsl
// RGB to YIQ
const RGB_TO_Y: vec3<f32> = vec3<f32>(0.299, 0.587, 0.114);
const RGB_TO_I: vec3<f32> = vec3<f32>(0.596, -0.274, -0.322);
const RGB_TO_Q: vec3<f32> = vec3<f32>(0.211, -0.523, 0.312);

// YIQ to RGB
const YIQ_TO_R: vec3<f32> = vec3<f32>(1.0, 0.956, 0.621);
const YIQ_TO_G: vec3<f32> = vec3<f32>(1.0, -0.272, -0.647);
const YIQ_TO_B: vec3<f32> = vec3<f32>(1.0, -1.106, 1.703);
```

### 4.2 YPbPr Conversion

```wgsl
// RGB to YPbPr
const RGB_TO_Y_COMPONENT: vec3<f32> = vec3<f32>(0.299, 0.587, 0.114);
const RGB_TO_PB: vec3<f32> = vec3<f32>(-0.169, -0.331, 0.500);
const RGB_TO_PR: vec3<f32> = vec3<f32>(0.500, -0.419, -0.081);

// YPbPr to RGB
const YPBPR_TO_R: vec3<f32> = vec3<f32>(1.0, 0.0, 1.402);
const YPBPR_TO_G: vec3<f32> = vec3<f32>(1.0, -0.344, -0.714);
const YPBPR_TO_B: vec3<f32> = vec3<f32>(1.0, 1.772, 0.0);
```

### 4.3 Bandwidth Parameters

```wgsl
// Composite - based on 720px horizontal resolution
const COMPOSITE_BLUR_I: f32 = 0.003;      // I channel: 1.3 MHz
const COMPOSITE_BLUR_Q: f32 = 0.008;      // Q channel: 0.4 MHz
const COMPOSITE_DOT_CRAWL_AMP: f32 = 0.03;
const COMPOSITE_DOT_CRAWL_FREQ: f32 = 227.5;  // cycles per line (3.58MHz / 15.734kHz)

// S-Video
const SVIDEO_BLUR_CHROMA: f32 = 0.002;    // 1.5 MHz chroma bandwidth

// Component
const COMPONENT_BLUR_BASE: f32 = 0.0005;  // Minimal blur
const COMPONENT_LENGTH_FACTOR: f32 = 0.00002;  // Per-foot additional blur

// RGB
const RGB_BLUR_BASE: f32 = 0.0002;        // Minimal base blur
const RGB_LENGTH_FACTOR: f32 = 0.00003;   // Per-foot additional blur
```

### 4.4 NTSC Timing Constants

```wgsl
const NTSC_SUBCARRIER_HZ: f32 = 3579545.0;  // 3.579545 MHz
const NTSC_LINE_RATE_HZ: f32 = 15734.0;     // 15.734 kHz
const NTSC_FIELD_RATE_HZ: f32 = 59.94;      // 59.94 Hz
const NTSC_CYCLES_PER_LINE: f32 = 227.5;    // Subcarrier cycles per line
```

---

## 5. Test Signals and Expected Results

### 5.1 Colorbars Test

**Signal:** SMPTE color bars (white, yellow, cyan, green, magenta, red, blue, black)

| Cable Type | Expected Visual Result |
|------------|----------------------|
| Composite | Visible bleeding between bars, dot crawl on edges |
| S-Video | Slight softening on edges, no dot crawl |
| Component | Nearly clean, minimal visible difference |
| RGB | Cleanest, virtually identical to source |

### 5.2 Sweep Test

**Signal:** Horizontal frequency sweep (low to high)

| Cable Type | Expected Visual Result |
|------------|----------------------|
| Composite | HF rolloff in chroma, pattern visible in luma |
| S-Video | HF rolloff in chroma only |
| Component | Minimal HF rolloff |
| RGB | Least HF rolloff |

### 5.3 Checker Test

**Signal:** Checkerboard pattern (high frequency)

| Cable Type | Expected Visual Result |
|------------|----------------------|
| Composite | Color fringing, possible rainbow artifact |
| S-Video | Clean luma, slight chroma blur |
| Component | Clean |
| RGB | Cleanest |

---

## 6. Verification Checklist

Before marking v0.13 complete, verify each cable type:

### Composite
- [ ] Code exists: `grep 'fn composite_degrade' src/shaders/cables.wgsl`
- [ ] Uses correct YIQ matrices (from Section 4.1)
- [ ] Applies I channel blur (~0.003)
- [ ] Applies Q channel blur (~0.008, wider than I)
- [ ] Dot crawl at 227.5 cycles/line
- [ ] Workbench colorbars shows visible chroma bleeding
- [ ] Screenshot captured: `output/composite_colorbars_verified.png`

### S-Video
- [ ] Code exists: `grep 'fn svideo_degrade' src/shaders/cables.wgsl`
- [ ] NO dot crawl (Y/C separated)
- [ ] Chroma blur only (~0.002)
- [ ] Luma passes through unmodified
- [ ] Workbench colorbars shows cleaner than composite
- [ ] Screenshot captured: `output/svideo_colorbars_verified.png`

### Component
- [ ] Code exists: `grep 'fn component_degrade' src/shaders/cables.wgsl`
- [ ] Uses YPbPr (not YIQ)
- [ ] Cable-length dependent rolloff
- [ ] Minimal visible degradation at default length
- [ ] Screenshot captured: `output/component_colorbars_verified.png`

### RGB
- [ ] Code exists: `grep 'fn rgb_degrade' src/shaders/cables.wgsl`
- [ ] No color space conversion (direct RGB processing)
- [ ] Cable-length dependent HF rolloff
- [ ] Cleanest of all cable types
- [ ] Screenshot captured: `output/rgb_colorbars_verified.png`

---

## Document History

| Date | Version | Changes |
|------|---------|---------|
| 2026-01-28 | 1.0 | Initial specification based on NTSC research |
