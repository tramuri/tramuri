# Exemplo: Product Engineer

Este tutorial leva uma ideia fictícia até um protótipo local de uma plataforma SaaS de cursos
online. O caminho principal é copiar um prompt completo para o Codex; ele informa o produto, os
usuários, o MVP e os limites do trabalho para que você não precise conduzir o agente com uma
sequência de perguntas. Também mostra como acompanhar a Mission, revisar o relatório e fazer
handoff.

O projeto de exemplo se chama **Aulae**. O nome, os cursos, as pessoas e os dados são fictícios. O
tutorial não exige Git, entrevistas reais, pagamentos, credenciais nem deploy.

Este tutorial requer Tramuri 0.10.0 ou mais recente. Confira `tramuri --version` após instalar;
`uv tool install tramuri` instala a versão publicada mais recente. Se o Tramuri já estiver instalado,
use `uv tool upgrade tramuri` para atualizá-lo.

## 1. Instalar e preparar o projeto

```console
uv tool install tramuri
tramuri --version
mkdir aulae-cursos
cd aulae-cursos
tramuri install --project --platform codex
```

O instalador acrescenta as instruções, a skill e os hooks suportados pelo Codex, preservando os
arquivos gerenciados pelo usuário. A integração orienta o Codex a iniciar ou retomar uma Mission
antes de usar ferramentas locais do projeto. Ela não executa o produto nem armazena o texto do chat.

Não rode `tramuri init` nem `tramuri scan` para este exemplo vazio. Ao enviar o prompt abaixo, o
Codex deve iniciar o workspace local junto com a Mission. O Tramuri funciona sem Git.

## 2. Copiar este prompt para o Codex

Envie o bloco inteiro em uma única mensagem. Ele já escolhe a perspectiva **Product Engineer** e
fornece detalhes suficientes para o agente estruturar e prototipar o produto. Se o terminal ainda
mostrar o seletor de perspectiva, escolha **Product Engineer** e pressione Enter. Essa escolha
descreve a perspectiva do trabalho; não identifica quem está executando a tarefa.

