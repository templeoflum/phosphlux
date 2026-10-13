# What "faithful" means for the Cadet IV (draft, T-0279)

A proposal for Hedawn to confirm or change. Details of the first try are in `faithful/TRY1.md`.

**Reference.** For now the reference is the T-0278 ngspice simulation of LZX's schematic, with its trimmers calibrated so that
the ramp peaks at 1.0 V. Later it will be scope captures of a real, calibrated unit. Where the two disagree, the real unit wins.

**What is compared.**
- J8 H ramp against the app's Test Signal mode 0, and J3 V ramp against mode 1. The inverted ramps and triangles have
  no app counterpart yet.
- Only the active picture is compared, using assumed NTSC timing:
  - Horizontally: from 9.4 µs after the H sync edge, for 52.66 µs, split into 720 columns.
  - Vertically: from 18 lines after the V sync edge, for 240 lines, one row per line (one field).
- The simulation is read at each pixel's centre time. The app value is its shader output at the pixel centre.
- 0 V maps to 0.0 (black) and 1 V maps to 1.0 (white). This follows the LZX 0–1 V range, is linear, and is taken before the screen's sRGB encoding.

**How a match is judged.** Units are pixel value, so 0.01 = 10 mV.

| Measure | Tolerance | Reason |
|---|---|---|
| Largest raw pixel error | ≤ 0.02 | About 5 of 255 display steps. That is just visible on a ramp, and it allows for mV offsets the simulation cannot settle |
| Gain of a straight-line fit, app to reference | ±2 % | How closely a trimmer can be set by hand |
| Offset of that fit | ≤ 0.02 | Same reason as the raw error. It catches a ramp that starts above black |
| Linearity residual after the fit | ≤ 0.004 | About 1 display step. A constant-current ramp should be straight |
| Reset position inside the picture | same pixel ±1; a missing or extra reset fails | A visible edge in the wrong place is the plainest mismatch |

**First try (measured).** This used the T-0278 run with trimmers at 1.0, because no calibrated run exists. The shapes match:
both ramps are straight (residual ≤ 0.0001) and neither has a reset in the picture. The levels do not match. The
circuit's ramp is already at 0.07–0.08 when the picture starts, because it begins charging at the end of sync. The app
ramps exactly 0 → 1. The largest error is 0.084 on both ramps, so both fail on raw error and offset, and the H ramp also fails on gain.

**Guesses for Hedawn to confirm or change.**
1. All five tolerances.
2. That 1 V maps to 1.0 linearly, judged before the sRGB encoding rather than on screen.
3. The NTSC picture window. The LZX documents give none, and PAL needs its own numbers.
4. That the reference is a module calibrated to a 1.0 V peak.
5. That the app's ramp should start grey, as the circuit's does.
