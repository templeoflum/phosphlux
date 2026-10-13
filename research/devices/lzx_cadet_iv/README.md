# LZX Cadet IV Dual Ramp Generator: schematic probe in ngspice (T-0278)

A rough first look at what the Cadet IV's six outputs are, taken from LZX's published schematic and
simulated in ngspice. It is a proposal, not a verified reference. Three kinds of statement appear below
and are kept apart: **read** (from the schematic or BOM), **assumed** (my choice, where the
documents say nothing), and **simulated** (ngspice output in `results/`).

## Files

| Path | What |
|---|---|
| `source/cadet-iv-dual-ramp-generator-schematics.pdf` | Schematic, 2 pages. From https://raw.githubusercontent.com/lzxindustries/lzxtm/main/static/pdf/cadet-iv-dual-ramp-generator/cadet-iv-dual-ramp-generator-schematics.pdf, downloaded 2026-10-13 06:30 UTC. SHA-256 `27a0ab4d03fb198bf8c8109a517cb62a12517ae840f78b498963af7df9108704`. PDF metadata: author Lars Larsen, created 2016-05-03, modified 2018-12-07. No text layer, so I read it from page images rendered at 200 and 400 dpi |
| `source/cadet-iv-dual-ramp-generator-bom.csv` | BOM from the same lzxtm folder, …/cadet-iv-dual-ramp-generator-bom.csv, downloaded 2026-10-13. SHA-256 of the bytes as downloaded (CRLF line endings) and as committed: `438946189094ebf71586f1e01e333382dfbd9d33e1d605205dfb9906da15d62b`. The first commit (2e887df) stored an LF-normalised copy (SHA-256 `10347e050b09990b84f201fa3fff22704dbe9dacc7da8d0a79d29d854c14033c`) because of the repo's `text=auto eol=lf` rule. `.gitattributes` in this folder now sets `source/** -text`, so later commits store the downloaded bytes unchanged |
| `source/LZX-C4-V1.0-Bill-of-Materials.csv` | BOM from https://github.com/lzxindustries/lzxcadet/blob/main/C4/BOM/ ("LZX-C4 V1.0 Bill of Materials.csv"), downloaded 2026-10-13. SHA-256 `2ea60d4ade6c44726bd33c6cc28d594d9043dfa8ebd7fb9225a9d99447fc39cc` (as downloaded and as committed). It has the same rows and values as the lzxtm copy. The only difference is trailing spaces in a few cells |
| `sim/cadet_iv.lib` | Hand-transcribed netlist (subcircuits) plus the assumed device models |
| `sim/run_sims.py`, `sim/run.sh` | Builds one deck per case, runs `ngspice -b`, and writes the CSVs, SVG plots and summary |
| `results/line_<std>_pos<p>.csv/.svg` | Two video lines of all six outputs, sampled every 20 ns, with t=0 at an H sync leading edge |
| `results/frame_<std>_pos<p>.csv/.svg` | One frame (two fields) plus 1 ms of all six outputs: 0–34.37 ms on NTSC, 0–41 ms on PAL. Sampled every 5 µs, with t=0 at a V sync leading edge, after two fields of settling. The sim has no interlace, so the two fields come out identical (V peaks agree within 0.1 mV) |
| `results/summary.csv` | Min and max of every output in every run |
| `results/operating_point.txt` | Reference voltages and charging current that ngspice printed |

Licence: the lzxcadet README (quoted in the studio knowledge note, row B) says the designs are open-source
hardware. The Cadet II–X schematic PDFs carry no licence text. The files are kept here as research evidence.

## What was read from the schematic and BOM

The schematic title block reads "Cadet IV Ramp Generator", "4HP EuroRack Synthesizer Module", V1.0,
Lars Larsen, "Schematic 1/2" dated January 16th 2016 and "Schematic 2/2" dated January 16th 2015.
Page 1 is the horizontal channel, the −2 V reference and the 5 V regulator. Page 2 is the vertical
channel, power entry and sync input. Both channels use the same circuit (component renaming is listed at the top of `cadet_iv.lib`).

Line by line, using page 1 (H) names, with page 2 (V) names in brackets:

