# ADR-0001: Deterministic provider hook gate backed by SQLite

## Status

Accepted — 2026-11-07

## Context

Skills and persistent instruction files help an agent understand Tramuri, but they
do not guarantee that an agent starts the product workflow before it
changes project files. Codex, Claude Code and Gemini CLI expose different native
hook events and different JSON decision contracts. Hook availability also depends
on trust, version and host configuration.

## Decision

Install one small Python hook runtime and provider-specific registrations. The
runtime normalizes the incoming event, reads the project-local SQLite state, records
metadata in `hook_events`, and emits the provider's native context or deny shape.

The normalization and rendering are harness responsibilities. The decision itself
is produced by the provider-neutral policy in tramuri.core.policy, so the three
providers cannot silently drift in workflow rules.

For a product-intent turn, mutating tool calls are denied until a completed product run
exists. Tramuri bootstrap commands and writes under `tramuri_output/input/` remain allowed.
Read-only operations are not blocked. The hook never executes event values and does
not persist prompt text, commands, tool arguments or credentials.

## Rationale

- SQLite is already the source of truth for Tramuri runs and checkpoints.
- A shared runtime prevents three providers from implementing different workflow
  policies.
- Native adapters preserve each host's trust and permission model.
- The gate is deterministic while the Product Manager remains responsible for
  semantic analysis and artifact quality.

## Consequences

### Positive

- A provider cannot silently skip the first persisted product workflow before mutating
  the project when its hooks are active and trusted.
- Hook activity and denials are inspectable without storing sensitive input.
- Existing provider settings are merged and non-Tramuri entries are preserved.

### Negative

- Hook coverage is not universal: disabled, untrusted, unsupported or hosted paths
  may bypass the gate.
- Users must review/trust Codex project hooks and may need to restart a provider
  after configuration changes.
- The deterministic gate cannot judge whether the product analysis is correct.

## References

- [Codex hooks](https://learn.chatgpt.com/docs/hooks)
- [Claude Code hooks](https://code.claude.com/docs/en/hooks-guide)
- [Gemini CLI hooks](https://github.com/google-gemini/gemini-cli/blob/main/docs/hooks/index.md)
