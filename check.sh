#!/usr/bin/env bash
set -u

cd -- "$(dirname -- "${BASH_SOURCE[0]}")" || exit 1
export CARGO_TARGET_DIR=/tmp/phosphlux-check/target
export CARGO_TERM_COLOR=never
mkdir -p /tmp/phosphlux-check || exit 1
log_dir=$(mktemp -d /tmp/phosphlux-check/run.XXXXXX) || exit 1
failed=0

printf 'Phosphlux check (cargo build/test/clippy --locked)\n'
for step in build test clippy; do
    log="$log_dir/$step.log"
    cargo "$step" --locked >"$log" 2>&1
    code=$?
    warnings=$(awk '/^warning: / && !/^warning: `.*` .*generated [0-9]+ warnings?/ && !/^warning: the following packages contain code/ { n++ } END { print n+0 }' "$log")
    if [ "$code" -eq 0 ]; then
        result=PASS
    else
        result=FAIL
        failed=1
    fi
    printf '%s: %s (exit %s, %s warnings)\n' "$step" "$result" "$code" "$warnings"
    if [ "$step" = test ]; then
        awk '/^test result:/ { print }' "$log"
    fi
    if [ "$code" -ne 0 ]; then
        tail -n 20 "$log"
    fi
    awk '/^warning: the following packages contain code/ { print }' "$log"
done

printf 'Logs: %s\n' "$log_dir"
if [ "$failed" -eq 0 ]; then
    printf 'Overall: PASS\n'
else
    printf 'Overall: FAIL\n'
fi
exit "$failed"
