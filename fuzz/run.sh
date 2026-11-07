#!/bin/sh
# fuzz/run.sh — builds the fuzz image and targets, then fuzzes every target in the locked-down container on half the CPUs.
set -eu
repo=$(cd "$(dirname "$0")/.." && pwd)
seconds=${FUZZ_SECONDS:-1200}
jobs=${FUZZ_JOBS:-$(( $(nproc) / 2 ))}
docker build -q --target fuzz -f "$repo/.devcontainer/Dockerfile" -t southstar-fuzz "$repo" >/dev/null
network=--network=none
vendor_mount=
if [ -n "${FUZZ_VENDOR:-}" ]; then
    vendor_mount="-v $FUZZ_VENDOR:/vendor:ro"
else
    network=
fi
mkdir -p "$repo/fuzz/corpus" "$repo/fuzz/artifacts" "$repo/fuzz/logs"
exec docker run --rm $network --cpus="$(( jobs * ${FUZZ_FORK:-1} ))" --cap-drop=ALL --security-opt=no-new-privileges \
    --read-only --tmpfs /tmp:exec,size=8g --pids-limit=4096 \
    -e CARGO_HOME=/tmp/cargo -e CARGO_BUILD_JOBS="$jobs" -e ASAN_OPTIONS=detect_leaks=0:detect_odr_violation=0 -e FUZZ_SECONDS="$seconds" -e FUZZ_JOBS="$jobs" \
    -e FUZZ_TARGETS="${FUZZ_TARGETS:-}" -e FUZZ_FORK="${FUZZ_FORK:-1}" \
    -v "$repo":/src:ro -v "$repo/fuzz":/src/fuzz $vendor_mount -w /src/fuzz \
    southstar-fuzz sh /src/fuzz/container.sh
