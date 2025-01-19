#!/bin/sh
cp /tmp/ecc2/ecc2 /tmp/ecc-test -r
cd /tmp/ecc-test
apt-get install -y libopenmpi-dev

case $1 in
    "test-rust-avx512")
        RUSTFLAGS="-C target-cpu=native -C target-feature=+avx512f" cargo test
        ;;
esac