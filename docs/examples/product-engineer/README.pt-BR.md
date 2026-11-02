# Exemplo: Product Engineer

Este tutorial acompanha a criação de um produto fictício do zero: um aplicativo para vizinhos
emprestarem ferramentas. Ele percorre instalação, configuração, Mission, discovery, protótipo,
evidências, revisão e handoff. Não exige Git nem dados reais de usuários. Execução concluída não
significa resultado verificado.

Os comandos requerem Tramuri 0.5.0 ou mais recente. Depois de instalar, confira `tramuri --version`;
`uv tool install tramuri` instala a versão publicada mais recente.

## 1. Instalar e configurar

```console
uv tool install tramuri
tramuri --version
mkdir empresta-ferramentas
cd empresta-ferramentas
tramuri install --project --platform codex
```

A integração com Codex preserva instruções existentes e instala hooks nativos de ferramentas. Em
cada turno, ferramentas locais do projeto ficam bloqueadas até `tramuri start` ou `tramuri resume`
terminar com sucesso. O gate não lê nem armazena o prompt ou os registros da Mission. Se o Codex
desativar/não confiar nos hooks ou usar uma ferramenta hospedada fora do hook local, as instruções
servem apenas como orientação. Um pedido substancial em linguagem natural inicia a Mission
exploratória e inicializa o workspace automaticamente. Não rode `tramuri init` antes nem abra um
relatório vazio.

## 2. Começar com uma frase no Codex

Depois de instalar a integração, não digite `tramuri mission create`. Diga ao Codex o que quer
alcançar, em suas palavras. O assistente deve iniciar ou retomar a Mission antes de explorar arquivos
ou acionar outra ferramenta de criação. Ele começa com uma atualização curta de Estruturação do
produto e faz discovery útil antes de perguntar algo que mudaria materialmente o produto:

> Tenho uma ideia para um produto novo: um aplicativo para vizinhos emprestarem ferramentas que
> usam pouco. Quero começar esse projeto do zero como Product Engineer. Me guie desde a ideia e
> pergunte só o essencial.

O assistente deve iniciar a Mission com o Tramuri, usar a skill de Product Engineering e continuar
sem transformar discovery em um interrogatório. Ele separa fatos informados por você, observações,
hipóteses e dúvidas. O Tramuri preserva o contexto confirmado; o Codex ajuda a transformar a ideia
em software. Responda em linguagem natural e descreva o trabalho realizado; não é preciso montar
comandos. Ao final da sessão, o assistente atualiza os relatórios. Ele abre o HTML somente se ainda
não o tiver aberto nessa sessão; se já estiver aberto, atualiza os arquivos sem abrir outra janela.

Os comandos das próximas seções mostram uma alternativa manual e o que fica registrado. Use-os se
estiver sem assistente ou se quiser operar pelo terminal; não precisa repetir esse fluxo no Codex.

## Alternativa manual: iniciar pelo objetivo e fazer discovery

O objetivo fictício é criar a primeira versão do aplicativo. Comece sem critérios porque o fluxo do
produto ainda será descoberto:

```console
tramuri start --goal "Criar a primeira versão de um aplicativo para vizinhos emprestarem ferramentas" --perspective product_engineer --json
```

Guarde o ID retornado como `<MISSION_ID>` e registre uma lacuna:

```console
tramuri mission workflow activity --mission <MISSION_ID> --activity discover
tramuri readiness assess --mission <MISSION_ID> --activity discover --state needs_review --gap "Ainda não sabemos quais ferramentas os vizinhos estariam dispostos a emprestar" --actor "Engenheiro de Produto"
tramuri status
tramuri resume --mission <MISSION_ID>
```

Após uma conversa fictícia de discovery, registre uma decisão e associe-a explicitamente à Mission:

```console
tramuri knowledge add --type decision --statement "Começar pelo catálogo de ferramentas e pedidos de empréstimo, sem pagamentos na primeira versão" --human-input --json
tramuri mission link-knowledge --mission <MISSION_ID> --knowledge <KNOWLEDGE_ID> --actor "Engenheiro de Produto"
```

