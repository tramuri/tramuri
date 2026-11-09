# Tramuri

<p align="center">
  <img src="https://raw.githubusercontent.com/tramuri/tramuri/main/assets/logo.png" alt="Logo Tramuri" width="220">
</p>

<p align="center"><a href="https://github.com/tramuri/tramuri/blob/main/README.md">English</a> · <a href="https://github.com/tramuri/tramuri/blob/main/README.pt-BR.md">Português (Brasil)</a></p>

<p align="center"><strong>Da intenção a uma definição de produto verificável.</strong></p>

O Tramuri é uma ferramenta Python local-first para transformar uma ideia de
produto incompleta em um resultado estruturado, rastreável e consciente de
evidências. Ele persiste o workflow em SQLite, instala orientações nativas para
Codex, Claude Code e Gemini CLI e gera um relatório bilíngue com uma experiência
de produto offline completa.

O workflow preserva o domínio informado, permite pesquisa de mercado produzida
pelo provider e gera superfícies de marketing, cliente e administração quando
faz sentido. Product Engineering continua sendo uma etapa futura; a experiência
gerada não é uma aplicação de produção.

O runtime é dividido entre um core neutro de providers, o módulo de produto e
harnesses finos para cada plataforma. O core possui estado e política do
workflow; os harnesses possuem as projeções nativas de Codex, Claude Code e
Gemini CLI. A API pública expõe diretamente os novos limites de core, produto e
harness; fachadas legadas não fazem parte desta versão.

## Pré-requisitos

