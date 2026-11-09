# Python Library Engineering Harness

Harness local para iniciar uma biblioteca Python do zero e conduzir o trabalho até um artefato instalável e documentado.

## O que existe aqui

- harness/core/checklists/product-skills.md: contrato de qualidade das skills
  distribuídas pelo Tramuri.

- `AGENTS.md`: regras gerais para o agente principal e os especialistas.
- `.agents/skills/`: skills locais para packaging, testes, CI, lint, typing, documentação, supply chain e release.
- `.codex/agents/`: papéis de agentes com fronteiras de escrita explícitas.
- `harness/core/workflows/`: processos de entrega, arquitetura, coordenação e release.
- `harness/core/checklists/`: checklists de arquitetura, packaging, testes e integridade.
- `harness/core/templates/`: modelos de relatório de entrega e decisão arquitetural.
- `harness/core/decisions/`: decisões aprovadas do projeto.
- `harness/scripts/`: validação do harness e gate não destrutivo do projeto Python.
- `harness/skills-manifest.md`: origem e finalidade das skills instaladas.
- `harness/core/policies/skill-selection.md`: matriz de ativação contextual.
- `harness/core/checklists/experience.md`: aceite do portal e da demo.
- `.agents/skills/tramuri-report-design/`: padrão do harness para construir,
  revisar e validar relatórios; não é distribuído pelo CLI Tramuri.
- `../../docs/design/report-design-system.md`: tokens, composição, evidências,
  acessibilidade e impressão dos relatórios.
- `specs/005-product-research-complete-experience/`: contrato versionado para pesquisa
  com evidencias e experiencias completas.
- `harness/reference/harness-development.md`: como adicionar um provider sem duplicar o core.

## Fluxo recomendado

1. Se o repositório estiver vazio, execute `core/workflows/bootstrap-python-library.md`.
2. Defina o objetivo, a API pública inicial e os critérios de aceitação.
3. Execute o workflow de entrega de feature e delegue tarefas por fronteira: implementação, packaging, testes e documentação.
4. Rode o gate do projeto e peça as revisões de evidência e qualidade.
5. Para um release, use o workflow de release e aguarde autorização explícita antes de publicar.

## Product Experience

Quando uma alteração constrói portal, mapa ou demo, o agente deve selecionar as
skills de interface e revisão visual de forma contextual. A entrega esperada é
um artefato offline navegável em `portal/` ou `demo/`, com rastreabilidade para
os artefatos do produto, sensores determinísticos e uma tentativa explícita de
validação no navegador. Playwright é opcional no harness; sua ausência deve ser
registrada como `not_run`, nunca convertida em sucesso falso.

O `Product Demo Engine` é acionado junto à projeção existente do Product
Experience. Ele classifica o produto pelo contexto persistido, produz uma
especificação `tramuri.demo.v1` e seleciona um template offline por categoria.
Use `.agents/skills/tramuri-product-demo-engine/` como contrato de geração e
`harness/core/policies/skill-selection.md` para decidir quais competências
externas aplicar. O fluxo não repete discovery nem requer Node.js no CLI.

Quando o produto depende de contexto de mercado, o harness coordena uma pesquisa
profunda com o provider disponível e entrega ao core um `tramuri.research.v1`.
O pack precisa conter fontes, concorrentes, sinais, motivos de compra,
posicionamento, sizing e trade-offs build/buy/partner. O core valida e persiste o
pack; ele não inventa números nem exige uma API de busca para funcionar offline.

O mapa e o gerador de demo são capacidades do produto Tramuri. As skills usadas
para construí-los permanecem somente no harness e não são instaladas no projeto
do usuário pelo CLI.

## Comandos do harness

```powershell
python harness/scripts/validate_harness.py
python harness/scripts/check_product.py
python harness/scripts/check_product.py --run
```

O segundo comando mostra o plano sem modificar o projeto. O modo `--run` executa apenas verificações locais disponíveis. Sem um `pyproject.toml`, ele informa que o projeto ainda não foi criado e não tenta inventar código.

## Convenções do pacote

Para um projeto novo, prefira `src/<nome_do_pacote>/`, `tests/`, `pyproject.toml` e `uv.lock`. O pacote gerado deve ser verificável por wheel e sdist e instalável em ambiente isolado com `pip`.

O harness não publica artefatos, não cria credenciais e não altera o repositório remoto automaticamente.

## Contexto, memória e status do Tramuri

O harness deve avaliar mudanças usando o ciclo `Plan → Delegate → Execute →
Evaluate → Critique → Repair → Verify → Complete`, mas a memória durável do
produto pertence ao runtime Python Tramuri. O SQLite continua sendo a fonte de
verdade; o runtime gera uma projeção legível em
`tramuri_output/memorys/`. A skill distribuída `product-memory` consulta esse
estado antes de responder sobre contexto anterior. O harness não distribui
skills externas aos usuários finais.

Ao alterar workflows, use como evidência os contratos em
`../specs/004-resumable-context-status/`: SQLite é a fonte de verdade, sessões
e checkpoints são distintos de conhecimento e eventos, sensores não podem
converter `not_run` em `passed`, e `tramuri status --json` precisa refletir os
mesmos fatos da saída visual. O agente deve manter a autonomia padrão do
Tramuri e agrupar perguntas somente para gates de autorização, segurança,
privacidade, legalidade ou decisões sem default seguro.


