# Tramuri documentation

Tramuri turns an incomplete product idea into a structured, reviewable product
result. This documentation describes the first Python release.

## Runtime architecture

The package has one provider-neutral core and thin provider harnesses:

- tramuri.core owns contracts, workflow transitions, policy and local state rules.
- tramuri.modules owns product stages and reusable assets.
- tramuri.harness projects those assets into Codex, Claude Code and Gemini CLI.

The public runtime uses the core, product module and harness boundaries directly;
there are no legacy provider or `pm` facade packages. The harness never pretends
to be an LLM backend.

## Concepts

- [Product workflows](concepts/product-workflows.md) — the POC, MVP and Enterprise modes.

## How-to guides

- [Run the deterministic demo](howto/demo.md) — produce a report, portal and prototype locally.
- [Prepare a PyPI release](howto/release.md) — build and verify without publishing accidentally.

## Reference

- [CLI](reference/cli.md) — command grammar and output contracts.
- [Providers](reference/providers.md) — detected capabilities and fallbacks.
- [Hooks](reference/hooks.md) — deterministic provider gate and SQLite contract.
- [Product skills](reference/product-skills.md) — focused skills and provider discovery.

## Design

- [Report design system](design/report-design-system.md) — reusable visual,
  evidence, accessibility and validation rules for offline product reports.
- [Product Demo Engine](design/product-demo-system.md) — contextual categories,
  versioned demo specification, offline templates and simulation boundaries.

The local runtime also provides resumable context, provenance-labelled
knowledge, trusted deterministic sensors and a truthful `tramuri status`
projection. It generates readable project memory under
`tramuri_output/memorys/`. These mechanisms are provider-neutral and require no
LLM, Node.js, Rust or hosted service.

## Product Experience

Every completed product run can generate a central offline portal at
`tramuri_output/portal/index.html` and a separate exploratory demo at
`tramuri_output/demo/index.html`. The demo chooses its environment from the
persisted product context; see the [Product Demo Engine design](design/product-demo-system.md).
The portal links the Product Definition report to the Product Demo and makes
Product UX and Product Engineering explicit future stages. The demo is
synthetic and must not be presented as a production application or user
research.
