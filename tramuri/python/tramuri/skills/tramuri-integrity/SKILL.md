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
- After a coherent task has written or assessed Mission records, refresh and open the local report
  once with `tramuri status --open`, unless the user asked not to. Do not run it after every
  individual write or merely to inspect status. Tell the user that the report is local and derived;
  it does not change Mission or Outcome state.

For provenance, freshness, explicit context selection, and export safety, read the installed
Tramuri skill's `references/knowledge.md` and `references/context-and-evidence.md` as relevant.
