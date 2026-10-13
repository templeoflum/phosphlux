#!/usr/bin/env python3
"""One rough faithfulness check: Cadet IV ramps (T-0278 ngspice sim) vs the app's test-signal ramps.

Usage: python3 -I compare.py   (from anywhere; paths are relative to this file)
Standard library only. Reads ../results/{line,frame}_ntsc_pos1.0.csv and the app's
generators.rs / renderer.rs, writes try1_numbers.csv and try1_side_by_side.png here.
The rules applied are the ones in ../FAITHFUL.md.
"""
import csv
import os
import re
import struct
import sys
import zlib

HERE = os.path.dirname(os.path.abspath(__file__))
DEV = os.path.dirname(HERE)
REPO = os.path.join(DEV, "..", "..", "..")
APP = os.path.join(REPO, "apps", "phosphlux", "src")

# Picture grid (FAITHFUL.md "What is compared"): one field, 720 x 240.
W, H = 720, 240
# ASSUMED NTSC active picture (SMPTE 170M style numbers, not from the LZX docs).
LINE = 63.556e-6           # line period used by the T-0278 sim
H_ACTIVE_START = 9.4e-6     # sync leading edge -> first active sample (4.7 sync + 4.7 back porch)
H_ACTIVE_LEN = 52.66e-6
V_ACTIVE_START = 18 * LINE  # V sync leading edge -> first active line
V_ACTIVE_LEN = H * LINE
VOLTS_FULL_SCALE = 1.0      # LZX 0-1 V: 0 V -> 0.0 (black), 1 V -> 1.0 (white)

# Tolerances (FAITHFUL.md "How a match is judged").
TOL = {
    "max_abs_error": 0.02,
    "fit_gain_error": 0.02,
    "fit_offset": 0.02,
    "linearity_residual": 0.004,
    "reset_in_picture_px": 1,
}


def read_csv(name, col):
    with open(os.path.join(DEV, "results", name)) as f:
        rows = list(csv.DictReader(f))
    return [float(r["time_s"]) for r in rows], [float(r[col]) for r in rows]


def interp(ts, vs, t):
    # ts is sorted and evenly spaced
    i = int((t - ts[0]) / (ts[1] - ts[0]))
    i = max(0, min(len(ts) - 2, i))
    a = (t - ts[i]) / (ts[i + 1] - ts[i])
    return vs[i] + a * (vs[i + 1] - vs[i])


def app_shader_check():
    """Fail loudly if the app's ramp code no longer matches what is reproduced below."""
    gen = open(os.path.join(APP, "devices", "generators.rs")).read()
    comp = open(os.path.join(APP, "gpu", "renderer.rs")).read()
    need = [
        (gen, r"Mode 0: Horizontal ramp[\s\S]{0,80}vec3<f32>\(uv\.x\)"),
        (gen, r"Mode 1: Vertical ramp[\s\S]{0,80}vec3<f32>\(uv\.y\)"),
        (comp, r"out\.uv = vec2<f32>\(u, 1\.0 - v\)"),
    ]
    for text, pat in need:
        if not re.search(pat, text):
            sys.exit(f"app code changed; pattern not found: {pat}")


def app_value(i, n):
    # Fragment at pixel centre: uv = (i + 0.5) / n; uv.y = 0 at the top row.
    # Mode 0 returns uv.x, mode 1 returns uv.y (linear value, before the sRGB surface encode).
    return (i + 0.5) / n


