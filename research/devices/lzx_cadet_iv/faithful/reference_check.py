#!/usr/bin/env python3
"""Reference values for FAITHFUL.md (T-0293), read from the committed T-0278 simulation.

Usage: python3 -I reference_check.py   (standard library only; writes reference_check.txt here)
For each run in ../results it samples all six outputs at the pixel centres of the picture window
defined in FAITHFUL.md, then prints the quantities FAITHFUL.md compares, and checks the simulation
against the values derived from LZX's documents in ../CIRCUIT.md (the "document gate").
"""
import csv
import os

HERE = os.path.dirname(os.path.abspath(__file__))
RES = os.path.join(HERE, "..", "results")

# Picture windows (FAITHFUL.md, "Where the picture is"). ASSUMED, not from the LZX documents.
WINDOWS = {
    #       line period, H start after sync edge, H length, columns, V start (lines after V sync edge), rows
    "ntsc": (63.556e-6, 9.4e-6, 52.66e-6, 720, 18, 240),
    "pal": (64.0e-6, 10.5e-6, 52.0e-6, 720, 23, 288),
}
RUNS = [("ntsc", "0.0"), ("ntsc", "0.5"), ("ntsc", "1.0"), ("pal", "0.5")]

# Derived from the documents (../CIRCUIT.md stages 4, 5, 8, 9; nominal parts, typical TL431 Vref).
SLOPE_H = 17.86e-3 / 1e-6  # V/s at trimmer fraction 1.0 (1 mA / 56 nF)
SLOPE_V = 66.7e-3 / 1e-3   # V/s at trimmer fraction 1.0 (1 mA / 15 uF)
INV_CEILING = 1.006        # ramp + inverted ramp
TRI_ZERO = 0.502           # ramp level where the triangle reaches 0
LEVEL_MARGIN = 0.010
SLOPE_MARGIN = 0.01


def load(name):
    with open(os.path.join(RES, name)) as f:
        rows = list(csv.DictReader(f))
    t = [float(r["time_s"]) for r in rows]
    return t, {k: [float(r[k]) for r in rows] for k in rows[0] if k != "time_s"}


def interp(ts, vs, x):
    dt = ts[1] - ts[0]
    i = max(0, min(len(ts) - 2, int((x - ts[0]) / dt)))
    a = (x - ts[i]) / dt
    return vs[i] + a * (vs[i + 1] - vs[i])


def fit(y):
    n = len(y)
    xs = [(i + 0.5) / n for i in range(n)]
    mx, my = sum(xs) / n, sum(y) / n
    b = sum((x - mx) * (v - my) for x, v in zip(xs, y)) / sum((x - mx) ** 2 for x in xs)
    a = my - b * mx
    return a, b, max(abs(v - (a + b * x)) for x, v in zip(xs, y))


def trim_fraction(pos):
    # Wiper position p (0 = R2 end) -> fraction of the capacitor voltage (CIRCUIT.md stage 6).
    return (4.99 + 10.0 * float(pos)) / 14.99


def main():
    out = []
    for std, pos in RUNS:
        line, hs, hl, cols, vstart, rows = WINDOWS[std]
        lt, L = load(f"line_{std}_pos{pos}.csv")
        ft, F = load(f"frame_{std}_pos{pos}.csv")
        k = trim_fraction(pos)
        chans = {
            "H": ([interp(lt, L[c], hs + (i + 0.5) / cols * hl) for i in range(cols)]
                  for c in ("jh_ramp", "jh_inv", "jh_tri")),
            "V": ([interp(ft, F[c], (vstart + j + 0.5) * line) for j in range(rows)]
                  for c in ("jv_ramp", "jv_inv", "jv_tri")),
        }
        for ch, gen in chans.items():
            ramp, inv, tri = gen
            n = len(ramp)
            span = hl if ch == "H" else rows * line
            a, b, res = fit(ramp)
            slope = b / span
            want = (SLOPE_H if ch == "H" else SLOPE_V) * k
            s = [x + y for x, y in zip(ramp, inv)]
            m = min(range(n), key=lambda i: tri[i])
            resets = sum(1 for i in range(1, n) if ramp[i - 1] - ramp[i] > 0.5)
            def judged(name, val, lim):
                return f"{name} {val:+.4f} ({'pass' if abs(val) <= lim else 'FAIL'} at {lim})"

            gate = [
                judged("slope", slope / want - 1, SLOPE_MARGIN),
                judged("ramp+inv", max(abs(v - INV_CEILING) for v in s), LEVEL_MARGIN),
            ]
            # The triangle-zero check applies when the ramp crosses the derived zero inside the picture.
            # Applicability comes from the ramp's range, not from the triangle, so a bad triangle cannot hide it.
            if min(ramp) <= TRI_ZERO <= max(ramp):
                if tri[m] > LEVEL_MARGIN:
                    gate.append(f"tri zero FAIL: ramp crosses {TRI_ZERO} V but the triangle's lowest level is {tri[m]:.4f}")
                else:
                    gate.append(judged("tri zero", ramp[m] - TRI_ZERO, LEVEL_MARGIN))
            else:
                gate.append(f"tri zero n/a: ramp spans {min(ramp):.4f}..{max(ramp):.4f} V, never reaching {TRI_ZERO} V")
            out.append(
                f"{std} trim {pos} (k={k:.3f}) {ch}: ramp first {ramp[0]:.4f} last {ramp[-1]:.4f}; "
                f"slope {slope * (1e-6 if ch == 'H' else 1e-3) * 1e3:.2f} mV/{'us' if ch == 'H' else 'ms'}; "
                f"straightness {res:.5f}; ramp+inv {min(s):.4f}..{max(s):.4f}; "
                f"tri min {tri[m]:.4f} at {'col' if ch == 'H' else 'row'} {m} (ramp {ramp[m]:.4f}), "
                f"ends {tri[0]:.4f}/{tri[-1]:.4f}; resets in picture {resets}"
            )
            out.append("    document gate: " + "; ".join(gate))
    text = "\n".join(out) + "\n"
    with open(os.path.join(HERE, "reference_check.txt"), "w") as f:
        f.write(text)
    print(text, end="")


if __name__ == "__main__":
    main()
