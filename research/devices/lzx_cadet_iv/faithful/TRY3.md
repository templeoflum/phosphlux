# Third try of FAITHFUL.md (T-0352)

Run: `python3 -I compare3.py`. Numbers are in `try3_numbers.csv` and the picture is `try3_side_by_side.png`, labelled
**reference (circuit)**, **app today** and **difference** (mid grey means equal; lighter means the circuit is brighter,
scaled ×4). Inputs are the same as tries 1 and 2 ([`TRY1.md`](TRY1.md)): the T-0278 NTSC simulation at trimmer 1.0, and
the app's Test Signal ramps computed from the shader formula, not rendered on a GPU. The app has no Cadet IV device yet.

What changed is the rule: the reference is the simulated circuit, grey start included (Hedawn, T-0350). Row 2 now judges
the app's first pixel against the simulation's first pixel, not against 0 V, and row 1 judges every pixel again. Values
are exactly what `compare3.py` printed; every margin is a guess.

| Row | | H ramp (J8 vs mode 0) | V ramp (J3 vs mode 1) |
|---|---|---|---|
| 1 | Largest level error, every pixel (margin 0.010) | 0.0836 fail | 0.0837 fail |
| 2 | Start level, circuit 0.0843 / 0.0710 against app 0.0007 / 0.0021 (margin 0.010) | 0.0836 fail | 0.0689 fail |
| 3 | Slope, fit gain error (margin ±1 %) | −6.2 % fail | +1.5 % fail |
| 4 | Straightness (margin 0.004) | 0.0001 pass | 0.0000 pass |
| 7 | Resets inside the picture (margin ±1 px) | none in either, pass | none in either, pass |

Rows 5 and 6 do not apply: the app has no inverted ramp or triangle yet. **Not a match.** The app starts black where the
circuit starts grey (why: [`TRY2.md`](TRY2.md)), and its H ramp is 6 % less steep. The circuit also ends above 1.0
(1.021 H, 1.082 V) where the app ends at 1.0.
