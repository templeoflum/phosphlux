# Cadet IV: the app's real picture against the circuit (T-0375)

Written by `run_all.sh` through `faithful/compare_app.py`; do not edit by hand. Rule and margins: [`FAITHFUL.md`](../FAITHFUL.md) section 2 (every margin is a guess, untested on a screen).

- App side: the app's Cadet IV device, composed as the app composes it and rendered off-screen on the GPU at 720x240 NTSC and 720x288 PAL, read back as floats before any clamp or sRGB. GPU Apple M5 Max (Metal, IntegratedGpu, driver "").
- Reference side: the ngspice simulation in `results/`, sampled at each pixel centre.
- Checks judged: 120. Passed: 120. **Failed: 0.** Conditions not run: 0.

**Verdict: a match by FAITHFUL.md.**

CPU model tests (`cargo test cadet_iv`): passed: test result: ok. 8 passed; 0 failed; 3 ignored; 0 measured; 8 filtered out

Reference document gate (`faithful/reference_check.txt`): 28 pass, 0 fail.

## Every check

Measured is app minus circuit (for row 3, fit gain minus 1; for row 7, pixels). Headroom is margin minus |measured|; negative means it fails.

### ntsc trim 0.0

Picture: [`side_by_side_ntsc_pos0.0.png`](side_by_side_ntsc_pos0.0.png)

| Row | Check | Measured | Margin | Headroom | Result | Note |
|---|---|---|---|---|---|---|
| 1 | J8 H ramp: largest level error, every pixel | +0.00002 | 0.01 | +0.00998 | pass | at x=693 y=0 |
| 1 | J6 H inverted: largest level error, every pixel | -0.00132 | 0.01 | +0.00868 | pass | at x=716 y=0 |
| 1 | J10 H triangle: largest level error, every pixel | -0.00062 | 0.01 | +0.00938 | pass | at x=719 y=0 |
| 1 | J3 V ramp: largest level error, every pixel | -0.00042 | 0.01 | +0.00958 | pass | at x=0 y=232 |
| 1 | J2 V inverted: largest level error, every pixel | -0.00026 | 0.01 | +0.00974 | pass | at x=0 y=1 |
| 1 | J4 V triangle: largest level error, every pixel | +0.00033 | 0.01 | +0.00967 | pass | at x=0 y=237 |
| 2 | J8 H ramp: start level (circuit 0.0281) | +0.00001 | 0.01 | +0.00999 | pass |  |
| 2 | J6 H inverted: start level (circuit 0.9793) | -0.00131 | 0.01 | +0.00869 | pass |  |
| 2 | J3 V ramp: start level (circuit 0.0236) | -0.00026 | 0.01 | +0.00974 | pass |  |
| 2 | J2 V inverted: start level (circuit 0.9829) | -0.00026 | 0.01 | +0.00974 | pass |  |
| 3 | J8 H ramp: slope, fit gain error | -0.00000 | 0.01 | +0.01000 | pass |  |
| 4 | J8 H ramp: straightness, largest distance from fit | +0.00001 | 0.004 | +0.00399 | pass |  |
| 3 | J6 H inverted: slope, fit gain error | -0.00001 | 0.01 | +0.00999 | pass |  |
| 4 | J6 H inverted: straightness, largest distance from fit | +0.00001 | 0.004 | +0.00399 | pass |  |
| 3 | J3 V ramp: slope, fit gain error | +0.00048 | 0.01 | +0.00952 | pass |  |
| 4 | J3 V ramp: straightness, largest distance from fit | +0.00001 | 0.004 | +0.00399 | pass |  |
| 3 | J2 V inverted: slope, fit gain error | +0.00047 | 0.01 | +0.00953 | pass |  |
| 4 | J2 V inverted: straightness, largest distance from fit | +0.00001 | 0.004 | +0.00399 | pass |  |
| 5 | H ramp + inverted, every pixel | -0.00131 | 0.01 | +0.00869 | pass | at x=718 y=0 |
| 5 | V ramp + inverted, every pixel | -0.00052 | 0.01 | +0.00948 | pass | at x=0 y=98 |
| 6 | H triangle: ramp level at lowest pixel | +0.00001 | 0.01 | +0.00999 | pass | app col 719, circuit col 719 |
| 6 | H triangle: lowest level | -0.00062 | 0.01 | +0.00938 | pass | app 0.3249, circuit 0.3255 |
| 6 | V triangle: ramp level at lowest pixel | -0.00042 | 0.01 | +0.00958 | pass | app row 239, circuit row 239 |
| 6 | V triangle: lowest level | +0.00032 | 0.01 | +0.00968 | pass | app 0.2853, circuit 0.2850 |
| 7 | J8 H ramp: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J6 H inverted: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J10 H triangle: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J3 V ramp: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J2 V inverted: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J4 V triangle: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| - | J8 H ramp: RMS difference (not judged) | +0.00001 |  |  | info |  |
| - | J6 H inverted: RMS difference (not judged) | +0.00131 |  |  | info |  |
| - | J10 H triangle: RMS difference (not judged) | +0.00060 |  |  | info |  |
| - | J3 V ramp: RMS difference (not judged) | +0.00035 |  |  | info |  |
| - | J2 V inverted: RMS difference (not judged) | +0.00018 |  |  | info |  |
| - | J4 V triangle: RMS difference (not judged) | +0.00020 |  |  | info |  |

