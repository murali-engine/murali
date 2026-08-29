# Release And Docs Versioning

Use this checklist to publish a Murali crate, freeze matching Docusaurus documentation, and create
the matching GitHub Release. Set `VERSION` to the version being released, for example `0.2.2`.

For the Python package release, use [`PYPI_RELEASE.md`](./PYPI_RELEASE.md).

## 1. Prepare The Crate

- Update `Cargo.toml`, `Cargo.lock`, installation snippets, `CHANGELOG.md`, and any release post.
- Confirm license metadata matches the repository license files.
- Run the Rust checks:

```bash
cargo fmt --all -- --check
cargo test --all-targets
cargo check --no-default-features --all-targets
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
```

## 2. Validate And Freeze The Docs

Update `docs/docs/` first, then run from `docs/`:

```bash
npm ci
npm run typecheck
npm run build
npm run docusaurus -- docs:version VERSION
npm run build
```

Version docs only once the live pages represent the released API.

Murali currently keeps only the latest frozen release docs to avoid stale API guidance confusing
search engines and AI readers. After `docs:version VERSION`, prune older frozen docs unless there is
a deliberate reason to keep them:

```bash
# Keep only docs/versioned_docs/version-VERSION and
# docs/versioned_sidebars/version-VERSION-sidebars.json.
$EDITOR versions.json
rm -rf versioned_docs/version-OLD versioned_sidebars/version-OLD-sidebars.json
npm run build
```

Then return to the repository root and verify release metadata:

```bash
scripts/check-release-metadata.sh
cargo package --list
cargo publish --dry-run
```

Review the package contents before publishing.

## 3. Commit, Tag, Publish, And Push

```bash
git add -A
git commit -m "Release murali VERSION"
git tag vVERSION
cargo publish
git push origin main
git push origin vVERSION
```

Pushing `main` deploys the site through `.github/workflows/deploy.yml`.

## 4. Create The GitHub Release

GitHub Releases are not created automatically from tags. Create a release object for `vVERSION` so
the repository Releases page does not look stale.

If the GitHub CLI is available:

```bash
gh release create vVERSION \
  --title "Murali vVERSION" \
  --notes-file /tmp/murali-release-notes.md
```

Otherwise, use the GitHub UI:

1. Open `https://github.com/murali-engine/murali/releases/new`.
2. Select tag `vVERSION`.
3. Set the title to `Murali vVERSION`.
4. Paste release notes from `CHANGELOG.md`.
5. Include links to crates.io and the compare range `vPREVIOUS...vVERSION`.
6. Publish the release.

## 5. Post-Release Checks

- Verify the new crates.io version and a fresh dependency resolution.
- Verify the GitHub Releases page shows `vVERSION` as the latest release.
- Verify the deployed docs, version selector, and release post.
- Confirm README installation instructions point to the released version.
- Run `scripts/check-release-metadata.sh` once more against the tagged tree.
