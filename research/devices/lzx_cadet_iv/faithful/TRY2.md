# Second try of FAITHFUL.md (T-0345)

Run: `python3 -I compare2.py`. Numbers are in `try2_numbers.csv` and the picture is `try2_side_by_side.png`, now with
labels on each column. Inputs are the same as try 1 ([`TRY1.md`](TRY1.md)): the T-0278 NTSC run at trimmer 1.0, and the
app's Test Signal ramps computed from the shader formula, not rendered on a GPU. What changed is the rules: the margins
are the T-0293 ones, row 2 now judges black against 0 V, on Hedawn's word (T-0327), and row 1 leaves out the first
pixel, which row 2 judges. Nothing here was measured on hardware or read from a GPU. The reference values are the
simulation's output, interpolated at pixel centres; the app values are computed from the shader formula. Both are
exactly what `compare2.py` printed.

| Row | | H ramp (J8 vs mode 0) | V ramp (J3 vs mode 1) |
|---|---|---|---|
| 1 | Largest level error, first pixel left out (margin 0.010) | 0.0836 fail | 0.0837 fail |
| 2 | Black level, app's first pixel against 0 V (margin 0.006, a guess) | 0.0007 pass | 0.0021 pass |
| 2 | Black level, simulation's first pixel against 0 V | 0.0843 fail | 0.0710 fail |
| 3 | Slope, fit gain error (margin ±1 %) | −6.2 % fail | +1.5 % fail |
| 4 | Straightness (margin 0.004) | 0.0001 pass | 0.0000 pass |
| 7 | Resets inside the picture (margin ±1 px) | none in either, pass | none in either, pass |

Rows 5 and 6 do not apply: the app has no inverted ramp or triangle yet. Not a match. Leaving out the first pixel
changes nothing here, because the gap is an offset along the whole ramp: over every pixel the largest error is the
same, 0.0836 H and 0.0837 V (reported as a diagnostic).

## Why the simulation starts grey

Three causes were checked. The evidence is the simulation's own output (`../results/line_ntsc_pos1.0.csv`,
`frame_ntsc_pos1.0.csv`), printed by `compare2.py`.

- **The 0 V-to-pixel mapping: not the cause.** The script divides by 1.0, so 0 V is 0.0 and the sRGB step comes after.
- **An offset in the netlist: a small part only.** During sync the reset holds the H ramp at 0.0022 V and the V ramp at
  0.0055 V. My reading, not checked: the reset transistor does not pull fully to 0 while the current source keeps
  feeding it. Both are a few millivolts, below the black margin of 0.006.
- **Reset timing against the picture window: the cause.** The reset is driven only by sync, and the Blanking pin is not
  connected (CIR stage 2, "Timing"). The ramp is released at the end of sync and rises before the picture starts:
  - H: 0.0022 V at the end of H sync (4.7 µs), 0.0837 V where the assumed picture starts (9.4 µs). That is 4.7 µs of
    back porch at 17.81 mV/µs.
  - V: 0.0055 V at the end of V sync (3 lines), 0.0689 V where the assumed picture starts (18 lines).

So the grey is in **the circuit as drawn**, not in the simulation or the mapping. How grey it looks depends on where the
picture starts, and LZX does not state that (CIR stage 2a). The 9.4 µs and 18 lines are assumptions. Any picture that
starts after the back porch, as a display's does, starts above black. Moving the window to the end of sync to hide this
would mean showing the back porch, so it was not done.

**Known difference.** Hedawn expects true black. The circuit as drawn, under the assumed timing, does not give it.
This is kept as a known difference. Nothing was changed to remove it. The app is not changed either: it has no Cadet
IV device yet, and building one is outside this task.
