# Contributing to Tramuri

Thanks for helping improve Tramuri. Contributions should preserve its local-first design and its
separation between objectives, execution, evidence, criterion assessment, and verified outcomes.
The Rust core owns domain behavior; the CLI and Python SDK expose that shared behavior.

## Before opening a change

1. Check existing issues and open a focused issue for substantial changes.
2. Keep changes small and explain the problem they solve.
3. For changes to architecture, public APIs, storage schemas, security behavior, or public
   contracts, discuss the compatibility impact and document the decision before implementation.
4. Do not include real customer data, credentials, or sensitive repository contents in examples or
   tests.

## Development checks

Development requires stable Rust, Python 3.11 or newer, and `uv`. On Windows, use the MSVC C++
build tools. Install the locked development environment and build the CLI and Python extension:

```bash
uv sync --locked --no-install-project
cargo build --locked --package tramuri-cli
uv run maturin develop --locked --manifest-path tramuri/bindings/python/Cargo.toml
```

Run the checks used by CI:

```bash
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
uv run pytest
```

Python tests that compare the SDK with the native CLI use `TRAMURI_CLI` to locate the compiled
executable. The GitHub Actions workflow configures this for its platform matrix.

## Pull requests

- Describe the behavior change and any compatibility or migration impact.
- Add or update tests for changed behavior, including CLI/SDK parity when applicable.
- Report the commands you ran and any checks you could not run.
- Keep generated files, local databases, build output, and unrelated edits out of the pull request.

By contributing, you agree that your contributions are made under the project's Apache-2.0
license.
