#!/bin/bash

EXAMPLE_TAG_PREFIX="// murali-example-tags:"

example_name_for_file() {
    basename "$1" .rs
}

example_tags_for_file() {
    local file="$1"
    local tag_line
    tag_line=$(grep -m 1 "^$EXAMPLE_TAG_PREFIX" "$file" || true)
    tag_line=${tag_line#"$EXAMPLE_TAG_PREFIX"}
    echo "$tag_line" | tr ',' ' '
}

list_example_tags() {
    local file
    for file in "$@"; do
        example_tags_for_file "$file"
    done | tr ' ' '\n' | sed '/^$/d' | sort -u
}

array_contains() {
    local needle="$1"
    shift
    local value
    for value in "$@"; do
        if [ "$value" = "$needle" ]; then
            return 0
        fi
    done
    return 1
}

example_has_tag() {
    local file="$1"
    local tag="$2"
    local example_tags
    read -r -a example_tags <<< "$(example_tags_for_file "$file")"
    array_contains "$tag" ${example_tags[@]+"${example_tags[@]}"}
}

example_requires_experimental() {
    local file="$1"
    example_has_tag "$file" "linear-algebra"
}

example_matches_filters() {
    local file="$1"

    local example_name
    example_name=$(example_name_for_file "$file")

    if [ "${#EXAMPLE_NAMES[@]}" -gt 0 ] && ! array_contains "$example_name" "${EXAMPLE_NAMES[@]}"; then
        return 1
    fi

    if [ "${#INCLUDE_TAGS[@]}" -gt 0 ]; then
        local matched_include=false
        local tag
        for tag in ${INCLUDE_TAGS[@]+"${INCLUDE_TAGS[@]}"}; do
            if example_has_tag "$file" "$tag"; then
                matched_include=true
                break
            fi
        done
        if [ "$matched_include" = false ]; then
            return 1
        fi
    fi

    local tag
    for tag in ${EXCLUDE_TAGS[@]+"${EXCLUDE_TAGS[@]}"}; do
        if example_has_tag "$file" "$tag"; then
            return 1
        fi
    done

    return 0
}
