#!/bin/sh
# Rerun the whole Cadet IV chain from the committed sources (T-0375):
#   1. the ngspice simulation of the schematic        -> results/
#   2. the reference values and their document gate    -> faithful/reference_check.txt
#   3. the device's CPU-model tests against them       -> log in <repo>/.scratch/cadet_iv_app/
#   4. an off-screen GPU render of the app's device     -> raw floats in <repo>/.scratch/cadet_iv_app/
#   5. FAITHFUL.md's checks, app against circuit       -> comparison/ (report.md, numbers.csv, pictures)
# Needs ngspice (tested with ngspice-47), python3 (standard library only), cargo and a GPU.
# A failed check is reported in comparison/report.md; it does not stop the script. A step that cannot
# run (no ngspice, no GPU, a build error) does stop it, with a non-zero exit.
set -e
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
scratch="$repo/.scratch"
mkdir -p "$scratch"
[ -f "$scratch/.gitignore" ] || printf '*\n' >"$scratch/.gitignore"
dump="$scratch/cadet_iv_app"
rm -rf "$dump"
mkdir -p "$dump"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$scratch/phosphlux-check/target}"
command -v cargo >/dev/null 2>&1 || PATH="$HOME/.cargo/bin:$PATH"

echo "== 1/5 ngspice simulation ($(ngspice -v 2>/dev/null | grep -o 'ngspice-[0-9]*' | head -1))"
sh "$here/sim/run.sh"

echo "== 2/5 reference values"
python3 -I "$here/faithful/reference_check.py"

echo "== 3/5 CPU model tests"
cd "$repo"
if cargo test --locked -p phosphlux --bins cadet_iv >"$dump/cpu_tests.log" 2>&1; then
    grep '^test result' "$dump/cpu_tests.log" | head -1 | sed -e 's/^/passed: /' -e 's/; finished in.*//' >"$dump/cpu_tests.txt"
else
    { printf 'FAILED: '; grep -E '^test result|FAILED|panicked' "$dump/cpu_tests.log" | head -5 | tr '\n' ' '; } >"$dump/cpu_tests.txt"
fi
cat "$dump/cpu_tests.txt"; echo

echo "== 4/5 off-screen GPU render of the app's Cadet IV"
CADET_IV_DUMP_DIR="$dump" cargo test --locked -p phosphlux --bins cadet_iv_gpu_dump -- --ignored --nocapture \
    >"$dump/gpu_render.log" 2>&1 || { tail -20 "$dump/gpu_render.log"; echo "GPU render failed"; exit 1; }
cat "$dump/manifest.txt"

echo "== 5/5 FAITHFUL.md checks"
python3 -I "$here/faithful/compare_app.py" "$dump" "$here/comparison" "$dump/cpu_tests.txt"
