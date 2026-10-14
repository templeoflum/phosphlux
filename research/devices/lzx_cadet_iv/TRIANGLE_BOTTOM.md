# H triangle crossover (T-0432)

**The app changes.** The near-zero interval is a dynamic effect of the drawn U2/D5/D6
precision rectifier with capacitive diodes, rather than a pause imposed by the generic
op-amp's slew limit. Its exact duration is model-dependent. The evidence supports retaining
it in the nominal simulation reference; it does not establish the waveform of a real unit.

CIRCUIT.md stage 9 derives a sharp absolute value using ideal op-amps and ideal diode
switches. During the actual handover, U2.1 must move its output from positive to negative
to transfer conduction from D6 to D5. D5/D6 junction capacitance carries current during
that movement. Feedback through the diodes weakens near zero, the summing input moves
away from virtual ground, and J10 stays close to zero before the rising branch recovers.
The saved internal voltages in `triangle/profiles.csv` show this movement. Removing
capacitance removes the near-zero interval; increasing slew does not.

## Controlled evidence

`python3 research/devices/lzx_cadet_iv/triangle/probe.py` reruns the complete circuit with
one change at a time. All 22 sensitivity runs returned ngspice exit **0**. Measurements
use the settled visible rise, excluding reset. Width below means the time from first to
last sample with |J10| < 1 mV near the bottom, sampled at 20 ns. It is not the precise
width of a mathematically flat segment. Full numbers: `triangle/sensitivity.csv`.

| Change from reference | Width, trim 0.5 / 1.0 (µs) | Largest local difference from ideal V, trim 0.5 / 1.0 (mV) |
|---|---:|---:|
| Original model | 0.50 / 0.40 | 12.74 / 15.72 |
| Maximum timestep 50 → 5 ns | 0.50 / 0.40 | 12.76 / 15.75 |
| Trapezoidal integration, 5 ns | 0.50 / 0.40 | 12.76 / 15.73 |
| LM6172 slew 3000 → 30000 V/µs | 0.50 / 0.40 | 12.74 / 15.72 |
| LM6172 GBW 100 → 1000 MHz | 0.44 / 0.06 | 12.32 / 15.27 |
| LM6172 GBW 100 → 10 MHz | 0 / 0 | 21.26 / 26.49 |
| Diode Cjo 2 → 0 pF | 0 / 0 | 5.24 / 6.48 |
| Diode Cjo 2 → 1 pF | 0.30 / 0.22 | 9.46 / 11.70 |
| Diode Is ×100 | 0.34 / 0.26 | 8.91 / 11.19 |
| TI LM6172, nominal offset/bias removed | 0.38 / 0.28 | 12.28 / 15.19 |

The unchanged TI model lifts the minimum to about 6 mV and shifts its ramp location to
0.512–0.513 V. It therefore has no samples below 1 mV. Removing its offset-generating
RAX mismatch (1.005990 kΩ → 1 kΩ) and setting FIN1/FIN2 bias currents to zero retains
its amplifier dynamics and restores the near-zero interval. This isolates the behavior
under the reference's conditions, which omit offsets (FAITHFUL.md §3). It is a diagnostic
variant, not an assertion that a physical LM6172 has zero offset or bias.

ST's [1N5711 datasheet](https://www.st.com/resource/en/datasheet/1n5711.pdf), page 1,
gives **2 pF maximum** at zero reverse voltage, 1 MHz; it is not a typical capacitance
or a full SPICE parameter set. TI's [LM6172 page](https://www.ti.com/product/LM6172)
gives typical 100 MHz GBW and 3000 V/µs slew. Those values alone do not specify the
rectifier crossover. The manufacturer model was downloaded from
[TI SNOM234](https://www.ti.com/lit/zip/snom234); the unmodified `lm6172.lib` is retained
in `triangle/` with its redistribution notice. Library SHA-256:
`16b5a02e9ea470c9d06c7ced18dafbf2f83968cccf3d9e8bf031bc728dcb9c60`.

## App change and checks

The CPU and GPU use the same 9 × 121 table of **rectifier corrections**, interpolated
by H trim and ramp voltage relative to the triangle zero. `triangle/build_table.py`
generates it from the isolated U2/D5/D6 stage, transcribed directly from `sim/cadet_iv.lib`.
It sweeps nine ramp slopes over trims 0–1, retaining J10 minus stage 9's ideal absolute
value within ±30 mV of the zero. It does not store complete reference pictures or ramp
traces. Outside that interval the ideal transfer remains. The V crossover is below the
current comparison's resolution/margin; its ideal triangle transfer remains.

The capacitor model also now uses **0.99753 mA collector current**, keeping the source's
ideal 1 mA emitter value separately. CIRCUIT.md stage 4 explicitly distinguishes emitter
and collector current. A linear fit to `results/line_ntsc_pos1.0.csv` over 10–60 µs gives
17813.0095 V/s × 56 nF = 0.9975285 mA. This is a fit to the generic transistor reference,
not a measured hardware current. The previous ideal current advanced the rapid handover
by about one column: a table-only first attempt still failed at 13.52 / 15.80 mV.
Correcting the upstream charging assumption resolves that timing error for both ramps.

Three complete-circuit trims absent from the isolated-stage table were checked at all
720 H pixel centres: 0.3125, 0.5625 and 0.9375. Largest H triangle errors were
**3.749, 2.606 and 2.551 mV**, respectively (`triangle/held_out.csv`). These are CPU-formula
checks, separate from the GPU comparison.

`sh research/devices/lzx_cadet_iv/run_all.sh` returned exit **0**. The reference document
gate reported **28 pass, 0 fail**; CPU tests reported **8 passed, 0 failed**. The actual
Apple M5 Max GPU render reported **90 judged checks passed, 0 failed**. H triangle row 1
errors at trims 0.5 and 1.0 are **4.19 and 2.34 mV**, both within the unchanged 10 mV margin.
The rendered trim-1 comparison was inspected, and the saved float profiles confirm the
app's near-zero bottom. FAITHFUL.md and the reference netlist/results are unchanged.

PAL remains unrun in the app, already assigned to T-0433. No real Cadet IV was measured;
its diode capacitance, low-current diode behavior, offsets and crossover width remain
unknown. The app now matches the accepted nominal simulation rather than proving a
hardware match. No additional task is needed beyond the existing PAL comparison task.
