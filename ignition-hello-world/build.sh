#!/bin/bash
set -e

cargo_flags="-Z build-std=core,alloc --target wasm32-unknown-unknown"
build_dir=../target/wasm32-unknown-unknown/release
cargo +nightly build $cargo_flags
cargo +nightly build $cargo_flags --release
wasm-opt -o $build_dir/opt.wasm -Oz $build_dir/ignition_hello_world.wasm
ls -lh $build_dir/opt.wasm
