#!/bin/sh
# Re-run the Cadet IV probe: writes ../results/*.csv, *.svg, summary.csv, operating_point.txt.
# Needs ngspice (tested with ngspice-47) and python3 (standard library only).
# Intermediate decks and logs go to <repo>/.scratch/cadet_iv_runs/.
set -e
cd "$(dirname "$0")"
python3 -I run_sims.py
