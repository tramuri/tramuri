# Workflow — Orquestração de agentes

## Ordem padrão

1. O agente principal define objetivo, critérios de aceitação e arquivos dentro do escopo.
2. O agente principal classifica a tarefa e ativa somente as skills da matriz contextual.
3. `python_library_engineer` define ou implementa a API e o comportamento.
4. `python_packaging_engineer` configura metadados, dependências e artefatos sem reescrever a implementação.
5. `python_test_engineer` adiciona testes para o contrato e os casos de falha.
6. Para portal/demo, `product_experience_engineer` implementa a experiência e
   `product_lead_reviewer` revisa coerência sem editar os artefatos.
7. Para decisões dependentes de mercado, `research_evidence_engineer` coleta ou
   organiza fontes, concorrentes, motivos de compra, sizing e build/buy/partner;
   `evidence_reviewer` rejeita claims sem source ID.
8. `python_docs_engineer` atualiza README, guias, exemplos e docstrings.
9. `evidence_reviewer` verifica segurança, rastreabilidade e conteúdo não confiável.
10. `quality_engineer` revisa os resultados e executa o gate final.
11. `python_release_engineer` prepara release apenas depois de aprovação explícita.

## Fronteiras

- Implementação: `src/` e, quando necessário, interfaces públicas documentadas.
- Packaging: `pyproject.toml`, lockfile, configuração de build e metadados de distribuição.
- Testes: `tests/`, fixtures e configuração exclusiva do runner.
- Documentação: `README*`, `docs/`, exemplos e docstrings.
- Revisores: sem edição, salvo autorização posterior e explícita.

Cada agente deve entregar arquivos alterados, critérios cobertos, comandos executados e lacunas. Se duas tarefas precisarem do mesmo arquivo, execute-as sequencialmente ou faça o agente principal integrar a mudança.

## Roteamento das skills distribuídas

O skill Tramuri é a porta de entrada do produto. Ele consulta status e artefatos
persistidos, classifica o pedido e seleciona somente a capacidade necessária.
Consulte o checklist de product skills ao criar ou revisar as skills distribuídas
em src/tramuri/templates/product_skills/.

Perguntas sobre o que o Tramuri lembra ou sobre o trabalho entre sessões usam a
capacidade `product-memory`: consulte `tramuri memory show`, busque o assunto e
confirme o progresso com `tramuri status` antes de responder.

Não crie uma persona PM. Skills como framing, pesquisa, posicionamento, UX,
experiência e status representam tarefas distintas; a mera existência de uma
definição de agente não prova que o provider a invocou. Se o mecanismo nativo
não estiver disponível, execute a capacidade inline e registre a limitação.

## Fluxo de Product Experience

`Product Definition → Research Handoff → Product Review → Experience Generation →
Browser Validation → Critical Review → Correction → Delivery`. Cada transição
deve deixar evento ou evidência. Pesquisa ausente fica `not_run` ou `blocked`,
nunca vira dado de mercado; ausência de Playwright fica `not_run`, nunca é
convertida em validação positiva.

## Fluxo de relatórios

Ao alterar um relatório, ative `.agents/skills/tramuri-report-design/SKILL.md`
e consulte `docs/design/report-design-system.md`. Inspecione gerador, projeção
de estado, contratos de dados, marca e artefato renderizado antes de editar.
Implemente primeiro os testes da projeção, mantenha a renderização centralizada
e preserve o conteúdo, os caminhos e a persistência existentes. Relações e
métricas só podem derivar de dados registrados. Depois valide o HTML offline,
idiomas, temas, teclado, tamanhos de tela e impressão no navegador; capture e
revise screenshots de desktop e mobile. Skills externas e essa skill interna
continuam no harness e não entram nos recursos instalados pelo produto.
