---
name: tramuri-navigation
description: Turn the defined core journey into a scoped offline navigation prototype.
---

## When to use
Create a lightweight, clickable model of the primary journey after product scope
and key user decisions are defined.

## When not to use
Do not create a production application, unsupported screens or a full experience
when the request is only a product definition.

## Inputs
In-scope capabilities, primary user journey, product domain, states and offline
artifact constraints.

## Process
Choose only screens required to demonstrate the main journey. Use semantic HTML,
responsive CSS and local JavaScript with visibly synthetic data; make controls
and links produce observable behavior without a backend.

## Output contract
Return a locally navigable prototype, screen-to-scope mapping and known simulation
limits.

## Quality checklist
- [ ] Every screen is traceable to an in-scope capability.
- [ ] Main journey can be completed with working controls.
- [ ] Synthetic data and simulated integrations are labeled.
- [ ] The prototype works offline and has basic accessible semantics.
