# Cadet IV: the app's real picture against the circuit (T-0375)

Written by `run_all.sh` through `faithful/compare_app.py`; do not edit by hand. Rule and margins: [`FAITHFUL.md`](../FAITHFUL.md) section 2 (every margin is a guess, untested on a screen).

- App side: the app's Cadet IV device, composed as the app composes it and rendered off-screen on the GPU at 720x240, read back as floats before any clamp or sRGB. GPU Apple M5 Max (Metal, IntegratedGpu, driver "").
- Reference side: the ngspice simulation in `results/`, sampled at each pixel centre.
- Checks judged: 90. Passed: 88. **Failed: 2.** Conditions not run: 1.

**Verdict: not a match.**

CPU model tests (`cargo test cadet_iv`): passed: test result: ok. 8 passed; 0 failed; 3 ignored; 0 measured; 8 filtered out

Reference document gate (`faithful/reference_check.txt`): 28 pass, 0 fail.

## Failures

| Condition | Row | Check | Measured | Margin | Note |
|---|---|---|---|---|---|
| ntsc trim 0.5 | 1 | J10 H triangle: largest level error, every pixel | +0.01566 | 0.01 | at x=521 y=0 |
| ntsc trim 1.0 | 1 | J10 H triangle: largest level error, every pixel | +0.01879 | 0.01 | at x=327 y=0 |

## Not run

- pal trim 0.5: the app's Cadet IV has NTSC timing only; PAL is in the CPU model, not the app.

## Every check

Measured is app minus circuit (for row 3, fit gain minus 1; for row 7, pixels). Headroom is margin minus |measured|; negative means it fails.

### ntsc trim 0.0

Picture: [`side_by_side_ntsc_pos0.0.png`](side_by_side_ntsc_pos0.0.png)

| Row | Check | Measured | Margin | Headroom | Result | Note |
|---|---|---|---|---|---|---|
| 1 | J8 H ramp: largest level error, every pixel | +0.00086 | 0.01 | +0.00914 | pass | at x=719 y=0 |
| 1 | J6 H inverted: largest level error, every pixel | -0.00216 | 0.01 | +0.00784 | pass | at x=718 y=0 |
| 1 | J10 H triangle: largest level error, every pixel | -0.00231 | 0.01 | +0.00769 | pass | at x=719 y=0 |
| 1 | J3 V ramp: largest level error, every pixel | +0.00047 | 0.01 | +0.00953 | pass | at x=0 y=239 |
| 1 | J2 V inverted: largest level error, every pixel | -0.00100 | 0.01 | +0.00900 | pass | at x=0 y=239 |
| 1 | J4 V triangle: largest level error, every pixel | -0.00146 | 0.01 | +0.00854 | pass | at x=0 y=239 |
| 2 | J8 H ramp: start level (circuit 0.0281) | +0.00008 | 0.01 | +0.00992 | pass |  |
| 2 | J6 H inverted: start level (circuit 0.9793) | -0.00138 | 0.01 | +0.00862 | pass |  |
| 2 | J3 V ramp: start level (circuit 0.0236) | -0.00020 | 0.01 | +0.00980 | pass |  |
| 2 | J2 V inverted: start level (circuit 0.9829) | -0.00032 | 0.01 | +0.00968 | pass |  |
| 3 | J8 H ramp: slope, fit gain error | -0.00247 | 0.01 | +0.00753 | pass |  |
| 4 | J8 H ramp: straightness, largest distance from fit | +0.00001 | 0.004 | +0.00399 | pass |  |
| 3 | J6 H inverted: slope, fit gain error | -0.00248 | 0.01 | +0.00752 | pass |  |
| 4 | J6 H inverted: straightness, largest distance from fit | +0.00001 | 0.004 | +0.00399 | pass |  |
| 3 | J3 V ramp: slope, fit gain error | -0.00199 | 0.01 | +0.00801 | pass |  |
| 4 | J3 V ramp: straightness, largest distance from fit | +0.00001 | 0.004 | +0.00399 | pass |  |
| 3 | J2 V inverted: slope, fit gain error | -0.00200 | 0.01 | +0.00800 | pass |  |
| 4 | J2 V inverted: straightness, largest distance from fit | +0.00001 | 0.004 | +0.00399 | pass |  |
| 5 | H ramp + inverted, every pixel | -0.00131 | 0.01 | +0.00869 | pass | at x=582 y=0 |
| 5 | V ramp + inverted, every pixel | -0.00052 | 0.01 | +0.00948 | pass | at x=0 y=66 |
| 6 | H triangle: ramp level at lowest pixel | +0.00086 | 0.01 | +0.00914 | pass | app col 719, circuit col 719 |
| 6 | H triangle: lowest level | -0.00231 | 0.01 | +0.00769 | pass | app 0.3232, circuit 0.3255 |
| 6 | V triangle: ramp level at lowest pixel | +0.00047 | 0.01 | +0.00953 | pass | app row 239, circuit row 239 |
| 6 | V triangle: lowest level | -0.00146 | 0.01 | +0.00854 | pass | app 0.2835, circuit 0.2850 |
| 7 | J8 H ramp: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J6 H inverted: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J10 H triangle: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J3 V ramp: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J2 V inverted: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J4 V triangle: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| - | J8 H ramp: RMS difference (not judged) | +0.00052 |  |  | info |  |
| - | J6 H inverted: RMS difference (not judged) | +0.00178 |  |  | info |  |
| - | J10 H triangle: RMS difference (not judged) | +0.00158 |  |  | info |  |
| - | J3 V ramp: RMS difference (not judged) | +0.00024 |  |  | info |  |
| - | J2 V inverted: RMS difference (not judged) | +0.00068 |  |  | info |  |
| - | J4 V triangle: RMS difference (not judged) | +0.00087 |  |  | info |  |