1. **Power entry (p2):** a 16-pin Eurorack header J9. +12 V passes through D1 1N4001 and ferrite bead FB1. −12 V passes through D2 1N4001 and FB2. C2 and C6 are 10 µF.
2. **5 V (p1):** U3 7805 from +12 V, with C1 330 n and C10 100 n. It powers only U7, a 74HC14.
3. **Sync input (p2):** J1 is a 14-pin "Video Sync Input Header" with pins GND, Blanking, Odd/Even, Burst Gate, V Sync, H Sync and C Sync. Only V Sync (pin 9) and H Sync (pin 11) are used. Each goes through one 74HC14 inverter (U7.1 and U7.3) to make `_VSYNC` and `_HSYNC`. The other 74HC14 inputs are tied to ground.
4. **−2 V reference (p1):** U11 TL431 with REF tied to K, fed from +12 V through R38 4.99 K, so about 2.495 V. U1.1 (TL072) inverts it with R40 12.4 K in and R41 10 K feedback, giving −2.495·10/12.4 = −2.01 V. The schematic labels this "−2V Reference". U1.2 is unused, wired as a follower to ground.
5. **Current-source reference:** R39 [R22] 10 K from +12 V to REF of U10 [U9], then R42 [R23] 10 K to node X, then R43 [R24] 1 K to ground. TL431 K goes to +12 V and A goes to node X. The TL431 holds 2.495 V across R42, so R39 also drops about 2.495 V. That puts node X at +12 − 4.99 V.
6. **Constant current:** U12.1 [U13.1] (TL074) drives the base of Q1 [Q2], a 2N3906 PNP, and holds the emitter at node X. The emitter returns to +12 V through R44 [R25] 4.99 K. Current = 4.99 V / 4.99 K = **1.0 mA, independent of the +12 V rail**.
7. **Timing capacitor:** the Q1 collector charges C29 **56 nF** [C18 **15 µF**] to ground. BOM: C29 is X7R ±10 %; C18 is an X5R ±20 % MLCC.
8. **Reset:** Q3 [Q4], a BS170 N-MOSFET, shorts the capacitor to ground. Its gate is driven from `_HSYNC` [`_VSYNC`] through R45 [R27] 100 R.
9. **Buffer and amplitude trimmer:** U12.2 is a follower. Its output feeds trimmer **R1 [R3] 10 K** (Bourns 3362P per the BOM), which goes to ground through R2 [R4] 4.99 K. The wiper feeds U12.3, a follower. So the ramp is scaled by between 4.99/14.99 = 0.333 and 1.0.
10. **Ramp output:** U12.3 → R50 499 R → **J8** [U13.3 → R32 → **J3**].
11. **Inverted ramp:** U12.4 is an inverting summer with R47 10 K from the ramp, R48 20 K from −2 V and R46 10 K feedback. So out = 1.005 V − ramp. It goes through R49 499 R to **J6** [R31 → **J2**].
12. **Triangle:** U2.1 is an LM6172 with + at ground. Its − input sums the ramp through R13 499 R and −2 V through R11 2 K. Two feedback paths each have a 1N5711 Schottky: D5 from node B (anode) to the output (cathode), with R10 1 K back to −; and D6 from the output (anode) to node C (cathode), with R26 1 K back to −. U2.2 takes node C on its + input (via R18 499 R) and node B on its − input (via R20 1 K, with R16 1 K feedback). Output goes through R15 499 R to **J10** [U5 → R33 → **J4**]. Hand analysis gives out = 2.004·|ramp − 0.50 V| (the −2 V/2 K current cancels the ramp/499 R current at a ramp of 0.50 V).
13. **Jacks:** six PJ302M jacks. The schematic gives no panel labels, so names like "J8 H ramp" in this README are mine. The BOM lists **no front-panel potentiometers**. The only adjustments are the two internal trimmers R1 and R3, so in this README "controls" means those trimmers.

## What was assumed

