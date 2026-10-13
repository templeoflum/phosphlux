# What "faithful" means for the Cadet IV (proposal, T-0293)

For Hedawn to agree to or change. It replaces the T-0279 draft (in git at `115ce3d`). Sources are cited by key:

- **CIR**: [`CIRCUIT.md`](CIRCUIT.md), what LZX's own documents say, stage by stage (T-0288)
- **AUD**: [`../../validation/AUDIO_MATCH_PRACTICE.md`](../../validation/AUDIO_MATCH_PRACTICE.md), how audio emulations judge a match (T-0289)
- **PRB**: [`probe-t0290/README.md`](probe-t0290/README.md), whether the circuit simulates on this Mac (T-0290)
- **SIM**: [`README.md`](README.md), the T-0278 simulation and its assumptions. **REF**: [`faithful/reference_check.txt`](faithful/reference_check.txt), the reference values on this page, written by `python3 -I faithful/reference_check.py` from the T-0278 runs in `results/`

## In plain words

The app's Cadet IV counts as faithful if each of its six outputs matches a simulation of LZX's schematic at every pixel
of the picture, with the trimmers set the same, to within 10 millivolts. That is one hundredth of full white.
The simulation is trusted only because it agrees with the values that can be worked out by hand from LZX's documents,
and it does. A measured real module would outrank the simulation. Nobody has measured one, though, so that is an
option for you and not part of the standard yet. Things no source puts a number on are left out for now: noise, the
shape of the reset edge, and the differences between one unit and another.

## 1. What the app is compared against

**The schematic simulation, checked against the documents.** The reference picture is the ngspice simulation of LZX's
schematic (SIM, deck `sim/cadet_iv.lib`), run with the trimmers where the app's are. The simulation must first pass a
**document check**. It has to agree with three values derived from the documents, within the margins of §2:
- the slope, 1 mA divided by the timing capacitor (CIR stages 4–5, Q3–Q4)
- ramp plus inverted ramp, 1.006 V (CIR stage 8, Q7)
- the triangle's zero at a ramp of 0.502 V (CIR stage 9, Q8)

Today it passes in all 8 cases. The slope is 0.25–0.26 % low, the sum is at most 0.0030 off, and the zero is at most
0.0045 off (REF). If a rebuilt simulation fails this check, the documents win and the simulation gets fixed.

Why this choice:
- **Documents alone are not enough.** They give no figure for straightness, the reset, the calibrated peak, noise or spread (CIR Q2, Q6, Q11, Q16).
  Their derived values assume ideal parts and an instant reset (CIR "What derived assumes"), so they cannot give a level for each pixel.
- **No real module has been measured.** No measured Cadet IV trace was found in any source (CIR, "Searched and not found").
- **Audio work uses the same approach.** It checks a circuit rebuilt from a schematic against SPICE (Esqueda 2017, AUD §1). Chowdhury
  used SPICE when the real unit was too costly (AUD §5). SPICE shows that the model follows the schematic. Only a real unit can show
  that it matches the hardware (AUD "What the sources have in common", item 1).
- **Its limits are known.** The op-amp, TL431, transistor and diode models are behavioural or generic, not the makers' own
  (SIM "What was assumed"; PRB model table). TI's models load, but a run with them stops (PRB "Exact vendor failure"). So the
  simulation is trusted for the shape and for what the document check covers, not for its millivolt offsets or reset edges (§3).
  It does rerun: PRB got J8 at 0.0022–1.0484 V on ngspice-47, the same as SIM's `results/summary.csv`.

**Option, not yet chosen: a real module and a scope.** This needs a Cadet IV (the design is open, so it can be built from
the BOM), a Cadet I to supply sync, and a scope or a video capture device. It would add the values no source has:
- the calibrated peak (CIR Q6)
- the reset's timing and the level it leaves (Q11)
- how straight the real ramp is (Q2)
- noise, edges and differences between units (Q16)
- the real sync the Cadet I sends (CIR stage 2a)

Record the capture chain on its own too, so its offset is not blamed on the module (AUD §4, §2.5). A measured unit would
outrank the simulation (AUD item 1). The margins would then have to be reset around what it shows. No audio source turns
part tolerances into a pass band (AUD item 3). This would mean spending money, so the decision is yours.

## 2. What is compared, and what counts as a match

**Conditions.** Every row below is checked under every combination of these conditions:
- **Outputs:** all six. J8 H ramp, J6 H inverted ramp, J10 H triangle, J3 V ramp, J2 V inverted ramp, J4 V triangle (CIR stages 7–9, 12).
- **Trimmer settings:** 0.0, 0.5 and 1.0 on NTSC, and 0.5 on PAL. These are the runs that exist (SIM). The app's trimmer control p is the wiper position, with 0
  at the R2 end, so it scales the ramp by (4.99 + 10p)/14.99, from 0.333 to 1.0 (CIR stage 6, Q14). The trimmers are the module's only controls (CIR "The module as a whole").
