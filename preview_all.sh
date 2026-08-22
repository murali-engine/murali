#!/bin/bash

# This script runs all Murali examples sequentially with the --preview flag.
# With --auto, each preview closes five seconds after its timeline completes.

EXAMPLES_DIR="examples"
AUTO_CLOSE=false
START_NO=1
END_NO=""
START_SET=false

usage() {
    echo "Usage: $0 [--auto] [--start N] [--end N]"
    echo "       $0 [--auto] [START] [END]"
}

is_positive_int() {
    [[ "$1" =~ ^[1-9][0-9]*$ ]]
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --auto)
            AUTO_CLOSE=true
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
        -*)
            echo "Error: unknown option '$1'."
            usage
            exit 2
            ;;
        *)
            if ! is_positive_int "$1"; then
                echo "Error: range values must be positive integers."
                usage
                exit 2
            fi
            if [ "$START_SET" = false ]; then
                START_NO="$1"
                START_SET=true
            elif [ -z "$END_NO" ]; then
                END_NO="$1"
            else
                echo "Error: too many range values."
                usage
                exit 2
            fi
            shift
            ;;
    esac
done

if [ ! -d "$EXAMPLES_DIR" ]; then
    echo "Error: examples directory not found."
    exit 1
fi

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

echo "Previewing examples $START_NO-$END_NO of $total_examples."

# Find all .rs files in the examples directory
sequence_no=0
for f in "${examples[@]}"; do
    sequence_no=$((sequence_no + 1))
    if [ "$sequence_no" -lt "$START_NO" ] || [ "$sequence_no" -gt "$END_NO" ]; then
        continue
    fi

    # Extract filename without extension
    example_name=$(basename "$f" .rs)
    
    echo "===================================================="
    echo "▶ Running Example $sequence_no/$total_examples: $example_name"
    echo "===================================================="
    
    # Run the example with the preview flag.
    if [ "$AUTO_CLOSE" = true ]; then
        cargo run --example "$example_name" -- --preview --auto-close
    else
        cargo run --example "$example_name" -- --preview
    fi
    
    # Check if the process was interrupted
    status=$?
    if [ $status -ne 0 ]; then
        echo "Example $sequence_no/$total_examples ($example_name) exited with status $status. Stopping."
        exit $status
    fi

    echo "OK Completed $sequence_no/$total_examples: $example_name"
done

echo "Done! Selected examples have been previewed."