| Assumption | Why / effect |
|---|---|
| Op-amps (TL072, TL074, LM6172) are a behavioural single-pole model I wrote: TL07x GBW 3 MHz and slew 13 V/µs; LM6172 GBW 100 MHz and slew 3000 V/µs; open-loop gain 2·10⁵; output swing to 1.5 V inside the rails; 25 Ω output. These are not vendor macro-models: no offsets, input bias or supply current | Vendor models were not downloaded (outside ALLOWED). Edges, reset glitches and offsets in the mV range are therefore not trustworthy |
| TL431 is a behavioural shunt that conducts once V(REF−A) > 2.495 V. Its ~2 µA REF current is ignored | Sets −2.013 V and the 1.0024 mA charging current |
| The 2N3906 model card is a widely circulated generic one, written from memory and not checked against a vendor file. The BS170 is a level-1 NMOS (Vto 2 V, about 2 Ω on at 5 V gate drive) plus a body diode. The 1N5711 has Is 1.2e-10, Rs 30 Ω. The 1N4001 uses a generic card | Affects reset speed, the pedestal left after reset, and the rectifier corner |
| Bus rails are exactly ±12 V. Ferrite beads are 0.1 Ω. The 7805 is not modelled: `_HSYNC`/`_VSYNC` are ideal 0/5 V pulses with 10 ns edges | The simulated +12 V rail after D1 is 11.28 V. The −12 V rail shows −12.00 V because the op-amp model draws no supply current, so D2 never conducts |
| **Sync is active-low at J1** (Cadet I sync generator not checked), so `_HSYNC`/`_VSYNC` go high during the sync pulse and reset the capacitor | If sync were active-high, the capacitor would be held shorted for most of each line, so the circuit only works this way round |
| Sync timing: NTSC is 63.556 µs lines with 4.7 µs H sync, and 16.683 ms fields with a 3-line V sync. PAL is 64 µs / 4.7 µs and 20 ms with a 2.5-line V sync. H sync keeps running through vertical sync; there are no equalising or serration pulses and no interlace half-line offset | The ramp peak depends directly on the period, so this sets the amplitudes |
| Jacks are unloaded: the voltage is read after the 499 R, with no load | A 1 K or 75 Ω load would divide the output; see open questions |
| Trimmer positions 0.0, 0.5 and 1.0 (0 = wiper at the R2 end). R1 and R3 are set together in each run. Nominal component values, 25 °C | These are the "control extremes and centre" asked for |
| The frame view samples every 5 µs, so the H outputs in the frame plots are aliased (they look like a dense band). Use the line view for H shape | Plotting only |

## What the simulation shows (simulated, ngspice-47)

Operating point: −2 V reference **−2.013 V**; node X **6.279 V** (+12 rail 11.281 V); charging current
**1.0024 mA**. Measured slopes match 1 mA/C: H ramp at R1 = 1.0 rises **17.8 mV/µs** (hand calc 17.9); V ramp
at R3 = 1.0 rises **66.5 mV/ms** (hand calc 66.8). Settled peaks also match slope × (period − sync width): NTSC V at
R3 = 1.0 is 1.103 V (hand 1.097). A first run that settled only one field showed V peaks 2–5 % high; that was a
start-up artifact and has been replaced.

All six outputs are positive-going sawtooth-family signals in roughly the 0–1 V LZX video range:

| Output | Shape | NTSC, trimmer 0.0 | NTSC, trimmer 0.5 | NTSC, trimmer 1.0 | PAL, trimmer 0.5 |
|---|---|---|---|---|---|
| J8 H ramp | Rises linearly over each line, snaps to ≈0 V at H sync, holds at 0 for the sync width, then rises again starting ≈5 µs after the sync edge. It peaks at the next sync edge | 0.001–0.349 V | 0.001–0.699 V | 0.002–1.048 V | 0.001–0.704 V |
| J6 H inverted ramp | 1.005 V − ramp: falls across the line | 0.658–1.006 V | 0.309–1.005 V | **−0.040**–1.004 V | 0.304–1.005 V |
| J10 H triangle | A "V": 2.004·\|ramp − 0.5 V\|. It reaches 0 V where the ramp crosses 0.5 V and returns toward 1 V at both ends. There is a brief dip to 0 V during the reset edge | 0.307–1.005 V, **never reaches 0**; minimum at line end | 0–1.004 V; dip at 47.3 µs | 0–**1.094** V; dip at 33.2 µs (about mid-line) | 0–1.004 V; dip at 47.3 µs |
| J3 V ramp | The same as J8, over one field. Reset is slower: the ramp is still 2–10 mV above 0 at 0.2 ms | 0.002–0.367 V | 0.004–0.735 V | 0.006–**1.103** V | 0.008–0.888 V |
| J2 V inverted ramp | 1.005 V − V ramp | 0.639–1.005 V | 0.272–1.003 V | **−0.096**–1.001 V | 0.118–0.998 V |
| J4 V triangle | A "V" across the field, plus a spike to 0 at V reset | 0.271–1.003 V, no zero | 0–0.999 V; dip at 11.4 ms | 0–**1.204** V; dip at 7.7 ms | 0–0.990 V; dip at 11.3 ms |

