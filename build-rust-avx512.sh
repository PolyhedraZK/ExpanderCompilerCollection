#!/bin/sh
cd "$(dirname "$0")"
cd expander_compiler/ec_go_lib
RUSTFLAGS="-C target-cpu=native -C target-features=+avx512f" cargo build --release
cd ..
cp target/release/libec_go_lib.so ../ecgo/rust/wrapper/