# What "faithful" means for the Cadet IV (proposal, T-0293; black level changed in T-0345)

For Hedawn to agree to or change. It replaces the T-0279 draft (git `115ce3d`). The full reasons and reference values are in
[`faithful/RATIONALE.md`](faithful/RATIONALE.md), section by section. Sources:
- **CIR** [`CIRCUIT.md`](CIRCUIT.md): what LZX's documents say
- **AUD** [`AUDIO_MATCH_PRACTICE.md`](../../validation/AUDIO_MATCH_PRACTICE.md): how audio emulations judge a match
- **PRB** [`probe-t0290/README.md`](probe-t0290/README.md): the simulation probe on this Mac
- **REF** [`faithful/reference_check.txt`](faithful/reference_check.txt): the reference values, from `faithful/reference_check.py`
- **TRY2** [`faithful/TRY2.md`](faithful/TRY2.md): the second comparison, and why the simulation starts grey

**In plain words.** The app's Cadet IV is faithful if each of its six outputs matches a simulation of LZX's schematic at
every pixel, with the trimmers set the same, to within 10 millivolts. That is 1 % of the signal's range. The simulation
counts only because it agrees with the values that can be worked out by hand from LZX's documents, and it does. A measured
real module would beat the simulation. None has been measured, so that is an option, not part of the standard. Noise,
reset edges and differences between units are left out because no source gives a number for them. One exception to
following the simulation: each ramp must start at true black, on Hedawn's word, even though the simulated circuit starts grey.

## 1. Reference

**The ngspice simulation of the schematic, checked against the documents.** The simulation must agree, within §2's
margins, with three values derived in CIR:
- the slope, 1 mA ÷ C
- ramp plus inverted ramp, 1.006 V
- the triangle's zero at a ramp of 0.502 V

It does. Slope and sum are checked in all 8 runs. The zero is checked in the 6 runs where the ramp reaches 0.502 V, and
does not apply at trimmer 0.0 (REF). If they ever disagree, the documents win.

Why each other choice is ruled out, with sources:
- **Documents alone:** they give no values for straightness, the reset, the peak, noise or spread (CIR Q2, Q6, Q11, Q16).
- **A real unit:** no measurement of one exists (CIR "Searched and not found").
- **SPICE is accepted practice:** audio work uses it to check circuits rebuilt from schematics, and when the real unit is
  out of reach (AUD §1, §5, item 1).
- **The simulation's limits:** its part models are generic, and TI's own models do not run yet (PRB).

**Option:** a real Cadet IV plus a Cadet I and a scope or capture device. It would add the calibrated peak, the reset,
straightness, noise and spread. It would outrank the simulation. It costs money, so the decision is Hedawn's
(RATIONALE §1; AUD items 1 and 3).

## 2. What is compared and what counts as a match

**Conditions:** all six outputs, at trimmer settings 0.0, 0.5 and 1.0 on NTSC and 0.5 on PAL. The app's trimmer p is the
wiper position, so k = (4.99 + 10p)/14.99 (CIR stage 6). Sync and the picture window are assumed, because LZX states
neither (CIR stage 2a):
- NTSC columns run from 9.4 µs after H sync for 52.66 µs (720 columns). Rows run from 18 lines after V sync (240 rows).
- PAL columns run from 10.5 µs for 52 µs. Rows run from 23 lines (288 rows). The PAL numbers are unchecked.

**How values are taken:**
- **Reference:** the simulated jack voltage, unloaded, interpolated at each pixel's centre time.
- **App:** the device's output at the pixel centre, read back from the GPU as a float, before any clamp and before sRGB.
  0 V is 0.0 and 1 V is 1.0 (CIR stage 12).

An 8-bit screenshot is not enough. Near white, its rounding is up to 0.0045, nearly half the margin (RATIONALE §2).

