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

3. Commit and push a tag matching the package version, such as `vX.Y.Z`.

`.github/workflows/publish.yml` verifies that the tag matches the package,
re-runs tests, builds the wheel/sdist, and publishes through the protected
`pypi` environment using GitHub OIDC. It does not store a PyPI token in the
repository. `skip-existing` makes rerunning a tag safe.

## Rehearse with TestPyPI

The repository also includes `.github/workflows/testpypi.yml`. It is deliberately
manual (`workflow_dispatch`) so a normal push cannot publish an artifact, even to
the staging index. The workflow runs the same tests and distribution checks as the
production release, then publishes through the protected `testpypi` environment
using GitHub OIDC.

Before the first run, register a separate trusted publisher at
<https://test.pypi.org/manage/account/publishing/> with these exact values:

- Repository owner and name: the GitHub repository that contains Tramuri.
- Workflow filename: `testpypi.yml`.
- Environment: `testpypi`.

After the workflow finishes, verify installation from both indexes because Tramuri's
runtime dependencies are published on production PyPI:

```bash
uv tool install \
  --index-url https://test.pypi.org/simple/ \
  --extra-index-url https://pypi.org/simple/ \
  --index-strategy unsafe-best-match \
  tramuri
tramuri --version
```

TestPyPI is a disposable staging registry; it is not a source of production
dependencies and its projects may be periodically removed.

## Dependabot

`.github/dependabot.yml` updates the `uv.lock` dependency set and GitHub Actions
weekly. Minor and patch Python updates are grouped to reduce review noise, major
updates remain separate, and a short cooldown avoids adopting newly published
versions immediately. Security updates can still be proposed outside the cooldown.
Dependabot pull requests must pass CI and receive normal human review; they are not
auto-merged.
