# ADR-0007: Prefix focused skill identifiers with Tramuri

## Status

Accepted

## Context

Tramuri installs its entry skill as `$tramuri`, alongside 20 focused product
skills. The focused skill metadata used unprefixed identifiers such as
`product-status` and `product-memory`. Their files were installed, but the
identifiers did not show that they belong to Tramuri or make them easy to find
beside the entry skill in Codex.

## Decision

Give every focused Tramuri skill a public `tramuri-` identifier. Remove a
redundant leading `product-` from existing product skill slugs, so
`product-status` becomes `tramuri-status` and `product-memory` becomes
`tramuri-memory`. Slugs without `product-` keep their full slug after the
prefix, such as `market-research` becoming `tramuri-market-research`.

Keep the template directories and provider installation paths stable. Change
the `name` metadata and routing references so `tramuri update` updates existing
installations in place without leaving duplicate skill directories. Keep the
general entry identifier `$tramuri` unchanged.

## Consequences

### Positive

- Focused skills appear under a consistent Tramuri namespace.
- Users can invoke a focused skill directly, for example `$tramuri-status` or
  `$tramuri-memory`.
- Existing provider files update in place through the normal `tramuri update`
  command.

### Negative

- Existing prompts that explicitly mention an old identifier need to use its
  Tramuri-prefixed name after updating the installation.
- Codex may need a restart before the updated names appear in its selector.

## Verification

- Contract tests require all 20 focused identifiers to use the `tramuri-`
  prefix for Codex, Claude Code and Gemini CLI plans.
- An installation migration test verifies that an existing skill file updates
  its metadata in place without creating a second directory.

## Related decisions

- ADR-0005: Use capability-based product skills, not a PM persona.
- ADR-0006: Curated project memory and individually discoverable skills.