| # | Quantity | Margin (proposed) | Sources |
|---|---|---|---|
| 1 | Level at every pixel, all outputs | ≤ 0.010 | CIR stages 3 and 8 (the TL431 spread is about ±5 mV); AUD §6 null test |
| 2 | **Black level, must match:** the first column or row of each rising ramp, against 0 V (true black), not against the simulation | ≤ 0.006 (**a guess**, placeholder) | Hedawn, T-0327: "reference clearly has darker blacks"; TRY2 |
| 3 | Slope, the gain of a straight-line fit | ±1 % | CIR Q3 (current −0.5/+1.3 %); AUD §3 |
| 4 | Straightness, the largest distance from that line | ≤ 0.004 (one 8-bit step of the linear signal) | CIR Q2 (no figure); PRB (generic transistor) |
| 5 | Ramp + inverted ramp, at every pixel | ≤ 0.010 from the reference | CIR Q7, Q14 |
| 6 | Triangle bottom: the ramp's level at the triangle's lowest pixel, and the lowest level | ±0.010 V each | CIR Q8, Q9, stage 10 |
| 7 | Resets inside the picture | none; otherwise the same pixel ±1 | CIR "Timing"; AUD "Does not carry over" |

A match means every row passes in every condition. Rows 1 and 5 leave out the first pixel of each ramp, which row 2
judges against black instead of against the simulation's grey. The error over every pixel is still reported. The RMS difference is reported but not judged, because it hides an
offset (AUD item 4). At the screen, 0.010 ranges from about 1 sRGB code near white to about 25 near black, so whether it
is visible is untested (RATIONALE §2).

**Known difference: the simulated circuit starts grey.** The ramp is released at the end of sync and rises through the back
porch before the assumed picture starts: 0.0837 V at the first H column and 0.069 V at the first V row, at trimmer 1.0.
This is in the circuit as drawn (reset driven only by sync, Blanking not connected), not in the simulation or the
mapping. How grey depends on the assumed picture window, which LZX does not state (TRY2). The simulation therefore
fails row 2 itself. It is kept as a known difference, not copied into the app.

**Why 0.006 (guess).** It is just above the few millivolts the reset leaves in the simulation (0.0022 V H, 0.0055 V V),
so the reset's own floor passes, and well below the 0.07–0.08 grey start. Near black, 0.006 is about 18 sRGB codes
(*computed here*), so it may be too loose to look black. It has not been tested on a screen.

## 3. Left out for now

- **Behaviour inside blanking:** reset edge, pedestal and spikes (CIR Q11; PRB).
- **Noise, bandwidth and millivolt offsets** (CIR Q16; PRB).
- **Differences between units** (CIR Q5).
- **Which trimmer setting is "calibrated"** (CIR Q6, stage 10).
- **The Cadet I's real sync** (CIR stage 2a).
- **Loading** (CIR stage 12).
- **Light on the screen** (RATIONALE §2).
- **A viewing test** (AUD).
- **Temperature and the supply** (PRB; CIR stage 4).

## 4. Against `docs/EMULATION_FIDELITY.md` (agent-written, not Hedawn's)

**Agrees:** 0–1 V maps to 0.0–1.0, and values outside 0–1 may pass between devices.

**Disagrees:**
- **How faithful is defined.** It defines faithful by how a model is built (six tiers). Its checks, a screenshot and one
  parameter sweep, name no reference and no margin. This page defines faithful by what comes out against a reference.
- **Noise, tolerance and crosstalk.** It requires them. Here they are left out, because there is nothing to compare them with (CIR Q16).
- **Tolerances.** Its ±5 % resistors do not match the BOM's 1 % (RATIONALE §4).
- **Soft clipping.** It asks for tanh clipping at every stage. These stages are linear in the picture (REF).
- **Clamping.** It clamps final output. The circuit leaves 0–1, so outputs are compared unclamped (RATIONALE §4).
- **Exposed parameters.** It wants every physical quantity exposed. Here only the two trimmers are judged.

## For you to decide

1. **The reference:** the simulation checked against the documents (recommended), or add a measured real unit.
2. **The margins:** 10 mV for levels and ±1 % for slope. The other margins follow from these.
3. **The grey start: answered.** Hedawn, T-0327: "reference clearly has darker blacks". True black is expected, so the
   grey start is not the standard (row 2). The picture he saw had no labels, and in it the darker column was the app,
   not the simulation; the new picture is labelled (`faithful/try2_side_by_side.png`).
