#!/usr/bin/env python3
"""Repeat the full-circuit crossover sensitivity and held-out trim checks.
Logs/decks stay in .scratch; measured evidence is written beside this script.
"""
import bisect
import csv
import importlib.util
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parents[1]
WORK = ROOT.parents[2] / '.scratch' / 'cadet_iv_triangle_probe'
WORK.mkdir(parents=True, exist_ok=True)
spec = importlib.util.spec_from_file_location('sim', ROOT / 'sim/run_sims.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)
lib = (ROOT / 'sim/cadet_iv.lib').read_text()
vendor = (ROOT / 'triangle/lm6172.lib').read_text()
vlib = (lib + '\n' + vendor).replace(
    'XU2A 0 na o1 vcc vee OPAMP gbw=100e6 sr=3000e6', 'XU2A 0 na vcc vee o1 LM6172/NS').replace(
    'XU2B p5 n6 o2 vcc vee OPAMP gbw=100e6 sr=3000e6', 'XU2B p5 n6 vcc vee o2 LM6172/NS')
variants = {
    'baseline': lib,
    'step_5n': lib,
    'trap_5n': lib,
    'slew_10x': lib.replace('sr=3000e6', 'sr=30000e6'),
    'gbw_10x': lib.replace('gbw=100e6', 'gbw=1000e6'),
    'gbw_0.1x': lib.replace('gbw=100e6', 'gbw=10e6'),
    'cjo_zero': lib.replace('Rs=30 Cjo=2p', 'Rs=30 Cjo=0'),
    'cjo_half': lib.replace('Rs=30 Cjo=2p', 'Rs=30 Cjo=1p'),
    'is_100x': lib.replace('Is=1.2e-10 N=1 Rs=30', 'Is=1.2e-8 N=1 Rs=30'),
    'vendor_u2': vlib,
    # FAITHFUL.md excludes offsets; keep vendor dynamics but remove generated offset/bias.
    'vendor_nominal': vlib.replace('RAX 122 10 MRAX 1.005990e+03', 'RAX 122 10 MRAX 1K').replace(
        'FIN1 18 5 VTEMP 0.991667', 'FIN1 18 5 VTEMP 0').replace('FIN2 19 5 VTEMP 1.00833', 'FIN2 19 5 VTEMP 0'),
}

def run(name, text, pos):
    tag = f'{name}_{pos}'
    lp = WORK / (tag + '.lib')
    lp.write_text(text)
    out, deck = sim.deck('ntsc', pos, 'line')
    dat = WORK / (tag + '.dat')
    deck = deck.replace(str(ROOT / 'sim/cadet_iv.lib'), str(lp)).replace(out, str(dat))
    deck = deck.replace('v(jv_tri)', 'v(jv_tri) v(m2) v(xh.na) v(xh.o1)')
    if name in ('step_5n', 'trap_5n'):
        deck = deck.replace('50n', '5n')
    if name == 'trap_5n':
        deck = deck.replace('method=gear', 'method=trap')
    cir = WORK / (tag + '.cir')
    cir.write_text(deck)
    p = subprocess.run(['ngspice', '-b', str(cir)], capture_output=True, text=True, timeout=120)
    dat.with_suffix('.log').write_text(p.stdout + p.stderr)
    if p.returncode:
        raise RuntimeError(f'{tag}: exit {p.returncode}')
    rows = [[float(x) for x in l.split()] for l in dat.read_text().splitlines()[1:]]
    t0 = 15 * sim.STANDARDS['ntsc']['line']
    rows = [r for r in rows if t0 + 9.4e-6 < r[0] < t0 + 62.06e-6]
    print(f'{tag}: ngspice exit {p.returncode}', flush=True)
    return rows, t0

summary = []
profiles = []
for name, text in variants.items():
    for pos in (0.5, 1.0):
        rows, t0 = run(name, text, pos)
        local = [r for r in rows if 0.48 < r[1] < 0.525]
        flat = [r for r in local if abs(r[3]) < 0.001]
        width = (flat[-1][0] - flat[0][0]) * 1e6 if flat else 0
        err = max(abs(r[3] - 1000 / 499 * abs(r[1] - abs(r[7]) * 499 / 2000)) for r in local)
        low = min(local, key=lambda r: r[3])
        summary.append([name, pos, 0, width, err, low[3], low[1]])
        profiles.extend([name, pos, (r[0]-t0)*1e6, r[1], r[3], r[8], r[9]] for r in local)
with (ROOT / 'triangle/sensitivity.csv').open('w') as f:
    w = csv.writer(f, lineterminator='\n')
    w.writerow(['variant', 'trim', 'ngspice_exit', 'below_1mV_width_us', 'max_error_from_ideal_V', 'min_tri_V', 'ramp_at_min_V'])
    w.writerows(summary)
with (ROOT / 'triangle/profiles.csv').open('w') as f:
    w = csv.writer(f, lineterminator='\n')
    w.writerow(['variant','trim','time_us','ramp_V','triangle_V','U2_inverting_V','U2_output_V'])
    w.writerows(profiles)
# Full circuit at trim settings absent from the isolated-stage table.
table = (ROOT / 'triangle/correction.rs').read_text().split('= [', 1)[1]
table = re.sub(r'//[^\n]*', '', table)
vals = [float(x) for x in re.findall(r'-?\d+\.\d+', table)]
assert len(vals) == 1089
zero = (2.495*10/12.4)*499/2000

def model(t, pos):
    ramp = (4.99+10*pos)/14.99 * 0.99753e-3/56e-9 * (t-4.7e-6)
    delta = ramp-zero
    v = 1000/499*abs(delta)
    if -0.03 <= delta <= 0.03:
        x = (delta+0.03)/0.0005
        y = pos*8
        i = min(int(x), 119)
        j = min(int(y), 7)
        def row(r):
            a,b = vals[r*121+i:r*121+i+2]
            return a+(b-a)*(x-i)
        v += row(j)+(row(j+1)-row(j))*(y-j)
    return v

held = []
for pos in (0.3125, 0.5625, 0.9375):
    rows,t0 = run('held_out', lib, pos)
    times = [r[0]-t0 for r in rows]
    errors = []
    for i in range(720):
        t = 9.4e-6+(i+0.5)/720*52.66e-6
        k = max(1,min(bisect.bisect_left(times,t),len(rows)-1))
        v = rows[k-1][3]+(rows[k][3]-rows[k-1][3])*(t-times[k-1])/(times[k]-times[k-1])
        errors.append(abs(model(t,pos)-v))
    held.append([pos,max(errors),max(errors)<=0.010])
    print(f'held-out trim {pos}: max H triangle error {max(errors):.6f} V',flush=True)
with (ROOT / 'triangle/held_out.csv').open('w') as f:
    w=csv.writer(f,lineterminator='\n'); w.writerow(['trim','max_H_triangle_error_V','within_0.010']);w.writerows(held)