### ntsc trim 0.5

Picture: [`side_by_side_ntsc_pos0.5.png`](side_by_side_ntsc_pos0.5.png)

| Row | Check | Measured | Margin | Headroom | Result | Note |
|---|---|---|---|---|---|---|
| 1 | J8 H ramp: largest level error, every pixel | +0.00006 | 0.01 | +0.00994 | pass | at x=523 y=0 |
| 1 | J6 H inverted: largest level error, every pixel | -0.00214 | 0.01 | +0.00786 | pass | at x=524 y=0 |
| 1 | J10 H triangle: largest level error, every pixel | +0.00419 | 0.01 | +0.00581 | pass | at x=522 y=0 |
| 1 | J3 V ramp: largest level error, every pixel | -0.00083 | 0.01 | +0.00917 | pass | at x=0 y=234 |
| 1 | J2 V inverted: largest level error, every pixel | +0.00031 | 0.01 | +0.00969 | pass | at x=0 y=236 |
| 1 | J4 V triangle: largest level error, every pixel | -0.00114 | 0.01 | +0.00886 | pass | at x=0 y=238 |
| 2 | J8 H ramp: start level (circuit 0.0562) | +0.00002 | 0.01 | +0.00998 | pass |  |
| 2 | J6 H inverted: start level (circuit 0.9519) | -0.00211 | 0.01 | +0.00789 | pass |  |
| 2 | J3 V ramp: start level (circuit 0.0473) | -0.00051 | 0.01 | +0.00949 | pass |  |
| 2 | J2 V inverted: start level (circuit 0.9593) | -0.00000 | 0.01 | +0.01000 | pass |  |
| 3 | J8 H ramp: slope, fit gain error | -0.00000 | 0.01 | +0.01000 | pass |  |
| 4 | J8 H ramp: straightness, largest distance from fit | +0.00004 | 0.004 | +0.00396 | pass |  |
| 3 | J6 H inverted: slope, fit gain error | -0.00001 | 0.01 | +0.00999 | pass |  |
| 4 | J6 H inverted: straightness, largest distance from fit | +0.00003 | 0.004 | +0.00397 | pass |  |
| 3 | J3 V ramp: slope, fit gain error | +0.00048 | 0.01 | +0.00952 | pass |  |
| 4 | J3 V ramp: straightness, largest distance from fit | +0.00001 | 0.004 | +0.00399 | pass |  |
| 3 | J2 V inverted: slope, fit gain error | +0.00047 | 0.01 | +0.00953 | pass |  |
| 4 | J2 V inverted: straightness, largest distance from fit | +0.00001 | 0.004 | +0.00399 | pass |  |
| 5 | H ramp + inverted, every pixel | -0.00211 | 0.01 | +0.00789 | pass | at x=524 y=0 |
| 5 | V ramp + inverted, every pixel | -0.00053 | 0.01 | +0.00947 | pass | at x=0 y=220 |
| 6 | H triangle: ramp level at lowest pixel | +0.00001 | 0.01 | +0.00999 | pass | app col 518, circuit col 518 |
| 6 | H triangle: lowest level | -0.00000 | 0.01 | +0.01000 | pass | app -0.0005, circuit -0.0005 |
| 6 | V triangle: ramp level at lowest pixel | +0.00207 | 0.01 | +0.00793 | pass | app row 162, circuit row 161 |
| 6 | V triangle: lowest level | -0.00032 | 0.01 | +0.00968 | pass | app 0.0022, circuit 0.0025 |
| 7 | J8 H ramp: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J6 H inverted: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J10 H triangle: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J3 V ramp: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J2 V inverted: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J4 V triangle: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| - | J8 H ramp: RMS difference (not judged) | +0.00002 |  |  | info |  |
| - | J6 H inverted: RMS difference (not judged) | +0.00212 |  |  | info |  |
| - | J10 H triangle: RMS difference (not judged) | +0.00075 |  |  | info |  |
| - | J3 V ramp: RMS difference (not judged) | +0.00069 |  |  | info |  |
| - | J2 V inverted: RMS difference (not judged) | +0.00019 |  |  | info |  |
| - | J4 V triangle: RMS difference (not judged) | +0.00087 |  |  | info |  |