### ntsc trim 0.5

Picture: [`side_by_side_ntsc_pos0.5.png`](side_by_side_ntsc_pos0.5.png)

| Row | Check | Measured | Margin | Headroom | Result | Note |
|---|---|---|---|---|---|---|
| 1 | J8 H ramp: largest level error, every pixel | +0.00171 | 0.01 | +0.00829 | pass | at x=719 y=0 |
| 1 | J6 H inverted: largest level error, every pixel | -0.00381 | 0.01 | +0.00619 | pass | at x=717 y=0 |
| 1 | J10 H triangle: largest level error, every pixel | +0.01566 | 0.01 | -0.00566 | FAIL | at x=521 y=0 |
| 1 | J3 V ramp: largest level error, every pixel | +0.00095 | 0.01 | +0.00905 | pass | at x=0 y=239 |
| 1 | J2 V inverted: largest level error, every pixel | -0.00147 | 0.01 | +0.00853 | pass | at x=0 y=239 |
| 1 | J4 V triangle: largest level error, every pixel | +0.00243 | 0.01 | +0.00757 | pass | at x=0 y=239 |
| 2 | J8 H ramp: start level (circuit 0.0562) | +0.00016 | 0.01 | +0.00984 | pass |  |
| 2 | J6 H inverted: start level (circuit 0.9519) | -0.00225 | 0.01 | +0.00775 | pass |  |
| 2 | J3 V ramp: start level (circuit 0.0473) | -0.00040 | 0.01 | +0.00960 | pass |  |
| 2 | J2 V inverted: start level (circuit 0.9593) | -0.00012 | 0.01 | +0.00988 | pass |  |
| 3 | J8 H ramp: slope, fit gain error | -0.00247 | 0.01 | +0.00753 | pass |  |
| 4 | J8 H ramp: straightness, largest distance from fit | +0.00004 | 0.004 | +0.00396 | pass |  |
| 3 | J6 H inverted: slope, fit gain error | -0.00248 | 0.01 | +0.00752 | pass |  |
| 4 | J6 H inverted: straightness, largest distance from fit | +0.00003 | 0.004 | +0.00397 | pass |  |
| 3 | J3 V ramp: slope, fit gain error | -0.00199 | 0.01 | +0.00801 | pass |  |
| 4 | J3 V ramp: straightness, largest distance from fit | +0.00001 | 0.004 | +0.00399 | pass |  |
| 3 | J2 V inverted: slope, fit gain error | -0.00200 | 0.01 | +0.00800 | pass |  |
| 4 | J2 V inverted: straightness, largest distance from fit | +0.00001 | 0.004 | +0.00399 | pass |  |
| 5 | H ramp + inverted, every pixel | -0.00211 | 0.01 | +0.00789 | pass | at x=524 y=0 |
| 5 | V ramp + inverted, every pixel | -0.00053 | 0.01 | +0.00947 | pass | at x=0 y=200 |
| 6 | H triangle: ramp level at lowest pixel | -0.00396 | 0.01 | +0.00604 | pass | app col 512, circuit col 518 |
| 6 | H triangle: lowest level | +0.00052 | 0.01 | +0.00948 | pass | app 0.0000, circuit -0.0005 |
| 6 | V triangle: ramp level at lowest pixel | +0.00049 | 0.01 | +0.00951 | pass | app row 161, circuit row 161 |
| 6 | V triangle: lowest level | -0.00151 | 0.01 | +0.00849 | pass | app 0.0010, circuit 0.0025 |
| 7 | J8 H ramp: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J6 H inverted: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J10 H triangle: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J3 V ramp: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J2 V inverted: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J4 V triangle: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| - | J8 H ramp: RMS difference (not judged) | +0.00104 |  |  | info |  |
| - | J6 H inverted: RMS difference (not judged) | +0.00306 |  |  | info |  |
| - | J10 H triangle: RMS difference (not judged) | +0.00299 |  |  | info |  |
| - | J3 V ramp: RMS difference (not judged) | +0.00047 |  |  | info |  |
| - | J2 V inverted: RMS difference (not judged) | +0.00088 |  |  | info |  |
| - | J4 V triangle: RMS difference (not judged) | +0.00131 |  |  | info |  |