- **Sync:** the timing the simulation assumed, which LZX's documents do not state (CIR stage 2a, Q11).
  - NTSC: 63.556 µs lines, 4.7 µs H sync, 3-line V sync
  - PAL: 64 µs lines, 4.7 µs H sync, 2.5-line V sync
  - Sync is active-low (SIM "What was assumed")

**Where the picture is.** These windows are assumed: no LZX document gives one (CIR stage 2a). The NTSC numbers come from T-0279.
- NTSC: 720 columns, starting 9.4 µs after the H sync edge and lasting 52.66 µs. 240 rows, one per line, starting 18 lines after the V sync edge (one field).
- PAL: 720 columns from 10.5 µs, lasting 52 µs. 288 rows from 23 lines after the V sync edge. *These PAL numbers are new here and have not been checked against the broadcast standard.*

**How each value is taken.**
- **Reference:** the simulated voltage at the jack (after the 499 Ω, unloaded), interpolated at each pixel's centre time.
  The H outputs come from the `results/line_*` files (20 ns steps). The V outputs come from the `results/frame_*` files
  (5 µs steps), read at the middle of each row's line (`reference_check.py`).
- **App:** the device's output at the same pixel centre, read back from the GPU as a floating-point number. It is read before any clamp and before the
  screen's sRGB encoding. 0 V is 0.0 and 1 V is 1.0, following the Cadet series' "0-1V DC" (CIR stage 12).
  An 8-bit screenshot is not good enough: its rounding alone uses up to 0.002 of the margin.
- **Scale:** 0.010 = 10 mV, about 2.5 steps of an 8-bit display.

| # | Quantity | Reference now (REF, NTSC trimmer 1.0, H / V) | Match margin | Why this margin (sources) |
|---|---|---|---|---|
| 1 | **Level at every pixel**, all six outputs. This is a null test in picture form (AUD §6) | the full curves (REF; `results/`) | \|app − reference\| ≤ **0.010** at every pixel | The documents fix levels only to about ±5 mV. The TL431's spread alone moves the inverted ramp's ceiling over 1.001–1.011 V (CIR stages 3 and 8, with the TI datasheet). The simulation leaves out op-amp offsets, worth a few mV (SIM "What the simulation cannot settle"). A tighter margin would ask for precision the reference does not have. A looser one would let visible steps through: try 1's gap of 0.084 is eight times this margin ([TRY1](faithful/TRY1.md)). Esqueda's 1 mV is not borrowed, because it belongs to another circuit (AUD "Carries over directly") |
| 2 | **Start level** of each ramp: its value at the first column or row | 0.0843 / 0.0710 (0.0281 / 0.0236 at trimmer 0.0) | ≤ **0.010** from the reference | Same reason as row 1. It is listed separately because it is the plainest visible fact. The ramp starts charging when sync ends, before the picture starts, so the picture's left and top edges are grey, not black (TRY1). In video, the DC level is the brightness, so it is judged on its own (AUD, ESR under "Carries over only with changes"). The value depends entirely on the assumed sync width and window |
| 3 | **Slope** of each ramp and inverted ramp: the gain of a straight-line fit across the picture | 17.81 mV/µs / 66.53 mV/ms | within **±1 %** of the reference | The charging current varies by −0.5 % to +1.3 % from the TL431 spread alone (0.995–1.013 mA, CIR Q3). 1 % of a 1 V ramp is 10 mV, which matches row 1. The slope is the ramp's counterpart of Werner's "tuning" (AUD §3) |
| 4 | **Straightness**: the largest distance from that fitted line | ≤ 0.00006 in every case | ≤ **0.004** | The documents give no figure (CIR Q2). The simulation's straightness rests on a generic transistor model (PRB), so the app is not asked to copy its 0.06 mV, only to stay within about one display step. Broadcast video measures this as luminance nonlinearity (AUD "Does not carry over") |
| 5 | **Inverted ramp ceiling**: ramp plus inverted ramp, at every pixel | 1.0089–1.0090 / 1.0066 (documents: 1.006) | ≤ **0.010** from the reference | The ceiling stays fixed whatever the trimmer setting (CIR Q7, Q14). This row checks that the app's inverted output follows its ramp, rather than being drawn separately |
| 6 | **Triangle bottom**: the reference ramp's level at the triangle's lowest pixel, and the triangle's lowest level | lowest at a ramp of 0.5065 V (column 324) / 0.5023 V (row 102), level 0.000. At trimmer 0.0 it never reaches 0: lowest 0.3255 at the last column | ramp level within **±0.010 V**, and lowest level within **0.010** | The zero point is set by the −2 V reference and slides with the trimmer (CIR Q8, Q9, Q14). It is the forum's calibration mark (CIR stage 10). It is judged in volts so that the margin is the same at every trimmer setting. At trimmer 1.0, H, one column is about 1.3 mV of ramp (73 ns × 17.81 mV/µs), so 10 mV is about 8 columns |
| 7 | **Resets inside the picture** | none, in all 8 cases | none in the app. If a later reference has one, it must land on the same pixel ±1; a missing or extra reset fails | The assumed sync puts every reset inside blanking (SIM; CIR "Timing"). An edge in the wrong place is the plainest mismatch. It stands in for audio's aliasing score (AUD "Does not carry over") |