### ntsc trim 1.0

Picture: [`side_by_side_ntsc_pos1.0.png`](side_by_side_ntsc_pos1.0.png)

| Row | Check | Measured | Margin | Headroom | Result | Note |
|---|---|---|---|---|---|---|
| 1 | J8 H ramp: largest level error, every pixel | +0.00009 | 0.01 | +0.00991 | pass | at x=328 y=0 |
| 1 | J6 H inverted: largest level error, every pixel | -0.00294 | 0.01 | +0.00706 | pass | at x=330 y=0 |
| 1 | J10 H triangle: largest level error, every pixel | +0.00234 | 0.01 | +0.00766 | pass | at x=328 y=0 |
| 1 | J3 V ramp: largest level error, every pixel | -0.00126 | 0.01 | +0.00874 | pass | at x=0 y=239 |
| 1 | J2 V inverted: largest level error, every pixel | +0.00072 | 0.01 | +0.00928 | pass | at x=0 y=237 |
| 1 | J4 V triangle: largest level error, every pixel | -0.00196 | 0.01 | +0.00804 | pass | at x=0 y=239 |
| 2 | J8 H ramp: start level (circuit 0.0843) | +0.00003 | 0.01 | +0.00997 | pass |  |
| 2 | J6 H inverted: start level (circuit 0.9246) | -0.00291 | 0.01 | +0.00709 | pass |  |
| 2 | J3 V ramp: start level (circuit 0.0710) | -0.00077 | 0.01 | +0.00923 | pass |  |
| 2 | J2 V inverted: start level (circuit 0.9356) | +0.00025 | 0.01 | +0.00975 | pass |  |
| 3 | J8 H ramp: slope, fit gain error | -0.00000 | 0.01 | +0.01000 | pass |  |
| 4 | J8 H ramp: straightness, largest distance from fit | +0.00006 | 0.004 | +0.00394 | pass |  |
| 3 | J6 H inverted: slope, fit gain error | -0.00001 | 0.01 | +0.00999 | pass |  |
| 4 | J6 H inverted: straightness, largest distance from fit | +0.00002 | 0.004 | +0.00398 | pass |  |
| 3 | J3 V ramp: slope, fit gain error | +0.00048 | 0.01 | +0.00952 | pass |  |
| 4 | J3 V ramp: straightness, largest distance from fit | +0.00002 | 0.004 | +0.00398 | pass |  |
| 3 | J2 V inverted: slope, fit gain error | +0.00047 | 0.01 | +0.00953 | pass |  |
| 4 | J2 V inverted: straightness, largest distance from fit | +0.00002 | 0.004 | +0.00398 | pass |  |
| 5 | H ramp + inverted, every pixel | -0.00291 | 0.01 | +0.00709 | pass | at x=330 y=0 |
| 5 | V ramp + inverted, every pixel | -0.00054 | 0.01 | +0.00946 | pass | at x=0 y=236 |
| 6 | H triangle: ramp level at lowest pixel | +0.00001 | 0.01 | +0.00999 | pass | app col 324, circuit col 324 |
| 6 | H triangle: lowest level | -0.00002 | 0.01 | +0.00998 | pass | app -0.0006, circuit -0.0006 |
| 6 | V triangle: ramp level at lowest pixel | -0.00100 | 0.01 | +0.00900 | pass | app row 102, circuit row 102 |
| 6 | V triangle: lowest level | +0.00129 | 0.01 | +0.00871 | pass | app 0.0014, circuit 0.0001 |
| 7 | J8 H ramp: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J6 H inverted: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J10 H triangle: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J3 V ramp: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J2 V inverted: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J4 V triangle: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| - | J8 H ramp: RMS difference (not judged) | +0.00003 |  |  | info |  |
| - | J6 H inverted: RMS difference (not judged) | +0.00292 |  |  | info |  |
| - | J10 H triangle: RMS difference (not judged) | +0.00090 |  |  | info |  |
| - | J3 V ramp: RMS difference (not judged) | +0.00104 |  |  | info |  |
| - | J2 V inverted: RMS difference (not judged) | +0.00052 |  |  | info |  |
| - | J4 V triangle: RMS difference (not judged) | +0.00156 |  |  | info |  |

