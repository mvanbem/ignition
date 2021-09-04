#!/bin/bash
set -e

build_dir=../target/wasm32-unknown-unknown/release
cargo +nightly build --target wasm32-unknown-unknown
cargo +nightly build --target wasm32-unknown-unknown --release
wasm-opt -o $build_dir/opt.wasm -Oz $build_dir/ignition_hello_world.wasm
ls -lh $build_dir/opt.wasm
