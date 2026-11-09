# Tramuri Product Skills

Tramuri installs focused capabilities for product decisions. Invoke the
Tramuri entry skill for a complete workflow; use a specific capability for a
narrow request. A configured skill is not proof that a provider invoked it.

| Skill | Use when | Main output |
| --- | --- | --- |
| `tramuri-framing` | The product problem, audience or value is unclear | Problem, audience, need and value hypothesis |
| `tramuri-discovery` | Missing evidence could change a decision | Prioritized gaps and validation work |
| `tramuri-jobs-to-be-done` | The progress a user seeks is unclear | Job statement, situation, alternatives and evidence state |
| `tramuri-market-research` | Current market evidence affects a product decision | Source-backed research pack or not_run |
| `tramuri-competitive-analysis` | Alternatives or differentiation affect positioning | Evidence-linked comparison and implications |
| `tramuri-buying-motivation` | Purchase triggers, objections or switching reasons matter | Evidence-backed motivations and hypotheses |
| `tramuri-audience-sizing` | Addressable audience size changes viability or scope | Sizing method, range, assumptions and confidence |
| `tramuri-positioning` | The product needs a clear market position | Audience, outcome, alternative and differentiation |
| `tramuri-hypotheses` | An uncertain product claim needs validation | Testable hypothesis and validation signal |
| `tramuri-prioritization` | Several product items compete for scope | In, deferred and excluded items with rationale |
| `tramuri-mvp-definition` | A first usable increment must test an outcome | Critical path, scope boundary and learning signal |
| `tramuri-ux` | Product decisions need connected journeys and states | Actor journeys, states and accessibility needs |
| `tramuri-navigation` | A defined journey needs a screen map | Scoped navigation structure |
| `tramuri-experience` | Product artifacts are ready for an offline demo | Navigable product surfaces with synthetic data labels |
| `tramuri-metrics` | Success needs observable measures | Metric definitions and verification state |
| `tramuri-brief` | Existing artifacts need a coherent summary | Reviewable product brief with open validations |
| `tramuri-critique` | Product artifacts need an adversarial review | Read-only findings with evidence and repair guidance |
| `tramuri-traceability` | Claims, screens and validations need linkage | Traceability map and missing links |
| `tramuri-status` | A run must be resumed or explained | Persisted progress, current stage and next action |
| `tramuri-memory` | The user asks what Tramuri remembers, what was being done or to resume context | Retrieved project memory, verified status and next action |

Use `$tramuri` for the complete workflow or invoke one focused skill directly,
for example `$tramuri-status` or `$tramuri-memory`. Template and installed
directory slugs remain stable so `tramuri update` can replace skill metadata
in place.

The skills reuse current intent, run state, decisions, evidence and artifacts.
They ask only for information that has no safe default or requires a human
decision. They do not invent a product, market data, interviews, willingness to
pay, experiments or user validation.

Prose follows the user's language. Identifiers, research-pack fields and
persisted status values remain stable across languages.

Provider adapters install the entry skill and focused product skills side by
side. In Codex, run `tramuri update --provider codex` to install or refresh
`.agents/skills/<skill-name>/SKILL.md`. Codex detects project skills from that
directory; restart Codex if a newly installed skill does not appear.
