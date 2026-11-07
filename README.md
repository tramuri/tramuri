# Tramuri

<p align="center">
  <img src="https://raw.githubusercontent.com/tramuri/tramuri/main/assets/logo.png" alt="Tramuri logo" width="220">
</p>

<p align="center"><a href="https://github.com/tramuri/tramuri/blob/main/README.md">English</a> · <a href="https://github.com/tramuri/tramuri/blob/main/README.pt-BR.md">Português (Brasil)</a></p>

<p align="center"><strong>From intent to a verified product definition.</strong></p>

<p align="center">
  <a href="https://github.com/tramuri/tramuri/actions/workflows/ci.yml"><img src="https://github.com/tramuri/tramuri/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI"></a>
  <a href="https://pypi.org/project/tramuri/"><img src="https://img.shields.io/pypi/v/tramuri" alt="PyPI version"></a>
</p>

Tramuri is a local-first Python tool for turning an incomplete product idea into
a structured, reviewable Product Management result. It keeps workflow state in
SQLite, installs provider-native guidance for Codex, Claude Code and Gemini CLI,
and generates an offline report plus a navigable prototype.

The first release implements Product Management only. It does not pretend to
implement UX or Engineering modules, and it does not require a hosted model or
duplicate provider credentials.

## Install

For an isolated CLI:

```bash
uv tool install tramuri
tramuri --version
```

Alternatively:

```bash
pipx install tramuri
# or
python -m pip install tramuri
```

## Run the deterministic demo

```bash
tramuri pm demo --mode mvp
tramuri runs list
tramuri report open <run-id> --no-open
```

The demo requires no network or provider authentication. Generated hypotheses
and example data are explicitly marked as synthetic.

## Install provider components

```bash
tramuri providers list
tramuri init --provider codex
tramuri init --provider claude
tramuri init --provider gemini
tramuri init --provider all --dry-run
```

Installation is project-local by default, idempotent, conflict-aware and
manifested. Existing unmanaged files are preserved.

## Development

```bash
uv sync --group dev
uv run pytest
uv run ruff check .
uv run mypy src/tramuri
uv build
```

See [docs/index.md](docs/index.md) for the architecture, provider capability
matrix, PM workflow and release procedure.

## License

Apache-2.0. See [LICENSE](LICENSE).
