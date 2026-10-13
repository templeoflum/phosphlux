#!/usr/bin/env python3
"""Run the Cadet IV ngspice probe and write CSVs, SVG plots and a summary.

Usage: python3 -I run_sims.py   (from anywhere; paths are relative to this file)
Needs ngspice on PATH (tested with ngspice-47). Uses only the standard library.
"""
import csv
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
RESULTS = os.path.join(ROOT, "results")
WORK = os.path.join(ROOT, "..", "..", "..", ".scratch", "cadet_iv_runs")

# ASSUMED sync timing (separate H and V sync from the Cadet I header J1;
# H sync keeps running at line rate through vertical sync; no equalising pulses).
STANDARDS = {
    "ntsc": dict(line=63.556e-6, hsw=4.7e-6, field=16.6833e-3, vsw=3 * 63.556e-6),
    "pal": dict(line=64.0e-6, hsw=4.7e-6, field=20.0e-3, vsw=2.5 * 64.0e-6),
}

CASES = [  # (standard, trimmer position)
    ("ntsc", 0.0),
    ("ntsc", 0.5),
    ("ntsc", 1.0),
    ("pal", 0.5),
]

OUTS = [  # (vector, label)
    ("jh_ramp", "J8 H ramp"),
    ("jh_inv", "J6 H inverted ramp"),
    ("jh_tri", "J10 H triangle"),
    ("jv_ramp", "J3 V ramp"),
    ("jv_inv", "J2 V inverted ramp"),
    ("jv_tri", "J4 V triangle"),
]


def deck(std, pos, view):
    s = STANDARDS[std]
    if view == "line":
        # settle 15 lines, then keep 2 lines starting at a sync edge
        t0 = 15 * s["line"]
        tstop = t0 + 2 * s["line"]
        tran = f".tran 20n {tstop:.6e} {t0:.6e} 50n"
    else:
        # two full fields (one frame) plus 1 ms, after two fields of settling,
        # starting at a V sync edge
        t0 = 2 * s["field"]
        tstop = t0 + 2 * s["field"] + 1e-3
        tran = f".tran 5u {tstop:.6e} {t0:.6e} 1u"
    out = os.path.join(WORK, f"{view}_{std}_pos{pos:.1f}.dat")
    vecs = " ".join(f"v({v})" for v, _ in OUTS)
    return out, f"""* Cadet IV probe {view} {std} pos={pos}
.include {os.path.join(HERE, 'cadet_iv.lib')}
XPWR vcc vee POWER
XREF vcc vee m2 MINUS2REF
* _HSYNC / _VSYNC as 74HC14 outputs on +5 V (7805), high during the sync pulse
VHS hsn 0 PULSE(0 5 0 10n 10n {s['hsw']:.6e} {s['line']:.6e})
VVS vsn 0 PULSE(0 5 0 10n 10n {s['vsw']:.6e} {s['field']:.6e})
XH vcc vee m2 hsn jh_ramp jh_inv jh_tri RAMPCH cramp=56n pos={pos}
XV vcc vee m2 vsn jv_ramp jv_inv jv_tri RAMPCH cramp=15u pos={pos}
.options method=gear reltol=1e-4
{tran}
.control
set wr_singlescale
set wr_vecnames
run
print v(m2)[0] v(xh.nx)[0] v(xh.em)[0] v(vcc)[0] v(vee)[0]
let ich = (v(vcc)-v(xh.em))/4.99k
print ich[0]
linearize
wrdata {out} {vecs}
.endc
.end
"""


def run(std, pos, view):
    os.makedirs(WORK, exist_ok=True)
    out, text = deck(std, pos, view)
    cir = out.replace(".dat", ".cir")
    with open(cir, "w") as f:
        f.write(text)
    p = subprocess.run(["ngspice", "-b", cir], capture_output=True, text=True)
    with open(out.replace(".dat", ".log"), "w") as f:
        f.write(p.stdout + p.stderr)
    if p.returncode != 0 or not os.path.exists(out):
        sys.exit(f"ngspice failed for {cir} (exit {p.returncode}); see log")
    ops = [l for l in p.stdout.splitlines() if "[0]" in l or l.strip().startswith("ich")]
    rows = []
    with open(out) as f:
        header = f.readline().split()
        for line in f:
            rows.append([float(x) for x in line.split()])
    return header, rows, ops


