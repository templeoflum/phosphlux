# What "faithful" means for the Cadet IV (draft, T-0279)

A proposal for Hedawn to confirm or change. Statements are marked **read** (from a document or code),
**assumed** (my choice) or **measured** (output of `faithful/compare.py`).

## Reference: what Phosphlux's Cadet IV is compared against

1. **Now:** the T-0278 ngspice simulation of LZX's published schematic (`results/`). It is a simulation
   built on generic part models, so it is a stand-in, not ground truth (see `README.md`, "What the simulation cannot settle").
2. **Later:** scope captures of a real Cadet IV with its trimmers calibrated. Where a real unit and the simulation
   disagree, the real unit wins, and the simulation is fixed or its limits are written down.

The reference is the module with its trimmers **calibrated** so the ramp peaks at 1.0 V. For this first try I used
the committed NTSC run with trimmers at 1.0 (peaks 1.048 V H, 1.103 V V), because it was the nearest available run. No calibrated run exists yet.

## What is compared

- **Signals:** J8 H ramp against the app's Test Signal mode 0 (H ramp), and J3 V ramp against mode 1 (V ramp).
  The inverted ramps (J6, J2) and triangles (J10, J4) have no app counterpart yet. Later they get the same treatment.
- **Time to pixels (assumed, NTSC):** only the active picture is compared. Horizontally that is 9.4 µs after the H sync
  leading edge (4.7 µs sync plus 4.7 µs back porch), for 52.66 µs, split into 720 columns. Vertically it is 18 lines
  after the V sync leading edge, for 240 lines, one row per line (one field, no interlace).
  These are standard NTSC figures. The LZX documentation note does not state picture timing; I searched it for this.
- **Sampling:** the simulated voltage is sampled at each pixel's centre time, with linear interpolation of the T-0278 CSVs
  (20 ns steps per line, 5 µs steps per frame). The app value is its shader's output at the pixel centre,
  `uv = (i + 0.5) / n`. That gives `uv.x` for mode 0 and `uv.y` for mode 1, with `uv.y = 0` on the top row (read in
  `generators.rs` and `renderer.rs`). The value is computed in Python from that formula, not rendered on a GPU. The
  script stops if those shader lines change.
- **Volts to pixel value (LZX 0–1 V; the Visual Cortex manual says "0-1V, DC Coupled"):** 0 V maps to 0.0 (black) and 1 V maps to 1.0 (white).
  The mapping is linear and is not clipped for measuring. The comparison uses the value before the screen's
  sRGB encoding, which the app applies to both alike (it picks an sRGB surface in `renderer.rs`).

## How a match is judged

All pass/fail limits are in pixel value units, where 0.01 = 10 mV.

| Measure | Tolerance | Reason |
|---|---|---|
| Largest pixel error, raw | ≤ 0.02 | About 5 of 255 display steps. That is just visible on a smooth ramp, and loose enough for the mV offsets the simulation cannot settle |
| Gain of the best straight-line fit, app → reference | within ±2 % | A calibrated unit's peak is set by hand on a trimmer, and 2 % is my guess at how precisely that can be done |
| Offset of that fit | ≤ ±0.02 | The same reasoning as the raw error. It catches a ramp that starts above black |
| Linearity: largest residual after the fit | ≤ 0.004 | About 1 display step. A ramp is a linearity test, and a constant-current charge should be straight |
| Reset or wrap inside the picture | same pixel ±1 | A visible edge in the wrong place is the most obvious mismatch there is |

## First try (measured: `python3 -I faithful/compare.py`, results in `faithful/try1_numbers.csv` and `faithful/try1_side_by_side.png`)

| | H ramp | V ramp |
|---|---|---|
| Left/top pixel: reference vs app | 0.084 vs 0.001 | 0.071 vs 0.002 |
| Right/bottom pixel: reference vs app | 1.021 vs 0.999 | 1.082 vs 0.998 |
| Largest raw error | 0.084 **fail** | 0.084 **fail** |
| Fit gain error / offset | −6.2 % **fail** / 0.084 **fail** | +1.5 % pass / 0.069 **fail** |
| Linearity residual | 0.0001 pass | 0.0000 pass |
| Reset inside picture | none in either, pass | none in either, pass |

**The gap, plainly:** the shapes agree. Both are straight ramps with no reset in the picture. The levels do not
agree. In the circuit, the ramp starts charging at the end of sync, so by the time the picture starts it is
already 0.07–0.08 up: grey, not black. It keeps rising until the next sync, which falls after the picture
ends. The app's ramp runs exactly 0 → 1 across the picture. Part of the gain and top-end error comes from the
uncalibrated trimmer setting. By hand arithmetic (not run), scaling the H ramp to a 1.0 V peak still leaves it at about
0.08 → 0.97 across the picture. So the offset is part of the circuit, not a calibration artefact.

The image has three columns: reference, app, and difference ×4 (mid grey = equal; lighter = reference higher). Both
picture columns are sRGB-encoded as the app would show them.

## Guesses Hedawn should confirm or change

1. Every tolerance in the table above: 0.02 raw, ±2 % gain, 0.02 offset, 0.004 linearity and ±1 pixel. None comes from a
   measurement of how much a real unit varies.
2. Value units: 1 V → 1.0, linear, compared before the sRGB encode. One alternative is to judge what reaches the screen.
3. The picture window: NTSC timings, 720 × 240 per field, and the 18-line vertical start. LZX may define this differently,
   and PAL needs its own numbers.
4. That the reference is the module calibrated to a 1.0 V peak, rather than the trimmers at some other setting.
5. That the app's ramp should show the circuit's grey start (≈0.08) at all. Faithful says yes, but it changes how the
   test signal looks today.
