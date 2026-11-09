# ADR-0006: Curated project memory and individually discoverable skills

## Status

Accepted

## Context

Tramuri already persists workflow state, checkpoints, decisions, evidence and
knowledge in the project-local SQLite database. The provider entry skill can
inspect workflow status, but there is no focused capability for answering
questions such as "what do you remember?" or "what were we doing?" The
database is also difficult for people to inspect directly. In the current
checkout, the provider installation plan contains focused product skills, but
the Codex repository skill directory is empty until those components are
installed.

## Decision drivers

- Keep the SQLite database as the canonical source of workflow and project
  memory.
- Make useful saved facts and current work visible under Tramuri's existing
  output root without storing full conversation transcripts.
- Make recall and progress questions retrieve persisted state before answering.
- Let users save explicit, durable preferences and decisions with provenance.
- Expose Tramuri's focused product skills through Codex's native project skill
  directory.
- Preserve existing database schema, output paths, and offline operation.

## Considered options

### Option 1: Keep memory only in SQLite

This preserves a single source of truth but does not provide the requested
human-readable memory files or a focused recall workflow.

### Option 2: Make Markdown files the canonical store

This makes files easy to browse but duplicates workflow state, creates
consistency rules between files and SQLite, and risks losing status fidelity.

### Option 3: Keep SQLite canonical and generate a Markdown projection

This makes memory inspectable while keeping updates and workflow status in the
existing database. Files can be regenerated after a checkpoint or memory save.

## Decision

Use SQLite as the canonical source. Add `tramuri memory show`, `tramuri memory
search`, and `tramuri memory save`. The save command records a short,
provenance-labelled project knowledge entry in SQLite. The runtime generates a
readable projection under `tramuri_output/memorys/`, including current work,
project memories, and per-run summaries. It does not copy raw conversation
transcripts or credentials into memory files.

Add a `product-memory` skill. It handles recall, cross-session context, explicit
memory requests and current-work questions. It reads `tramuri memory show`,
searches relevant records, checks `tramuri status` for active workflow state,
and saves only durable, useful information. When the Codex installation is
updated, Tramuri product skills remain individually discoverable in
`.agents/skills/` alongside the `$tramuri` entry skill.

This decision supersedes ADR-0005 only where ADR-0005 rejected a second
project-memory file. The Markdown files are generated views, not a second source
of truth. ADR-0005's capability-based skill selection and no-persona decision
remain in force.

## Consequences

### Positive

- Users can inspect project memory with ordinary tools and ask Codex to retrieve
  a topic or resume from the recorded next action.
- Explicit user memories retain their classification, source reference and
  update history in SQLite.
- Current progress snapshots are refreshed after verified workflow
  checkpoints.
- No migration or new runtime dependency is required.

### Negative

- The memory projection must remain bounded, redacted and regenerated from
  SQLite.
- Skill files must be installed into the active repository before Codex can
  discover them there; Codex may need to restart after installation.

## Compatibility and recovery

The CLI additions are additive. Existing databases already have the required
knowledge and supersession fields. Removing `tramuri_output/memorys/` is safe;
the files can be regenerated from SQLite by `tramuri memory show` or the next
checkpoint. Existing output directories and workflow commands keep their
current names.
