# ADR-0004: Evidence-backed research packs and product experience projections

## Status

Accepted

## Context

The deterministic MVP workflow correctly prevents fabricated market evidence,
but it currently stops at generic product artifacts and a single hard-coded
journey. Provider-installed skills and agent definitions are not evidence that
those agents ran. The product therefore needs a truthful boundary between:

- research performed by a provider or supplied explicitly by a user;
- deterministic Tramuri state and validation; and
- offline product experiences generated from the resulting product contract.

The workflow must remain local-first and must not require a hosted LLM backend,
Node.js or a mandatory internet connection.

## Decision drivers

- Preserve the no-fabrication and few-question product philosophy.
- Keep SQLite and the provider-neutral core authoritative.
- Make source, timestamp and confidence visible for external research.
- Generate domain-coherent marketing, customer and admin surfaces when scope
  supports them.
- Keep demos distinct from Product Engineering and production systems.
- Make missing research and unavailable browser validation explicit.

## Considered options

### Option 1: Make the CLI scrape the web directly

Rejected. Search engines, authentication, robots policies and result quality
vary across environments. A mandatory network implementation would also violate
the local-first installation boundary.

### Option 2: Let agents research without a persisted contract

Rejected. Conversation output is not durable evidence, cannot be resumed
reliably and makes it impossible to distinguish a real source from an inference.

### Option 3: Provider research plus validated local research packs

Accepted. The provider can use its native web capabilities, then persist a
structured pack through Tramuri. Offline or deterministic runs can explicitly
show `not_run` without inventing findings.

## Decision

Introduce a provider-neutral research-pack contract. The core validates source
URLs, timestamps, claims, confidence and evidence references, persists the pack
as a versioned product artifact and projects its findings into the report and
status views. The Product Experience builder consumes the product definition,
research and scope to generate domain-specific offline surfaces.

The first implementation supports deterministic archetypes for learning
marketplaces, appointment services and a safe generic fallback. The fallback
must visibly state that domain-specific research and screens are pending rather
than presenting a generic experience as a validated product.

## Consequences

### Positive

- Research can be deep without pretending that Tramuri itself browsed when it
  did not.
- Agents have a clear handoff contract and the user can resume after closing a
  provider session.
- Reports can show buying motivation, competitors, positioning and sizing with
  provenance.
- Product demos become coherent with the idea and can include multiple surfaces.

### Negative

- A provider workflow needs one additional structured handoff step.
- Research packs require schema validation and source freshness handling.
- More generated files require stronger navigation, accessibility and browser
  validation sensors.

## Implementation notes

- The research artifact is `pm.market_research` and is optional for the
  deterministic offline demo, where it is explicitly `not_run`.
- `tramuri product research --run-id <id> --research-file <file>` validates and
  imports a pack, then resumes dependent stages.
- Provider-facing skills instruct the agent to research only when the product
  decision benefits from it and to ask at most one grouped input question when
  intent, audience or outcome is absent.
- Generated surfaces remain under `tramuri_output/demo/` and reports under
  `tramuri_output/reports/`.

## Related decisions

- [ADR-0001](0001-deterministic-provider-hook-gate.md)
- [ADR-0002](0002-core-harness-separation.md)
- [ADR-0003](0003-product-command-and-no-legacy-facades.md)
