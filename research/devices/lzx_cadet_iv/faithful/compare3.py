#!/usr/bin/env python3
"""Third run of the faithfulness check (T-0352): same inputs as compare.py, judged by the current ../FAITHFUL.md.

Usage: python3 -I compare3.py   (from anywhere; paths are relative to this file)
Standard library only. Reuses compare.py's readers, measures and PNG writer and compare2.py's label font; writes
try3_numbers.csv and try3_side_by_side.png here. Changes from try 2:
- row 2 judges the first pixel against the simulation's first pixel (the circuit's grey start), not against 0 V;
  T-0345's true-black rule is withdrawn on Hedawn's word (T-0350);
- row 1 judges every pixel again, the first included;
- the picture's labels are "reference (circuit)", "app today" and "difference".
"""
import csv
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)  # -I does not add the script folder; compare.py and compare2.py are ours, beside this file
import compare as c  # noqa: E402
import compare2 as c2  # noqa: E402

W, H = c.W, c.H

# FAITHFUL.md section 2. Every margin is a guess, untested on a screen.
TOL = {
    "max_abs_error": 0.010,         # row 1, level at every pixel, against the simulation
    "start_level": 0.010,           # row 2, first pixel against the simulation's first pixel
    "fit_gain_error": 0.01,         # row 3, slope
    "linearity_residual": 0.004,    # row 4, straightness
    "reset_in_picture_px": 1,       # row 7, resets
}


def main():
    c.self_test()
    c.app_shader_check()
    lt, lh = c.read_csv("line_ntsc_pos1.0.csv", "jh_ramp")
    ft, fv = c.read_csv("frame_ntsc_pos1.0.csv", "jv_ramp")
    h_ref = [c.interp(lt, lh, c.H_ACTIVE_START + (i + 0.5) / W * c.H_ACTIVE_LEN) for i in range(W)]
    v_ref = [c.interp(ft, fv, c.V_ACTIVE_START + (j + 0.5) / H * c.V_ACTIVE_LEN) for j in range(H)]
    h_app = [c.app_value(i, W) for i in range(W)]
    v_app = [c.app_value(j, H) for j in range(H)]

    res = {
        "H ramp (J8 vs mode 0)": (h_ref, h_app, c.measures(h_ref, h_app, W)),
        "V ramp (J3 vs mode 1)": (v_ref, v_app, c.measures(v_ref, v_app, H)),
    }

    out_csv = os.path.join(HERE, "try3_numbers.csv")
    with open(out_csv, "w", newline="") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["signal", "row", "measure", "value", "margin", "pass"])

        def put(sig, row, k, v, tol):
            ok = "" if tol is None else ("yes" if abs(v) <= tol else "NO")
            w.writerow([sig, row, k, f"{v:.4f}", "" if tol is None else tol, ok])
            print(f"{sig:24s} {row:3s} {k:28s} {v:+.4f}  margin {tol if tol is not None else '-':>6}  {ok}")

        for sig, (ref, app, m) in res.items():
            put(sig, "1", "max_abs_error", m["max_abs_error"], TOL["max_abs_error"])
            put(sig, "2", "start_level_error", ref[0] - app[0], TOL["start_level"])
            put(sig, "3", "fit_gain_error", m["fit_gain_error"], TOL["fit_gain_error"])
            put(sig, "4", "linearity_residual", m["linearity_residual"], TOL["linearity_residual"])
            put(sig, "7", "reset_in_picture_px", m["reset_in_picture_px"], TOL["reset_in_picture_px"])
            put(sig, "-", "ref_first_px", ref[0], None)
            put(sig, "-", "app_first_px", app[0], None)
            put(sig, "-", "rms_error", m["rms_error"], None)
            put(sig, "-", "fit_offset", m["fit_offset"], None)
            put(sig, "-", "ref_last_px", ref[-1], None)
            put(sig, "-", "app_last_px", app[-1], None)

    # Side-by-side with a label strip: columns reference (circuit) | app today | difference x4; rows H ramp, V ramp.
    gap, top = 8, 24
    pw = W * 3 + gap * 2
    ph = top + H * 2 + gap
    img = [[255] * pw for _ in range(ph)]
    labels = ["REFERENCE (CIRCUIT)", "APP TODAY", "DIFFERENCE"]
    for k, text in enumerate(labels):
        c2.draw_text(img, k * (W + gap) + 4, 5, text)
    rows = [(h_ref, h_app, lambda i, j: i), (v_ref, v_app, lambda i, j: j)]
    for row, (ref, app, idx) in enumerate(rows):
        y0 = top + row * (H + gap)
        for j in range(H):
            line = img[y0 + j]
            for i in range(W):
                r, a = ref[idx(i, j)], app[idx(i, j)]
                line[i] = round(255 * c.srgb(r))
                line[W + gap + i] = round(255 * c.srgb(a))
                line[2 * (W + gap) + i] = round(255 * min(1.0, max(0.0, 0.5 + 4 * (r - a))))
    c.write_png(os.path.join(HERE, "try3_side_by_side.png"), pw, ph, img)
    print("wrote", os.path.relpath(out_csv), "and try3_side_by_side.png")


if __name__ == "__main__":
    main()
