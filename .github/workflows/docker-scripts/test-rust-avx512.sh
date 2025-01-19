#!/bin/sh
cd /tmp/ecc
RUSTFLAGS="-C target-cpu=native -C target-feature=+avx512f" cargo test