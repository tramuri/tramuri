# Skills manifest — Python library harness

As skills foram pesquisadas no catálogo [skills.sh](https://www.skills.sh/) e instaladas localmente em `.agents/skills/`. O catálogo informa que as skills são contribuições externas e devem ser revisadas antes de receber confiança automática.

## Núcleo Python

Fonte: [paldom/python-skills](https://www.skills.sh/paldom/python-skills)

- `python-packaging`: layout, `pyproject.toml`, wheel, sdist e instalação.
- `python-testing`: pytest, cobertura, propriedades e robustez dos testes.
- `python-ci`: workflows e gates de integração contínua.
- `python-lint`: lint e qualidade estática.
- `python-typing`: tipagem e verificação estática.
- `python-precommit`: hooks e validações antes do commit.
- `python-supply-chain`: dependências, lockfiles e risco de distribuição.
- `python-release`: preparação de release e publicação autorizada.
- `agent-guardrails`: limites, segurança e comportamento previsível de agentes.

## Documentação

- [`documentation-writing`](https://www.skills.sh/nilecui/skillsbase/documentation-writing), da `nilecui/skillsbase`: documentação orientada ao leitor.
- [`python-docstrings`](https://www.skills.sh/open-edge-platform/anomalib/python-docstrings), da `open-edge-platform/anomalib`: docstrings e API Python.

## Skills de suporte mantidas

- `architecture-decision-records`: decisões técnicas duráveis.
- `refactoring-analysis`: auditoria de code smells e plano de refatoração.
- `verification-before-completion`: evidência antes de declarar conclusão.
- `writing-agents-md`: manutenção das instruções dos agentes.

O manifesto é deliberadamente legível e não autoriza download ou execução automática de código. Mudanças de origem devem ser revisadas e validadas pelo `harness/scripts/validate_harness.py`.

## Skills externas de Product Experience

Estas skills são ferramentas de desenvolvimento do harness. O produto não as
redistribui nem depende delas em runtime. A descoberta abaixo foi feita pelos
`SKILL.md` locais e deve ser revalidada quando a origem mudar.

Para o Product Demo Engine, as dez skills solicitadas foram encontradas
instaladas. `frontend-design`, `mobile-design` e `web-design-guidelines` foram
aplicadas à composição contextual, moldura responsiva e revisão de semântica,
estados, foco, movimento reduzido, contraste e operação offline. `mobile-app`
contribuiu apenas com ideias de safe area e proporção: seu contrato é de uma
tela estática para iPhone, inadequada ao fluxo navegável. `data-visualization`
orienta hierarquia de etapas e tabelas; métricas e gráficos só entram quando há
dados de origem. `webapp-testing` foi selecionada; Playwright é uma capacidade
opcional e não faz parte das dependências runtime do produto. Se Playwright ou
navegador não estiver disponível numa execução, o estado correto é `not_run`.
`ui-ux-pro-max`
foi avaliada, mas os resultados de busca não cobriam o motor multiplataforma
offline. `theme-factory` requer seleção de tema pelo usuário, então tokens são
propostos por domínio; `silent-product-demo` produz vídeo e storyboard, não demo
interativa; `impeccable` foi reservada para a crítica de uma composição de
produto concreto, não para o sistema de templates.

| Skill | Estado local | Uso contextual | Dependência/limite |
| --- | --- | --- | --- |
| `frontend-design` | instalada | portal, mapa, demo e direção visual | exige plano visual antes do código |
| `web-design-guidelines` | instalada | auditoria de semântica, acessibilidade e usabilidade | revisão produz findings, não elogios genéricos |
| `webapp-testing` | instalada | jornada real no navegador | Playwright é uma dependência opcional de validação, não runtime |
| `skill-creator` | instalada | criação e avaliação de skills internas | permanece ferramenta do harness |
| `verification-before-completion` | instalada | gate de evidências em toda entrega | não aceitar apenas afirmação de subagente |
| `systematic-debugging` | instalada | investigação de causa raiz | ativar quando houver falha ou surpresa |
| `test-driven-development` | instalada | contratos Python e comportamento novo | teste vermelho antes da implementação |
| `python-testing` | instalada | pytest, cobertura e testes de integração | não substitui o gate do projeto |
| `subagent-driven-development` | instalada | divisão de tarefas independentes | usar apenas quando reduzir risco/tempo |
| `web-animation-design` | ausente | sem ativação | fallback: CSS mínimo e `prefers-reduced-motion` |

## Skills do produto

`Product Framing`, `Product Discovery`, `Market Research`, `Competitive Analysis`,
`Buying Motivation`, `Product Positioning`, `Audience Sizing`, `Product UX`,
`Product Experience`, `Product Status`, `Product Demo Generation`, `Experience Mapping`
e `Product Traceability` são capacidades
versionadas em `src/tramuri/templates/product_skills/`. Elas são instaladas
pelos adaptadores do Tramuri quando aplicável. Não copie conteúdo de skills
externas para esse diretório: implemente contratos próprios, compatíveis com o
runtime e com as licenças aplicáveis.
