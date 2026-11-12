#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
if [ "$#" -eq 0 ]; then
    set -- --bins
fi
cargo +stable build --locked "$@"
(
    unset CARGO_TARGET_DIR
    sh vendor/hibana-wash/scripts/build.sh
)
