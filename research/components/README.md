# Component Library

Reusable analog component models for video synthesis emulation.

## Structure

```
components/
├── ics/                    # Integrated circuits
│   ├── ca3046/            # RCA CA3046 transistor array
│   ├── ca3080/            # OTA (Operational Transconductance Amplifier)
│   ├── lm311/             # Comparator
│   └── ...
├── transistors/           # Discrete transistors
├── passives/              # R, C, L models (if needed for parasitics)
└── functional/            # Higher-level functional blocks
    ├── vca/               # Voltage Controlled Amplifier
    ├── comparator/        # Threshold comparator
    ├── ramp_generator/    # Horizontal/vertical ramps
    ├── sync_generator/    # Video sync
    └── amplitude_window/  # Paik/Abe style colorizer
```

## Component Entry Format

Each component folder contains:
- `spec.json` - Key specifications and transfer function
- `datasheet.pdf` - Original datasheet (if available)
- `model.wgsl` - WGSL shader implementation (core transfer functions)
- `*.wgsl` - Application shaders demonstrating component usage
- `test.md` - Test cases and validation notes
- `spice.txt` - SPICE model (if available)

## Available Components

### CA3019 - Ultra-fast Matched Diode Array
- `spec.json` - Pinout, diode specifications, transfer function
- `model.wgsl` - Diode conduction, amplitude window detection, multi-level classification
- `colorizer.wgsl` - **Paik/Abe PED Control** - the core colorizer effect
- `ped_control.wgsl` - Full PED Control circuit with chroma mixing

### CA3046 - NPN Transistor Array
- `model.wgsl` - BJT transfer function, differential pair, current mirror
- `colorizer.wgsl` - **Paik/Abe style colorizer** using amplitude windows

### CA3080 - Operational Transconductance Amplifier (OTA)
- `model.wgsl` - OTA transfer function with gm control
- `vca.wgsl` - **Voltage Controlled Amplifier** with LFO modulation

### LM311 - Voltage Comparator
- `model.wgsl` - Hard/soft comparator, window comparator
- `keyer.wgsl` - **Video keyer** with threshold and window modes

### CA3054 - Dual Differential Amplifier
- `spec.json` - Pinout, specifications, transfer function
- `model.wgsl` - Diff amp, keying functions, matrix mixing
- `matrix_mixer.wgsl` - **7-channel to RGB matrix mixer** (Paik/Abe page 8)
- `video_keyer.wgsl` - **Video keyer** with hard/soft keying modes

### Folding Amplifier (functional/folding_amp/)
- `spec.json` - Behavior specification, transfer functions
- `model.wgsl` - Core folding functions (triangle, sine, Paik/Abe style)
- `folder.wgsl` - **Wavefolder effect** with adjustable gain and softness

## Transfer Function Format

```json
{
  "name": "CA3046",
  "type": "transistor_array",
  "manufacturer": "RCA/Intersil",
  "description": "NPN Transistor Array (5 matched transistors)",

  "specs": {
    "vce_max": 15,
    "ic_max_ma": 50,
    "hfe_typical": 100,
    "ft_mhz": 300
  },

  "transfer_function": {
    "type": "exponential",
    "equation": "Ic = Is * exp(Vbe / Vt)",
    "parameters": {
      "Is": 1e-14,
      "Vt": 0.026
    }
  },

  "used_in": [
    "Paik/Abe Video Synthesizer",
    "Sandin Image Processor"
  ]
}
```

## Priority Components (from Paik/Abe schematics)

| Part | Type | Function | Status |
|------|------|----------|--------|
| CA3019 | Diode Array | Amplitude window detection (PED Control) | **Modeled** |
| CA3022 | Video Amplifier | Signal conditioning | Need datasheet |
| CA3023 | Video Amplifier | Signal conditioning | Need datasheet |
| CA3046 | Transistor Array | General purpose | **Modeled** |
| CA3054 | Differential Amp | Matrix mixing, keying | **Modeled** |
| CA3055 | Differential Amp | Matrix mixing | Need datasheet |
| CA3080 | OTA | Voltage controlled gain | **Modeled** |
| LM311 | Comparator | Threshold detection | **Modeled** |

## Sources

- [1978 RCA Linear IC Databook](https://www.smcelectronics.com/DOWNLOADS/1978-RCA%20LINEAR%20IC.PDF)
- [LZX Cadet Schematics](https://github.com/lzxindustries/lzxcadet)
- [Sandin IP Schematics](https://archive.org/details/sandin-image-processor-schematics_2)
- [AllDatasheet](https://www.alldatasheet.com)
