#!/bin/sh
# fuzz/container.sh — runs inside the fuzz container: builds the targets with ASan and fuzzes them in parallel.
set -eu
export FUZZ_FORK="${FUZZ_FORK:-1}"
mkdir -p "$CARGO_HOME"
if [ -d /vendor ]; then
    printf '[source.crates-io]\nreplace-with = "vendored-sources"\n\n[source.vendored-sources]\ndirectory = "/vendor"\n' > "$CARGO_HOME/config.toml"
fi
cargo fuzz build -a --sanitizer address
targets=${FUZZ_TARGETS:-$(cargo fuzz list)}
for target in $targets; do
    mkdir -p "corpus/$target" "artifacts/$target"
done
echo $targets | tr ' ' '\n' | xargs -P "$FUZZ_JOBS" -I{} sh -c \
    'dict=; [ -f dicts/{}.dict ] && dict=-dict=dicts/{}.dict; cargo fuzz run -a --sanitizer address {} corpus/{} -- $dict -fork="$FUZZ_FORK" -ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1 \
        -max_total_time="$FUZZ_SECONDS" -timeout=10 -rss_limit_mb=2048 -malloc_limit_mb=2048 -detect_leaks=0 \
        -max_len=65536 -artifact_prefix=artifacts/{}/ > logs/{}.log 2>&1; echo "done {}"'
