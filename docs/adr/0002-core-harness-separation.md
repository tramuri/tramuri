# ADR-0002: Separate the Tramuri core from provider harnesses

- Status: Accepted
- Date: 2026-11-07
- Decision owners: Tramuri maintainers

## Context

The first Python implementation placed provider detection, native file layouts, PM
content, hook payload parsing and installation in the same modules. That made it
hard to evolve the PM workflow once, test provider behavior independently, or add a
provider without copying domain logic. The AI-DLC project demonstrates a useful
boundary: a neutral core owns workflow knowledge and deterministic rules, while a
thin harness projects those rules into each host environment.

## Decision

Tramuri uses three runtime boundaries:

1. `tramuri.core` owns domain models, workflow contracts, state policy and neutral
   hook decisions. It must not import provider packages.
2. `tramuri.modules` owns product capabilities. `tramuri.modules.pm` owns PM stages,
   deterministic demo payloads and PM assets.
3. `tramuri.harness` owns provider detection, capability manifests, projections,
   native hook rendering and conflict-aware installation.

The `tramuri.hooks.runtime` module remains a real command boundary because it is
the installed hook entrypoint. The former `tramuri.providers` and `tramuri.pm`
facades are not retained; callers use `tramuri.harness` and
`tramuri.modules.pm` directly.

Provider hooks normalize host payloads into a core `HookEvaluation`; the core
returns a `HookDecision`; the harness renders that decision in the host's native
format. SQLite remains the local source of execution state and is opened only by
the runtime command, never by a shell hook that does not need it.

## Alternatives considered

- Keep the mixed modules: rejected because provider changes would continue to alter
  domain behavior and make contract tests weak.
- Copy the AI-DLC TypeScript/Bun architecture: rejected because Tramuri is a pure
  Python package and must not require Node, Rust or external services.
- Create fully independent provider implementations: rejected because it duplicates
  PM semantics and causes drift between Codex, Claude Code and Gemini CLI.

## Consequences

Positive:

- PM workflows can be tested without a provider or an LLM.
- Provider output differences are explicit and testable.
- Installation can be projected from a manifest and remains idempotent/conflict-aware.
- Adding a provider requires a harness adapter and contract tests, not a new core.

Costs and risks:

- Downstream imports of removed facade packages must migrate to the canonical
  core, module and harness boundaries.
- The native hook surfaces can change independently, so capability reports must be
  conservative and provider tests must assert the emitted shape.
- The boundary adds a small amount of indirection to the current CLI.

## Verification

The decision is verified by core-only imports/tests, golden provider projection and
hook tests, installer fixture tests, and the existing CLI/storage/PM test suite.
