# First try of FAITHFUL.md (T-0279)

Run: `python3 -I compare.py`. Numbers are in `try1_numbers.csv` and the picture is `try1_side_by_side.png`.
Every value below is **measured** unless it is marked otherwise.

**Inputs.** The reference is the committed T-0278 NTSC run with trimmers R1 and R3 at 1.0: `../results/line_ntsc_pos1.0.csv` and
`frame_ntsc_pos1.0.csv`. Its peaks are 1.048 V (H) and 1.103 V (V). It is the nearest available run to a calibrated
1.0 V peak; no calibrated run exists yet. The app side is computed in Python from the shader formula:
`uv.x` for mode 0 and `uv.y` for mode 1, at pixel centres. It is not rendered on a GPU. The script stops if those lines in
`generators.rs` or the UV mapping in `renderer.rs` change. Before comparing, it runs a self-test of the reset measure
with shifts of 0, 1 and 2 pixels, a missing reset and an extra reset.

| | H ramp (J8 vs mode 0) | V ramp (J3 vs mode 1) |
|---|---|---|
| First pixel: reference vs app | 0.084 vs 0.001 | 0.071 vs 0.002 |
| Last pixel: reference vs app | 1.021 vs 0.999 | 1.082 vs 0.998 |
| Largest raw error (limit 0.02) | 0.084 fail | 0.084 fail |
| Fit gain error (limit ±2 %) | −6.2 % fail | +1.5 % pass |
| Fit offset (limit 0.02) | 0.084 fail | 0.069 fail |
| Linearity residual (limit 0.004) | 0.0001 pass | 0.0000 pass |
| Reset position (limit ±1 px) | no reset in either, pass | no reset in either, pass |

**Why they differ.** The circuit's ramp starts charging when sync ends. That is before the picture starts, so the
first pixel is already grey. The ramp keeps rising until the next sync, which comes after the picture ends. Part of
the gain and top-end error comes from the trimmer being at 1.0. By hand arithmetic (not run), scaling the H ramp to a
1.0 V peak still gives about 0.08 → 0.97 across the picture. So the offset is part of the circuit itself.

**Picture.** The columns are reference, app, and difference ×4 (mid grey = equal; lighter = reference higher). The rows
are the H ramp and the V ramp. Both picture columns are sRGB-encoded, the way the app would show them.