### ntsc trim 1.0

Picture: [`side_by_side_ntsc_pos1.0.png`](side_by_side_ntsc_pos1.0.png)

| Row | Check | Measured | Margin | Headroom | Result | Note |
|---|---|---|---|---|---|---|
| 1 | J8 H ramp: largest level error, every pixel | +0.00256 | 0.01 | +0.00744 | pass | at x=719 y=0 |
| 1 | J6 H inverted: largest level error, every pixel | -0.00545 | 0.01 | +0.00455 | pass | at x=719 y=0 |
| 1 | J10 H triangle: largest level error, every pixel | +0.01879 | 0.01 | -0.00879 | FAIL | at x=327 y=0 |
| 1 | J3 V ramp: largest level error, every pixel | +0.00142 | 0.01 | +0.00858 | pass | at x=0 y=239 |
| 1 | J2 V inverted: largest level error, every pixel | -0.00196 | 0.01 | +0.00804 | pass | at x=0 y=239 |
| 1 | J4 V triangle: largest level error, every pixel | +0.00340 | 0.01 | +0.00660 | pass | at x=0 y=239 |
| 2 | J8 H ramp: start level (circuit 0.0843) | +0.00024 | 0.01 | +0.00976 | pass |  |
| 2 | J6 H inverted: start level (circuit 0.9246) | -0.00312 | 0.01 | +0.00688 | pass |  |
| 2 | J3 V ramp: start level (circuit 0.0710) | -0.00060 | 0.01 | +0.00940 | pass |  |
| 2 | J2 V inverted: start level (circuit 0.9356) | +0.00008 | 0.01 | +0.00992 | pass |  |
| 3 | J8 H ramp: slope, fit gain error | -0.00247 | 0.01 | +0.00753 | pass |  |
| 4 | J8 H ramp: straightness, largest distance from fit | +0.00006 | 0.004 | +0.00394 | pass |  |
| 3 | J6 H inverted: slope, fit gain error | -0.00248 | 0.01 | +0.00752 | pass |  |
| 4 | J6 H inverted: straightness, largest distance from fit | +0.00002 | 0.004 | +0.00398 | pass |  |
| 3 | J3 V ramp: slope, fit gain error | -0.00199 | 0.01 | +0.00801 | pass |  |
| 4 | J3 V ramp: straightness, largest distance from fit | +0.00002 | 0.004 | +0.00398 | pass |  |
| 3 | J2 V inverted: slope, fit gain error | -0.00201 | 0.01 | +0.00799 | pass |  |
| 4 | J2 V inverted: straightness, largest distance from fit | +0.00002 | 0.004 | +0.00398 | pass |  |
| 5 | H ramp + inverted, every pixel | -0.00291 | 0.01 | +0.00709 | pass | at x=330 y=0 |
| 5 | V ramp + inverted, every pixel | -0.00054 | 0.01 | +0.00946 | pass | at x=0 y=236 |
| 6 | H triangle: ramp level at lowest pixel | -0.00396 | 0.01 | +0.00604 | pass | app col 320, circuit col 324 |
| 6 | H triangle: lowest level | +0.00164 | 0.01 | +0.00836 | pass | app 0.0010, circuit -0.0006 |
| 6 | V triangle: ramp level at lowest pixel | +0.00024 | 0.01 | +0.00976 | pass | app row 102, circuit row 102 |
| 6 | V triangle: lowest level | +0.00094 | 0.01 | +0.00906 | pass | app 0.0011, circuit 0.0001 |
| 7 | J8 H ramp: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J6 H inverted: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J10 H triangle: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J3 V ramp: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J2 V inverted: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J4 V triangle: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| - | J8 H ramp: RMS difference (not judged) | +0.00155 |  |  | info |  |
| - | J6 H inverted: RMS difference (not judged) | +0.00434 |  |  | info |  |
| - | J10 H triangle: RMS difference (not judged) | +0.00412 |  |  | info |  |
| - | J3 V ramp: RMS difference (not judged) | +0.00071 |  |  | info |  |
| - | J2 V inverted: RMS difference (not judged) | +0.00110 |  |  | info |  |
| - | J4 V triangle: RMS difference (not judged) | +0.00178 |  |  | info |  |

The pictures have three columns, labelled reference (circuit), app and difference. Rows are the six jacks in panel order. Difference is mid grey where they agree, scaled x25 (lighter: the circuit is higher). The reference and app columns are sRGB-encoded and clipped to 0-1 for display only; the numbers are unclamped.