| Requisito | Mínimo | Verificação | Instalação |
| --- | --- | --- | --- |
| Python | 3.10+ | `python --version` | [python.org](https://www.python.org/downloads/) |
| uv (recomendado) | qualquer versão atual | `uv --version` | `curl -LsSf https://astral.sh/uv/install.sh \| sh` |
| pipx (alternativa) | versão atual | `pipx --version` | `python -m pip install --user pipx` |

Atalhos por plataforma:

```bash
# macOS (Homebrew)
brew install python@3.12 uv

# Windows (PowerShell)
winget install --id=astral-sh.uv -e

# Ubuntu/Debian
sudo apt install python3.12 python3-pip pipx
# ou instale o uv:
curl -LsSf https://astral.sh/uv/install.sh | sh
```

## Quickstart (30 segundos)

```bash
uv tool install tramuri
tramuri init --provider auto
```

Abra o seu assistente no mesmo projeto e execute a skill instalada:

```text
/tramuri .        # Claude Code ou Gemini CLI
$tramuri .        # Codex
```

Isso já é suficiente para começar. Use `--provider all` em vez de `auto` para
instalar explicitamente os componentes dos três providers.

## Instalação

### Importante

O pacote oficial no PyPI é `tramuri`. O comando instalado no `PATH` também é
`tramuri`. Este projeto não usa o nome de pacote nem o comando do Graphify.

Escolha um método para instalar o CLI isolado:

```bash
uv tool install tramuri
# Se `tramuri` não for encontrado, reinicie o shell ou execute:
# uv tool update-shell
tramuri --version
```

Alternativas:

```bash
pipx install tramuri
# ou
python -m pip install tramuri
```

Não é necessário Rust, Node.js, Docker, backend hospedado ou uma segunda chave
de API dos providers.

### Instalação com escopo do projeto

No projeto que o Tramuri deve acompanhar, instale os componentes nativos do
provider uma vez. A instalação é local ao projeto por padrão:

```bash
tramuri init --provider auto       # detecta providers disponíveis neste projeto
tramuri init --provider all        # instala explicitamente os três adapters
```

Depois, inicie o Tramuri dentro do seu assistente:

```text
Claude Code: /tramuri .
Gemini CLI:  /tramuri .
Codex:       $tramuri .
```

`/tramuri .` significa “use este projeto como contexto”. Claude Code e Gemini
expõem esse nome como comando nativo. O Codex expõe skills de projeto pela
notação oficial `$skill-name`, então `$tramuri .` é a forma garantida no Codex;
as instruções instaladas também reconhecem `/tramuri .` quando ele é enviado
como uma mensagem normal.

O Tramuri começa pela intenção de produto da pessoa. Se ela não estiver
presente, faz uma única pergunta curta e agrupada sobre o que criar ou melhorar,
para quem e qual resultado se espera. Ele não inventa um produto. Com a
intenção clara, usa MVP por padrão, registra suposições de baixo impacto e só
pergunta sobre autorizações, segurança ou decisões irreversíveis.

Todo o estado e os artefatos gerados ficam juntos:

```text
tramuri_output/
├── portal/index.html                 # mapa central do produto
├── reports/<run-id>/report.html      # relatório do produto
├── reports/<run-id>/prototype.html  # protótipo navegável offline
├── demo/                             # experiência de marketing, cliente e admin
├── database/tramuri.sqlite3          # memória durável do workflow
├── memorys/                          # memória legível e resumos de cada execução
├── manifests/                        # registros de instalação
└── backups/                          # backups seguros do instalador
```

Os arquivos nativos dos providers continuam em `.agents/`, `.claude/` ou
`.gemini/`, pois são os locais que os assistentes reconhecem para skills,
comandos e hooks.

A skill de entrada é instalada em:

```text
.agents/skills/tramuri/SKILL.md        # Codex
.claude/skills/tramuri/SKILL.md        # Claude Code
.gemini/skills/tramuri/SKILL.md        # Gemini CLI
.gemini/commands/tramuri.toml          # comando nativo /tramuri do Gemini
```

As capacidades de produto também são instaladas como skills individuais, por
exemplo `.agents/skills/product-framing/SKILL.md` e
`.agents/skills/product-memory/SKILL.md` no Codex. Os nomes públicos começam
com `tramuri-`, como `$tramuri-framing` e `$tramuri-memory`. Execute
`tramuri update --provider codex` para instalar ou atualizar a lista. Se as
skills novas não aparecerem, reinicie o Codex.

Também é possível acompanhar o workflow pelo terminal:

```bash
tramuri status
tramuri runs list
tramuri resume <run-id>
tramuri memory show
tramuri memory search "decisão de preço"
tramuri memory save --title "Região de lançamento" --content "Começar pelo Brasil" --classification decision
```

O Tramuri mantém memória curta do projeto e o estado do workflow no SQLite e
gera arquivos legíveis em `tramuri_output/memorys/`. Ele não salva transcrições
inteiras. Quando você perguntar o que ele lembra ou o que estávamos fazendo, a
skill `tramuri-memory` busca os registros e confere o status atual antes de
responder.

## Comandos comuns

Execute a skill instalada dentro de um assistente:

```text
/tramuri .                              # Claude Code ou Gemini CLI
$tramuri .                              # Codex
/tramuri explain "proposta de valor"   # explicar fatos do produto
$tramuri explain "escopo do MVP"         # equivalente no Codex
```

Execute operações determinísticas ou automações pelo shell:

```bash
tramuri --help
tramuri status
tramuri --json explain "proposta de valor"
tramuri product run --idea-file ideia.md --mode mvp
tramuri product research --run-id <run-id> --research-file pesquisa.json
tramuri product demo --mode mvp
tramuri experience check <run-id>
tramuri report open <run-id>
tramuri report open <run-id> --no-open
tramuri runs list
tramuri runs show <run-id>
tramuri resume <run-id>
tramuri providers list
tramuri doctor
tramuri init --provider all --dry-run
tramuri update --provider all
tramuri uninstall --provider codex
```

`explain` lê somente artefatos, decisões, dependências e evidências persistidos
em `tramuri_output`. Quando um assunto não foi registrado, ele informa a lacuna
em vez de inventar uma explicação.

Se `tramuri` não for encontrado após uma instalação isolada, reinicie o shell ou
execute `uv tool update-shell`. Se um provider não for detectado, execute
`tramuri doctor` e instale o CLI do provider seguindo as instruções oficiais
dele. O Tramuri nunca instala assistentes de terceiros ou servidores MCP
automaticamente.

## Demo determinística

```bash
tramuri product demo --mode mvp
tramuri runs list
tramuri report open <run-id> --no-open
```

A demo não exige rede nem autenticação. Hipóteses e dados de exemplo são
marcados explicitamente como sintéticos, e a pesquisa local aparece como
`not_run`. Um provider pode produzir um pack `tramuri.research.v1` com fontes,
concorrentes, sinais, motivos de compra, posicionamento, sizing e trade-offs
build/buy/partner; importe-o com `tramuri product research` para retomar o fluxo.

Cada demo também produz `tramuri_output/portal/index.html`, o mapa de Product
Experience, e `tramuri_output/demo/` com superfícies de marketing, cliente e
admin, links locais e interações simuladas. Abra o portal para navegar de
Product Definition até a Product Demo; Product Engineering permanece como etapa
futura.
Valide os artefatos com `tramuri --json experience check <run-id>`.

O workflow sobrevive ao fechamento da sessão do provider. Use `tramuri status`
para ver a etapa real, checkpoint, gates dos sensores e próxima ação; depois
execute `tramuri resume <run-id>` para continuar. O Tramuri registra knowledge
local com proveniência e trata documentos do repositório como dados não
confiáveis. Ambiguidades de baixo impacto viram hipóteses registradas; somente
decisões críticas de autorização ou segurança viram gates humanos.

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
providers, workflow de produto e release.

## Licença

Apache-2.0. Consulte [LICENSE](LICENSE).
