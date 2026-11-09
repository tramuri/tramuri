---
name: tramuri-status
description: Explain the real persisted state of a Tramuri product workflow.
---

# Product Status

For a cross-session progress question, first load `tramuri-memory` and read
`tramuri memory show`. Then read `tramuri status` and the latest checkpoint
before continuing. Distinguish
completed, skipped, not-run, blocked, failed, browser-not-run and pending
validation. Never turn a generated file or a subagent suggestion into a success
claim without a persisted validation receipt.

## When to use
Answer a progress question, orient a new provider session or decide how to
resume an interrupted Tramuri run.

## When not to use
Do not infer progress from conversation history or generated files alone, and do
not restart a workflow for a status question.

## Inputs
Persisted Tramuri status, run history, latest checkpoint, artifacts and
validation receipts.

## Process
Read current state, summarize completed, active, skipped, blocked, failed and
pending stages, then identify the next valid action or resume command.

## Output contract
Return the run identifier, actual persisted stage states, validated artifacts,
limitations and one actionable next step.

## Quality checklist
- [ ] Status comes from persisted state.
- [ ] Generated and validated artifacts are distinguished.
- [ ] Failed, skipped and not-run work remains visible.
- [ ] The next step matches the workflow's real dependencies.
