# Example: Forward Deployed Engineer

This tutorial simulates a fictional enterprise integration without Git or source code. You will
install Tramuri, record requirements and constraints, simulate validation, and hand the work off to
another FDE. Do not use credentials, real systems, or customer information.

The commands require Tramuri 0.5.0 or newer. Check `tramuri --version` after installation;
`uv tool install tramuri` installs the latest published release.

## 1. Install and prepare a workspace without Git

```console
uv tool install tramuri
tramuri --version
mkdir northstar-integration
cd northstar-integration
tramuri install --project --platform codex
tramuri init .
tramuri status --open
```

Agent integration is optional. The installer preserves existing instructions and installs native
Codex tool hooks. Local project tools are blocked until `tramuri start` or `tramuri resume` succeeds
for each turn. If hooks are disabled or bypassed, instructions are guidance only. No prompt or
Mission data is read or stored by the gate.

## 2. Start with one sentence in Codex

After installation, do not type `tramuri mission create`. Tell Codex what you need to do:

> I am working as an FDE and want to simulate an order-event integration for the fictional company
> Northstar. Use fabricated data only, with no Git and no system connections. Start an exploratory
> Mission and guide me through discovery, asking only what is needed.

The assistant should create the exploratory Mission, record requirements and constraints you
confirm, and guide the next steps in natural language. You do not need to enter commands manually.
To demonstrate a handoff, ask Codex to open the report at the end, then have another person resume
the Mission in an independent session.

The next sections show the equivalent manual flow. Use them without an assistant integration or if
you prefer the terminal; do not repeat the command sequence in Codex.

## Manual alternative: create the Mission and record discovery

“Northstar” is a fictional company that wants to send synthetic order events to a test endpoint.
Start without criteria because the details are not known yet:

```console
tramuri mission create --objective "Validate synthetic order-event integration for Northstar" --context "Fictional scenario; no production systems or credentials" --json
```

Save the ID as `<MISSION_ID>` and record known gaps:

```console
tramuri readiness assess --mission <MISSION_ID> --activity discover --state needs_review --gap "The event fields are not confirmed" --gap "The retention window is not agreed" --actor "FDE 1"
```

Record only synthetic Knowledge with human-input provenance:

```console
tramuri knowledge add --type fact --statement "Synthetic events contain order_id, status, and updated_at" --human-input --json
tramuri knowledge add --type fact --statement "The fictional customer prohibits personal data in the test payload" --human-input --json
tramuri knowledge add --type question --statement "Who approves the synthetic acceptance test?" --human-input --json
```

Use the returned IDs to explicitly link each relevant record:

```console
tramuri mission link-knowledge --mission <MISSION_ID> --knowledge <REQUIREMENT_ID> --actor "FDE 1"
tramuri mission link-knowledge --mission <MISSION_ID> --knowledge <RESTRICTION_ID> --actor "FDE 1"
tramuri mission link-knowledge --mission <MISSION_ID> --knowledge <QUESTION_ID> --actor "FDE 1"
```

## 3. Define acceptance criteria

Create `success-contract.json` in the workspace:

```json
{
  "criteria": [
    {"description":"The test endpoint receives the three agreed synthetic fields","required":true,"evidence_expectation":"Synthetic endpoint test record"},
    {"description":"No personal data is sent in the test payload","required":true,"evidence_expectation":"Reviewed synthetic payload check"},
    {"description":"The customer-side owner accepts the test outcome","required":true,"evidence_expectation":"Synthetic human acceptance reference"}
  ]
}
```

Revise the contract and retrieve the criterion IDs:

```console
tramuri mission contract revise --mission <MISSION_ID> --criteria-file success-contract.json --json
tramuri mission show --mission <MISSION_ID> --json
```

Save `<CONTRACT_REVISION>` and `<CRITERION_1>`, `<CRITERION_2>`, and `<CRITERION_3>`. Assess
readiness for the next activity. If a critical blocker exists, record `blocked` instead of hiding
it as uncertainty:

```console
tramuri readiness assess --mission <MISSION_ID> --activity prototype --state ready --criterion-applied "Fields and privacy restriction are recorded" --accept-uncertainty "Customer approval contact is being confirmed" --actor "FDE 1"
```

Readiness is advisory; it does not replace security approval or authorize access to systems.

## 4. Simulate the integration and review evidence

Run a local validation manually, without connecting to services, and record the execution:

```console
tramuri execution record --mission <MISSION_ID> --contract-revision <CONTRACT_REVISION> --actor-kind human --actor "FDE 1" --work-perspective fde --status completed --notes "Mapped synthetic fields and ran a local payload check" --json
```

Save `<EXECUTION_ID>` and add a fictional reference. Tramuri does not open or validate the
locator's contents:

```console
tramuri evidence add --mission <MISSION_ID> --execution <EXECUTION_ID> --kind external_reference --locator "synthetic-test-record:northstar-run-01" --json
```

Save `<EVIDENCE_ID>`. Assess the two demonstrated criteria and leave acceptance pending:

```console
tramuri outcome assess --mission <MISSION_ID> --contract-revision <CONTRACT_REVISION> --criterion <CRITERION_1> --result met --evidence <EVIDENCE_ID> --verifier-kind mechanism --verifier "Synthetic payload test"
tramuri outcome assess --mission <MISSION_ID> --contract-revision <CONTRACT_REVISION> --criterion <CRITERION_2> --result met --evidence <EVIDENCE_ID> --verifier-kind human --verifier "Synthetic privacy reviewer"
tramuri outcome assess --mission <MISSION_ID> --contract-revision <CONTRACT_REVISION> --criterion <CRITERION_3> --result unverified --verifier-kind human --verifier "Synthetic customer owner" --explanation "No acceptance has been recorded"
tramuri outcome show --mission <MISSION_ID>
```

The Outcome should remain pending: a completed execution is not customer acceptance.

## 5. Hand off to another FDE

In an independent session, ask another FDE to reconstruct the state through the public interfaces:

```console
tramuri mission list
tramuri resume --mission <MISSION_ID>
tramuri knowledge list
tramuri execution list --mission <MISSION_ID>
tramuri evidence list --mission <MISSION_ID>
tramuri outcome show --mission <MISSION_ID>
tramuri status --open
```

The second FDE should find the requirements, privacy restriction, execution, evidence, and pending
acceptance without depending on an oral history from the first. `actor` is a declared label, not
authenticated identity. Reports are `.tramuri/reports/tramauri.html`, `tramauri.md`, and
`tramauri.json`; they are projections and do not change or verify the Outcome.