```text
Quero criar do zero um produto web fictício chamado Aulae: uma plataforma SaaS de cursos online
com vários instrutores e alunos.

Trabalhe nesta Mission pela perspectiva Product Engineer. Comece ou retome com o Tramuri antes de
usar outras ferramentas. Use a Mission atual para todo o trabalho; não crie outra Mission para o
mesmo produto. Siga o workflow DEFINE do Core: consulte a próxima etapa, carregue a skill indicada,
envie o resultado pelo CLI e avance somente quando ele for aceito. Preserve o que eu disse,
separando informação fornecida, hipótese, recomendação, decisão aprovada e questão em aberto.

CONTEXTO INICIAL DO PRODUTO

- Objetivo: permitir que instrutores publiquem cursos e que alunos encontrem, assinem e acompanhem
  cursos em um só lugar.
- Problema a investigar: alunos podem ter dificuldade para encontrar cursos adequados e retomar o
  aprendizado; instrutores podem depender de várias ferramentas para publicar e acompanhar cursos.
  Trate isso como hipótese, não como pesquisa ou problema validado.
- Público inicial de trabalho: alunos adultos no Brasil interessados em habilidades práticas e
  profissionais, e instrutores independentes que desejam oferecer cursos online. É uma hipótese
  para orientar o protótipo, não uma persona validada por entrevistas.
- Papéis: aluno, instrutor e administrador da plataforma.
- Jornada do aluno: descobrir e filtrar cursos → consultar detalhes → simular uma assinatura ou
  matrícula → acessar aulas → marcar uma aula concluída e acompanhar o progresso.
- Jornada do instrutor: criar um curso em rascunho → adicionar módulos e aulas → enviar para revisão
  → acompanhar se foi aprovado ou precisa de ajustes.
- Jornada do administrador: revisar cursos enviados e aprovar ou solicitar ajustes. Faça uma
  demonstração simples; não crie permissões ou segurança reais.
- Modelo comercial de trabalho: acesso recorrente dos alunos a um catálogo com cursos de vários
  instrutores. Use R$ 200/mês somente como preço ilustrativo na interface, claramente marcado como
  exemplo fictício, não como preço aprovado. A regra de repasse aos instrutores e qualquer comissão
  continuam em aberto. Não invente nem registre uma decisão financeira.

PROTÓTIPO LOCAL — ESCOPO

1. Página de catálogo em português do Brasil, responsiva, com busca, categorias e cartões de cursos.
2. Detalhe de curso com instrutor fictício, descrição, nível, duração, módulos e aulas.
3. Fluxo demonstrativo de assinatura/matrícula, sem checkout nem cobrança, que leve à área do aluno.
4. Área do aluno com cursos acessíveis, progresso e botão para continuar uma aula.
5. Player demonstrativo com conteúdo fictício e ação para marcar aula como concluída.
6. Área simples do instrutor para criar/editar um curso fictício e enviá-lo para revisão.
7. Fila simples de revisão administrativa para aprovar um curso ou pedir ajustes.
8. Use conteúdo sintético, por exemplo: “Excel para pequenos negócios”, “Vendas pelo WhatsApp” e
   “Fundamentos de UX”. Os nomes e instrutores também devem ser inventados.

FORA DO ESCOPO

- Pagamentos, assinaturas ou repasses reais.
- Login real, coleta de dados pessoais ou envio de e-mails.
- Hospedagem de vídeos, serviços externos, backend de produção ou deploy.
- Afirmações de que alunos ou instrutores reais validaram o produto.
- Definir preço final, comissão ou divisão de receita sem minha decisão explícita.

CONTRATO DE SUCESSO DESTE TUTORIAL

- Uma pessoa consegue pesquisar/filtrar o catálogo e abrir um curso.
- Uma matrícula simulada deixa o curso visível na área do aluno.
- A pessoa consegue marcar uma aula fictícia como concluída e ver o progresso mudar.
- O instrutor consegue criar um rascunho e enviá-lo para revisão; a tela administrativa consegue
  alterar seu estado demonstrativo.
- A interface funciona em tamanhos de tela comuns e usa textos claros em pt-BR.

Para este exercício, autorizo esses critérios como Success Contract da demonstração local. Eles
avaliam somente o protótipo; não comprovam demanda, aprendizagem ou sucesso comercial. Não altere
esse contrato sem me consultar e não registre o preço ou a remuneração como decisões aprovadas.

COMO CONDUZIR O TRABALHO

- Faça primeiro o trabalho de Product Engineering disponível com este contexto; não transforme a
  conversa em questionário. Durante o DEFINE, pergunte apenas se uma resposta realmente mudar a
  direção do produto ou evitar um risco importante, e reúna dúvidas relacionadas em uma pergunta.
  Para detalhes reversíveis, escolha uma hipótese clara e continue. Não repita informação já dada.
- Dê atualizações curtas de fase: o que está fazendo, o que produziu e o que vem depois. Não mostre
  raciocínio interno detalhado.
- Conclua o Product Brief pelo fluxo do Tramuri e continue para construir o protótipo local neste
  diretório; não pare no Brief nem espere eu perguntar qual é o próximo passo.
- Depois que o Brief for aceito, registre os critérios acima no Success Contract da Mission. Ao
  terminar a implementação, registre uma Execution com o estado realmente alcançado e Evidence das
  verificações executadas. Não declare Outcome verificado apenas porque o código foi escrito.
- Inspecione arquivos existentes depois de iniciar a Mission e preserve-os. Como este exemplo começa
  vazio, não faça scan sem necessidade. Use as ferramentas de criação disponíveis; se uma ferramenta
  ou MCP não estiver disponível, informe a limitação e continue por um caminho local viável, sem
  fingir que a ferramenta foi chamada. Prefira a estrutura mais simples já disponível e não adicione
  dependências sem necessidade.
- Antes de terminar, execute a aplicação localmente quando possível, percorra os fluxos acima e rode
  os testes que já existirem. Relate exatamente o que foi verificado e o que não foi. Não faça deploy.
- Ao concluir uma etapa do DEFINE, confira o Status/relatório do Tramuri. No fim, deixe claro o que
  foi registrado, os arquivos produzidos, as hipóteses ainda abertas, os testes executados e como
  iniciar o protótipo.
```

## 3. Acompanhar sem conduzir cada passo

Depois de enviar o prompt:

