# ADR-0003: Use `product` as the public workflow boundary

- Status: Accepted
- Date: 2026-11-07
- Decision owners: Tramuri maintainers

## Context

The product workflow was initially exposed as `pm`, which makes the public
experience sound like a profession instead of a product-building stage. The
Core/Harness split also temporarily kept import facades under
`tramuri.pm` and `tramuri.providers`.

## Decision

The public workflow commands are:

- `tramuri product run`
- `tramuri product demo`

The old `pm` command group and the top-level `tramuri.pm` and
`tramuri.providers` facade packages are removed. Canonical implementations live
under `tramuri.modules.pm` for the Product Management capability and
`tramuri.harness` for provider integrations. The internal capability name does
not create a second public command or import boundary.

## Consequences

- New users see product-oriented language in CLI help, installed skills and
  documentation.
- Existing callers importing the removed facade packages must migrate to
  `tramuri.modules.pm` or `tramuri.harness`.
- Persisted SQLite records keep their existing internal artifact identifiers;
  this naming decision does not rewrite stored history.

## Verification

CLI tests assert that `product` is present and `pm` is absent from root help.
Contract and integration tests import only canonical module and harness paths.
