#!/bin/sh
cd /tmp/ecc
apt-get install -y libopenmpi-dev
RUSTFLAGS="-C target-cpu=native -C target-feature=+avx512f" cargo test