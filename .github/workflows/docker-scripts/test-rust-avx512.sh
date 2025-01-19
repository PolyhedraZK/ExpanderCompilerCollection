#!/bin/sh
cd "$(dirname "$0")"
pwd
ls -al
. setup.sh
RUSTFLAGS="-C target-cpu=native -C target-feature=+avx512f" cargo test