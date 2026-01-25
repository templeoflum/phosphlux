# Techniques

Abstract methods and approaches used in analog video synthesis, independent of specific hardware implementations.

## Categories

### Signal Processing
- **Amplitude Windowing** - Detecting when signal falls within specific voltage ranges
- **Wavefolding** - Nonlinear waveshaping that "folds" signals back on themselves
- **Soft Clipping** - Gradual saturation using tanh-like transfer functions
- **Hard Clipping** - Abrupt voltage limiting

### Colorization
- **Luminance-to-Color Mapping** - Converting grayscale levels to color bands
- **Matrix Mixing** - Weighted combinations of multiple inputs
- **Keying** - Using one signal to select between others

### Modulation
- **Amplitude Modulation (AM)** - Controlling signal level with another signal
- **Frequency Modulation (FM)** - Controlling oscillator frequency with another signal
- **Ring Modulation** - Four-quadrant multiplication

### Raster Manipulation
- **Scan Displacement** - Deflecting CRT electron beam with external signals
- **Z-Axis Modulation** - Using brightness to control deflection
- **Wobbulation** - Periodic scan line displacement

### Feedback
- **Video Feedback** - Pointing camera at its own output
- **Electronic Feedback** - Signal path loops with delay/processing

---

## Technique Details

### Amplitude Windowing
Used in: Paik/Abe PED Control, Jones Colorizer

Creates color bands by detecting when luminance falls within specific voltage ranges.
Each "window" is defined by upper and lower thresholds.

```
Window 1: 0.0V - 0.2V → Color A
Window 2: 0.2V - 0.4V → Color B
Window 3: 0.4V - 0.6V → Color C
...
```

**Key circuit elements:**
- Comparators (LM311, CA3019 diodes)
- Reference voltage dividers
- OR/AND logic for window detection

### Wavefolding
Used in: Paik/Abe Folding Amplifier, Buchla 259, Serge Wave Multipliers

Creates harmonic overtones by "folding" a signal back when it exceeds thresholds.

```
Transfer function (single fold):
if (input > threshold):
    output = 2*threshold - input
else if (input < -threshold):
    output = -2*threshold - input
else:
    output = input
```

Multiple folding stages create increasingly complex waveforms.

### Scan Displacement (Wobbulation)
Used in: Paik/Abe Wobbulator, Rutt/Etra

Modulates the horizontal and/or vertical deflection of CRT scan lines.

```
displaced_x = x + h_amplitude * sin(2π * h_freq * y + time)
displaced_y = y + v_amplitude * sin(2π * v_freq * x + time)
```

**Video-driven mode:** Uses input luminance to modulate displacement amount.

---

## Research Sources

When researching a technique, look for:
1. **Original papers** - Academic publications describing the method
2. **Patents** - Often contain detailed implementation descriptions
3. **Schematics** - Show specific circuit realizations
4. **Demo videos** - Visual examples of the technique in action

## File Organization

Each technique should have:
```
techniques/
├── technique_name/
│   ├── README.md        # Overview, history, variations
│   ├── theory.md        # Mathematical foundations
│   ├── implementations/ # Links to component/device implementations
│   └── references.md    # Papers, patents, videos
```
