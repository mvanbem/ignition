#!/bin/bash
set -e

build_dir=../target/wasm32-unknown-unknown/release
cargo build --target wasm32-unknown-unknown --release
wasm-strip $build_dir/ignition_hello_world.wasm
wasm-opt -o $build_dir/opt.wasm -Oz $build_dir/ignition_hello_world.wasm
ls -lh $build_dir/opt.wasm