Use o ID da primeira chamada como `<KNOWLEDGE_ID>`. O Tramuri não infere associações.

## 3. Definir o Success Contract

Crie `success-contract.json` no diretório do exemplo:

```json
{
  "criteria": [
    {"description":"Uma pessoa consegue consultar ferramentas disponíveis e solicitar um empréstimo","required":true,"evidence_expectation":"Demonstração local do fluxo fictício de solicitação"},
    {"description":"A pessoa proprietária de uma ferramenta consegue aprovar ou recusar um pedido","required":true,"evidence_expectation":"Demonstração local revisada do fluxo de resposta"}
  ]
}
```

Registre o contrato e consulte a revisão e os IDs vigentes:

```console
tramuri mission contract revise --mission <MISSION_ID> --criteria-file success-contract.json --json
tramuri mission show --mission <MISSION_ID> --json
```

Guarde `<CONTRACT_REVISION>`, `<CRITERION_1>` e `<CRITERION_2>`. Avalie de novo a prontidão depois
da revisão do contrato:

```console
tramuri readiness assess --mission <MISSION_ID> --activity prototype --state ready --criterion-applied "O fluxo inicial de solicitação está definido" --criterion-applied "A demonstração local foi escolhida como forma de avaliação" --accept-uncertainty "O protótipo ainda não foi testado por vizinhos reais" --actor "Engenheiro de Produto"
```

`ready` exige uma verificação aplicada e não permite lacunas ou bloqueios em aberto. Incertezas
aceitas permanecem visíveis; readiness é consultiva e não impede trabalho.

## 4. Registrar execução e evidência

Depois do experimento sintético, registre a execução e uma referência fictícia. Guarde os IDs
retornados como `<EXECUTION_ID>` e `<EVIDENCE_ID>`:

```console
tramuri execution record --mission <MISSION_ID> --contract-revision <CONTRACT_REVISION> --actor-kind human --actor "Engenheiro de Produto" --work-perspective product_engineer --status completed --notes "Implementou e demonstrou o protótipo local fictício" --json
tramuri evidence add --mission <MISSION_ID> --execution <EXECUTION_ID> --kind external_reference --locator "demonstracao-ficticia:empresta-ferramentas-v1" --json
tramuri execution list --mission <MISSION_ID>
tramuri evidence list --mission <MISSION_ID>
```

O locator é uma referência: o Tramuri não abre URLs ou lê arquivos para validá-lo. Avalie critérios
apenas depois de revisar evidências:

```console
tramuri outcome assess --mission <MISSION_ID> --contract-revision <CONTRACT_REVISION> --criterion <CRITERION_1> --result met --evidence <EVIDENCE_ID> --verifier-kind human --verifier "Revisor fictício do protótipo" --explanation "O fluxo foi demonstrado localmente com dados inventados"
tramuri outcome assess --mission <MISSION_ID> --contract-revision <CONTRACT_REVISION> --criterion <CRITERION_2> --result unverified --verifier-kind human --verifier "Revisor fictício do protótipo" --explanation "A resposta ao pedido ainda não foi demonstrada"
tramuri outcome show --mission <MISSION_ID>
```

O Outcome continua pendente enquanto um critério obrigatório não estiver verificado.

## 5. Handoff e relatório

Em uma sessão independente, outra pessoa deve reconstruir o trabalho sem receber o histórico desta
simulação:

```console
tramuri mission list
tramuri resume --mission <MISSION_ID>
tramuri status --open
```

Se o HTML já tiver sido aberto na sessão, use `tramuri status --refresh` para atualizar os arquivos
sem abrir o navegador novamente.

Peça que identifique a ideia do produto, a decisão vinculada, a execução, a evidência e o critério pendente. O relatório
fica em `.tramuri/reports/tramauri.html`, com versões Markdown e JSON ao lado. Ele não altera o
Outcome. O rótulo `actor` não autentica uma pessoa.
