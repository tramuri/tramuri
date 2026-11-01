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
- **Descoberta de capacidades** para catalogar skills, instruções, agentes, hooks e declarações MCP
  locais compatíveis, sem executá-los nem se conectar a eles.
- Uma CLI nativa em Rust e um SDK Python apoiados pelo mesmo core Rust.

O SDK Python publicado inclui uma extensão Rust pré-compilada nas plataformas suportadas; o usuário
não precisa instalar Rust.

O Tramuri mantém o estado local em `.tramuri/`, com o SQLite em
`.tramuri/database/tramuri.sqlite3` e o relatório da análise mais recente em
`.tramuri/discovery/`. Evidências são registradas como referências; o Tramuri não busca recursos
externos nem armazena o conteúdo dos arquivos analisados como evidência.
O catálogo de capacidades é um snapshot derivado; servidores MCP e hooks declarados não são
considerados operacionalmente verificados.

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

O pacote Tramuri inclui a distribuição da CLI e o instalador das integrações com assistentes.
Instale a CLI em um ambiente isolado:

```bash
uv tool install tramuri
tramuri --version
```

`pipx install tramuri` também está disponível como alternativa para uma instalação isolada.
Se o comando `tramuri` não for encontrado após a instalação, execute `uv tool update-shell` e
abra um novo terminal.

Quando uma nova versão do Tramuri for publicada, atualize usando o mesmo gerenciador da instalação:

```bash
uv tool upgrade tramuri
# Se instalou com pipx:
pipx upgrade tramuri
tramuri --version
```

Instale o Tramuri no projeto para o assistente utilizado, preservando instruções e hooks existentes:

```bash
tramuri install --platform codex --project
tramuri init .
```

Depois, descreva o resultado desejado no chat do assistente, por exemplo: “Quero atuar como Product
Engineer para melhorar o onboarding” ou “Vou assumir esta integração como FDE”. As skills do Tramuri
orientam a criação da Mission e do Success Contract, registram a perspectiva escolhida e conduzem o
trabalho; a varredura do projeto é opcional. Ao final de uma sessão coerente que altere registros da
Mission, o assistente atualiza e abre o relatório local uma vez. Você também pode pedir “abra o
relatório do Tramuri” ou executar `tramuri status --open`.

O relatório local pode ser lido em inglês ou português brasileiro e mostra a perspectiva de trabalho
(Product Engineer/FDE) separada do executor registrado. O nome do executor é informado por quem faz
o registro e não é autenticado; o Tramuri não presume quem é o responsável atual pela Mission.

`tramuri capabilities --type skill --json` filtra o catálogo declarativo mais recente. A varredura
respeita as regras de exclusão do projeto e de arquivos sensíveis; não lê configurações globais de
agentes, não executa hooks, não inicia servidores MCP nem acessa a rede.

Codex, Claude Code, Cursor e Gemini CLI recebem instruções nativas do projeto, uma skill geral do
Tramuri e skills específicas para configuração, descoberta, Missions e integridade, além de um hook
local de início de sessão. Os hooks emitem apenas um lembrete estático: não leem
o banco nem anexam dados das Missions. O agente executa comandos Tramuri explicitamente quando
precisar de contexto do workspace. Os hooks respeitam as configurações de confiança da plataforma,
e a instalação preserva instruções e hooks não relacionados. Cursor usa `.cursor/rules/tramuri.mdc`,
`.cursor/skills/tramuri/` e `.cursor/hooks.json`; os demais hosts usam seus caminhos nativos.

As instruções e skills do Copilot continuam disponíveis; o Tramuri não instala hooks do Copilot.

```bash
tramuri install --project --platform copilot
tramuri install --project --platform claude
tramuri install --project --platform cursor
tramuri install --project --platform gemini
```

`tramuri status` resume o workspace, `tramuri status --open` gera e abre o relatório local, e
`tramuri resume --mission <id>` recupera o trabalho registrado e a prontidão para a próxima
atividade escolhida. Uma Mission pode começar sem critérios; o Outcome permanece pendente até que
critérios obrigatórios sejam definidos e avaliados. A prontidão é consultiva e nunca bloqueia a
execução:

```bash
tramuri mission create --objective "Investigar abandono no onboarding"
tramuri readiness assess --mission <id> --activity discover --state needs_review \
  --gap "Os dados das entrevistas ainda não estão disponíveis"
tramuri status --open
```

Use `tramuri readiness show --mission <id>` para consultar a avaliação atual e o histórico.
Mudanças no objetivo, Success Contract ou Knowledge/Capabilities associados explicitamente marcam
a avaliação mais recente como desatualizada, sem apagar seu registro. `tramuri mission list` descobre as
Missions existentes. Os arquivos são `.tramuri/reports/tramauri.html`, `tramauri.md` e
`tramauri.json`. Ao abrir um banco antigo compatível, o Tramuri pode aplicar as migrações de schema
existentes; a leitura de status não altera Missions, execuções, evidências ou outcomes. Esses
comandos funcionam sem assistentes; as integrações são opcionais.

Conhecimentos e capacidades descobertas podem ser associados explicitamente a uma Mission com
`tramuri mission link-knowledge` e `tramuri mission link-capability`. O comando
`tramuri resume --mission <id>` inclui somente essas associações registradas. O Tramuri não infere
relevância para Missions; vínculos de capacidades guardam o hash observado e podem ficar alterados,
ausentes ou não verificados após novas varreduras. A atribuição do ator é informada pelo chamador e
não é autenticada.

Siga os tutoriais com projetos sintéticos, da configuração ao handoff, para [Product Engineer e
FDE](docs/examples/README.pt-BR.md).

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
descoberta de capacidades, missões, registros manuais de execução e evidências, e avaliações de critérios fazem parte do
produto atual. A exportação versionada de Context Packs e Evidence Packs está planejada e usará
seleção e autorização explícitas, com proteção segura por padrão.

## Comunidade

- [Como contribuir](CONTRIBUTING.md)
- [Código de Conduta](CODE_OF_CONDUCT.md)
- [Política de Segurança](SECURITY.md)
- [Suporte](SUPPORT.md)
- [Licença](LICENSE) (Apache-2.0)
