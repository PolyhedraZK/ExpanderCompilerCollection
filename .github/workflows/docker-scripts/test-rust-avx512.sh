#!/bin/sh
cd "$(dirname "$0")"
source setup.sh
RUSTFLAGS="-C target-cpu=native -C target-feature=+avx512f" cargo test