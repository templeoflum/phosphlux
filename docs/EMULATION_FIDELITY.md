# Emulation Fidelity Standards

Defines what "faithful emulation" means for Phosphlux. Every physical quantity becomes an exposed parameter for experimentation.

---

## Fidelity Tiers

All six tiers are required for a device to be considered "complete."

| Tier | Domain | What It Models | Examples |
|------|--------|----------------|----------|
| 1 | IC Behavioral Models | Transfer function of every IC on the schematic | CA3019 diode windows, CA3054 diff amp tanh curve, CA3080 OTA transconductance |
| 2 | Passive Network Effects | Bandwidth limiting, impedance loading, RC filtering per stage | 6 MHz video bandwidth rolloff, coupling cap high-pass, termination loading |
| 3 | Nonlinear Saturation | Soft clipping and slew rate at every gain stage | `tanh()` saturation, dV/dt limiting, supply rail compression |
| 4 | Noise Floor | Thermal (resistors), shot (junctions), 1/f (flicker) | Per-pixel gaussian noise scaled by signal level and temperature |
| 5 | Component Tolerance | +/-5% resistors, +/-20% caps, aging drift | Randomized parameter offsets per synthesizer "unit" |
| 6 | Signal Path Crosstalk | Adjacent channel bleed, ground coupling | Subtle leakage between matrix mixer channels, power supply ripple |

### Tier Details

**Tier 1: IC Behavioral Models** are the foundation. Each IC gets a `model.wgsl` with pure transfer functions derived from datasheet curves. No shader boilerplate - just math.

**Tier 2: Passive Network Effects** model the RC networks between stages. A video amplifier's 6 MHz bandwidth means high-frequency detail is attenuated. A coupling capacitor blocks DC. These are as important as the ICs themselves.

**Tier 3: Nonlinear Saturation** captures what happens when signals exceed the linear operating range. Every gain stage has a `tanh()` or polynomial soft clip. Slew rate limiting smooths fast transitions.

**Tier 4: Noise Floor** adds the analog character that distinguishes hardware from digital. Thermal noise from resistors (Johnson-Nyquist), shot noise from semiconductor junctions, and 1/f flicker noise at low frequencies.

**Tier 5: Component Tolerance** means no two synthesizer "units" sound or look identical. Resistors vary +/-5%, capacitors +/-20%, and aging shifts everything further. This is a feature, not a bug.

**Tier 6: Signal Path Crosstalk** models the unintended coupling between channels. Adjacent traces on a PCB bleed signal. Shared ground returns create common-mode interference. These artifacts contribute to the device's character.

---

## Exposed Parameter Philosophy

> Every physical quantity that would change if you swapped a component on the PCB must be an exposed parameter.

### Component-to-Parameter Mapping

| Physical Component | Parameter Effect | Typical Range |
|--------------------|-----------------|---------------|
| Resistor value | Gain, bias point, threshold | +/-5% tolerance |
| Capacitor value | Bandwidth, time constant, coupling | +/-20% tolerance |
| Supply voltage | Headroom, saturation point, slew rate | +/-10% regulation |
| Temperature | Drift, noise amplitude, Vbe shift | -2mV/C for silicon |
| Component age | Tolerance spread, leakage increase | Gradual over decades |
| PCB layout | Crosstalk, parasitic capacitance | Geometry-dependent |

### Implementation Rule

When modeling an IC or functional block:
1. Identify every external component that sets a parameter (feedback resistors, bias networks, filter caps)
2. Express that parameter as a normalized control (0.0 to 1.0 for workbench compatibility)
3. Document the mapping from normalized value to physical quantity in the shader header comment
4. Provide a sensible default that matches the original Paik/Abe circuit values

---

## Per-IC Modeling Checklist

Every IC in the component library must have:

- [ ] `spec.json` - Full electrical parameters from datasheet (voltage limits, gain, bandwidth, pinout, transfer function)
- [ ] `model.wgsl` - Core transfer functions as pure functions (no shader boilerplate, no textures, no vertex/fragment)
- [ ] Application `.wgsl` - Workbench-format shader demonstrating the IC in its Paik/Abe context (full vertex/fragment, Params struct, texture bindings)
- [ ] Header comment documenting parameter mapping (which `Params` field maps to which physical control)
- [ ] `provenance.json` - Version metadata and source artifact references
- [ ] Workbench screenshot verifying expected visual behavior
- [ ] At least one parameter sweep demonstrating smooth continuous response

### Completed ICs

| IC | spec.json | model.wgsl | App .wgsl | Provenance | Screenshot | Sweep |
|----|-----------|------------|-----------|------------|------------|-------|
| CA3019 | Y | Y | colorizer, ped_control | Y | Y | Y |
| CA3046 | Y | Y | colorizer | Y | Y | Y |
| CA3054 | Y | Y | matrix_mixer, video_keyer | Y | Y | Y |
| CA3080 | Y | Y | vca | Y | Y | Y |
| LM311 | Y | Y | keyer | Y | Y | Y |
| CA3022 | Y | Y | video_amp | Y | - | - |
| CA3023 | Y | Y | subcarrier_amp | Y | - | - |

---

## Workbench Uniform Convention

All workbench shaders use a standardized 16-field `Params` struct. The first 8 fields are control parameters; the last 8 are padding for GPU alignment:

```wgsl
struct Params {
    gain: f32,       // Primary amplitude / gain control
    offset: f32,     // DC offset / bias point
    mix: f32,        // Wet/dry blend or secondary control
    fold: f32,       // Nonlinearity / distortion amount
    bias: f32,       // Operating point / secondary bias
    frequency: f32,  // Rate / frequency control
    phase: f32,      // Phase / angle control
    threshold: f32,  // Threshold / clip level
    _pad0: f32, _pad1: f32, _pad2: f32, _pad3: f32,
    _pad4: f32, _pad5: f32, _pad6: f32, _pad7: f32,
}
```

Each application shader documents its specific mapping from these generic names to physical controls in its header comment block.

---

## Signal Level Convention

All signals in the GPU domain are normalized to 0.0-1.0 (corresponding to 0V-1V NTSC video). The model functions operate on this normalized range. Physical voltage scaling is documented in `spec.json` but not applied in shaders.

- `0.0` = blanking level (black)
- `1.0` = peak white
- Values outside 0-1 are permitted internally for intermediate processing (super-white, sync tips) but final output is clamped.

---

*Established January 2026. Update this document when adding new fidelity tiers or changing conventions.*
