#!/usr/bin/env python3
"""Offline reproducible probe. Python standard library and ngspice only."""
from pathlib import Path
import subprocess, csv, json, hashlib
p=Path(__file__).resolve().parent
for deck,log in [('vendor-attempt.cir','vendor-attempt.log'),('ramp.cir','ngspice.log')]:
 r=subprocess.run(['ngspice','-b',deck],cwd=p,capture_output=True,text=True)
 (p/log).write_text(r.stdout+r.stderr)
 print(f'{deck}: exit {r.returncode}')
 if deck=='ramp.cir' and (r.returncode or 'aborted' in r.stdout+r.stderr):
  raise SystemExit('Ramp simulation failed; see ngspice.log')
rows=[[float(v) for v in line.split()] for line in (p/'ramp.dat').read_text().splitlines()[1:]]
assert len(rows)>6000 and all(len(r)==5 for r in rows)
t0=953.34e-6
with (p/'ramp.csv').open('w') as f:
 w=csv.writer(f);w.writerow(['time_from_sync_s','J8_V','reset_drive_V','capacitor_V','R44_current_A'])
 w.writerows([[r[0]-t0]+r[1:] for r in rows])
# First saved complete line; centered least-squares fit excludes edge/reset intervals.
line=[r for r in rows if 0<=r[0]-t0<63.556e-6]
fit=[r for r in line if 10e-6<=r[0]-t0<=55e-6]
mx=sum(r[0] for r in fit)/len(fit);my=sum(r[1] for r in fit)/len(fit)
slope=sum((r[0]-mx)*(r[1]-my) for r in fit)/sum((r[0]-mx)**2 for r in fit)
res=max(abs(r[1]-my-slope*(r[0]-mx)) for r in fit)
edges=[rows[i][0]-t0 for i in range(1,len(rows)) if rows[i-1][2]<2.5<=rows[i][2]]
# Falling 0.5 V output crossings locate observable resets; interpolate between samples.
falls=[]
for a,b in zip(rows,rows[1:]):
 if a[1]>=.5>b[1]: falls.append(a[0]-t0+(.5-a[1])*(b[0]-a[0])/(b[1]-a[1]))
m={'samples':len(rows),'J8_min_V':min(r[1] for r in line),'J8_max_V':max(r[1] for r in line),'slope_V_per_us':slope/1e6,'fit_max_residual_V':res,'reset_crossings_us':[v*1e6 for v in falls],'measured_reset_period_us':(falls[1]-falls[0])*1e6,'assumed_sync_period_us':63.556,'assumed_sync_high_us':4.7,'charging_interval_us':58.856,'R44_current_A':rows[0][4]}
(p/'numbers.json').write_text(json.dumps(m,indent=2)+'\n'); print(json.dumps(m,indent=2))
W,H=960,400;left,right,top,bottom=78,930,65,335
X=lambda t:left+t/127.112e-6*(right-left)
Y=lambda v:bottom-v/1.2*(bottom-top)
s=['<svg xmlns="http://www.w3.org/2000/svg" width="960" height="400" font-family="sans-serif" font-size="16">','<rect width="960" height="400" fill="white"/>','<text x="78" y="23" font-size="12">CIRCUIT SIMULATION · T-0290</text>','<text x="78" y="49" font-size="21">Cadet IV · J8 horizontal ramp</text>']
for v in [0,.5,1]:
 s+=[f'<line x1="{left}" x2="{right}" y1="{Y(v)}" y2="{Y(v)}" stroke="#777" stroke-dasharray="2 4"/>',f'<text x="65" y="{Y(v)+5}" text-anchor="end">{v:g} V</text>']
for t in [0,31.778,63.556,95.334,127.112]:
 x=X(t*1e-6);s +=[f'<text x="{x}" y="359" text-anchor="{"end" if t == 127.112 else "middle"}">{t:g}</text>']
pts=' '.join(f'{X(r[0]-t0):.2f},{Y(r[1]):.2f}' for r in rows)
s += [f'<polyline points="{pts}" fill="none" stroke="black" stroke-width="2"/>','<text x="78" y="390">Time from assumed H sync edge (µs) · R1 maximum · unloaded · behavioral models</text>','</svg>']
(p/'ramp.svg').write_text('\n'.join(s))
print('SHA256 ramp.dat',hashlib.sha256((p/'ramp.dat').read_bytes()).hexdigest())
# Minimal vector PDF, using built-in Helvetica; no plotting packages required.
ops=['1 1 1 rg 0 0 960 400 re f','0 0 0 rg']
def text_at(x,y,size,text):
 safe=text.replace('\\','\\\\').replace('(','\\(').replace(')','\\)')
 ops.append(f'BT /F1 {size} Tf {x} {400-y} Td ({safe}) Tj ET')
text_at(78,23,12,'CIRCUIT SIMULATION - T-0290')
text_at(78,49,21,'Cadet IV - J8 horizontal ramp')
for v in [0,.5,1]:
 y=400-Y(v);ops.append(f'0.4 G 0.5 w {left} {y} m {right} {y} l S');text_at(30,Y(v)+5,16,f'{v:g} V')
for t in [0,31.778,63.556,95.334,127.112]:text_at(X(t*1e-6)-(55 if t==127.112 else 20),359,16,f'{t:g}')
ops.append('0 G 1.5 w')
ops.append(' '.join(f'{X(r[0]-t0):.2f} {400-Y(r[1]):.2f} '+('m' if i==0 else 'l') for i,r in enumerate(rows))+' S')
text_at(78,390,14,'Time from H sync edge (us) - R1 maximum - unloaded - behavioral models')
stream='\n'.join(ops).encode('ascii')
objects=[b'<< /Type /Catalog /Pages 2 0 R >>',b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>',b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 960 400] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>',b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>',b'<< /Length '+str(len(stream)).encode()+b' >>\nstream\n'+stream+b'\nendstream']
blob=b'%PDF-1.4\n';offsets=[0]
for i,obj in enumerate(objects,1):
 offsets.append(len(blob));blob+=f'{i} 0 obj\n'.encode()+obj+b'\nendobj\n'
start=len(blob);blob+=b'xref\n0 6\n0000000000 65535 f \n'+b''.join(f'{n:010d} 00000 n \n'.encode() for n in offsets[1:])+f'trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{start}\n%%EOF\n'.encode()
(p/'ramp.pdf').write_bytes(blob)
