#!/bin/bash

# This script exports all Murali examples sequentially with the --export flag.
# Each example writes its artifacts under rendered_output/ before the next one starts.

set -u

EXAMPLES_DIR="examples"
START_NO=1
END_NO=""
START_SET=false

if [ ! -d "$EXAMPLES_DIR" ]; then
    echo "Error: examples directory not found."
    exit 1
fi

usage() {
    echo "Usage: $0 [--release] [--start N] [--end N] [extra export args...]"
    echo "       $0 [--release] [START] [END] [extra export args...]"
}

is_positive_int() {
    [[ "$1" =~ ^[1-9][0-9]*$ ]]
}

run_mode="debug"
extra_args=()

while [ "$#" -gt 0 ]; do
    case "$1" in
        --release)
            run_mode="release"
            shift
            ;;
        --start)
            if [ "$#" -lt 2 ] || ! is_positive_int "$2"; then
                echo "Error: --start requires a positive integer."
                usage
                exit 2
            fi
            START_NO="$2"
            START_SET=true
            shift 2
            ;;
        --end)
            if [ "$#" -lt 2 ] || ! is_positive_int "$2"; then
                echo "Error: --end requires a positive integer."
                usage
                exit 2
            fi
            END_NO="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            if is_positive_int "$1" && [ "$START_SET" = false ]; then
                START_NO="$1"
                START_SET=true
            elif is_positive_int "$1" && [ -z "$END_NO" ]; then
                END_NO="$1"
            else
                extra_args+=("$1")
            fi
            shift
            ;;
    esac
done

shopt -s nullglob
examples=("$EXAMPLES_DIR"/*.rs)
total_examples=${#examples[@]}

if [ "$total_examples" -eq 0 ]; then
    echo "Error: no examples found."
    exit 1
fi

if [ -z "$END_NO" ]; then
    END_NO="$total_examples"
fi

if [ "$START_NO" -gt "$END_NO" ]; then
    echo "Error: start number ($START_NO) cannot be greater than end number ($END_NO)."
    exit 2
fi

echo "Exporting examples $START_NO-$END_NO of $total_examples ($run_mode)."

sequence_no=0
for f in "${examples[@]}"; do
    sequence_no=$((sequence_no + 1))
    if [ "$sequence_no" -lt "$START_NO" ] || [ "$sequence_no" -gt "$END_NO" ]; then
        continue
    fi

    example_name=$(basename "$f" .rs)

    echo "===================================================="
    echo "▶ Exporting Example $sequence_no/$total_examples: $example_name ($run_mode)"
    echo "===================================================="

    cargo_cmd=(cargo run)
    if [ "$run_mode" = "release" ]; then
        cargo_cmd+=(--release)
    fi
    cargo_cmd+=(--example "$example_name" -- --export)

    if [ "${#extra_args[@]}" -gt 0 ]; then
        cargo_cmd+=("${extra_args[@]}")
    fi

    "${cargo_cmd[@]}"

    status=$?
    if [ $status -ne 0 ]; then
        echo "Example $sequence_no/$total_examples ($example_name) exited with status $status. Stopping."
        exit $status
    fi

    echo "OK Completed $sequence_no/$total_examples: $example_name"
done

echo "Done! Selected examples have been exported."