### pal trim 0.5

Picture: [`side_by_side_pal_pos0.5.png`](side_by_side_pal_pos0.5.png)

| Row | Check | Measured | Margin | Headroom | Result | Note |
|---|---|---|---|---|---|---|
| 1 | J8 H ramp: largest level error, every pixel | +0.00007 | 0.01 | +0.00993 | pass | at x=514 y=0 |
| 1 | J6 H inverted: largest level error, every pixel | -0.00214 | 0.01 | +0.00786 | pass | at x=516 y=0 |
| 1 | J10 H triangle: largest level error, every pixel | +0.00307 | 0.01 | +0.00693 | pass | at x=513 y=0 |
| 1 | J3 V ramp: largest level error, every pixel | +0.00074 | 0.01 | +0.00926 | pass | at x=0 y=0 |
| 1 | J2 V inverted: largest level error, every pixel | -0.00125 | 0.01 | +0.00875 | pass | at x=0 y=3 |
| 1 | J4 V triangle: largest level error, every pixel | -0.00197 | 0.01 | +0.00803 | pass | at x=0 y=0 |
| 2 | J8 H ramp: start level (circuit 0.0693) | +0.00002 | 0.01 | +0.00998 | pass |  |
| 2 | J6 H inverted: start level (circuit 0.9389) | -0.00211 | 0.01 | +0.00789 | pass |  |
| 2 | J3 V ramp: start level (circuit 0.0679) | +0.00074 | 0.01 | +0.00926 | pass |  |
| 2 | J2 V inverted: start level (circuit 0.9387) | -0.00125 | 0.01 | +0.00875 | pass |  |
| 3 | J8 H ramp: slope, fit gain error | -0.00000 | 0.01 | +0.01000 | pass |  |
| 4 | J8 H ramp: straightness, largest distance from fit | +0.00005 | 0.004 | +0.00395 | pass |  |
| 3 | J6 H inverted: slope, fit gain error | -0.00001 | 0.01 | +0.00999 | pass |  |
| 4 | J6 H inverted: straightness, largest distance from fit | +0.00002 | 0.004 | +0.00398 | pass |  |
| 3 | J3 V ramp: slope, fit gain error | +0.00045 | 0.01 | +0.00955 | pass |  |
| 4 | J3 V ramp: straightness, largest distance from fit | +0.00002 | 0.004 | +0.00398 | pass |  |
| 3 | J2 V inverted: slope, fit gain error | +0.00044 | 0.01 | +0.00956 | pass |  |
| 4 | J2 V inverted: straightness, largest distance from fit | +0.00002 | 0.004 | +0.00398 | pass |  |
| 5 | H ramp + inverted, every pixel | -0.00211 | 0.01 | +0.00789 | pass | at x=518 y=0 |
| 5 | V ramp + inverted, every pixel | -0.00053 | 0.01 | +0.00947 | pass | at x=594 y=165 |
| 6 | H triangle: ramp level at lowest pixel | +0.00001 | 0.01 | +0.00999 | pass | app col 509, circuit col 509 |
| 6 | H triangle: lowest level | -0.00000 | 0.01 | +0.01000 | pass | app -0.0005, circuit -0.0005 |
| 6 | V triangle: ramp level at lowest pixel | +0.00051 | 0.01 | +0.00949 | pass | app row 153, circuit row 153 |
| 6 | V triangle: lowest level | +0.00067 | 0.01 | +0.00933 | pass | app 0.0011, circuit 0.0005 |
| 7 | J8 H ramp: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J6 H inverted: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J10 H triangle: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J3 V ramp: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J2 V inverted: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| 7 | J4 V triangle: resets in picture (circuit 0, app 0) | 0 | 1 | 1 | pass |  |
| - | J8 H ramp: RMS difference (not judged) | +0.00002 |  |  | info |  |
| - | J6 H inverted: RMS difference (not judged) | +0.00212 |  |  | info |  |
| - | J10 H triangle: RMS difference (not judged) | +0.00075 |  |  | info |  |
| - | J3 V ramp: RMS difference (not judged) | +0.00054 |  |  | info |  |
| - | J2 V inverted: RMS difference (not judged) | +0.00106 |  |  | info |  |
| - | J4 V triangle: RMS difference (not judged) | +0.00160 |  |  | info |  |

The pictures have three columns, labelled reference (circuit), app and difference. Rows are the six jacks in panel order. Difference is mid grey where they agree, scaled x25 (lighter: the circuit is higher). The reference and app columns are sRGB-encoded and clipped to 0-1 for display only; the numbers are unclamped.
