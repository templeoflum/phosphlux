# Cable Degradation Research

Research artifacts for v0.13 Cable Degradation feature.

## Source Documents

| Source | Type | Used For |
|--------|------|----------|
| [NTSC Wikipedia](https://en.wikipedia.org/wiki/NTSC) | Web | Subcarrier frequency (3.579545 MHz), bandwidth specs |
| [YIQ Wikipedia](https://en.wikipedia.org/wiki/YIQ) | Web | Color space conversion matrices |
| [Dot Crawl Wikipedia](https://en.wikipedia.org/wiki/Dot_crawl) | Web | Y/C crosstalk artifact characteristics |
| [S-Video Wikipedia](https://en.wikipedia.org/wiki/S-Video) | Web | Y/C separation benefits |
| [RG-59 Wikipedia](https://en.wikipedia.org/wiki/RG-59) | Web | Cable attenuation values |

## Key Specifications

### NTSC Bandwidth Limits

| Channel | Bandwidth | Notes |
|---------|-----------|-------|
| Luminance (Y) | 4.2 MHz | Full baseband |
| I (orange-cyan) | 1.3 MHz | Vestigial sideband |
| Q (green-magenta) | 0.4 MHz | Double sideband only |

### YIQ Conversion Matrices (1953 NTSC)

**RGB to YIQ:**
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

### Cable Type Characteristics

| Type | Channels | Luma BW | Chroma BW | Artifacts |
|------|----------|---------|-----------|-----------|
| Composite | 1 | ~4 MHz | 0.4-1.3 MHz | Dot crawl, rainbow |
| S-Video | 2 | 5-7 MHz | ~1.5 MHz | None (Y/C separated) |
| Component | 3 | ~6 MHz | ~3 MHz | Minimal |
| RGB | 3 | 10+ MHz | N/A | Cable length only |

## Related Files

- Specification: `docs/CABLE_DEGRADATION_SPEC.md`
- Implementation: `src/shaders/cables.wgsl`
- Technical reference: `docs/VIDEO_SYNTHESIS_TECHNICAL_REFERENCE.md`
