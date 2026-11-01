---
name: tramuri-integrity
description: Record or review Tramuri Knowledge, provenance, freshness, evidence, and verified outcomes safely.
---

<!-- tramuri:managed-skill:v1 -->

# Preserve context and evidence integrity

Use this skill when recording decisions or Knowledge, associating context with Missions, capturing
evidence, or assessing whether a Mission's Success Contract was met.

- Preserve the distinction between user-provided statements, repository-derived facts, inferences,
  and unanswered questions. Include source provenance where available.
- Do not infer Mission associations from names, paths, or semantic similarity. Use explicit links
  only with user authorization.
- Treat locators as references. Do not fetch URLs, open referenced files, or export their contents
  merely to retrieve stored Knowledge or Evidence.
- Never copy credentials, secrets, customer data, or private conversation content into records or
  generated reports.
- Recording an Execution as completed does not verify an Outcome. Assess each current Success
  Contract criterion against reviewed evidence; leave it unverified when the evidence is insufficient.
- Do not mark a criterion met or claim a Verified Outcome without the responsible person's or
  authorized mechanism's actual review.
- Before recording a `human` review on behalf of an agent, require explicit authorization from the
  responsible person for that review. `verifier_reference` is a declared label, not authentication.
- After a coherent task has written or assessed Mission records, refresh reports once at the end.
  Use `tramuri status --open` if the browser has not been opened in this session and the user has not
  asked to skip it; otherwise use `tramuri status --refresh`. The latter updates the same local files
  without reopening the browser. Do not refresh after every individual write or merely to inspect
  status. Tell the user that reports are local projections; refreshing them does not change Mission
  or Outcome state.

For provenance, freshness, explicit context selection, and export safety, read the installed
Tramuri skill's `references/knowledge.md` and `references/context-and-evidence.md` as relevant.
