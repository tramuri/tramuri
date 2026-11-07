# Contribuindo com o Tramuri

Obrigado por contribuir. O Tramuri é uma biblioteca/CLI Python local-first para
Product Management agent-native. O core, o CLI, os adapters e os relatórios
devem permanecer independentes de um modelo específico de IA.

## Desenvolvimento

Requisitos: Python 3.12+ e `uv`. Não é necessário Rust, Node.js, Bun, Docker ou
um banco externo.

```bash
uv sync --group dev
uv run pytest --cov=tramuri --cov-report=term-missing
uv run ruff check src tests
uv run ruff format --check src tests
uv run mypy src/tramuri
uv run python -m build
```

Para a demo determinística:

```bash
uv run tramuri --project-dir .tramuri-demo pm demo --mode mvp
```

## Processo

Especificações do produto ficam em `tramuri-harness-kit/specs/`; as instruções,
skills e workflows de construção ficam em `tramuri-harness-kit/.specify/` e
`tramuri-harness-kit/.agents/`. O harness não é dependência de runtime e não
deve ser duplicado na raiz do pacote.

Mudanças de schema, contratos públicos, segurança, adapters ou instalação devem
incluir testes e, quando necessário, uma decisão arquitetural. Não adicione
MCPs, serviços externos ou credenciais aos testes.

## Pull requests

Descreva comportamento, compatibilidade e comandos executados. Preserve o
`assets/logo.png`, o README bilíngue e o fluxo de publicação PyPI. Não inclua
bancos locais, artefatos de build, credenciais ou dados de usuário.
