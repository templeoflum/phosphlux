#!/usr/bin/env python3
"""Second run of the faithfulness check (T-0345): same inputs as compare.py, judged by the current ../FAITHFUL.md.

Usage: python3 -I compare2.py   (from anywhere; paths are relative to this file)
Standard library only. Reuses compare.py's readers, measures and PNG writer; writes try2_numbers.csv and
try2_side_by_side.png here. Changes from try 1:
- margins are the T-0293 ones (level 0.010, slope +/-1 %, straightness 0.004, resets +/-1 px);
- row 2 is now "black level": the first pixel of each ramp is judged against 0 V (true black), not against the
  simulation, on Hedawn's word (T-0327). Its margin, 0.005, is a placeholder guess;
- the picture has labels, so the simulated circuit and the app cannot be mistaken for each other.
"""
import csv
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)  # -I does not add the script folder; compare.py is ours, beside this file
import compare as c  # noqa: E402

W, H = c.W, c.H

# FAITHFUL.md section 2 (T-0293 margins; row 2 from T-0345).
TOL = {
    "max_abs_error": 0.010,         # row 1, level at every pixel, against the simulation
    "black_level": 0.005,           # row 2, first pixel against 0 V. GUESS, placeholder (T-0345)
    "fit_gain_error": 0.01,         # row 3, slope
    "linearity_residual": 0.004,    # row 4, straightness
    "reset_in_picture_px": 1,       # row 7, resets
}

# 5x7 capitals for the labels; '#' is a lit pixel.
FONT = {
    "A": ".###.|#...#|#...#|#####|#...#|#...#|#...#", "C": ".###.|#...#|#....|#....|#....|#...#|.###.",
    "D": "####.|#...#|#...#|#...#|#...#|#...#|####.", "E": "#####|#....|#....|####.|#....|#....|#####",
    "F": "#####|#....|#....|####.|#....|#....|#....", "I": "#####|..#..|..#..|..#..|..#..|..#..|#####",
    "L": "#....|#....|#....|#....|#....|#....|#####", "M": "#...#|##.##|#.#.#|#.#.#|#...#|#...#|#...#",
    "N": "#...#|##..#|#.#.#|#..##|#...#|#...#|#...#", "P": "####.|#...#|#...#|####.|#....|#....|#....",
    "R": "####.|#...#|#...#|####.|#.#..|#..#.|#...#", "S": ".####|#....|#....|.###.|....#|....#|####.",
    "T": "#####|..#..|..#..|..#..|..#..|..#..|..#..", "U": "#...#|#...#|#...#|#...#|#...#|#...#|.###.",
    "X": "#...#|#...#|.#.#.|..#..|.#.#.|#...#|#...#", "4": "...#.|..##.|.#.#.|#..#.|#####|...#.|...#.",
    "G": ".###.|#...#|#....|#.###|#...#|#...#|.###.", "O": ".###.|#...#|#...#|#...#|#...#|#...#|.###.",
    "Y": "#...#|#...#|.#.#.|..#..|..#..|..#..|..#..", "Q": ".###.|#...#|#...#|#...#|#.#.#|#..#.|.##.#",
    "=": ".....|.....|#####|.....|#####|.....|.....",
    "H": "#...#|#...#|#...#|#####|#...#|#...#|#...#", "V": "#...#|#...#|#...#|#...#|#...#|.#.#.|..#..",
    "(": "..#..|.#...|#....|#....|#....|.#...|..#..", ")": "..#..|...#.|....#|....#|....#|...#.|..#..",
    ",": ".....|.....|.....|.....|.....|..#..|.#...", " ": ".....|.....|.....|.....|.....|.....|.....",
}


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


def main():
    c.self_test()
    c.app_shader_check()
    lt, lh = c.read_csv("line_ntsc_pos1.0.csv", "jh_ramp")
    ft, fv = c.read_csv("frame_ntsc_pos1.0.csv", "jv_ramp")
    h_ref = [c.interp(lt, lh, c.H_ACTIVE_START + (i + 0.5) / W * c.H_ACTIVE_LEN) for i in range(W)]
    v_ref = [c.interp(ft, fv, c.V_ACTIVE_START + (j + 0.5) / H * c.V_ACTIVE_LEN) for j in range(H)]
    h_app = [c.app_value(i, W) for i in range(W)]
    v_app = [c.app_value(j, H) for j in range(H)]

    # Where the grey start comes from: the ramp level at the end of reset, and its rise until the picture begins.
    sync_end_h = c.interp(lt, lh, 4.7e-6)
    sync_end_v = c.interp(ft, fv, 3 * c.LINE)
    print(f"H ramp at end of H sync (4.7 us): {sync_end_h:.4f} V; at picture start (9.4 us): "
          f"{c.interp(lt, lh, c.H_ACTIVE_START):.4f} V")
    print(f"V ramp at end of V sync (3 lines): {sync_end_v:.4f} V; at picture start (18 lines): "
          f"{c.interp(ft, fv, c.V_ACTIVE_START):.4f} V")

    res = {
        "H ramp (J8 vs mode 0)": (h_ref, h_app, c.measures(h_ref, h_app, W)),
        "V ramp (J3 vs mode 1)": (v_ref, v_app, c.measures(v_ref, v_app, H)),
    }

    out_csv = os.path.join(HERE, "try2_numbers.csv")
    with open(out_csv, "w", newline="") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["signal", "row", "measure", "value", "margin", "pass"])

        def put(sig, row, k, v, tol):
            ok = "" if tol is None else ("yes" if abs(v) <= tol else "NO")
            w.writerow([sig, row, k, f"{v:.4f}", "" if tol is None else tol, ok])
            print(f"{sig:24s} {row:3s} {k:26s} {v:+.4f}  margin {tol if tol is not None else '-':>6}  {ok}")

        for sig, (ref, app, m) in res.items():
            put(sig, "1", "max_abs_error", m["max_abs_error"], TOL["max_abs_error"])
            put(sig, "2", "black_level_app_first_px", app[0], TOL["black_level"])
            put(sig, "2", "black_level_ref_first_px", ref[0], TOL["black_level"])
            put(sig, "3", "fit_gain_error", m["fit_gain_error"], TOL["fit_gain_error"])
            put(sig, "4", "linearity_residual", m["linearity_residual"], TOL["linearity_residual"])
            put(sig, "7", "reset_in_picture_px", m["reset_in_picture_px"], TOL["reset_in_picture_px"])
            put(sig, "-", "rms_error", m["rms_error"], None)
            put(sig, "-", "fit_offset", m["fit_offset"], None)
            put(sig, "-", "ref_last_px", ref[-1], None)
            put(sig, "-", "app_last_px", app[-1], None)

    # Side-by-side with a label strip: columns simulated circuit | app | difference x4; rows H ramp, V ramp.
    gap, top = 8, 24
    pw = W * 3 + gap * 2
    ph = top + H * 2 + gap
    img = [[255] * pw for _ in range(ph)]
    labels = ["SIMULATED CIRCUIT (REFERENCE)", "APP", "DIFFERENCE X4, MID GREY = EQUAL"]
    for k, text in enumerate(labels):
        draw_text(img, k * (W + gap) + 4, 5, text)
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
    c.write_png(os.path.join(HERE, "try2_side_by_side.png"), pw, ph, img)
    print("wrote", os.path.relpath(out_csv), "and try2_side_by_side.png")


if __name__ == "__main__":
    main()
