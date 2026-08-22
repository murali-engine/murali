#!/bin/bash

# This script exports all Murali examples sequentially with the --export flag.
# Each example writes its artifacts under rendered_output/ before the next one starts.

set -u

EXAMPLES_DIR="examples"
START_NO=1
END_NO=""
START_SET=false
DRY_RUN=false
INCLUDE_TAGS=()
EXCLUDE_TAGS=()
EXAMPLE_NAMES=()
LIST_TAGS=false
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "$SCRIPT_DIR/scripts/example_filter.sh"

if [ ! -d "$EXAMPLES_DIR" ]; then
    echo "Error: examples directory not found."
    exit 1
fi

usage() {
    echo "Usage: $0 [--release] [--dry-run] [--tag TAG] [--skip-tag TAG] [--example NAME] [--start N] [--end N] [extra export args...]"
    echo "       $0 [--release] [START] [END] [extra export args...]"
    echo "       $0 --list-tags"
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
        --dry-run)
            DRY_RUN=true
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
        --tag)
            if [ "$#" -lt 2 ]; then
                echo "Error: --tag requires a tag."
                usage
                exit 2
            fi
            INCLUDE_TAGS+=("$2")
            shift 2
            ;;
        --skip-tag)
            if [ "$#" -lt 2 ]; then
                echo "Error: --skip-tag requires a tag."
                usage
                exit 2
            fi
            EXCLUDE_TAGS+=("$2")
            shift 2
            ;;
        --example)
            if [ "$#" -lt 2 ]; then
                echo "Error: --example requires an example name without .rs."
                usage
                exit 2
            fi
            EXAMPLE_NAMES+=("$2")
            shift 2
            ;;
        --list-tags)
            LIST_TAGS=true
            shift
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

if [ "$LIST_TAGS" = true ]; then
    list_example_tags "${examples[@]}"
    exit 0
fi

selected_examples=()
for f in "${examples[@]}"; do
    if example_matches_filters "$f"; then
        selected_examples+=("$f")
    fi
done
total_selected=${#selected_examples[@]}

if [ "$total_selected" -eq 0 ]; then
    echo "Error: no examples matched the selected filters."
    exit 1
fi

if [ -z "$END_NO" ]; then
    END_NO="$total_selected"
fi

if [ "$START_NO" -gt "$END_NO" ]; then
    echo "Error: start number ($START_NO) cannot be greater than end number ($END_NO)."
    exit 2
fi

echo "Exporting selected examples $START_NO-$END_NO of $total_selected ($total_examples total, $run_mode)."

sequence_no=0
for f in "${selected_examples[@]}"; do
    sequence_no=$((sequence_no + 1))
    if [ "$sequence_no" -lt "$START_NO" ] || [ "$sequence_no" -gt "$END_NO" ]; then
        continue
    fi

    example_name=$(basename "$f" .rs)

    echo "===================================================="
    echo "▶ Exporting Example $sequence_no/$total_selected: $example_name ($run_mode)"
    echo "===================================================="

    if [ "$DRY_RUN" = true ]; then
        echo "DRY RUN Would export $example_name"
        continue
    fi

    cargo_cmd=(cargo run)
    if [ "$run_mode" = "release" ]; then
        cargo_cmd+=(--release)
    fi
    if example_requires_experimental "$f"; then
        cargo_cmd+=(--features experimental)
    fi
    cargo_cmd+=(--example "$example_name" -- --export)

    if [ "${#extra_args[@]}" -gt 0 ]; then
        cargo_cmd+=("${extra_args[@]}")
    fi

    "${cargo_cmd[@]}"

    status=$?
    if [ $status -ne 0 ]; then
        echo "Example $sequence_no/$total_selected ($example_name) exited with status $status. Stopping."
        exit $status
    fi

    echo "OK Completed $sequence_no/$total_selected: $example_name"
done

echo "Done! Selected examples have been exported."
