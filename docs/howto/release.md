# Prepare a PyPI release

The published project name remains `tramuri`; its existing logo is preserved in
the README and packaged under `src/tramuri/assets/logo.png`.

1. Update the static version in `pyproject.toml`, `src/tramuri/__init__.py`
   fallback and `CHANGELOG.md`.
2. Run the locked checks:

   ```bash
   uv sync --locked --all-groups
   uv run pytest --cov=tramuri --cov-report=term-missing
   uv run ruff check src tests
   uv run ruff format --check src tests
   uv run mypy src/tramuri
   uv run python -m build
   uv run twine check dist/*
   ```

3. Commit and push a tag such as `v0.13.0`.

`.github/workflows/publish.yml` verifies that the tag matches the package,
re-runs tests, builds the wheel/sdist, and publishes through the protected
`pypi` environment using GitHub OIDC. It does not store a PyPI token in the
repository. `skip-existing` makes rerunning a tag safe.
