# Tramuri report design system

This specification defines the shared visual and content rules for Tramuri's
generated product reports. Reports are local product artifacts: they explain
the recorded run and remain usable without a server or network connection.

## Contents

- [Visual tokens](#visual-tokens)
- [Report structure](#report-structure)
- [Reusable templates](#reusable-templates)
- [Evidence and visualization](#evidence-and-visualization)
- [Accessibility and delivery](#accessibility-and-delivery)
- [Review checklist](#review-checklist)

## Visual tokens

The visual system uses Tramuri's existing identity. The deep teal anchors the
shell and headings; turquoise marks verified product structure; orange marks
attention or unresolved validation. Secondary signal colors are reserved for
meaningful states and remain paired with text labels.

| Token | Value | Use |
| --- | --- | --- |
| `--brand-deep` | `#0B2530` | Brand frame, strong contrast, primary shell |
| `--brand-teal` | `#00B3A6` | Active workflow and verified structure |
| `--brand-petrol` | `#006D6A` | Accessible links and text accents |
| `--brand-aqua` | `#4FD6C7` | Light emphasis and focus-adjacent accents |
| `--brand-orange` | `#FFA229` | Attention, pending work and decision highlights |
| `--brand-mint` | `#A8EFE1` | Selection and soft positive surfaces |

Use a system sans-serif for dense reading and a system serif for report titles
and major section headings. Do not fetch fonts or icons from a CDN. Keep the
real Tramuri logo embedded in the generated document. Use a quiet canvas,
generous reading width, clear section rhythm and hairline dividers; avoid
repeated metric-card grids, decorative gradients and dashboard filler.

The light and dark themes share the same semantic roles. Keep body copy at a
readable size and maintain at least 4.5:1 contrast for ordinary text. Every
status uses a text label and a shape or symbol in addition to color.

## Report structure

Use the following story unless a workflow has a justified, documented need for
a different order:

1. **Overview** — product name and intent, run state, persisted progress, source
   count, recorded decisions, and the next action.
2. **PDLC journey** — actual stage order, status, owner, attempts and persisted
   dependencies.
3. **Product intelligence** — problem, audience, needs, value and discovery
   gaps, with uncertainty explicit.
4. **Market context** — only imported, source-backed research; absent research
   has an honest empty state.
5. **Hypotheses and scope** — risks, tests, success signals, included and
   excluded capabilities.
6. **Decision and evidence map** — recorded links with a text equivalent and
   links back to the records.
7. **Execution and outcomes** — validations, checkpoints, sensor results,
   review findings, risks, events and generated artifacts.
8. **Product experience** — links to generated demo surfaces and a clear
   statement that synthetic experiences are not production software.

Preserve the source language and wording of product-authored content. The
English and Brazilian Portuguese switch translates report controls and
explanatory UI; it does not rewrite stored product claims.

## Reusable templates

The shared Jinja presentation layer lives in
`src/tramuri/reports/templates/`:

- `report_base.html` owns document metadata, brand shell, navigation, controls,
  logo and common layout.
- `report_components.html` owns repeated state labels, section headings and
  empty states.
- `report.css` owns all brand tokens, components, responsive layouts, themes,
  reduced-motion behavior and print rules.
- `report.js` owns the local language/theme controls, current-section cue and
  print action. It has no network or backend dependency.
- `report.html` owns the product report content and data bindings.

Keep rendering in `ReportGenerator`; add presentation-only projections beside
it instead of copying business rules into templates. Keep generated output
self-contained. Preserve report paths, archive behavior and `generate()` return
keys when adding another report type.

## Evidence and visualization

- Render only values present in persisted run details or explicitly imported
  evidence packs. A missing value is an empty state, not a guessed number.
- Keep facts, evidence, inferences, hypotheses, assumptions, decisions and
  pending validation visually distinguishable when the source class exists.
- Draw graph edges only from stored stage dependencies, artifact lineage or
  exact evidence references. Do not infer a semantic relationship because two
  records share a topic or appear next to each other.
- Give every relationship graphic a navigable text equivalent. Unlinked
  records stay visible as independent records.
- Use charts only when actual quantitative data makes a comparison clearer
  than a labeled table. Include units, source, time context and a text/table
  alternative.
- Permit clickable external links only for absolute HTTP or HTTPS URLs. Render
  other source strings as escaped text.

## Accessibility and delivery

- Use semantic landmarks, a skip link, hierarchical headings and visible
  `:focus-visible` states.
- Give controls accessible names and use buttons for actions and anchors for
  navigation. Make scrollable visualizations keyboard reachable.
- Support narrow screens without page-level horizontal overflow. Tables and
  the relationship map may scroll within their own labeled containers.
- Respect `prefers-reduced-motion`; avoid decorative movement.
- Keep print rules readable and hide interactive chrome. Validate browser print
  and PDF output separately from screen layout.
- Escape all stored content, sanitize link protocols and avoid rendering event
  payloads or other data that may contain secrets.
- Keep CSS, JavaScript, logo and icons embedded; no Node, hosted service or
  CDN is required to read the report.

## Review checklist

- [ ] Real Tramuri identity and logo are used; no new brand palette is invented.
- [ ] Overview and journey reflect stored state and are not simulated.
- [ ] Hypotheses, evidence, decisions, risks and open validation are easy to
  distinguish and trace.
- [ ] Graph links have a persisted source; disconnected records remain clear.
- [ ] English and Brazilian Portuguese controls work; source claims are kept
  intact.
- [ ] Light/dark, keyboard focus, reduced motion, mobile and print are checked.
- [ ] Desktop and mobile screenshots were inspected after browser rendering.
- [ ] External skills remain in the harness and are absent from package assets.
- [ ] Relevant Python tests, browser checks and security checks have fresh
  evidence; unavailable tools are marked `not_run`.
