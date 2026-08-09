# Release And Docs Versioning

Use this checklist to publish a Murali crate and freeze matching Docusaurus documentation. Set
`VERSION` to the version being released.

## 1. Prepare The Crate

- Update `Cargo.toml`, `Cargo.lock`, installation snippets, `CHANGELOG.md`, and any release post.
- Confirm license metadata matches the repository license files.
- Run:

```bash
scripts/check-release-metadata.sh
cargo fmt --all -- --check
cargo test --all-targets
cargo check --no-default-features --all-targets
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
cargo package --list
cargo publish --dry-run
```

Review the package contents and publish with `cargo publish` only after the dry run succeeds.

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

## 3. Commit, Tag, And Push

```bash
git add -A
git commit -m "Release murali VERSION"
git tag vVERSION
git push origin main
git push origin vVERSION
```

Pushing `main` deploys the site through `.github/workflows/deploy.yml`.

## 4. Post-Release Checks

- Verify the new crates.io version and a fresh dependency resolution.
- Verify the deployed docs, version selector, and release post.
- Confirm README installation instructions point to the released version.
- Run `scripts/check-release-metadata.sh` once more against the tagged tree.
