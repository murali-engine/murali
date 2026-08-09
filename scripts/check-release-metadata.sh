#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)
if [[ -z "$version" ]]; then
  echo "Could not read the package version from Cargo.toml" >&2
  exit 1
fi

require_text() {
  local file=$1
  local expected=$2

  if ! grep -Fq "$expected" "$file"; then
    echo "$file is missing: $expected" >&2
    exit 1
  fi
}

lock_version=$(awk '
  $0 == "name = \"murali\"" { found = 1; next }
  found && /^version = / { gsub(/^version = \"|\"$/, ""); print; exit }
' Cargo.lock)

if [[ "$lock_version" != "$version" ]]; then
  echo "Cargo.lock has murali $lock_version, expected $version" >&2
  exit 1
fi

require_text Cargo.toml 'license = "MIT OR Apache-2.0"'
require_text README.md "murali = \"$version\""
require_text README.md 'dual-licensed under either the MIT License or the Apache License'
require_text docs/docs/installation.md "murali = \"$version\""
require_text docs/docs/intro.mdx "murali = \"$version\""
require_text docs/docusaurus.config.ts "lastVersion: '$version'"

first_docs_version=$(sed -n 's/.*"\([^"]*\)".*/\1/p' docs/versions.json | head -n 1)
if [[ "$first_docs_version" != "$version" ]]; then
  echo "docs/versions.json starts with $first_docs_version, expected $version" >&2
  exit 1
fi

for path in \
  "docs/versioned_docs/version-$version" \
  "docs/versioned_sidebars/version-$version-sidebars.json"; do
  if [[ ! -e "$path" ]]; then
    echo "Missing frozen documentation artifact: $path" >&2
    exit 1
  fi
done

for file in LICENSE-APACHE LICENSE-MIT; do
  if [[ ! -s "$file" ]]; then
    echo "Missing non-empty $file" >&2
    exit 1
  fi
done

require_text LICENSE-APACHE 'Apache License'
require_text LICENSE-MIT 'MIT License'

echo "Release metadata is consistent for murali $version"