**How the trimmer acts (simulated):** R1 and R3 scale the ramp's height, from 0.333 to 1.0 of the capacitor
voltage. The slope changes and the timing does not. The inverted ramp is always anchored at 1.005 V, and the
triangle's zero point is always at 0.5 V of ramp. So the trimmer also **slides the triangle's dip along
the line (or field) and changes how lopsided it is**. The triangle is symmetric (centred and reaching
about 1 V at both ends) only when the ramp peak is calibrated to about 1.0 V. My reading is that this calibration is
what the trimmers are for, which is consistent with the calibration thread (/t/1152, not read in this task).

**How the sync standard acts (simulated):** with the same trimmer setting, the PAL V ramp is 21 % taller than
NTSC (0.888 vs 0.735 V at 0.5), because the capacitor charges for longer. The H ramp barely changes (0.704 vs 0.699 V).
From hand arithmetic on the simulated peaks (not separate runs), a 1.0 V peak needs these wiper positions:
R1 ≈ 0.93 (NTSC) or 0.92 (PAL); R3 ≈ 0.86 (NTSC) or 0.63 (PAL).

**Inference, not simulated:** the headroom is small. At R1 = 1.0, the nominal H ramp is only 4.8 % above 1 V, and C29 is ±10 %.
If C29 is 10 % high (61.6 nF, H peak ≈ 0.95 V), or C18 is 20 % high (18 µF, NTSC V peak ≈ 0.92 V), the ramp cannot reach 1.0 V,
and the triangle's dip moves away from centre. Unit-to-unit spread in real modules is therefore plausible.

## What the simulation cannot settle, and what a real unit or forum reading would add

- **Real edge and reset behaviour.** How fast the capacitor dumps, the pedestal left after reset, and the glitch
  on the triangle at each reset all depend on the BS170, LM6172 and 1N5711 models, which are generic. A scope trace of
  J8, J6 and J10 at the H sync edge from a real unit would settle this.
- **Offsets and absolute levels in mV.** There are no op-amp offsets or bias currents and no TL431 tolerance (the TL431B is ±0.5 %). Real
  outputs could sit a few to tens of mV away from these numbers. The 0 V floor and the 1.005 V ceiling of the inverted ramp need measuring.
- **What sync the Cadet I actually sends**: its polarity, pulse widths, whether H sync continues through vertical, and
  serration and equalising pulses (the J1 header also carries C Sync and Odd/Even, which are unused here). Interlace would make alternate
  fields 0.5 line different in length. The Cadet I schematic and `main.asm`, or a scope reading, would settle this.
- **Loading.** Outputs go through 499 R. Whether LZX inputs are high-impedance (no drop) or terminated changes the levels
  a patched module sees. The LZX patchable standard reference design (forum /t/1160) or a loaded measurement would settle this.
- **Where the trimmers are set on real units.** The factory or builder calibration procedure is in forum thread /t/1152. I did not read it
  in this task. It would say what peak voltage builders are told to set (1.0 V presumably), and on which standard.
- **Frequency response and noise:** not modelled at all.
- **Front-panel names of the six jacks.** The schematic does not label them. The panel artwork in lzxcadet `C4/Frontpanel` would.

## How to rerun

`sh sim/run.sh` runs 8 cases and takes about 30 s on the machine used. Decks and ngspice logs go to `<repo>/.scratch/cadet_iv_runs/`.
