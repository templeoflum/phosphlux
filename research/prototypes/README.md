# Prototypes

Working modules that are not authentic circuit emulations. These are valuable creative tools that emerged during development but should not be confused with historically accurate recreations.

## Classification

| Status | Meaning |
|--------|---------|
| `working` | Functional, produces useful output |
| `experimental` | Under development, may have issues |
| `archived` | Superseded or no longer maintained |

## Prototypes

### paik_abe_colorizer_v1

**Status:** Working
**Created:** 2026-01-25
**Classified:** 2026-01-29

Triangle wave folding colorizer. Originally thought to be authentic Paik/Abe emulation based on online descriptions mentioning "folding" behavior. Schematic analysis revealed actual Paik/Abe uses CA3019 amplitude windows (PED Control), not triangle wave folding.

**Keep for:** Creative/artistic use, potential wavefolder module base

---

## Versioning Convention

Prototypes use semantic versioning: `MAJOR.MINOR.PATCH`

- **MAJOR**: Breaking changes to parameters or behavior
- **MINOR**: New features, parameter additions
- **PATCH**: Bug fixes, tuning adjustments

## Promotion Path

Prototypes may be promoted to production devices if:
1. They are validated against actual schematics
2. They fill a gap not covered by authentic emulations
3. They are clearly labeled as "inspired by" rather than "emulation of"
