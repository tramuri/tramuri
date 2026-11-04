# Tramuri

<p align="center">
  <img src="https://raw.githubusercontent.com/tramuri/tramuri/main/assets/logo.png" alt="Logo do Tramuri" width="220">
</p>

<p align="center"><a href="https://github.com/tramuri/tramuri/blob/main/packages/typescript/README.md">English</a> · <a href="https://github.com/tramuri/tramuri/blob/main/packages/typescript/README.pt-BR.md">Português (Brasil)</a></p>

<p align="center"><strong>Da intenção a resultados verificados.</strong></p>

<p align="center">
  <a href="https://www.npmjs.com/package/@tramuri/typescript"><img src="https://img.shields.io/npm/v/@tramuri/typescript" alt="Versão no npm"></a>
  <a href="https://pypi.org/project/tramuri/"><img src="https://img.shields.io/pypi/v/tramuri" alt="Versão no PyPI"></a>
</p>

O Tramuri é uma infraestrutura open source e local-first para transformar objetivos de produto e negócio em trabalho coordenado e resultados verificáveis. Ele ajuda pessoas, agentes e ferramentas a preservar contexto entre sessões e a distinguir trabalho concluído de resultados que foram realmente avaliados.

O Tramuri foi pensado para Product Engineers, Forward Deployed Engineers (FDEs) e equipes de produto. O produto completo não exige um repositório de código, um LLM nem um framework específico de agentes.

> **Escopo deste pacote:** `@tramuri/typescript` é uma prévia inicial da CLI em TypeScript. Atualmente, permite escolher uma perspectiva de trabalho (Product Engineer, FDE ou misto) no terminal ou obter a escolha em JSON. Ele **ainda não oferece** Missions, SQLite, Knowledge, Execution, Evidence, Outcome, integrações ou o workflow DEFINE completos disponíveis na distribuição Python. Este pacote npm não exige Python nem Rust; a CLI completa distribuída no PyPI requer Python 3.11+ e usa wheels pré-compiladas com Rust nas plataformas compatíveis.

Está começando? Siga o [tutorial de Product Engineer](https://github.com/tramuri/tramuri/tree/main/docs/examples/product-engineer) ou o [tutorial de FDE](https://github.com/tramuri/tramuri/tree/main/docs/examples/fde). Ambos têm instruções em português brasileiro.

## O que o produto completo Tramuri oferece

- **Missions e Success Contracts** para descrever objetivos e critérios de aceitação.
- **Conhecimento persistente** com referências às fontes e histórico.
- **Registros de execução e evidências** que mantêm separados o trabalho, suas referências de apoio e sua avaliação.
- **Resultados verificados** com base na avaliação explícita dos critérios. Concluir uma execução não marca uma Mission como alcançada.
- **Descoberta de projetos opcional** para inventariar um projeto de código sem executar arquivos do repositório.
- **Descoberta de capacidades** para catalogar skills, instruções, agentes, hooks e declarações MCP compatíveis no projeto, sem executá-los nem se conectar a eles.
- Uma CLI nativa em Rust e um SDK Python apoiados pelo mesmo core Rust.

O pacote Python publicado no PyPI inclui uma extensão Rust pré-compilada nas plataformas compatíveis; o usuário não precisa instalar Rust. O estado local fica em `.tramuri/`. Evidências são registradas como referências, não como conteúdo buscado externamente. Hooks e servidores MCP declarados não são considerados operacionalmente verificados.

## O que esta prévia npm faz

O pacote TypeScript atual fornece uma operação explícita:

```text
tramuri-cli perspective choose [--perspective <value>] [--json]
```

Valores disponíveis:

| Valor | Significado |
| --- | --- |
| `product_engineer` | Foco no problema do produto e nos resultados para usuários. |
| `fde` | Foco em uma necessidade de cliente, sistemas, restrições e aceite. |
| `mixed` | Conecta a evolução do produto a uma integração de cliente. |

Sem `--perspective`, a CLI apresenta um seletor de opções navegável pelo teclado. Com `--json`, retorna o resultado versionado `tramuri.work-perspective-selection.v1`. A escolha é devolvida para quem chamou o comando; ela não é persistida e não cria nem seleciona uma Mission.

Esta prévia não instala instruções, skills ou hooks de agentes e ainda não implementa os workflows completos descritos acima. Para essas capacidades, use a distribuição Python.

## Instalar a CLI TypeScript

### Node.js

Requer Node.js 22.14 ou mais recente. Não é necessário instalar Bun.

```bash
npm install --global @tramuri/typescript
tramuri-cli --help
tramuri-cli perspective choose --perspective product_engineer --json
```

Para executar uma vez sem instalação global:

```bash
npx --yes --package @tramuri/typescript tramuri-cli --help
```

O nome anterior `tramuri-ts` continua disponível como alias de compatibilidade.

### Bun

Bun é opcional. Você pode adicionar o pacote a um projeto JavaScript/TypeScript e executar o comando sem instalar Python ou Rust:

```bash
bun add @tramuri/typescript
bun run tramuri-cli --help
bun run tramuri-cli perspective choose --perspective fde --json
```

O pacote também é testado com Bun 1.3.14. Quem usa Node.js pode instalar e executar `tramuri-cli` com npm/npx sem Bun.

## Instalar a CLI completa e o SDK Python

O produto completo atual é distribuído como `tramuri` no PyPI, em um pacote separado de `@tramuri/typescript`.

Para instalar a CLI e suas integrações locais com agentes em um ambiente isolado:

```bash
uv tool install tramuri
tramuri --version
```

A instalação requer Python 3.11 ou mais recente e uma wheel pré-compilada compatível; não é necessário instalar Rust nas plataformas suportadas. Para usar o SDK em um projeto Python:

```bash
uv add tramuri
```

Ou use pip:

```bash
python -m pip install tramuri
```

Atualize com o mesmo gerenciador usado na instalação:

```bash
uv tool upgrade tramuri
# ou, se instalou com pipx:
pipx upgrade tramuri
```

Instale instruções, skills e hooks compatíveis no projeto para o host escolhido, preservando configurações não relacionadas:

```bash
tramuri install --project --platform codex
```

As integrações da distribuição Python incluem Codex, Claude Code, Cursor, Gemini CLI e GitHub Copilot, respeitando o suporte nativo e as configurações de confiança de cada host. Hooks não funcionam em todas as superfícies e não garantem que um agente seguiu as instruções. Quando existe um hook compatível, ativo e confiável, o workflow completo do Tramuri pode exigir iniciar ou retomar uma Mission antes do uso de ferramentas do projeto. Em hosts sem um hook aplicável, instruções e skills servem como orientação.

Para tutoriais detalhados, consulte [Product Engineer](https://github.com/tramuri/tramuri/tree/main/docs/examples/product-engineer), [FDE](https://github.com/tramuri/tramuri/tree/main/docs/examples/fde) e a [página do pacote Python](https://pypi.org/project/tramuri/).

## Desenvolvimento

No diretório `packages/typescript`:

```bash
bun install --frozen-lockfile
bun run ci
npm pack --dry-run
```

O pacote usa a licença Apache-2.0. Consulte o [repositório](https://github.com/tramuri/tramuri), a [visão do produto](https://github.com/tramuri/tramuri/blob/main/VISION.md) e o [changelog](./CHANGELOG.md).