1. Responda ao seletor de perspectiva, se aparecer, escolhendo **Product Engineer**.
2. Deixe o Codex iniciar/retomar a Mission e executar as etapas autorizadas do DEFINE. O Core valida
   a ordem e a estrutura submetida; ele não valida mercado, entrevistas ou a verdade das hipóteses.
3. Se o agente perguntar algo que já está no prompt ou que pode ser decidido provisoriamente, responda:

   > Use uma hipótese reversível, marque-a como hipótese e continue. Só volte a perguntar se isso
   > alterar materialmente o produto ou envolver um risco importante.

4. Depois do Brief, o Codex deve continuar para o protótipo local, usando ferramentas disponíveis ou
   uma alternativa local viável. Se uma integração de criação de sites falhar, isso não prova um bug
   do Tramuri; verifique se o agente consegue continuar com os recursos locais disponíveis.

Você não precisa escrever `tramuri mission create`, montar JSON de cada etapa nem perguntar “o que
faço agora?”. O Codex consulta o contrato de cada etapa pelo CLI e submete o resultado ao Tramuri.
Uma etapa só conta como concluída quando o Core aceita a submissão. Uma etapa validada do DEFINE
significa que o Product Brief foi estruturado; não significa que o SaaS foi construído ou que seu
resultado de negócio foi verificado.

## 4. Conferir os critérios do protótipo

Depois que o Product Brief for aceito, confira se os critérios do Success Contract são os cinco que
você autorizou no prompt. O agente deve registrar essa revisão do contrato antes de avaliar os
resultados. A alteração é explícita e cria uma revisão; não substitui o histórico.

Quando o protótipo estiver pronto, percorra os critérios você mesmo. Registre somente o que
realmente testou. Por exemplo, se pesquisou um curso e o abriu, pode avaliar esse critério como
`met`; se não verificou a revisão administrativa, mantenha esse critério como `unverified`. A
referência abaixo é apenas um rótulo local — o Tramuri não abre nem valida automaticamente o recurso.

Uma Execution descreve o trabalho feito; Evidence aponta para o que foi observado; Outcome é a
avaliação explícita dos critérios. Se não houver evidência suficiente, deixe o resultado sem
verificação.

## 5. Ver o estado da Mission e o relatório

O Codex deve atualizar o estado conforme o trabalho avança. Se quiser conferir no terminal:

```console
tramuri status
tramuri mission list
tramuri resume --mission <MISSION_ID>
```

Os relatórios ficam em:

```text
.tramuri/reports/tramauri.html
.tramuri/reports/tramauri.md
.tramuri/reports/tramauri.json
```

As gravações aceitas pelo CLI atualizam os arquivos do relatório de forma best-effort. Uma aba HTML
já aberta é uma página estática; use o botão de atualizar do relatório ou atualize a página do
navegador para buscar o estado mais recente. Para abrir o relatório no navegador ao final, execute
`tramuri status --open` se ele ainda não tiver sido aberto nessa sessão. Não é necessário reabrir a
janela a cada etapa.

## 6. Testar o protótipo e fazer handoff

Use a instrução de execução que o Codex deixar no final. Teste manualmente catálogo, detalhe,
matrícula simulada, progresso da aula e revisão do curso. Registre como evidência somente o que você
realmente observou. Se não percorreu um fluxo, mantenha o critério como não verificado. Uma referência
de demonstração não é validada automaticamente pelo Tramuri.

Para simular um handoff, em outra sessão do Codex ou com outra pessoa, não envie este histórico.
Peça que a pessoa retome a Mission pelo Tramuri e identifique objetivo, decisões, hipóteses, pendências,
arquivos e testes. Ela pode usar:

```console
tramuri mission list
tramuri resume --mission <MISSION_ID>
tramuri status --open
```

O rótulo de perspectiva `Product Engineer` não informa quem executou o trabalho. `actor` é um rótulo
declarado, não uma identidade autenticada.

## 7. Alternativa manual: registrar contrato, execução e evidência

O caminho principal pede ao Codex que faça esses registros depois do seu aceite e das verificações.
Se estiver operando manualmente, crie `success-contract.json` somente com os critérios que você
aprovou:

