#!/bin/sh
cd "$(dirname "$0")"
cd expander_compiler/ec_go_lib
cargo build --release --features avx256
cd ..
cp target/release/libec_go_lib.so ../ecgo/rust/wrapper/