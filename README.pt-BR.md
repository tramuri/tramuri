# Tramuri

<p align="center">
  <img src="https://raw.githubusercontent.com/tramuri/tramuri/main/assets/logo.png" alt="Logo Tramuri" width="220">
</p>

<p align="center"><a href="https://github.com/tramuri/tramuri/blob/main/README.md">English</a> · <a href="https://github.com/tramuri/tramuri/blob/main/README.pt-BR.md">Português (Brasil)</a></p>

<p align="center"><strong>Da intenção a uma definição de produto verificável.</strong></p>

O Tramuri é uma ferramenta Python local-first para transformar uma ideia de
produto incompleta em um resultado estruturado e revisável de Product
Management. Ele persiste o workflow em SQLite, instala orientações nativas para
Codex, Claude Code e Gemini CLI e gera um relatório offline com protótipo
navegável.

A primeira versão implementa apenas Product Management. Ela não simula módulos
de UX ou Engenharia, não exige um modelo hospedado e não pede credenciais
duplicadas dos providers.

## Instalação

```bash
uv tool install tramuri
tramuri --version
```

Alternativas:

```bash
pipx install tramuri
# ou
python -m pip install tramuri
```

## Demo determinística

```bash
tramuri pm demo --mode mvp
tramuri runs list
tramuri report open <run-id> --no-open
```

A demo não exige rede nem autenticação. Hipóteses e dados de exemplo são
marcados explicitamente como sintéticos.

## Componentes dos providers

```bash
tramuri providers list
tramuri init --provider codex
tramuri init --provider claude
tramuri init --provider gemini
tramuri init --provider all --dry-run
```

A instalação é local ao projeto, idempotente, consciente de conflitos e
registrada em manifest. Arquivos existentes não gerenciados são preservados.

## Desenvolvimento

```bash
uv sync --group dev
uv run pytest
uv run ruff check .
uv run mypy src/tramuri
uv build
```

Consulte [docs/index.md](docs/index.md) para arquitetura, capabilities dos
providers, workflow de PM e release.

## Licença

Apache-2.0. Consulte [LICENSE](LICENSE).