def write_csv(path, header, rows, t0):
    names = ["time_s"] + [v for v, _ in OUTS]
    with open(path, "w", newline="") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(names)
        for r in rows:
            w.writerow([f"{r[0] - t0:.9e}"] + [f"{x:.5f}" for x in r[1:]])


def svg_plot(path, title, xs, series, xlabel, xscale):
    W, H = 900, 140 * len(series) + 60
    ml, mr, mt, ph = 70, 20, 40, 110
    x0, x1 = xs[0], xs[-1]
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" '
           f'font-family="sans-serif" font-size="12">',
           f'<rect width="{W}" height="{H}" fill="white"/>',
           f'<text x="{ml}" y="22" font-size="15">{title}</text>']
    for i, (label, ys) in enumerate(series):
        top = mt + i * 140
        lo, hi = -0.2, 1.4
        def X(x):
            return ml + (x - x0) / (x1 - x0) * (W - ml - mr)
        def Y(y):
            return top + ph - (min(max(y, lo), hi) - lo) / (hi - lo) * ph
        out.append(f'<rect x="{ml}" y="{top}" width="{W - ml - mr}" height="{ph}" '
                   f'fill="none" stroke="#999"/>')
        for g in (0.0, 0.5, 1.0):
            out.append(f'<line x1="{ml}" x2="{W - mr}" y1="{Y(g):.1f}" y2="{Y(g):.1f}" '
                       f'stroke="#ddd"/><text x="{ml - 6}" y="{Y(g) + 4:.1f}" '
                       f'text-anchor="end">{g:.1f} V</text>')
        pts = " ".join(f"{X(x):.1f},{Y(y):.1f}" for x, y in zip(xs, ys))
        out.append(f'<polyline points="{pts}" fill="none" stroke="#1f4e9e" stroke-width="1.2"/>')
        out.append(f'<text x="{ml + 6}" y="{top + 14}">{label}  (min {min(ys):.3f} V, '
                   f'max {max(ys):.3f} V)</text>')
    out.append(f'<text x="{ml}" y="{H - 12}">{xlabel}: 0 to {(x1 - x0) * xscale:.2f}</text>')
    out.append("</svg>")
    with open(path, "w") as f:
        f.write("\n".join(out))


def measure(rows, col):
    ys = [r[col] for r in rows]
    return min(ys), max(ys)


def main():
    os.makedirs(RESULTS, exist_ok=True)
    summary = [["view", "standard", "trimmer_pos"] +
               [f"{v}_{m}" for v, _ in OUTS for m in ("min", "max")]]
    oplines = []
    for std, pos in CASES:
        for view in ("line", "frame"):
            header, rows, ops = run(std, pos, view)
            t0 = rows[0][0]
            tag = f"{view}_{std}_pos{pos:.1f}"
            write_csv(os.path.join(RESULTS, tag + ".csv"), header, rows, t0)
            xs = [r[0] for r in rows]
            series = [(lab, [r[i + 1] for r in rows]) for i, (_, lab) in enumerate(OUTS)]
            if view == "line":
                xl, sc = "time from H sync leading edge, microseconds", 1e6
            else:
                xl, sc = "time from V sync leading edge, milliseconds", 1e3
            svg_plot(os.path.join(RESULTS, tag + ".svg"),
                     f"Cadet IV sim: {view}, {std.upper()} timing, trimmers R1/R3 at {pos:.1f}",
                     xs, series, xl, sc)
            summary.append([view, std, pos] +
                           [f"{x:.4f}" for i in range(len(OUTS)) for x in measure(rows, i + 1)])
            if view == "line" and pos == 0.5 and std == "ntsc":
                oplines = ops
            print(tag, "rows", len(rows))
    with open(os.path.join(RESULTS, "summary.csv"), "w", newline="") as f:
        csv.writer(f, lineterminator="\n").writerows(summary)
    with open(os.path.join(RESULTS, "operating_point.txt"), "w") as f:
        f.write("ngspice prints at the first saved point, line view, NTSC, pos 0.5\n")
        f.write("\n".join(oplines) + "\n")
    print("\n".join(oplines))


if __name__ == "__main__":
    main()