```json
{
  "criteria": [
    {"description":"Uma pessoa consegue pesquisar o catálogo e abrir os detalhes de um curso","required":true,"evidence_expectation":"Fluxo exercitado no protótipo local com dados fictícios"},
    {"description":"Uma matrícula simulada deixa o curso visível na área do aluno","required":true,"evidence_expectation":"Demonstração local da matrícula fictícia"},
    {"description":"Uma aula concluída atualiza o progresso do aluno","required":true,"evidence_expectation":"Demonstração local do progresso atualizado"},
    {"description":"Um instrutor consegue enviar um rascunho de curso para revisão administrativa","required":true,"evidence_expectation":"Demonstração local do envio e da mudança de estado"},
    {"description":"A interface funciona em tamanhos de tela comuns e usa textos claros em pt-BR","required":true,"evidence_expectation":"Revisão manual em tamanhos de tela representativos"}
  ]
}
```

Registre o contrato e consulte a revisão e os IDs dos critérios. Use esses valores nos comandos
seguintes:

```console
tramuri mission contract revise --mission <MISSION_ID> --criteria-file success-contract.json --json
tramuri mission show --mission <MISSION_ID> --json
tramuri execution record --mission <MISSION_ID> --contract-revision <CONTRACT_REVISION> --actor-kind <ACTOR_KIND> --actor "<rótulo real do executor>" --work-perspective product_engineer --status completed --notes "Protótipo Aulae implementado e fluxos locais percorridos"
tramuri evidence add --mission <MISSION_ID> --execution <EXECUTION_ID> --kind external_reference --locator "demo-local:aulae-v1" --json
```

Substitua `<ACTOR_KIND>` por `human`, `agent` ou `tool` e use o rótulo que corresponde a quem
realmente fez o trabalho; esse rótulo não é autenticado. Só use `status completed` se a Execution
terminou. Se você não executou o teste, não registre que ele foi observado. Avalie cada critério
usando o ID retornado por `mission show`:

```console
tramuri outcome assess --mission <MISSION_ID> --contract-revision <CONTRACT_REVISION> --criterion <CRITERION_ID> --result met --evidence <EVIDENCE_ID> --verifier-kind human --verifier "<rótulo do revisor humano>" --explanation "Descreva o fluxo que foi realmente testado"
tramuri outcome show --mission <MISSION_ID>
```

O comando `met` é um exemplo para um critério que você realmente verificou; não o copie para todos
os critérios sem testar. Use `unverified` para o que não foi comprovado. Uma aprovação registrada
com referência local não significa que o Tramuri validou o aplicativo, a evidência ou o sucesso
comercial.

## 8. Alternativa manual: consultar o workflow pelo CLI

Use esta parte se não estiver usando o Codex. A CLI persiste e valida o workflow, mas não faz sozinha
a análise de produto nem gera conteúdo de LLM. Para cada etapa, produza o JSON conforme o contrato
retornado por `next` e submeta-o; a referência técnica explica o fluxo completo.

```console
tramuri start --goal "Criar uma plataforma SaaS de cursos online para alunos e instrutores" --perspective product_engineer --context "Exemplo fictício Aulae; catálogo, detalhe de curso, matrícula simulada, área do aluno, progresso e revisão de cursos por instrutores; sem pagamentos reais ou deploy" --json
tramuri mission define status --mission <MISSION_ID> --json
tramuri mission define next --mission <MISSION_ID> --json
```

Depois que as etapas necessárias forem aceitas, indique a atividade de prototipação e registre a
incerteza do discovery:

```console
tramuri mission workflow activity --mission <MISSION_ID> --activity prototype
tramuri readiness assess --mission <MISSION_ID> --activity prototype --state needs_review --gap "Não há validação com alunos ou instrutores reais" --accept-uncertainty "O preço de R$ 200/mês é apenas ilustrativo" --actor "Product Engineer"
tramuri status
tramuri resume --mission <MISSION_ID>
```

Para ver o restante das operações de Execution, Evidence e Outcome, use os exemplos da seção
anterior e registre somente resultados que você observou. A conclusão do código não deve ser marcada
como prova de sucesso comercial.

Consulte o [fluxo técnico de Product Engineering](../../reference/product-engineering-execution-flow.pt-BR.md)
para os contratos do DEFINE, revisão de etapas e persistência.
