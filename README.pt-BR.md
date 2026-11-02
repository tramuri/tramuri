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

> **Começando agora?** Siga o [tutorial de Product Engineer](docs/examples/product-engineer/README.pt-BR.md)
> ou o [tutorial de FDE](docs/examples/fde/README.pt-BR.md), do setup ao handoff.

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
```

Depois, descreva um objetivo substancial no chat do assistente, por exemplo: “Quero melhorar o
onboarding” ou “Vou assumir esta integração de cliente”. As instruções iniciam ou retomam uma Mission
mesmo que você não mencione o Tramuri. Para uma Mission nova, escolha uma perspectiva: Product
Engineer (entende o problema do produto e ajuda a decidir o que construir), FDE (resolve uma
necessidade de cliente conectando sistemas) ou misto (combina evolução do produto e integração).
No terminal, escolha com as setas e confirme com Enter; em uma sessão de agente, responda no chat.
A escolha fica registrada na Mission. Preferências e inferências podem sugerir, mas não substituir
essa escolha. Uma Mission existente com objetivo igual e correspondência inequívoca pode ser retomada
com a perspectiva já registrada. Em Missions de Product Engineering, uma skill específica conduz
framing e discovery, separando fatos, hipóteses e dúvidas.
Ela orienta o agente a iniciar ou retomar a Mission antes de explorar o projeto ou usar outras
ferramentas de criação. Não é necessário rodar
`tramuri init` antes nem aprender a sintaxe da CLI. A Mission pode começar como exploratória e a
varredura do projeto é opcional. Ao final de uma sessão coerente que altere registros da Mission,
o assistente atualiza os relatórios. Ele abre o HTML se ainda não o abriu na sessão; caso já esteja
aberto, atualiza os arquivos sem abrir outra janela. Você também pode pedir “abra o relatório do
Tramuri”, executar `tramuri status --open`, ou usar `tramuri status --refresh` para atualizar sem
abrir o navegador depois de iniciar ou retomar a Mission.

O relatório local pode ser lido em inglês ou português brasileiro e mostra a perspectiva de trabalho
(Product Engineer/FDE) separada do executor registrado. O nome do executor é informado por quem faz
o registro e não é autenticado; o Tramuri não presume quem é o responsável atual pela Mission.

`tramuri capabilities --type skill --json` filtra o catálogo declarativo mais recente. A varredura
respeita as regras de exclusão do projeto e de arquivos sensíveis; não lê configurações globais de
agentes, não executa hooks, não inicia servidores MCP nem acessa a rede.

Codex, Claude Code, Cursor, Gemini CLI e GitHub Copilot recebem instruções nativas e skills do
Tramuri; há hooks locais nas plataformas e superfícies que os suportam. Quando habilitado e
confiável, o hook bloqueia ferramentas locais compatíveis até `tramuri start` ou `tramuri resume`
terminar com sucesso no turno atual. A cobertura depende do host: caminhos desabilitados, não
confiáveis, contornados ou hospedados não são impostos. O gate usa apenas metadados do evento e o
resultado da chamada correspondente à CLI; não lê prompts nem registros da Mission. Instruções e
skills orientam o agente, mas não obrigam hosts sem um hook aplicável. A instalação preserva
instruções e hooks não relacionados. Cursor usa `.cursor/rules/tramuri.mdc`,
`.cursor/skills/tramuri/` e `.cursor/hooks.json`; Copilot usa `.github/hooks/tramuri.json` nos
runtimes que suportam esse hook; os demais hosts usam seus caminhos nativos.

```bash
tramuri install --project --platform copilot
tramuri install --project --platform claude
tramuri install --project --platform cursor
tramuri install --project --platform gemini
```

`tramuri status` resume o workspace, `tramuri status --open` gera e abre o relatório local,
`tramuri status --refresh` atualiza os relatórios sem abrir o navegador, e `tramuri resume --mission <id>`
recupera o trabalho registrado e a prontidão para a próxima
atividade escolhida. Uma Mission pode começar sem critérios; o Outcome permanece pendente até que
critérios obrigatórios sejam definidos e avaliados. A prontidão é consultiva e nunca bloqueia a
execução:

```bash
tramuri start --goal "Investigar abandono no onboarding" --perspective product_engineer
tramuri readiness assess --mission <id> --activity discover --state needs_review \
  --gap "Os dados das entrevistas ainda não estão disponíveis"
