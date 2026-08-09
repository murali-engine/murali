#!/bin/bash

# This script runs all Murali examples sequentially with the --preview flag.
# With --auto, each preview closes five seconds after its timeline completes.

EXAMPLES_DIR="examples"
AUTO_CLOSE=false

usage() {
    echo "Usage: $0 [--auto]"
}

case "${1:-}" in
    "")
        ;;
    --auto)
        AUTO_CLOSE=true
        ;;
    -h|--help)
        usage
        exit 0
        ;;
    *)
        echo "Error: unknown option '$1'."
        usage
        exit 2
        ;;
esac

if [ "$#" -gt 1 ]; then
    echo "Error: too many arguments."
    usage
    exit 2
fi

if [ ! -d "$EXAMPLES_DIR" ]; then
    echo "Error: examples directory not found."
    exit 1
fi

# Find all .rs files in the examples directory
for f in "$EXAMPLES_DIR"/*.rs; do
    # Extract filename without extension
    example_name=$(basename "$f" .rs)
    
    echo "===================================================="
    echo "▶ Running Example: $example_name"
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
        echo "Example $example_name exited with status $status. Stopping."
        exit $status
    fi
done

echo "Done! All examples have been previewed."
