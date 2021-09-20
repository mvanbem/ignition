#!/bin/bash
set -e

if [ $# -ne 1 ]; then
    echo "USAGE:" >&2
    echo "    $0 <crate_name>" >&2
    exit 1
fi

# Rust crates named with hyphens generate build artifacts with underscores.
crate_name="$1"
wasm_filename="$(sed 's/-/_/g' <<< "$crate_name").wasm"

# Build the wasm crate and optimize the output.
(cd wasm && cargo build -p "$crate_name" --release)
build_dir=wasm/target/wasm32-unknown-unknown/release
wasm-opt -o $build_dir/opt.wasm -Oz $build_dir/$wasm_filename
ls -lh $build_dir/opt.wasm

# Run the host and pass it the path to the optimized wasm.
(cd host && cargo run -p ignition-host --release -- ../$build_dir/$wasm_filename)
