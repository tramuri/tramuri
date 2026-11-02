# Documentação do Tramuri

## Comece por um tutorial

Os tutoriais usam dados fictícios e acompanham a configuração, a Mission, o registro do trabalho,
as evidências, a avaliação e o handoff. O exemplo de FDE não exige Git nem repositório de código.

| Perspectiva de trabalho | Português (Brasil) | English |
| --- | --- | --- |
| Product Engineer | [Tutorial de Product Engineer](examples/product-engineer/README.pt-BR.md) | [Product Engineer tutorial](examples/product-engineer/README.md) |
| Forward Deployed Engineer | [Tutorial de FDE](examples/fde/README.pt-BR.md) | [FDE tutorial](examples/fde/README.md) |

Primeiro uso:

1. Instale a CLI e, se usar um assistente, instale a integração do Tramuri no projeto.
2. Descreva o trabalho no chat do assistente ou execute `tramuri start --goal "..."` no terminal.
3. Para uma Mission nova, escolha explicitamente Product Engineer, FDE ou trabalho misto.
4. Continue a Mission e consulte o relatório local quando for útil.

Missions novas de Product Engineer usam o workflow DEFINE persistido de cinco etapas. O Core
controla a ordem e valida a estrutura enviada; ele não faz pesquisa de produto nem garante que o
assistente carregou uma skill. Os tutoriais diferenciam evidência registrada de resultado verificado.

## Referências técnicas

- [Fluxo de Product Engineering: DEFINE → BUILD → SHIP](reference/product-engineering-execution-flow.pt-BR.md)
- [Fluxo operacional completo e comportamento PE/FDE](reference/fluxo-operacional-completo.pt-BR.md)
- [Revisão do DEFINE, propagação stale e avaliação do Product Brief](reference/define-hardening.pt-BR.md)
- [Product Engineering: DEFINE → BUILD → SHIP (English)](reference/product-engineering-execution-flow.md)
- [Decisões de arquitetura](adr/)

Consulte o [README principal](../README.pt-BR.md) para conhecer o produto e a
[visão do Tramuri](../VISION.md).