A match means every row passes under every condition. The RMS difference is reported as a summary but not judged, because
a single number like that hides an offset (AUD, ESR under "Carries over only with changes"; item 4). The full reference values
for all 8 cases are in REF. Under these margins, try 1 still fails rows 1–3. That run compared the app's Test Signal ramps,
which are not a Cadet IV device. The app has no Cadet IV device yet.

## 3. Left out for now

- **Inside blanking:** the speed of the reset edge, the level left after reset, and the triangle's spike at reset.
  These are outside the picture, and the simulation uses generic models for them (CIR Q11; SIM; PRB).
- **Noise, bandwidth, edge shape and the real parts' millivolt offsets.** The simulation has none of these and no
  source gives a figure (CIR Q16; PRB "Reset pedestal, edge speed, noise, and millivolt offsets remain unvalidated").
- **Differences between units.** C29 at ±10 % and C18 at ±20 % move the peak a long way (CIR Q5, "Timing"). The app is compared
  with the nominal circuit. Building a tolerance band would need simulations across the tolerance range, or several
  units measured (AUD "Carries over directly").
- **Which trimmer setting counts as "calibrated", and the app's default.** No source gives a value (CIR Q6). The forum
  centres the triangle's dark point on the screen (CIR stage 10). Comparing at three settings keeps the standard free of this choice.
- **The Cadet I's real sync:** its polarity, pulse widths, equalising pulses and interlace. These are assumed (CIR stage 2a; SIM).
- **Loading.** The comparison is unloaded. Into a 100k input, the 499 Ω costs 0.5 % (CIR stage 12).
- **How the value becomes light:** the sRGB encoding, the display or a CRT. Values are compared before encoding.
- **A viewing test** like MUSHRA. It would need a capture of a real unit as its hidden reference (AUD "Carries over only with changes").
- **Temperature, ageing and supply-rail changes.** The simulation runs at exactly ±12 V and 27 °C (PRB). The charging current does not depend on the rail (CIR stage 4).

## 4. Against `docs/EMULATION_FIDELITY.md` (agent-written, not Hedawn's decision)

**Agrees.** Both use the 0–1 scale: 0.0 is 0 V black and 1.0 is 1 V white. Both let values outside 0–1 pass between devices.

**Disagrees.**
- **How faithful is defined.** That page defines it by how a model is built: six tiers, all required. This page defines it by what comes out, compared
  with a named reference within stated margins. That page has no reference and no margin. Its only check is a "Workbench screenshot verifying expected visual behavior".
- **Noise, tolerance and crosstalk.** That page requires them (tiers 4–6). This page leaves them out, because no source gives a number to compare them with (CIR Q16).
  If they were modelled now, nobody could judge whether they were right.
- **Tolerances.** Tier 5 assumes ±5 % resistors and ±20 % capacitors. The Cadet IV BOM gives every fixed resistor at 1 % and the two trimmers at 10 % (`source/cadet-iv-dual-ramp-generator-bom.csv` lines 12–19 and 29, CIR's BOM key), C29 56 nF ±10 % and C18 15 µF ±20 % (CIR Q5).
- **Soft clipping.** Tier 3 asks for tanh soft clipping at every gain stage. In the picture, the Cadet IV's stages are linear (straightness ≤ 0.00006, REF),
  so adding clipping would make this device less faithful.
- **Clamping.** It says "final output is clamped". The circuit goes outside 0–1: J6 reaches −0.040 V and J10 reaches 1.094 V at trimmer 1.0 (SIM, `results/summary.csv`).
  So the device's output is compared unclamped. What the screen does with it is a separate question.
- **Exposed parameters.** It says "Every physical quantity … must be an exposed parameter". Here only the two trimmers count, because they are the module's only controls.
  Other parameters may be exposed, but they are not judged.
- **Bandwidth.** Its tier 2 example, a 6 MHz video bandwidth, does not affect a ramp's level across the picture. It is left out here (§3).

## For you to decide

1. **The reference.** The simulation checked against the documents (recommended), or a measured real module as well (the option in §1).
2. **The margins.** The 10 mV level margin and the ±1 % slope margin. Rows 2 and 4–6 follow from these. All of them are proposals.
3. **The grey start.** Whether the grey at the picture's left and top edges (row 2) is part of the circuit, for the app to copy.