tramuri status --open
```

### Workflow Product Engineering DEFINE

Ao iniciar uma Mission com perspectiva `product_engineer`, o Core cria automaticamente um workflow
DEFINE persistente. O agente integrado consulta a etapa autorizada, usa a skill indicada e envia o
resultado estruturado para validação. As cinco etapas são: objetivo e contexto; problema, público e
negócio; requisitos e critérios; síntese de fatos e hipóteses; Product Brief. O usuário não precisa
criar essas etapas manualmente nem completar um Success Contract para começar.

```bash
tramuri start --goal "Quero criar um SaaS para vender cursos online" --perspective product_engineer --json
tramuri mission define status --mission <id> --json
tramuri mission define next --mission <id> --json
```

`next` retorna a etapa liberada, a skill correspondente e o contrato de saída. Depois de produzir o
JSON, o host envia-o a `tramuri mission define submit --mission <id> --step <etapa> --file
<resultado.json>`. O Core rejeita campos obrigatórios ausentes, referências inválidas e submissões
fora de ordem; uma rejeição mantém a etapa atual aberta. Para uma decisão material, o host pode
registrar uma pergunta e sua resposta com `tramuri mission define question request` e
`tramuri mission define question answer`. O limite de duas perguntas comuns persiste entre sessões.

`status` e `next` retomam o progresso salvo. Cada transição pela CLI tenta atualizar
`.tramuri/reports/tramauri.html`, `tramauri.md` e `tramauri.json` sem reabrir o navegador. O SDK
Python oferece as mesmas operações `mission_define_*`; ao usá-lo diretamente, atualize os relatórios
com `tramuri status --refresh`.

O Core controla sequência, validação estrutural e persistência. Ele não prova que o host carregou
uma skill, que uma pesquisa é verdadeira ou que o Product Brief está correto. O host realiza a
análise e usa as ferramentas disponíveis. Concluir DEFINE salva o Product Brief, mas não constrói,
publica nem verifica o produto e não marca o Outcome da Mission como alcançado.

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

Siga os [tutoriais de Product Engineer e FDE](docs/README.pt-BR.md), com passos de configuração,
trabalho, evidências, avaliação e handoff usando dados fictícios.

### Como um assistente pode usar o Tramuri

O instalador do projeto adiciona instruções nativas e skills do Tramuri. Elas orientam o assistente
a iniciar ou retomar uma Mission antes de usar ferramentas do projeto. Quando a plataforma oferece
um hook nativo confiável e habilitado, ele bloqueia chamadas locais compatíveis até que
`tramuri start` ou `tramuri resume` termine com sucesso no turno atual. A cobertura depende dos
hooks de cada plataforma; caminhos desabilitados, não confiáveis, contornados ou hospedados não são
impostos. Os hooks usam somente metadados do evento e o resultado da chamada correspondente à CLI;
não leem nem armazenam prompts ou registros de Missions. Esse gate ajuda a manter a ordem do fluxo,
mas não é autenticação nem garante que a análise de produto do assistente esteja correta. Consulte
os [detalhes das integrações](docs/reference/fluxo-operacional-completo.pt-BR.md).

| Interface | Uso |
| --- | --- |
| CLI | Executar descoberta local e fluxos de conhecimento e missões. |
| Skill e instruções do assistente | Orientar um assistente configurado sobre quando e como usar a CLI do Tramuri. |
| Hooks nativos | Bloquear ferramentas locais compatíveis até iniciar ou retomar uma Mission, quando ativos e confiáveis na plataforma. |
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
