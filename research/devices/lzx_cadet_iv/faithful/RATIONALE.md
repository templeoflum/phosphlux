# Why the Cadet IV standard says what it says (T-0293)

This is supporting material for [`../FAITHFUL.md`](../FAITHFUL.md). It gives the full reasons, the reference values and
the sources for each choice. The section numbers match that page. Source keys:

- **CIR**: [`../CIRCUIT.md`](../CIRCUIT.md) (T-0288)
- **AUD**: [`../../../validation/AUDIO_MATCH_PRACTICE.md`](../../../validation/AUDIO_MATCH_PRACTICE.md) (T-0289)
- **PRB**: [`../probe-t0290/README.md`](../probe-t0290/README.md) (T-0290)
- **SIM**: [`../README.md`](../README.md) (T-0278)
- **REF**: [`reference_check.txt`](reference_check.txt), written by `python3 -I reference_check.py`
- **TRY1**: [`TRY1.md`](TRY1.md)
- **TRY2**: [`TRY2.md`](TRY2.md) (T-0345)

**Labels.** Every margin is a **proposal**. No source sets a pass mark (AUD item 2), and none turns part tolerances into one
(AUD item 3). Numbers marked *computed here* are my arithmetic. None of the remarks about what is visible on screen has been tested on a viewer.

## 1. Reference

**The document check.** Before any comparison, the simulation has to agree with values derived from the documents,
within the margins in §2. REF reports the check for each of the 8 runs:
- **Slope:** the derived slope is 1 mA/C × k (CIR stages 4–6, Q3–Q4). Checked in all 8 cases, 0.25–0.26 % low.
- **Ramp plus inverted ramp:** the derived value is 1.006 V (CIR stage 8, Q7). Checked in all 8 cases, at most 0.0030 off.
- **Triangle zero:** the derived value is a ramp of 0.502 V (CIR stage 9, Q8). This check applies only when the ramp crosses
  0.502 V inside the picture. That is decided from the ramp's range, so a broken triangle cannot hide it. If the
  ramp crosses, two things are judged. The triangle's lowest level must be within ±0.010 of 0, on either side.
  The ramp's level at that lowest pixel must be within ±0.010 V of 0.502 V. Here it applies in 6 cases: the lowest
  level is at most 0.0025 from 0, and the position is at most 0.0045 V off. It does not apply in the 2 trimmer-0.0 cases: their ramps reach only 0.340 V (H) and 0.360 V (V),
  so they never get to 0.502 V (CIR Q8, the "peak below 0.502 V" case).
- *Checked here:* in scratch copies, adding +0.1 or −0.1 to one run's triangle made the check report "tri lowest level +0.0994 (FAIL)" and "−0.1006 (FAIL)".

**Why not the documents alone.**
- No source gives a figure for straightness, the reset, the calibrated peak, noise or spread (CIR Q2, Q6, Q11, Q16).
- The derived values assume ideal parts and an instant reset (CIR "What derived assumes").

**Why not a real unit.** No measured Cadet IV trace exists in any source read (CIR "Searched and not found").

**Why a simulation is acceptable.**
- Audio work checks a circuit rebuilt from its schematic against SPICE (Esqueda, AUD §1).
- Chowdhury used SPICE because the real pedal was too costly (AUD §5).
- SPICE shows that the model follows the schematic. Only a real unit shows that it matches the hardware (AUD item 1).

**Limits of the simulation.**
- The op-amps, TL431, transistor and diode are behavioural or generic models (SIM "What was assumed"; PRB model table).
- A run with TI's own models stops (PRB "Exact vendor failure").
- The run is reproducible. PRB got J8 at 0.0022–1.0484 V on ngspice-47, the same as `../results/summary.csv`.

**Option: a real unit.** It would need:
- a Cadet IV, which can be built from the open BOM
- a Cadet I for sync
- a scope or a video capture device

It would supply the values no source gives: the calibrated peak (Q6), reset timing and level (Q11), straightness (Q2),
noise and differences between units (Q16), and the real sync (CIR stage 2a). Measure the capture chain on its own as
well (AUD §4, §2.5). A measured unit would outrank the simulation (AUD item 1), and the margins would have to be set again.

## 2. Measures

**Conditions.**
- **Outputs:** all six (CIR stages 7–9, 12).
- **Trimmer settings:** 0.0, 0.5 and 1.0 on NTSC, and 0.5 on PAL (the runs in SIM). The app's trimmer p maps to
  k = (4.99 + 10p)/14.99 (CIR stage 6, Q14).
- **Sync:** NTSC 63.556 µs lines, 4.7 µs H sync, 3-line V sync. PAL 64 µs, 4.7 µs, 2.5 lines. Active-low.
  These are assumptions made in SIM and are not stated by LZX (CIR stage 2a, Q11).

**Picture window (assumed).**
- **NTSC:** 9.4 µs after the H sync edge, 52.66 µs long, 720 columns. Rows start 18 lines after V sync, 240 of them. These numbers come from T-0279.
- **PAL:** 10.5 µs, 52 µs, 720 columns. Rows start 23 lines after V sync, 288 of them. These are new here and *not checked* against the broadcast standard.

**Scale, signal against screen** (*computed here* with the standard sRGB formula). 0.010 is 10 mV, which is 1 % of the
0–1 V signal. On screen, after the app's sRGB encoding, the same change is a different number of 8-bit codes
depending on the level:

