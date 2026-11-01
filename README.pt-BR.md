# Tramuri

<p align="center">
  <img src="https://raw.githubusercontent.com/tramuri/tramuri/main/assets/logo.png" alt="Logo do Tramuri" width="220">
</p>

<p align="center"><a href="https://github.com/tramuri/tramuri/blob/main/README.md">English</a> · <a href="https://github.com/tramuri/tramuri/blob/main/README.pt-BR.md">Português (Brasil)</a></p>

<p align="center"><strong>Da intenção a resultados verificados.</strong></p>

<p align="center">
  <a href="https://github.com/tramuri/tramuri/actions/workflows/ci.yml"><img src="https://github.com/tramuri/tramuri/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI"></a>
  <a href="https://pypi.org/project/tramuri/"><img src="https://img.shields.io/pypi/v/tramuri" alt="Versão no PyPI"></a>
</p>

O Tramuri é uma infraestrutura open source e local-first para transformar objetivos de produto e
negócio em trabalho coordenado e resultados verificáveis. Ele ajuda pessoas, agentes e ferramentas
a preservar contexto entre sessões e a distinguir trabalho concluído de resultados que foram
realmente avaliados.

O Tramuri foi pensado para Product Engineers, Forward Deployed Engineers (FDEs) e equipes de
produto. Ele não exige um repositório de código, um LLM nem um framework específico de agentes.

## O que o Tramuri oferece

- **Missões e Contratos de Sucesso** para descrever objetivos e critérios de aceitação.
- **Conhecimento persistente** com referências às fontes e histórico.
- **Registros de execução e evidências** que mantêm separados o trabalho, suas referências de apoio
  e sua avaliação.
- **Resultados verificados** com base na avaliação explícita dos critérios. Concluir uma execução
  não marca uma missão como alcançada.
- **Descoberta de projetos** como recurso opcional para inventariar um projeto de código. A análise
  não executa arquivos do repositório e exclui caminhos sensíveis por padrão.
- Uma CLI nativa em Rust e um SDK Python apoiados pelo mesmo core Rust.

O SDK Python publicado inclui uma extensão Rust pré-compilada nas plataformas suportadas; o usuário
não precisa instalar Rust.

O Tramuri mantém o estado local em `.tramuri/`, com o SQLite em
`.tramuri/database/tramuri.sqlite3` e o relatório da análise mais recente em
`.tramuri/discovery/`. Evidências são registradas como referências; o Tramuri não busca recursos
externos nem armazena o conteúdo dos arquivos analisados como evidência.

## Pré-requisitos

| Requisito | Versão | Verificação | Instalação |
| --- | --- | --- | --- |
| Python | 3.11 ou mais recente | `python --version` | [python.org](https://www.python.org/downloads/) |
| `uv` (recomendado) | Qualquer versão atual | `uv --version` | [guia de instalação do uv](https://docs.astral.sh/uv/getting-started/installation/) |

Instalações pelo PyPI usam wheels pré-compiladas nas plataformas suportadas. Não é necessário
instalar Rust.

## Instalação

### SDK Python

Adicione o Tramuri a um projeto Python existente:

```bash
uv add tramuri
```

Ou use pip:

```bash
python -m pip install tramuri
```

### CLI e integrações com assistentes

O Tramuri 0.3.0 inclui a distribuição da CLI e o instalador das integrações com assistentes.
Instale a CLI em um ambiente isolado:

```bash
uv tool install tramuri
tramuri --version
tramuri init .
tramuri scan .
```

`pipx install tramuri` também está disponível como alternativa para uma instalação isolada.
Se o comando `tramuri` não for encontrado após a instalação, execute `uv tool update-shell` e
abra um novo terminal.

Para adicionar as instruções do Tramuri ao projeto, escolha uma plataforma de assistente:

```bash
tramuri install --project --platform codex
tramuri install --project --platform copilot
tramuri install --project --platform claude
tramuri install --project --platform gemini
tramuri integrations status
```

O instalador adiciona instruções da plataforma, uma skill do Tramuri e suas referências sob demanda.
Ele preserva o conteúdo existente e não instala hooks. Essas integrações são opcionais: o Tramuri
não exige um assistente para usar a CLI ou o SDK.

### Como um assistente pode usar o Tramuri

O instalador do projeto foi pensado para fornecer instruções persistentes e nativas da plataforma
ao assistente sobre quando o Tramuri é útil. A skill explica o fluxo; a CLI executa operações do
Tramuri sobre o estado local do projeto. Ambos são opcionais, e o assistente continua livre para
usar outras ferramentas.

| Interface | Uso |
| --- | --- |
| CLI | Executar descoberta local e fluxos de conhecimento e missões. |
| Skill e instruções do assistente | Ajudar um assistente configurado a escolher e usar a CLI do Tramuri. |
| SDK Python | Chamar os mesmos recursos apoiados pelo core Rust a partir de aplicações Python. |
| MCP | O Tramuri ainda não oferece um servidor MCP; use a CLI ou o SDK. |

Criar uma missão não exige um repositório Git nem uma análise do projeto. Consulte a
[visão do produto](VISION.md) para conhecer o escopo e os princípios do Tramuri, e o
[changelog](CHANGELOG.md) para ver o histórico de versões.

## Estado do projeto

O Tramuri está em desenvolvimento inicial. Descoberta de projetos, conhecimento persistente,
missões, registros manuais de execução e evidências, e avaliações de critérios fazem parte do
produto atual. A exportação versionada de Context Packs e Evidence Packs está planejada e usará
seleção e autorização explícitas, com proteção segura por padrão.

## Comunidade

- [Como contribuir](CONTRIBUTING.md)
- [Código de Conduta](CODE_OF_CONDUCT.md)
- [Política de Segurança](SECURITY.md)
- [Suporte](SUPPORT.md)
- [Licença](LICENSE) (Apache-2.0)
