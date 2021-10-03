#!/bin/bash
set -e
shopt -s extglob

function usage {
    echo "USAGE:" >&2
    echo "    $0 ( <multiplier> | <crate_name> )..." >&2
    echo >&2
    echo "    <multiplier> An integer that sets the repeat count for the next <crate_name>" >&2
    echo "    <crate_name> The name of a crate in the wasm Cargo workspace" >&2
    echo >&2
    echo "    At least one crate must be specified after multipliers are considered."
    echo >&2
    echo "EXAMPLES:" >&2
    echo "    $0 ignition-impulse-bench" >&2
    echo "    $0 20 ignition-impulse-bench" >&2
    echo "    $0 ignition-rpc-echo-server 20 ignition-rpc-echo-client" >&2
    exit 1
}

profile=debug
multiplier=1
declare -a crates
while [ $# -gt 0 ]; do
    case $1 in
        --debug)
            profile=debug
            shift
            ;;
        --release)
            profile=release
            shift
            ;;
        --*)
            usage
            ;;
        +([0-9]))
            multiplier=$1
            shift
            ;;
        *)
            for ((i=1; i <= multiplier; i++)); do
                crates+=("$1")
            done
            multiplier=1
            shift
    esac
done

if [ ${#crates[@]} -eq 0 ]; then
    usage
fi

case $profile in
    release)
        extra_cargo_flags=--release
        build_dir=wasm/target/wasm32-unknown-unknown/release
        ;;
    debug)
        extra_cargo_flags=
        build_dir=wasm/target/wasm32-unknown-unknown/debug
        ;;
esac


declare -a wasm_paths
declare -A built_crates
for crate_name in "${crates[@]}"; do
    # Rust crates named with hyphens generate build artifacts with underscores.
    build_artifact_name="$(sed 's/-/_/g' <<< "$crate_name")"

    # Build the wasm crate and optimize the output.
    case $profile in
        release)
            final_wasm_path="$build_dir/$build_artifact_name.opt.wasm"
            ;;
        debug)
            final_wasm_path="$build_dir/$build_artifact_name.wasm"
            ;;
    esac

    wasm_paths+=("$final_wasm_path")
    if [ -v "built_crates["$crate_name"]" ]; then
        continue
    fi
    built_crates["$crate_name"]=

    (cd wasm && cargo build -p "$crate_name" $extra_cargo_flags)
    if [ $profile = release ]; then
        wasm-opt -o "$final_wasm_path" -Oz "$build_dir/$build_artifact_name.wasm"
    fi
    ls -lh "$final_wasm_path"
done

# Run the host and pass it the path to the optimized wasm.
cargo run -p ignition-host --release -- "${wasm_paths[@]}"
