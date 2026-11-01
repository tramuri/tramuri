# Exemplo: Forward Deployed Engineer

Este tutorial simula uma integração empresarial fictícia sem Git ou código-fonte. Você instalará o
Tramuri, registrará requisitos e restrições, simulará uma validação e fará handoff a outro FDE. Não
use credenciais, sistemas reais ou informações de clientes.

Os comandos requerem Tramuri 0.5.0 ou mais recente. Confira `tramuri --version` após instalar;
`uv tool install tramuri` instala a versão publicada mais recente.

## 1. Instalar e preparar um workspace sem Git

```console
uv tool install tramuri
tramuri --version
mkdir northstar-integration
cd northstar-integration
tramuri install --project --platform codex
tramuri init .
tramuri status --open
```

A integração de agente é opcional. O instalador preserva instruções existentes; o hook Codex apenas
exibe um lembrete e não lê dados da Mission nem executa comandos.

## 2. Começar com uma frase no Codex

Depois da instalação, não digite `tramuri mission create`. Diga ao Codex o que precisa fazer:

> Estou atuando como FDE e quero simular uma integração de eventos de pedidos para a empresa
> fictícia Northstar. Use somente dados inventados, sem Git e sem conectar a sistemas. Comece uma
> Mission exploratória e conduza o discovery comigo, fazendo só as perguntas necessárias.

O assistente deve abrir a Mission exploratória, registrar requisitos e restrições que você confirmar
e guiar os próximos passos em linguagem natural. Não precisa preencher comandos manualmente. Para
demonstrar um handoff, peça ao Codex que abra o relatório ao final e depois solicite a outra pessoa
que retome a Mission em uma sessão independente.

As próximas seções mostram o fluxo manual equivalente. Use-as se estiver sem assistente ou quiser
operar pelo terminal; não repita a sequência de comandos no Codex.

## Alternativa manual: criar Mission e registrar discovery

“Northstar” é uma empresa fictícia que quer enviar eventos de pedidos sintéticos a um endpoint de
teste. Comece sem critérios, pois os detalhes ainda serão descobertos:

```console
tramuri mission create --objective "Validar integração fictícia de eventos de pedidos da Northstar" --context "Cenário fictício; sem sistemas de produção ou credenciais" --json
```

Guarde o ID como `<MISSION_ID>` e registre lacunas conhecidas:

```console
tramuri readiness assess --mission <MISSION_ID> --activity discover --state needs_review --gap "Os campos do evento ainda não foram confirmados" --gap "O período de retenção ainda não foi acordado" --actor "FDE 1"
```

Registre somente conhecimento sintético, com proveniência de entrada humana:

```console
tramuri knowledge add --type fact --statement "Os eventos fictícios contêm order_id, status e updated_at" --human-input --json
tramuri knowledge add --type fact --statement "O cliente fictício proíbe dados pessoais no payload de teste" --human-input --json
tramuri knowledge add --type question --statement "Quem aprova o teste de aceite fictício?" --human-input --json
```

Use os IDs retornados para associar cada registro relevante explicitamente:

```console
tramuri mission link-knowledge --mission <MISSION_ID> --knowledge <REQUIREMENT_ID> --actor "FDE 1"
tramuri mission link-knowledge --mission <MISSION_ID> --knowledge <RESTRICTION_ID> --actor "FDE 1"
tramuri mission link-knowledge --mission <MISSION_ID> --knowledge <QUESTION_ID> --actor "FDE 1"
```

## 3. Definir critérios de aceite

Crie `success-contract.json` no workspace:

```json
{
  "criteria": [
    {"description":"O endpoint de teste recebe os três campos fictícios acordados","required":true,"evidence_expectation":"Registro fictício do teste do endpoint"},
    {"description":"Nenhum dado pessoal é enviado no payload de teste","required":true,"evidence_expectation":"Verificação revisada do payload fictício"},
    {"description":"A pessoa responsável pelo cliente aceita o resultado do teste","required":true,"evidence_expectation":"Referência fictícia de aceite humano"}
  ]
}
```

Revise o contrato e consulte os IDs dos critérios:

```console
tramuri mission contract revise --mission <MISSION_ID> --criteria-file success-contract.json --json
tramuri mission show --mission <MISSION_ID> --json
```

Guarde `<CONTRACT_REVISION>` e `<CRITERION_1>`, `<CRITERION_2>` e `<CRITERION_3>`. Avalie a
prontidão para a próxima atividade. Se houver bloqueio crítico, registre `blocked` em vez de
ocultá-lo como incerteza:

```console
tramuri readiness assess --mission <MISSION_ID> --activity prototype --state ready --criterion-applied "Os campos e a restrição de privacidade estão registrados" --accept-uncertainty "A pessoa de contato para aprovação do cliente está sendo confirmada" --actor "FDE 1"
```

Readiness é consultiva e não substitui aprovação de segurança ou autorização para acessar sistemas.

## 4. Simular integração e avaliar evidências

Faça uma validação manual local, sem conexão com serviços, e registre a execução:

```console
tramuri execution record --mission <MISSION_ID> --contract-revision <CONTRACT_REVISION> --actor-kind human --actor "FDE 1" --work-perspective fde --status completed --notes "Mapeou campos fictícios e verificou o payload localmente" --json
```

Guarde `<EXECUTION_ID>` e adicione uma referência fictícia. O Tramuri não abre nem valida o
conteúdo do locator:

```console
tramuri evidence add --mission <MISSION_ID> --execution <EXECUTION_ID> --kind external_reference --locator "registro-de-teste-ficticio:northstar-execucao-01" --json
```

Guarde `<EVIDENCE_ID>`. Avalie os dois critérios demonstrados e deixe o aceite pendente:

```console
tramuri outcome assess --mission <MISSION_ID> --contract-revision <CONTRACT_REVISION> --criterion <CRITERION_1> --result met --evidence <EVIDENCE_ID> --verifier-kind mechanism --verifier "Teste fictício do payload"
tramuri outcome assess --mission <MISSION_ID> --contract-revision <CONTRACT_REVISION> --criterion <CRITERION_2> --result met --evidence <EVIDENCE_ID> --verifier-kind human --verifier "Pessoa revisora fictícia de privacidade"
tramuri outcome assess --mission <MISSION_ID> --contract-revision <CONTRACT_REVISION> --criterion <CRITERION_3> --result unverified --verifier-kind human --verifier "Pessoa responsável fictícia pelo cliente" --explanation "Nenhum aceite foi registrado"
tramuri outcome show --mission <MISSION_ID>
```

O Outcome deve continuar pendente: execução concluída não é aceite do cliente.

## 5. Fazer handoff para outro FDE

Em uma sessão independente, peça a outro FDE que reconstrua o estado usando as interfaces públicas:

```console
tramuri mission list
tramuri resume --mission <MISSION_ID>
tramuri knowledge list
tramuri execution list --mission <MISSION_ID>
tramuri evidence list --mission <MISSION_ID>
tramuri outcome show --mission <MISSION_ID>
tramuri status --open
```

O segundo FDE deve encontrar requisitos, restrição de privacidade, execução, evidência e aceite
pendente sem depender do histórico oral do primeiro. `actor` é um rótulo declarado, não identidade
autenticada. Os relatórios locais são `.tramuri/reports/tramauri.html`, `tramauri.md` e
`tramauri.json`; são projeções e não alteram nem verificam o Outcome.
