# Releasing `@tramuri/typescript`

This package is versioned independently from the Rust/Python workspace. Do not use the root
`vX.Y.Z` tags; they trigger the Python release workflow. TypeScript package tags use
`typescript-vX.Y.Z`.

## Current registry state

`@tramuri/typescript@0.1.0` is published. Its preferred command for the next version is
`tramuri-cli`; `tramuri-ts` remains as a compatibility alias.

Before staging a later version, commit the package, lockfile, and workflow changes; review the
archive with `npm pack --dry-run`; configure GitHub Actions Trusted Publishing for owner `tramuri`,
repo `tramuri`, workflow filename `npm-publish.yml`, and environment `npm-publish`; and protect the
GitHub environment `npm-publish` with maintainer approval. Permit **staged publishing only**. The
workflow uses OIDC and does not need an npm token.

## Later versions

1. Update `version` in `package.json` and add the corresponding section to this changelog.
2. Run `bun run ci` and `npm pack --dry-run` from this directory.
3. Commit the version and changelog update.
4. Create and push the matching package tag, for example `typescript-v0.1.2`.
5. The `npm-publish.yml` workflow validates the tag, tests and stages the archive using npm OIDC.
6. Review and approve the staged package in npm. Staging does not make it public until a maintainer
   approves it.

Do not create a tag until the corresponding version is committed. Never reuse a published npm
version. A workflow run is not proof that npm staging or maintainer approval succeeded; verify the
package page and provenance after the release.

## Development dependency security note

`package.json#overrides.minimatch` pins the development toolchain to `11.0.1` because the current
ESLint dependency ranges still resolve to the affected `10.2.6` release (GHSA-g3hv-75wg-m249).
Remove this override after the upstream ESLint dependency ranges accept a fixed release, then run
`bun audit` and the full CI checks again. This override applies to the development toolchain; the
published CLI has no runtime dependencies.
