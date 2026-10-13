#!/usr/bin/env python3
"""FAITHFUL.md's checks on the app's real Cadet IV picture (T-0375), against the simulated circuit.

Usage: python3 -I compare_app.py <dump dir> <out dir> [<cpu test log>]
Standard library only. <dump dir> holds the off-screen GPU render that the ignored Rust test
`cadet_iv_gpu_dump` writes (ntsc_pos<p>_<jack>.f32, 720x240 raw f32, and manifest.txt). The reference is
../results/{line,frame}_<std>_pos<p>.csv from the ngspice run, sampled at pixel centres as reference_check.py
does. Writes, in <out dir>: report.md, numbers.csv, app_profiles.csv and side_by_side_ntsc_pos<p>.png.
The rule and margins are FAITHFUL.md section 2's; they are copied here, not changed.
"""
import array
import csv
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)  # -I does not add the script folder; these modules are ours, beside this file
import compare as c  # noqa: E402
import compare2 as c2  # noqa: E402
import reference_check as rc  # noqa: E402

W, H = 720, 240
JACKS = [  # (app jack, sim vector, panel name), panel order (CIRCUIT.md stage 12)
    ("h_ramp", "jh_ramp", "J8 H ramp"),
    ("h_inv", "jh_inv", "J6 H inverted"),
    ("h_tri", "jh_tri", "J10 H triangle"),
    ("v_ramp", "jv_ramp", "J3 V ramp"),
    ("v_inv", "jv_inv", "J2 V inverted"),
    ("v_tri", "jv_tri", "J4 V triangle"),
]
CONDITIONS = [("ntsc", "0.0"), ("ntsc", "0.5"), ("ntsc", "1.0"), ("pal", "0.5")]  # FAITHFUL.md section 2
APP_STANDARDS = {"ntsc"}  # the app's Cadet IV runs NTSC timing only (app/README.md)

# FAITHFUL.md section 2. Every margin is a guess, untested on a screen.
LEVEL = 0.010      # rows 1, 2, 5, 6
SLOPE = 0.01       # row 3
STRAIGHT = 0.004   # row 4
RESET_PX = 1       # row 7

# Extra glyphs for the labels (compare2.FONT has the rest).
FONT = dict(c2.FONT)
FONT.update({
    "J": "..###|...#.|...#.|...#.|#..#.|#..#.|.##..", "B": "####.|#...#|#...#|####.|#...#|#...#|####.",
    "W": "#...#|#...#|#...#|#.#.#|#.#.#|##.##|#...#", "K": "#...#|#..#.|#.#..|##...|#.#..|#..#.|#...#",
    "0": ".###.|#...#|#..##|#.#.#|##..#|#...#|.###.", "1": "..#..|.##..|..#..|..#..|..#..|..#..|.###.",
    "2": ".###.|#...#|....#|...#.|..#..|.#...|#####", "3": "####.|....#|....#|.###.|....#|....#|####.",
    "5": "#####|#....|####.|....#|....#|#...#|.###.", "6": "..##.|.#...|#....|####.|#...#|#...#|.###.", "8": ".###.|#...#|#...#|.###.|#...#|#...#|.###.",
    ".": ".....|.....|.....|.....|.....|.##..|.##..", "-": ".....|.....|.....|.###.|.....|.....|.....",
    "+": ".....|..#..|..#..|#####|..#..|..#..|.....",
})
DIFF_GAIN = 25  # difference picture: mid grey = equal, +-0.02 V = white/black


def draw_text(img, x0, y0, text, scale=2):
    x = x0
    for ch in text:
        for r, bits in enumerate(FONT[ch].split("|")):
            for k, b in enumerate(bits):
                if b == "#":
                    for dy in range(scale):
                        for dx in range(scale):
                            img[y0 + r * scale + dy][x + k * scale + dx] = 0
        x += 6 * scale


def reference(std, pos):
    """Simulated jack voltages at pixel centres: H jacks per column, V jacks per row (reference_check.py)."""
    line, hs, hl, cols, vstart, rows = rc.WINDOWS[std]
    lt, L = rc.load(f"line_{std}_pos{pos}.csv")
    ft, F = rc.load(f"frame_{std}_pos{pos}.csv")
    out = {}
    for jack, vec, _ in JACKS:
        if jack.startswith("h"):
            out[jack] = [rc.interp(lt, L[vec], hs + (i + 0.5) / cols * hl) for i in range(cols)]
        else:
            out[jack] = [rc.interp(ft, F[vec], (vstart + j + 0.5) * line) for j in range(rows)]
    return out


