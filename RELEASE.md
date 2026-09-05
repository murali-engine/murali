# Release

This is the only release guide for the engine repository. It covers both artifacts:

| Artifact | Users get it from | How it is published |
| --- | --- | --- |
| Rust crate `murali` | crates.io | `cargo publish` from your machine |
| Python package `murali-engine` | PyPI | GitHub Actions, on a `v*` tag |

`pip install murali-engine` never reads GitHub Actions. CI **builds** the wheels; the tag job
**uploads** them to PyPI; pip then picks a matching wheel. Do not `maturin upload` a local macOS
wheel for a version that CI will also publish.

Release **engine first**, then [murali-kit](https://github.com/murali-engine/murali-kit). Kit
depends on a published `murali-engine` range.

Set the version once:

```bash
export VERSION=0.3.0
```

The git tag is `v${VERSION}`. Cargo.toml, pyproject.toml, and the Python wheel must use the same
number.

## How The Python Wheels Work

`.github/workflows/wheels.yml` builds:

- macOS arm64 and x86_64
- Linux x86_64 and aarch64 (manylinux 2_28)
- Windows x86_64
- an sdist for other platforms

The package is `abi3` for CPython 3.10+, so one wheel per OS/arch covers 3.10 and newer. Those
installs do not need Rust. The sdist still does (Rust 1.85+).

PRs and `main` **build and smoke-test** wheels. Only `v*` tags **publish**.

## One-Time Setup

### crates.io

```bash
cargo login
```

Use a crates.io API token. This is only for `cargo publish`. It is not used by the wheel workflow.

### PyPI trusted publishing

Configure once at
`https://pypi.org/manage/project/murali-engine/settings/publishing/`
before the first tag upload. GitHub → Add a new publisher. The fields must match the job
**exactly** (a mismatch is `invalid-publisher`):

| Field | Value |
| --- | --- |
| Owner | `murali-engine` |
| Repository | `murali` |
| Workflow name | `wheels.yml` (filename only, not `.github/workflows/wheels.yml`) |
| Environment name | `pypi` (must not be blank) |

In GitHub, create an Environment named `pypi` (`Settings → Environments`). Protection rules are
optional.

If a `v*` publish already failed with `invalid-publisher`, save the publisher on PyPI, then
**Re-run failed jobs** on that workflow run. Do not cut a new tag. Wheel artifacts from the same
run are reused.

Do not put a PyPI token in the repo or in chat. The publish job uses OIDC.

## 1. Bump Metadata

Update the same version in:

- `Cargo.toml` (`version`)
- `Cargo.lock` (`cargo update -p murali` or a normal build that rewrites the lock)
- `pyproject.toml`
- `CHANGELOG.md`
- install pins in `README.md`, `docs/docs/intro.mdx`, and `docs/docs/installation.md`

```bash
rg "$VERSION" Cargo.toml Cargo.lock pyproject.toml README.md docs/docs
scripts/check-release-metadata.sh
```

That script checks crate/Python pins, licenses, `lastVersion` matching the crate, and a Next
docs line. Historical 0.2.x pages stay in the version dropdown.

## 2. Checks

```bash
cargo fmt --all --check
cargo test --all-targets
cargo test --features python python
cargo clippy --all-targets --all-features -- -A warnings -D clippy::correctness -D clippy::suspicious
cargo check --no-default-features --all-targets
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
npm run build --prefix docs
scripts/check-release-metadata.sh
cargo package --list
cargo publish --dry-run
uv lock --check
```

Review `cargo package --list`. The crate excludes `docs/**`, `examples/**`, and `RELEASE.md`.

A local wheel is optional and only for the machine you are on:

```bash
uv run --locked maturin build --release --features python --locked
```

Do not upload it.

## 3. Cargo Publish (Rust Crate)

Commit the version bump, then publish the crate **before or immediately with** the git tag. crates.io
does not use GitHub Actions.

```bash
git add -A
git commit -m "Release murali ${VERSION}"
git tag "v${VERSION}"
cargo publish
```

If `cargo publish` fails, do not push the tag. Yank or fix, then try again. A published crates.io
version cannot be replaced; only yanked.

Wait until `https://crates.io/crates/murali/${VERSION}` exists and `cargo search murali` sees it.

## 4. Push The Tag (Python Wheels)

```bash
git push origin main
git push origin "v${VERSION}"
```

Pushing `main` deploys docs via `.github/workflows/deploy.yml`.
Pushing `v${VERSION}` runs `.github/workflows/wheels.yml` and, if every platform job passes,
uploads wheels and the sdist to PyPI.

Watch the **Python wheels** workflow. Each platform job installs the wheel and runs
`python/tests/test_bindings.py`.

If one platform fails, nothing new is published (`needs: [linux, windows, macos, sdist]`). Fix,
tag a new version if the failed tag already created partial files (PyPI files are immutable;
`skip-existing` only skips identical filenames).

## 5. GitHub Release

Tags do not create a GitHub Release object. Add one so the Releases page is not empty:

```bash
gh release create "v${VERSION}" \
  --title "Murali v${VERSION}" \
  --notes-file /tmp/murali-release-notes.md
```

Notes usually come from `CHANGELOG.md`. Link crates.io, PyPI, and `vPREVIOUS...v${VERSION}`.

## 6. Verify

Use a plain `venv` and `pip` for this release check intentionally: it verifies the installation
path used by people who do not have uv.

```bash
python3 -m venv /tmp/murali-pypi-test
/tmp/murali-pypi-test/bin/python -m pip install --upgrade pip
/tmp/murali-pypi-test/bin/python -m pip install "murali-engine==${VERSION}"
/tmp/murali-pypi-test/bin/python -c "from murali_engine import Scene, Circle, Timeline; print('pypi ok')"
```

Confirm all platform wheels are listed:

```text
https://pypi.org/project/murali-engine/#files
https://crates.io/crates/murali
https://github.com/murali-engine/murali/releases
https://muraliengine.com/docs/intro
```

Then release kit: [murali-kit RELEASE.md](https://github.com/murali-engine/murali-kit/blob/main/RELEASE.md).

## Docs Freeze

After a named release, lock that docs set and put ongoing work under Next:

```bash
cd docs
npm ci
npm run docusaurus -- docs:version ${VERSION}
```

That copies `docs/docs/` to `versioned_docs/version-${VERSION}`. Then in `docusaurus.config.ts`:

```ts
lastVersion: '${VERSION}',
versions: {
  current: {
    label: 'Next 🚧',
    path: 'next',
  },
},
```

`/docs` is the frozen release. `/docs/next` is unreleased. Keep `0.2.4` in the dropdown for Rust
authoring. Do not freeze on every patch.

## Do Not

- Upload one local wheel with `maturin upload` / `maturin publish`
- Reuse a yanked or failed version number for different files
- Release kit before `murali-engine==${VERSION}` is installable from PyPI
- Put PyPI or crates.io tokens in the repo, the workflow file, or chat

## AI Helper Prompt

```text
I am releasing murali ${VERSION} from the engine repository.
Follow RELEASE.md only. Publish the Rust crate with cargo publish, then push tag v${VERSION}
so GitHub Actions uploads murali-engine wheels to PyPI. Do not maturin upload a local wheel.
Do not ask me to paste tokens. Trusted publishing is wheels.yml / environment pypi.
```