| Level | Codes for a change of 0.010 |
|---|---|
| near white (0.99) | about 1.1 |
| mid grey (0.5) | about 1.7 |
| the ramp's grey start (0.084) | about 4.6 |
| black | about 25 |

An 8-bit sRGB screenshot rounds by up to half a code. Near white, that is 0.0045 in linear units, close to half the
margin. That is why the app's values are read back as floats before encoding.

| # | Reference now (REF, NTSC trimmer 1.0, H / V) | Why this margin |
|---|---|---|
| 1 Level at every pixel ≤ 0.010 | the full curves | The documents fix levels to about ±5 mV. The TL431 spread alone moves the inverted ceiling across 1.001–1.011 V (CIR stages 3 and 8; *computed here*: 2.483 V and 2.507 V × 10/24.8). The simulation leaves out op-amp offsets of a few mV (SIM "What the simulation cannot settle"). Try 1's gap of 0.084 is 8× this margin (TRY1). Esqueda's 1 mV belongs to another circuit and is not borrowed (AUD "Carries over directly"). It is a null test in picture form (AUD §6) |
| 2 Black level ≤ 0.006 against 0 V (T-0345; the margin is a guess) | 0.0843 / 0.0710, so the simulation itself fails | Changed on Hedawn's word (T-0327): true black is expected, so the first pixel is judged against 0 V, not against the simulation. The grey start is in the circuit as drawn: the ramp is released at the end of sync and rises through the back porch before the assumed picture starts (TRY2). DC level is brightness, so it is judged on its own (AUD, ESR under "Carries over only with changes"). 0.006 sits just above the reset's few millivolts in the simulation (0.0022 V H, 0.0055 V V) and well below the grey start; near black it is about 18 sRGB codes (*computed here*), untested on a screen. Rows 1 and 5 leave out the first pixel, which this row judges |
| 3 Slope ±1 % | 17.81 mV/µs / 66.53 mV/ms | The charging current spans 0.995–1.013 mA (CIR Q3). 1 % of a 1 V ramp is 10 mV, the same as row 1. It is the ramp's counterpart of "tuning" (AUD §3) |
| 4 Straightness ≤ 0.004 | ≤ 0.00006 in all runs | No source gives a figure (CIR Q2). The simulation's value rests on a generic transistor (PRB), so it is not copied. 0.004 is one 8-bit step of the *linear signal* (1/255), not a screen step. After sRGB it is under one code above mid grey and more near black (about 13 codes at 0). It is about 2.5× tighter than row 1, so curvature is caught separately from offset. Broadcast video calls this luminance nonlinearity (AUD "Does not carry over") |
| 5 Ramp + inverted ≤ 0.010 from the reference | 1.0089–1.0090 / 1.0066 | The ceiling stays fixed at any trimmer setting (CIR Q7, Q14). This row checks that the inverted output follows the ramp |
| 6 Triangle bottom ±0.010 V | the ramp is at 0.5065 V at column 324 / 0.5023 V at row 102; the lowest level is 0.000. At trimmer 0.0 the triangle never reaches 0 (lowest 0.3255) | The zero is set by the −2 V reference and slides with the trimmer (CIR Q8, Q9). It is the forum's calibration mark (CIR stage 10). Judging it in volts keeps the margin equal at every setting. *Computed here:* one column is about 1.3 mV of ramp at trimmer 1.0, so 10 mV is about 8 columns |
| 7 Resets ±1 px | none in any of the 8 runs | The assumed sync puts each reset in blanking (SIM; CIR "Timing"). It is the counterpart of audio's aliasing score (AUD "Does not carry over") |

The RMS difference is reported but not judged, because one number hides an offset (AUD item 4 and ESR). Under these
margins, try 1 still fails rows 1–3. It compared the app's Test Signal ramps; the app has no Cadet IV device yet.

## 3. Left out (sources)

- **Blanking-time behaviour:** CIR Q11; SIM; PRB.
- **Noise, bandwidth, edges and offsets:** CIR Q16. PRB: "Reset pedestal, edge speed, noise, and millivolt offsets remain unvalidated."
- **Differences between units:** CIR Q5, "Timing". A tolerance band would need simulations across the tolerances or several measured units (AUD "Carries over directly").
- **The calibrated setting and the app's default:** CIR Q6, stage 10.
- **The real sync:** CIR stage 2a; SIM.
- **Loading:** into 100k, the output drops by 0.5 % (CIR stage 12).
- **Light on the screen:** see "Scale, signal against screen" above.
- **A viewing test:** AUD "Carries over only with changes".
- **Temperature and supply:** PRB (27 °C, exactly ±12 V); CIR stage 4 (the current does not depend on the rail).

## 4. Against `docs/EMULATION_FIDELITY.md`

- **Clamping.** It says "final output is clamped". The circuit goes outside 0–1: J6 reaches −0.040 V and J10 reaches 1.094 V at trimmer 1.0 (`../results/summary.csv`).
- **Its checks.** Its checklist asks for a "Workbench screenshot verifying expected visual behavior" and "At least one parameter sweep demonstrating smooth continuous response". Neither names a reference to compare against or a margin for a match.
- **Tolerances.** It assumes ±5 % resistors and ±20 % capacitors. The BOM gives 1 % fixed resistors and 10 % trimmers (`../source/cadet-iv-dual-ramp-generator-bom.csv` lines 12–19, 29), C29 at ±10 % and C18 at ±20 % (CIR Q5).
