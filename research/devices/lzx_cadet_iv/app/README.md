# Cadet IV in the app: first pass (T-0374)

`cadet_iv_six_jacks.png` is the app's Cadet IV device rendered off-screen on the GPU. It is not a
window screenshot. The patch is composed the way the app composes it (`ShaderComposer::compose_plan`),
and the frame is drawn at 720×240, the NTSC window that FAITHFUL.md §2 assumes. Both trimmers are at 1.0.
The six jacks are laid out in panel order:

| | | |
|---|---|---|
| J8 H ramp | J6 H inverted | J10 H triangle |
| J3 V ramp | J2 V inverted | J4 V triangle |

The values are read back as floats and then sRGB-encoded, which is how the app's sRGB surface shows them.
Values outside 0–1 are clipped in the PNG only.

To make it again (needs a GPU):

    CADET_IV_FRAME=research/devices/lzx_cadet_iv/app/cadet_iv_six_jacks.png \
      cargo test cadet_iv_gpu_frame -- --ignored --nocapture

The same test checks that the GPU agrees with the CPU model at every pixel (to within 1e-4). The CPU model's own
tests check it against `faithful/reference_check.txt` within FAITHFUL.md's margins. The device is
`apps/phosphlux/src/devices/lzx/cadet_iv.rs`, and every formula there cites its CIRCUIT.md stage.

What is not from LZX:
- The sync widths and the picture window. These are assumed, as in the simulation and in FAITHFUL.md §2.
- The reset switch's 2.33 Ω. It is fitted to the simulation's V-ramp discharge (CIRCUIT.md Q11: not stated).
- PAL uses the assumed timing constants in `cadet_iv.rs`, with a 720×288 picture window.

Patching a jack into a scan modulator (for example the Wobulator, which runs in its own pass) hands on
that jack, not the first one (fixed after the T-0374 check). `each_jack_reaches_a_scan_modulator` checks
the composed passes. `cadet_iv_gpu_into_wobulator` (ignored, needs a GPU) reads back the texture the
Wobulator samples for each of the six jacks.

## Timing (T-0433)

The app device exposes **Timing**, an NTSC / PAL switch (stored as 0 / 1), alongside the two trimmers.
Values below 0.5 select NTSC; values at or above 0.5 select PAL. NTSC is the default.
This setting represents the sync standard supplied to the module, not a physical panel control.
Both standards use the same circuit constants and the unchanged H-triangle crossover correction.

`../run_all.sh` renders all six jacks at NTSC trims 0.0, 0.5, 1.0 and PAL trim 0.5.
The GPU dump checks every pixel against the CPU model before the circuit comparison.
PAL floats have 720×288 pixels; NTSC floats have 720×240 pixels.
The comparison retains separate profiles and pictures for each standard and trim.
