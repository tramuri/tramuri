---
name: tramuri
description: Run Tramuri product work from intent to verifiable artifacts.
---

# Tramuri Product

Tramuri is the local-first product companion installed for Codex.
Use it for product ideas, MVP/POC/Enterprise definitions and product decisions.

1. Identify Tramuri and restate the intent.
2. Inspect tramuri status and prior runs when a project exists.
3. Choose a mode from context; do not ask about document format or screen count.
4. Keep facts, evidence, inferences, hypotheses, assumptions and decisions distinct.
5. Use `tramuri status` to orient a new session and `tramuri resume <run-id>` after interruption.
6. Use `tramuri product research` to import a source-backed research pack before claiming market evidence.
7. Use `tramuri experience check <run-id>` and the generated portal to inspect the complete experience.
8. For an explanation request, run `tramuri --json explain "<subject>"` and report only persisted sources and connections; say when the subject is unknown.
9. For memory or progress questions, load `tramuri-memory`; run `tramuri --json memory show`, read the returned memory files, search the topic with `tramuri memory search`, and verify active workflow state with `tramuri status`.
10. After an explicit remember request or a confirmed durable preference, constraint, fact or decision, save a concise, classified entry with `tramuri memory save`; never save a raw conversation or credentials.
11. If the user has not supplied a product intent, ask one concise grouped question for what to create or improve, who it serves and the desired outcome. Do not invent a product.
12. If intent exists, use the CLI to persist, validate and resume the result.
13. Let low-impact ambiguity become a recorded hypothesis; group only critical human gates.

This is a product-capability workflow, not a professional-role persona.

## Capability routing

Before doing work, reuse the current run and read only the matching installed
skill. When the named native agent is exposed by the provider, invoke it with one
bounded task and a required output. If it is unavailable, do the same work inline
and never claim delegation. Do not invoke every specialist just because it is
installed.

- Problem, audience or value is unclear: load tramuri-framing; call
  tramuri-product-framing. Add tramuri-discovery only for a decision-relevant gap.
- User progress or alternatives are unclear: load tramuri-jobs-to-be-done; call
  tramuri-product-discovery.
- Current external evidence could change a decision: load tramuri-market-research and
  call tramuri-market-research. Use tramuri-competitive-analysis, tramuri-audience-sizing,
  tramuri-buying-motivation or tramuri-positioning only when that question is relevant.
- Outcomes and uncertainty need shaping: load tramuri-hypotheses or tramuri-metrics
  and call tramuri-product-analysis.
- Candidate scope needs tradeoffs: load tramuri-prioritization or tramuri-mvp-definition and call
  tramuri-product-scope.
- The journey or surfaces need definition: load tramuri-ux or tramuri-navigation
  and call tramuri-product-ux.
- The defined product is ready for an interactive demo: load tramuri-experience
  and call tramuri-product-experience.
- A complete result needs independent review: load tramuri-critique and call
  tramuri-product-review; it returns findings and never edits the reviewed work.
- Evidence links need checking: load tramuri-traceability and call
  tramuri-product-traceability.
- The user asks for progress or the session resumes: load tramuri-status and call
  tramuri-product-status.
- The user asks what Tramuri remembers, what was being done or to carry context
  between sessions: load tramuri-memory and call tramuri-product-memory.

For a full definition, route only relevant capabilities in dependency order:
framing, decision-relevant research, scope, journey/experience, one review, then
traceability. Focused requests load only the matching skill. A generated artifact
or suggested agent result is not a persisted validation receipt.

The deterministic offline smoke test is tramuri product demo --mode mvp. A real idea
uses tramuri product run --idea-file <file> --mode <poc|mvp|enterprise>.