def load_app(dump, pos, jack):
    a = array.array("f")
    with open(os.path.join(dump, f"ntsc_pos{pos}_{jack}.f32"), "rb") as f:
        a.frombytes(f.read())
    if sys.byteorder != "little":
        a.byteswap()
    if len(a) != W * H:
        sys.exit(f"{jack} pos {pos}: expected {W * H} floats, got {len(a)}")
    return a


def full_ref(jack, ref):
    """The reference broadcast to every pixel, row-major: H jacks vary by column, V jacks by row."""
    r = ref[jack]
    if jack.startswith("h"):
        return lambda i: r[i % W]
    return lambda i: r[i // W]


def profile(jack, img):
    """The app's 1D profile along the jack's axis (row 0 for H, column 0 for V)."""
    return list(img[:W]) if jack.startswith("h") else [img[j * W] for j in range(H)]


def judge(rows, cond, row, check, value, margin, note=""):
    ok = abs(value) <= margin
    rows.append({"condition": cond, "row": row, "check": check, "value": value, "margin": margin,
                 "headroom": margin - abs(value), "result": "pass" if ok else "FAIL", "note": note})


def run_condition(dump, std, pos, rows, profiles):
    cond = f"{std} trim {pos}"
    ref = reference(std, pos)
    app = {jack: load_app(dump, pos, jack) for jack, _, _ in JACKS}
    prof = {jack: profile(jack, app[jack]) for jack, _, _ in JACKS}
    profiles[pos] = (ref, prof)
    names = {jack: name for jack, _, name in JACKS}

    # Row 1: level at every pixel, all six outputs.
    for jack, _, name in JACKS:
        r = full_ref(jack, ref)
        img = app[jack]
        worst = max(range(W * H), key=lambda i: abs(img[i] - r(i)))
        judge(rows, cond, "1", f"{name}: largest level error, every pixel", img[worst] - r(worst), LEVEL,
              f"at x={worst % W} y={worst // W}")
    # Row 2: start level of each ramp (and inverted ramp), first column (H) or first row (V), whole edge.
    for jack in ("h_ramp", "h_inv", "v_ramp", "v_inv"):
        r, img = ref[jack][0], app[jack]
        edge = [img[j * W] for j in range(H)] if jack.startswith("h") else list(img[:W])
        d = max(edge, key=lambda v: abs(v - r)) - r
        judge(rows, cond, "2", f"{names[jack]}: start level (circuit {r:.4f})", d, LEVEL)
    # Rows 3, 4: slope and straightness of each straight output (compare.measures: ref fitted on app).
    for jack in ("h_ramp", "h_inv", "v_ramp", "v_inv"):
        n = W if jack.startswith("h") else H
        m = c.measures(ref[jack], prof[jack], n)
        judge(rows, cond, "3", f"{names[jack]}: slope, fit gain error", m["fit_gain_error"], SLOPE)
        judge(rows, cond, "4", f"{names[jack]}: straightness, largest distance from fit", m["linearity_residual"],
              STRAIGHT)
    # Row 5: ramp + inverted ramp at every pixel, against the reference's sum.
    for ch in ("h", "v"):
        rr, ri = full_ref(f"{ch}_ramp", ref), full_ref(f"{ch}_inv", ref)
        ar, ai = app[f"{ch}_ramp"], app[f"{ch}_inv"]
        worst = max(range(W * H), key=lambda i: abs(ar[i] + ai[i] - rr(i) - ri(i)))
        judge(rows, cond, "5", f"{ch.upper()} ramp + inverted, every pixel",
              ar[worst] + ai[worst] - rr(worst) - ri(worst), LEVEL, f"at x={worst % W} y={worst // W}")
    # Row 6: triangle bottom: the ramp's level at the triangle's lowest pixel, and the lowest level.
    for ch in ("h", "v"):
        rt, at = ref[f"{ch}_tri"], prof[f"{ch}_tri"]
        rm = min(range(len(rt)), key=lambda i: rt[i])
        am = min(range(len(at)), key=lambda i: at[i])
        axis = "col" if ch == "h" else "row"
        judge(rows, cond, "6", f"{ch.upper()} triangle: ramp level at lowest pixel",
              prof[f"{ch}_ramp"][am] - ref[f"{ch}_ramp"][rm], LEVEL, f"app {axis} {am}, circuit {axis} {rm}")
        judge(rows, cond, "6", f"{ch.upper()} triangle: lowest level", at[am] - rt[rm], LEVEL,
              f"app {at[am]:.4f}, circuit {rt[rm]:.4f}")
    # Row 7: resets inside the picture, all six outputs.
    for jack, _, name in JACKS:
        n = W if jack.startswith("h") else H
        rr, ar = c.find_resets(ref[jack]), c.find_resets(prof[jack])
        judge(rows, cond, "7", f"{name}: resets in picture (circuit {len(rr)}, app {len(ar)})",
              c.reset_distance(rr, ar, n), RESET_PX)
    # Not judged (FAITHFUL.md: RMS hides an offset), reported only.
    for jack, _, name in JACKS:
        r, img = full_ref(jack, ref), app[jack]
        rms = (sum((img[i] - r(i)) ** 2 for i in range(W * H)) / (W * H)) ** 0.5
        rows.append({"condition": cond, "row": "-", "check": f"{name}: RMS difference (not judged)", "value": rms,
                     "margin": "", "headroom": "", "result": "info", "note": ""})


def picture(path, std, pos, ref, prof):
    """Six rows (panel order) x three columns: reference (circuit) | app | difference."""
    lw, top, gap = 150, 40, 6
    pw = lw + W * 3 + gap * 2
    ph = top + (H + gap) * 6
    img = [[255] * pw for _ in range(ph)]
    draw_text(img, 4, 4, f"NTSC TRIM {pos}")
    for k, text in enumerate(["REFERENCE (CIRCUIT)", "APP", f"DIFFERENCE X{DIFF_GAIN}"]):
        draw_text(img, lw + k * (W + gap) + 4, 22, text)
    for row, (jack, _, name) in enumerate(JACKS):
        y0 = top + row * (H + gap)
        draw_text(img, 4, y0 + H // 2 - 7, name.upper().replace("INVERTED", "INV").replace("TRIANGLE", "TRI"))
        r, a = ref[jack], prof[jack]
        for j in range(H):
            line = img[y0 + j]
            for i in range(W):
                k = i if jack.startswith("h") else j
                rv, av = r[k], a[k]
                line[lw + i] = round(255 * c.srgb(rv))
                line[lw + W + gap + i] = round(255 * c.srgb(av))
                line[lw + 2 * (W + gap) + i] = round(255 * min(1.0, max(0.0, 0.5 + DIFF_GAIN * (rv - av))))
    c.write_png(path, pw, ph, img)


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    dump, out = sys.argv[1], sys.argv[2]
    cpu_log = sys.argv[3] if len(sys.argv) > 3 else None
    os.makedirs(out, exist_ok=True)
    c.self_test()

    rows, profiles = [], {}
    for std, pos in CONDITIONS:
        if std not in APP_STANDARDS:
            rows.append({"condition": f"{std} trim {pos}", "row": "all", "check": "every check", "value": "",
                         "margin": "", "headroom": "", "result": "NOT RUN",
                         "note": "the app's Cadet IV has NTSC timing only; PAL is in the CPU model, not the app"})
            continue
        run_condition(dump, std, pos, rows, profiles)
        ref, prof = profiles[pos]
        picture(os.path.join(out, f"side_by_side_{std}_pos{pos}.png"), std, pos, ref, prof)

    fmt = lambda v: f"{v:+.5f}" if isinstance(v, float) else str(v)
    with open(os.path.join(out, "numbers.csv"), "w", newline="") as f:
        w = csv.writer(f, lineterminator="\n")
        keys = ["condition", "row", "check", "value", "margin", "headroom", "result", "note"]
        w.writerow(keys)
        for r in rows:
            w.writerow([fmt(r[k]) if k in ("value", "headroom") else r[k] for k in keys])

    with open(os.path.join(out, "app_profiles.csv"), "w", newline="") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["condition", "jack", "axis", "index", "circuit_v", "app_v"])
        for pos, (ref, prof) in profiles.items():
            for jack, _, _ in JACKS:
                axis = "col" if jack.startswith("h") else "row"
                for i, (rv, av) in enumerate(zip(ref[jack], prof[jack])):
                    w.writerow([f"ntsc trim {pos}", jack, axis, i, f"{rv:.6f}", f"{av:.6f}"])

    judged = [r for r in rows if r["result"] in ("pass", "FAIL")]
    fails = [r for r in judged if r["result"] == "FAIL"]
    not_run = [r for r in rows if r["result"] == "NOT RUN"]
    manifest = open(os.path.join(dump, "manifest.txt")).read().splitlines()[0]
    lines = [
        "# Cadet IV: the app's real picture against the circuit (T-0375)",
        "",
        "Written by `run_all.sh` through `faithful/compare_app.py`; do not edit by hand. Rule and margins: "
        "[`FAITHFUL.md`](../FAITHFUL.md) section 2 (every margin is a guess, untested on a screen).",
        "",
        f"- App side: the app's Cadet IV device, composed as the app composes it and rendered off-screen on the GPU "
        f"at 720x240, read back as floats before any clamp or sRGB. GPU {manifest.split(': ', 1)[1]}.",
        "- Reference side: the ngspice simulation in `results/`, sampled at each pixel centre.",
        f"- Checks judged: {len(judged)}. Passed: {len(judged) - len(fails)}. **Failed: {len(fails)}.** "
        f"Conditions not run: {len(not_run)}.",
        "",
        "**Verdict: " + ("not a match" if fails else
                         ("every judged check passes, but not every condition was run, so not yet a match "
                          "by FAITHFUL.md" if not_run else "a match by FAITHFUL.md")) + ".**",
        "",
    ]
    if cpu_log:
        lines += [f"CPU model tests (`cargo test cadet_iv`): {open(cpu_log).read().strip()}", ""]
    gate = open(os.path.join(HERE, "reference_check.txt")).read()
    lines += [f"Reference document gate (`faithful/reference_check.txt`): "
              f"{gate.count('(pass')} pass, {gate.count('(FAIL')} fail.", ""]
    if fails:
        lines += ["## Failures", "", "| Condition | Row | Check | Measured | Margin | Note |", "|---|---|---|---|---|---|"]
        lines += [f"| {r['condition']} | {r['row']} | {r['check']} | {fmt(r['value'])} | {r['margin']} | {r['note']} |"
                  for r in fails]
        lines.append("")
    if not_run:
        lines += ["## Not run", ""] + [f"- {r['condition']}: {r['note']}." for r in not_run] + [""]
    lines += ["## Every check", "",
              "Measured is app minus circuit (for row 3, fit gain minus 1; for row 7, pixels). Headroom is margin "
              "minus |measured|; negative means it fails.", ""]
    for std, pos in CONDITIONS:
        cond = f"{std} trim {pos}"
        sub = [r for r in rows if r["condition"] == cond and r["result"] != "NOT RUN"]
        if not sub:
            continue
        lines += [f"### {cond}", "", f"Picture: [`side_by_side_{std}_pos{pos}.png`](side_by_side_{std}_pos{pos}.png)",
                  "", "| Row | Check | Measured | Margin | Headroom | Result | Note |", "|---|---|---|---|---|---|---|"]
        lines += [f"| {r['row']} | {r['check']} | {fmt(r['value'])} | {r['margin']} | {fmt(r['headroom'])} | "
                  f"{r['result']} | {r['note']} |" for r in sub]
        lines.append("")
    lines += ["The pictures have three columns, labelled reference (circuit), app and difference. Rows are the six "
              f"jacks in panel order. Difference is mid grey where they agree, scaled x{DIFF_GAIN} (lighter: the "
              "circuit is higher). The reference and app columns are sRGB-encoded and clipped to 0-1 for display "
              "only; the numbers are unclamped.", ""]
    with open(os.path.join(out, "report.md"), "w") as f:
        f.write("\n".join(lines))
    print("\n".join(lines[:12]))
    print(f"wrote {out}/report.md, numbers.csv, app_profiles.csv and side_by_side_*.png")


if __name__ == "__main__":
    main()
