# Tramuri Product Vision

**Positioning:** From intent to verified outcomes.

Tramuri is open-source, local-first, agent-native infrastructure for turning product and business
objectives into coordinated work and verifiable outcomes. It connects persistent knowledge,
mission context, people, agents, tools, execution records, evidence, and explicit review.

Tramuri serves Product Engineers, Forward Deployed Engineers (FDEs), product teams, and engineering
and product leaders. Code contributors are an important part of the open-source community, but the
product is not limited to software repositories or programming agents.

## Product commitments

- A Mission and its Success Contract can exist without source code, a Git repository, an LLM, or an
  agent framework.
- Project Discovery is one optional source capability. It does not define Tramuri's domain boundary.
- Mission state, execution state, evidence, criterion assessments, and verified outcomes remain
  distinct. A completed execution does not establish that its intended outcome was achieved.
- An outcome assessment records the criteria reviewed, linked evidence, the responsible person or
  verification mechanism, the assessment date, and the result.
- Evidence identifies what supports a claim and its provenance. It does not turn an inference into
  a fact or a tool's self-report into independent verification.
- Context and Evidence Pack export uses explicit selection and authorization. Source contents and
  credentials are excluded. Human-authored statements are excluded unless the user selects them
  and authorizes that export.
- The local product remains useful without a hosted service. Future managed services must not make
  the open-source core depend on a provider.

## Domain vocabulary

- **Mission:** an objective and the scope of work intended to achieve it.
- **Success Contract:** the explicit, reviewable criteria used to assess a Mission. Criteria are
  descriptive in the MVP; Tramuri does not execute a generic rules engine.
- **Context Pack:** a selected, versioned set of Mission context and approved knowledge references.
- **Execution:** a record of work carried out by a person or tool. The MVP records execution; it
  does not run or orchestrate arbitrary tools.
- **Evidence Pack:** selected, versioned evidence references and provenance, with freshness and
  validation state preserved.
- **Verified Outcome:** an assessment of a Mission's criteria with evidence and an identified
  reviewer or verification mechanism. Mission closure and execution completion remain separate
  from this assessment.
- **Process Pack:** a reusable process definition. This is a future concept, pending evidence that
  teams repeat processes that Tramuri should encode.

## Business direction

The long-term direction may evolve from software as a service toward service as a software:

1. **Tramuri Open Source:** local, free infrastructure independent of vendors.
2. **Managed Workflows:** future managed coordination and verification services.
3. **Service as a Software:** future specialized services that deliver measurable, verifiable
   results.

The second and third levels are strategic possibilities. They do not authorize Cloud, billing,
enterprise authentication, subscriptions, or workflow hosting in the MVP.

## MVP boundary

The MVP builds a local, inspectable path from intent to reviewed result:

1. Initialize local Tramuri state in any chosen workspace directory.
2. Create a Mission and Success Contract without requiring a repository scan.
3. Attach relevant persistent knowledge and explicitly selected Context Pack material.
4. Record an execution and evidence references without launching tools.
5. Assess each required criterion and report a Verified Outcome only when the required evidence and
   review conditions are met.
6. Preserve these distinctions through versioned Context and Evidence Pack contracts and equivalent
   Rust CLI and Python SDK operations.

Process Packs, automatic orchestration, managed workflows, Cloud, billing, and subscription
systems remain outside the MVP.

## Architecture commitments

- Rust is the shared domain core and native CLI implementation.
- Python uses PyO3 and maturin as an interface to the same Rust core.
- SQLite is the initial local store; schema changes require compatibility analysis and an additive
  migration plan.
- CLI, SDK, and pack behavior must stay compatible through shared core operations and versioned
  contracts.
- The core does not require an LLM, agent framework, Codex, AI-DLC, Spec Kit, Graphify, or another
  harness.
- The project is licensed under Apache 2.0.

## Validation with users

Validate the product direction separately with Product Engineers and FDEs. Use real, non-sensitive
work examples to learn how each group defines success, loses context, accepts evidence, and decides
who can validate an outcome. Treat interviews and prototypes as evidence about usability and
workflow fit, not as proof of commercial demand. Do not build managed services until repeated
needs and willingness to adopt have been observed.
