#!/bin/bash

working_dir=$(pwd)
if [[ $working_dir == *"ExpanderCompilerCollection/scripts" ]]
then
        cd ..
fi

RUSTFLAGS="-C target-cpu=native -C target-feature=+avx512f" cargo build --release
mkdir -p ~/.cache/ExpanderCompilerCollection
cp target/release/libec_go_lib.so ~/.cache/ExpanderCompilerCollection
