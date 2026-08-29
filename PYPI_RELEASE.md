# PyPI Release Checklist

Use this checklist to publish the `murali-engine` Python package from this repository.

Set the release version once at the start:

```bash
export VERSION=0.2.5
```

## Accounts And Tokens

You need a PyPI account to publish the real package:

- create an account at `https://pypi.org/account/register/`
- verify the account email address
- create an API token at `https://pypi.org/manage/account/token/`
- prefer a project-scoped token after the first upload creates the project

For a dry run, create a separate TestPyPI account:

- TestPyPI account: `https://test.pypi.org/account/register/`
- TestPyPI token: `https://test.pypi.org/manage/account/token/`

PyPI and TestPyPI are separate services. Accounts and tokens do not carry across.

## 1. Confirm Metadata

From the repository root:

```bash
rg "$VERSION|0\.2\.4" Cargo.toml Cargo.lock pyproject.toml README.md docs/docs
```

Confirm these files use the release version:

- `Cargo.toml`
- `Cargo.lock`
- `pyproject.toml`
- current install snippets in `README.md` and `docs/docs`

The archived Docusaurus `versioned_docs/version-0.2.4` directory can remain historical.

## 2. Run Checks

```bash
cargo test --features python python
npm run build --prefix docs
git diff --check
```

Optional but recommended before the first public PyPI release:

```bash
cargo test --all-targets
cargo package --list
```

## 3. Build Locally

Create or reuse the local release environment:

```bash
python3 -m venv .venv
.venv/bin/python -m pip install --upgrade pip
.venv/bin/python -m pip install maturin
```

Build the release wheel:

```bash
.venv/bin/maturin build --release --features python
```

Artifacts are written to:

```text
target/wheels/
```

## 4. Test The Built Wheel

Use a fresh environment so the test does not accidentally import the local checkout:

```bash
python3 -m venv /tmp/murali-wheel-test
/tmp/murali-wheel-test/bin/python -m pip install --upgrade pip
/tmp/murali-wheel-test/bin/python -m pip install target/wheels/murali_engine-${VERSION}-*.whl
/tmp/murali-wheel-test/bin/python -c "from murali_engine import Scene, Circle, Timeline; print('murali-engine import ok')"
```

## 5. Optional TestPyPI Dry Run

Use the TestPyPI token:

```bash
export MATURIN_PYPI_TOKEN="pypi-..."
.venv/bin/maturin upload --repository-url https://test.pypi.org/legacy/ target/wheels/murali_engine-${VERSION}-*.whl
```

Verify from TestPyPI:

```bash
python3 -m venv /tmp/murali-testpypi-test
/tmp/murali-testpypi-test/bin/python -m pip install --upgrade pip
/tmp/murali-testpypi-test/bin/python -m pip install \
  --index-url https://test.pypi.org/simple/ \
  --extra-index-url https://pypi.org/simple/ \
  murali-engine==${VERSION}
/tmp/murali-testpypi-test/bin/python -c "import murali_engine; print('testpypi import ok')"
```

## 6. Publish To PyPI

Use the real PyPI token:

```bash
export MATURIN_PYPI_TOKEN="pypi-..."
.venv/bin/maturin upload target/wheels/murali_engine-${VERSION}-*.whl
```

If you prefer to build and publish in one command:

```bash
export MATURIN_PYPI_TOKEN="pypi-..."
.venv/bin/maturin publish --release --features python
```

For the first release, uploading the already-tested wheel is usually calmer.

## 7. Verify The Real Install

```bash
python3 -m venv /tmp/murali-pypi-test
/tmp/murali-pypi-test/bin/python -m pip install --upgrade pip
/tmp/murali-pypi-test/bin/python -m pip install murali-engine==${VERSION}
/tmp/murali-pypi-test/bin/python -c "from murali_engine import Scene, Circle, Timeline; print('pypi import ok')"
```

Then check the project page:

```text
https://pypi.org/project/murali-engine/
```

## AI Helper Prompt

When asking an AI assistant for help during release, paste this:

```text
I am releasing the Python package `murali-engine` from the Murali Rust repository.
The package uses PyO3 and maturin. The import name is `murali_engine`.
The release version is 0.2.5.

Please help me follow PYPI_RELEASE.md exactly:
1. verify metadata,
2. run checks,
3. build the wheel,
4. test the built wheel in a fresh virtualenv,
5. optionally upload to TestPyPI,
6. upload to PyPI,
7. verify a fresh install from PyPI.

Do not change unrelated source files. If a command needs a PyPI token, ask me to provide it through an environment variable instead of pasting it into chat.
```

