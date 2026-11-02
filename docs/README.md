# Tramuri documentation

## Start with a tutorial

The tutorials use fictional data and walk through setup, a Mission, recorded work, evidence,
review, and handoff. You do not need Git or a source-code repository for the FDE example.

| Work perspective | English | Português (Brasil) |
| --- | --- | --- |
| Product Engineer | [Product Engineer tutorial](examples/product-engineer/README.md) | [Tutorial de Product Engineer](examples/product-engineer/README.pt-BR.md) |
| Forward Deployed Engineer | [FDE tutorial](examples/fde/README.md) | [Tutorial de FDE](examples/fde/README.pt-BR.md) |

Typical first use:

1. Install the CLI and add Tramuri to the project for your assistant, if you use one.
2. Describe the work in the assistant chat, or run `tramuri start --goal "..."` in a terminal.
3. Explicitly select Product Engineer, FDE, or mixed work for a new Mission.
4. Continue the Mission and review its local report when useful.

New Product Engineer Missions use the persisted five-step DEFINE workflow. The Core controls step
order and validates submitted structures; it does not perform product research or guarantee that an
assistant followed a skill. The tutorials distinguish recorded evidence from verified outcomes.

## Technical references

- [Product Engineering: DEFINE → BUILD → SHIP](reference/product-engineering-execution-flow.md)
- [Product Engineering: DEFINE → BUILD → SHIP (Português)](reference/product-engineering-execution-flow.pt-BR.md)
- [Complete operational flow and PE/FDE behavior (Português)](reference/fluxo-operacional-completo.pt-BR.md)
- [DEFINE review, stale propagation, and Product Brief assessment (Português)](reference/define-hardening.pt-BR.md)
- [Architecture decisions](adr/)

The repository overview is in the [main README](../README.md); the product vision is in
[`VISION.md`](../VISION.md).