def measures(ref, app, n):
    err = [r - a for r, a in zip(ref, app)]
    mx = sum(app) / n
    my = sum(ref) / n
    sxx = sum((x - mx) ** 2 for x in app)
    gain = sum((x - mx) * (y - my) for x, y in zip(app, ref)) / sxx
    offset = my - gain * mx
    resid = [y - (gain * x + offset) for x, y in zip(app, ref)]
    # A reset inside the picture shows as a drop between neighbouring pixels larger than half the range.
    ref_resets = [i for i in range(1, n) if ref[i - 1] - ref[i] > 0.5]
    app_resets = [i for i in range(1, n) if app[i - 1] - app[i] > 0.5]
    reset_px = 0 if ref_resets == app_resets else max(n, 1)
    return {
        "max_abs_error": max(abs(e) for e in err),
        "rms_error": (sum(e * e for e in err) / n) ** 0.5,
        "ref_first_px": ref[0], "app_first_px": app[0],
        "ref_last_px": ref[-1], "app_last_px": app[-1],
        "fit_gain_error": gain - 1.0,
        "fit_offset": offset,
        "linearity_residual": max(abs(r) for r in resid),
        "reset_in_picture_px": reset_px,
        "ref_resets": ref_resets, "app_resets": app_resets,
    }


def srgb(v):
    v = min(1.0, max(0.0, v))
    return 12.92 * v if v <= 0.0031308 else 1.055 * v ** (1 / 2.4) - 0.055


def write_png(path, w, h, rows):
    raw = b"".join(b"\x00" + bytes(r) for r in rows)

    def chunk(t, d):
        return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)

    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n")
        f.write(chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 0, 0, 0, 0)))
        f.write(chunk(b"IDAT", zlib.compress(raw, 9)))
        f.write(chunk(b"IEND", b""))


def main():
    app_shader_check()
    lt, lh = read_csv("line_ntsc_pos1.0.csv", "jh_ramp")
    ft, fv = read_csv("frame_ntsc_pos1.0.csv", "jv_ramp")

    h_ref = [interp(lt, lh, H_ACTIVE_START + (i + 0.5) / W * H_ACTIVE_LEN) / VOLTS_FULL_SCALE for i in range(W)]
    v_ref = [interp(ft, fv, V_ACTIVE_START + (j + 0.5) / H * V_ACTIVE_LEN) / VOLTS_FULL_SCALE for j in range(H)]
    h_app = [app_value(i, W) for i in range(W)]
    v_app = [app_value(j, H) for j in range(H)]

    res = {
        "H ramp (J8 vs mode 0)": measures(h_ref, h_app, W),
        "V ramp (J3 vs mode 1)": measures(v_ref, v_app, H),
    }

    out_csv = os.path.join(HERE, "try1_numbers.csv")
    keys = ["max_abs_error", "rms_error", "ref_first_px", "app_first_px", "ref_last_px", "app_last_px",
            "fit_gain_error", "fit_offset", "linearity_residual", "reset_in_picture_px"]
    with open(out_csv, "w", newline="") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["signal", "measure", "value", "tolerance", "pass"])
        for sig, m in res.items():
            for k in keys:
                tol = TOL.get(k)
                ok = "" if tol is None else ("yes" if abs(m[k]) <= tol else "NO")
                w.writerow([sig, k, f"{m[k]:.4f}", "" if tol is None else tol, ok])
                print(f"{sig:24s} {k:20s} {m[k]:+.4f}  tol {tol if tol is not None else '-':>6}  {ok}")

    # Side-by-side: row 1 H ramp, row 2 V ramp; columns reference | app | difference x4 (mid grey = 0, lighter = reference higher).
    gap = 8
    pw = W * 3 + gap * 2
    ph = H * 2 + gap
    img = [[255] * pw for _ in range(ph)]
    for row, (ref_at, app_at) in enumerate([
        (lambda i, j: h_ref[i], lambda i, j: h_app[i]),
        (lambda i, j: v_ref[j], lambda i, j: v_app[j]),
    ]):
        y0 = row * (H + gap)
        for j in range(H):
            line = img[y0 + j]
            for i in range(W):
                r, a = ref_at(i, j), app_at(i, j)
                line[i] = round(255 * srgb(r))
                line[W + gap + i] = round(255 * srgb(a))
                line[2 * (W + gap) + i] = round(255 * min(1.0, max(0.0, 0.5 + 4 * (r - a))))
    write_png(os.path.join(HERE, "try1_side_by_side.png"), pw, ph, img)
    print("wrote", os.path.relpath(out_csv), "and try1_side_by_side.png")


if __name__ == "__main__":
    main()
