<!-- tramuri:managed-reference:v1 -->

# Context and evidence packs

Context Packs and Evidence Packs are intended to carry selected, traceable information between work
and tools. Preserve source references and provenance; distinguish original evidence from summaries
and inferences.

Export is privacy-sensitive. Never export automatically. Require explicit selection of each item and
authorization for the export operation. Exclude source contents, credentials, `.env` values, keys,
and tokens. Keep human-authored statements excluded unless the user explicitly selects and authorizes
them. Apply safe defaults when authorization or sensitivity is unclear.

Mission context links are available separately from Context Packs. Use
`tramuri mission link-knowledge` or `tramuri mission link-capability` only when the user explicitly
asks to associate an item with a Mission. `tramuri resume --mission <id>` includes those links;
workspace `status` and generated reports do not export linked Knowledge text. Capability links refer
to the exact observed source hash and report changed, missing, or unverified freshness after scans.
Actor attribution is caller supplied, not authenticated. Never create links from name similarity,
directory proximity, inferred relevance, or agent guesses.

The public pack contracts and export operations are versioned/approval-gated work. Check the installed
CLI help and SDK documentation before relying on them; do not claim pack serialization or export is
available unless the installed version actually provides it.
