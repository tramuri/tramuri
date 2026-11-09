# Política de seleção contextual de skills

## Objetivo

O harness usa skills como competências especializadas de desenvolvimento. A
seleção é explícita, mínima e auditável; a mera presença de um diretório não é
evidência de que uma skill funciona no ambiente.

## Matriz

| Tipo de tarefa | Skills principais | Evidência exigida |
| --- | --- | --- |
| `frontend` | `frontend-design`, `web-design-guidelines`, `verification-before-completion` | tokens/direção visual, auditoria e verificação renderizada |
| `report-design` | `tramuri-report-design`, `frontend-design`, `ui-ux-pro-max`, `theme-factory`; acrescente `impeccable`, `data-visualization` e `web-design-guidelines` conforme a mudança; `webapp-testing` para validar no browser | relatório offline, dados rastreáveis, screenshots desktop/mobile e verificações de interação |
| `browser` | `webapp-testing`, `verification-before-completion` | Playwright executado ou `not_run` com motivo |
| `python` | `python-testing`, `test-driven-development`, `verification-before-completion` | teste de contrato, suíte e comandos frescos |
| `debugging` | `systematic-debugging`, `verification-before-completion` | hipótese causal, reprodução, reparo e revalidação |
| `skill-authoring` | `skill-creator`, `verification-before-completion` | contrato, gatilhos, casos de avaliação e resultado |
| `product-experience` | `frontend-design`, `web-design-guidelines`, `webapp-testing`, `python-testing`, `test-driven-development`, `verification-before-completion` | portal/demo, sensores, contratos Python e testes adequados |
| `product-demo-engine` | `tramuri-product-demo-engine`, `frontend-design`, `mobile-design` para mobile, `data-visualization` quando dados sustentam gráficos, `web-design-guidelines`, `webapp-testing` quando o navegador estiver disponível, `python-testing`, `test-driven-development`, `verification-before-completion` | spec versionada, categoria explicável, template contextual, interação local, sensores e screenshots ou `not_run` fundamentado |
| `product-research` | `systematic-debugging`, `python-testing`, `verification-before-completion` | research pack com fontes, claims rastreaveis e desconhecidos explicitos |
| `product-research-and-experience` | `frontend-design`, `web-design-guidelines`, `webapp-testing`, `python-testing`, `test-driven-development`, `systematic-debugging`, `verification-before-completion` | pesquisa validada, superficies coerentes, browser executado ou `not_run` |

## Algoritmo

1. Classifique a tarefa por arquivos e critérios de aceite antes de carregar contexto.
2. Descubra cada `SKILL.md`, valide frontmatter e registre hash do conteúdo.
3. Verifique dependências executáveis; `webapp-testing` sem Playwright fica
   selecionada como capacidade degradada, com `not_run` explícito.
4. Carregue somente a linha da matriz que cobre a tarefa. Skills não mapeadas
   não são carregadas por padrão.
5. Registre razões, indisponibilidades e verificações exigidas no resultado.
6. Antes de concluir, aplique a verificação associada e preserve evidências.
7. Para pesquisa, a skill externa pode coletar fontes; o produto recebe somente um pack
   versionado e validado. Uma resposta textual do agente não substitui o pack.
8. Para experiência, selecione a skill de frontend antes da implementação, a de guidelines
   depois da renderização e a de browser somente quando o runtime estiver disponível.
9. Para relatórios Tramuri, consulte `.agents/skills/tramuri-report-design/` e
   `docs/design/report-design-system.md`. Se a projeção Python mudar, use TDD e
   `python-testing`; use visualização de dados apenas quando registros reais
   justificarem o gráfico. A skill de relatório pertence ao harness e nunca é
   copiada para o pacote ou para as skills instaladas nos usuários.
10. Quando `ReportGenerator` gerar a experiência de um produto, aplique
    `.agents/skills/tramuri-product-demo-engine/` automaticamente. Classifique
    usando o contexto do PDLC, registre ambiguidade, reutilize decisões
    persistidas e gere a demo a partir da especificação estruturada. Selecione
    competências externas de interface por categoria; não carregue a matriz
    inteira indiscriminadamente.

## Limites

Ao editar uma skill distribuída do produto, selecione skill-creator e aplique
harness/core/checklists/product-skills.md. Isso não autoriza copiar conteúdo
da skill externa. Mantenha o nome como capacidade de produto, sem criar uma
persona profissional PM.

Skills externas são do harness. Nunca são copiadas para `src/tramuri` nem
instaladas no projeto usuário. As skills do produto possuem contratos próprios
e são distribuídas pelo CLI Tramuri.
