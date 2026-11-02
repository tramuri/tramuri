# DEFINE: revisão, validade e avaliação do Product Brief

Este documento descreve os contratos de revisão e avaliação introduzidos no workflow DEFINE.
O Core Rust continua sendo a autoridade de transições; SQLite continua sendo a fonte de verdade.

## Revisar uma etapa aceita

Use `tramuri mission define review` para substituir explicitamente o conteúdo vigente de uma etapa
aceita. A operação exige motivo e rótulo de autor declarado. Esse rótulo registra proveniência, mas
não autentica a pessoa. A origem e a revisão esperada também podem ser fornecidas. O Core valida o
novo payload antes de aceitá-lo e preserva as revisões anteriores. JSON editado ou SQL direto não é
uma operação de revisão suportada.

```powershell
tramuri mission define next --mission <MISSION_ID> --json
tramuri mission define review --mission <MISSION_ID> --step business_discovery `
  --file .\business-discovery.json --reason "Público-alvo refinado" `
  --author "operador" --origin user_request --json
tramuri mission define next --mission <MISSION_ID> --history --json
```

Mudanças em `approved_decisions` do Product Brief exigem `--authorize-decision-changes` explícito.
Uma submissão ou revisão inválida não substitui a versão aceita. `--expected-revision` permite
rejeitar atualizações concorrentes com base em estado obsoleto.

## Dependências e propagação `stale`

Cada artefato aceito registra as referências de etapa/revisão consumidas. A revisão compara essas
referências, não timestamps. Se uma revisão aceita modificar uma entrada consumida por etapas
posteriores, o Core marca as dependentes transitivas como `stale`, preserva seus artefatos e histórico
e faz a etapa desatualizada mais antiga ser o próximo trabalho autorizado. Etapas independentes não
são invalidadas. `define next` não entrega artefatos históricos/desatualizados como entradas atuais.
O DEFINE não pode ser concluído enquanto houver etapas requeridas stale.

Consulte o histórico por `tramuri mission define next --mission <MISSION_ID> --history --json`.
Sem `--history`, a resposta serve ao contrato de execução da etapa e inclui apenas entradas aceitas
atuais.

## Avaliação semântica do Product Brief

Uma avaliação é uma declaração de um avaliador hospedeiro, separada da validação estrutural e do
estado da Mission. O Core verifica formato, versão da rubrica, referências a campos existentes,
limites e coerência entre resultado geral e critérios. Não decide se a argumentação é verdadeira nem
verifica mercado, clientes ou adequação do produto.

A rubrica `tramuri.product-brief-rubric.v1` possui os critérios:

- `goal_alignment`
- `target_consistency`
- `evidence_assumption_separation`
- `success_criteria_fit`
- `requirements_traceability`
- `decision_authorization`
- `internal_consistency`

Cada critério recebe `pass`, `concern` ou `not_assessed`, justificativa e referências a campos de
artefatos aceitos. O resultado geral é `pass`, `concerns` ou `incomplete`. Uma avaliação não marca a
Mission como verificada. Quando um artefato citado muda, a avaliação deixa de ser atual.

O host pode enviar o JSON versionado usando:

```powershell
tramuri mission define review --mission <MISSION_ID> `
  --assessment-file .\brief-assessment.json --evaluator "codex" --json
```

O contrato de entrada inclui `schema`, `brief_artifact_id`, `brief_revision`, `rubric_version`,
`overall`, a lista completa `criteria`, `limitations` e `evaluation_boundary`. O Core valida a
referência ao Product Brief vigente e registra a avaliação no histórico. A identidade do avaliador
é declarada, não autenticada.

## Relatório

O contrato JSON do relatório é `tramuri.report.v5`. HTML, Markdown e JSON projetam os artefatos
persistidos: etapas e progresso, etapa atual, resumo minimizado do Product Brief, estado vigente ou
stale, hipóteses e questões pendentes, e avaliação semântica atual ou stale. Resumos são limitados e
há mascaramento heurístico de alguns padrões comuns de segredo; isso não é um detector abrangente de
segredos. Não armazene credenciais nos artefatos.

As escritas DEFINE atualizam os arquivos de relatório em modo best-effort sem abrir o navegador. O
HTML é um arquivo estático: enquanto permanecer aberto não recebe eventos ao vivo. Use seu controle
de atualização para recarregar os arquivos existentes. Uma falha de projeção não desfaz a gravação
do Core nem prova conclusão de etapa.

## Seleção de Mission em `start`

O Core reutiliza uma única Mission aberta somente quando o objetivo normalizado coincide exatamente
(espaços e caixa não significativos) e a perspectiva registrada é compatível. Não faz busca por
similaridade semântica. Sem correspondência exata, o comportamento normal de criação e roteamento é
mantido. Duplicatas exatas, perspectiva ausente ou conflito explícito de perfil retornam
`MissionSelectionRequired`; o chamador deve pedir escolha explícita, sem misturar Missions.

`mission_start` passou a retornar `schema: tramuri.mission.start.v2` e `mission_reused`. O enum
público Rust `TramuriError` ganhou a variante `MissionSelectionRequired`. Chamadores Rust que
constroem `MissionStartResult` por literal ou fazem `match` exaustivo em `TramuriError` precisam
adaptar o código; a mudança é fonte-incompatível, embora o formato seja JSON versionado. Não houve
migration nova para revisão, invalidação ou avaliações: elas reutilizam tabelas/eventos DEFINE já
presentes no schema 10.

## Limites de validação

- Validação determinística do Core confirma estrutura, referências, dependências e autorização
  explícita, não qualidade de produto.
- Avaliação semântica precisa ser executada e declarada pelo host; a rubrica não é garantia de
  correção e não equivale à validação com usuários.
- Atualização do relatório é best-effort e manual no navegador; não existe watcher/servidor ao vivo.
- Instalação das integrações e skills não prova comportamento end-to-end de um agente real.
- O workflow não revisa nem altera automaticamente Outcomes existentes.
