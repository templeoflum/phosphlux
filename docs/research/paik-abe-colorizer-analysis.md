# Paik/Abe Colorizer - Research Analysis

**Date:** 2026-01-25
**Purpose:** Validate our emulation against historical documentation

## Sources Consulted

1. [The Early Video Project - Paik/Abe Synthesizer](http://davidsonsfiles.org/PaikAbeSythesizer.html)
2. [Internet Archive - Paik-Abe Schematics](https://archive.org/details/ETC3311) (PDF not accessible via fetch)
3. [Video History Project - Colorizers Come of Age](https://www.videohistoryproject.org/colorizers-come-age)
4. [Vasulka Archive](https://www.vasulka.org/archive/eigenwelt/pdf/126-129.pdf)
5. [Fondation Langlois - Paik/Abe Documentation](https://www.fondation-langlois.org/html/e/page.php?NumPage=452)
6. Local model research queries (llama3.1)

## Historical Paik/Abe Architecture

### Input Stage
- **Seven black and white surveillance cameras** as inputs
- Each camera assigned to different color channels:
  - Camera 1 → Red
  - Camera 2 → Green
  - Camera 3 → Blue
  - Camera 4 → Red + Green
  - Camera 5 → Red + Blue
  - Camera 6 → Green + Blue (implied)
  - Camera 7 → Additional mixing

### Processing Stage
- Each video signal passes through a **nonlinear amplifier**
- Nonlinear behavior: "passes low level signals but folds over or inverts the polarity of higher level signals"
- Individual **gain controls** for each input to overdrive the nonlinear amps

### Color Generation
- Signals passed through a **matrix** into RGB
- **Phase shift on the whole system** - changes color value of each input
- RGB to NTSC color encoder with Sync and Blanking

### Additional Features
- Magnetic scan processing via extra deflection yokes
- Audio-driven modulation of scan patterns
- Video feedback loops possible

## Our Implementation

### Architecture
```
Single luminance input
    ↓
Extract luma from RGB
    ↓
┌─────────────────────────────────────┐
│ Three parallel wavefold paths:      │
│   R: luma × gain_r → triangle fold  │
│   G: luma × gain_g → triangle fold  │
│   B: luma × gain_b → triangle fold  │
└─────────────────────────────────────┘
    ↓
Op-amp saturation (LM741 model)
    ↓
Saturation mix with output luma
    ↓
Warmth bias (thermal drift)
    ↓
RGB output
```

### Key Functions

**Triangle Wave Folding:**
```wgsl
fn paik_abe_fold(x: f32, gain: f32, depth: f32) -> f32 {
    var v = x * gain;
    for stages {
        v = abs(fract(v * 0.5 + 0.25) * 2.0 - 1.0);
        v = v * 0.98 + 0.01; // asymmetry
    }
    return v;
}
```

**Op-amp Saturation:**
```wgsl
fn paik_abe_opamp_sat(v_in: f32, slew_factor: f32) -> f32 {
    let v_sat = 0.87;
    let normalized = v_in * 2.0 - 1.0;
    var saturated = tanh(normalized * 3.0) / tanh(3.0);
    // slew rate limiting
    return (saturated + 1.0) * 0.5 * v_sat + (1.0 - v_sat) * 0.5;
}
```

## Comparison: Historical vs Implementation

| Aspect | Historical Paik/Abe | Our Implementation |
|--------|---------------------|-------------------|
| **Input** | 7 camera inputs | Single RGB input |
| **Color source** | Multiple cameras assigned to R/G/B | Luminance folded 3 ways |
| **Nonlinearity** | "Folds over or inverts" at high levels | Triangle wave folding |
| **Gain control** | Per-input gain pots | Per-channel fold gain |
| **Phase shift** | System-wide color phase | Different gain offsets per channel |
| **Saturation** | Unknown op-amp type | LM741 model (tanh) |
| **Mixing** | RGB matrix | Implicit (parallel channels) |

## Assessment

### What We Got Right
1. **Nonlinear folding behavior** - "folds over or inverts" is exactly what triangle wave folding does
2. **Per-channel gain differences** - creates color variation across luminance
3. **Warm, organic output** - saturation and asymmetry simulate component tolerances
4. **Visual character** - produces similar psychedelic rainbow banding

### What's Different
1. **Single input vs multi-camera** - We process one signal; original had 7 inputs
2. **Phase shift mechanism** - We use gain offset; original had explicit phase control
3. **Op-amp model** - Local research mentioned CA3130; we modeled LM741
4. **No scan modulation** - Original had magnetic deflection; we don't emulate this

### What's Unknown
1. Exact transfer function of the "nonlinear amplifier"
2. Component values in the original circuit
3. RGB matrix mixing ratios
4. Phase shift circuit implementation
5. Whether LM741 or CA3130 or other op-amp was used

## Recommendations

### For Current Version (Artistic Emulation)
Our implementation is **valid as an artistic interpretation**. It produces the characteristic Paik/Abe "look" (rainbow banding from luminance) through a mathematically similar mechanism.

### For Accurate Historical Emulation
Would need:
1. Access to actual schematics from Internet Archive (ETC3311, ETC2370)
2. Identify exact op-amp used and model its specific characteristics
3. Implement actual phase shift mechanism instead of gain offset
4. Consider multi-input architecture for full authenticity

### Suggested Improvements
1. **Add phase control parameter** - explicit hue rotation of the color output
2. **Research the actual nonlinear amp** - may not be pure triangle fold
3. **Add scan modulation** - separate module for deflection effects
4. **Temperature simulation** - more sophisticated drift modeling

## Conclusion

Our Paik/Abe Colorizer is a **conceptually accurate** emulation that captures the key insight (nonlinear folding creates color from luminance) but uses a **simplified circuit model**. For artistic use, this is perfectly valid. For museum-quality historical accuracy, we'd need the actual schematics.

The triangle wave folding approach is commonly used in modern video synthesis (LZX, Gieskes) and produces results that are visually consistent with 1970s video art aesthetics.
