# Example: Product Engineer

This tutorial follows a fictional greenfield product project: an app for neighbors to lend tools to
each other. It covers installation, setup, Mission, discovery, prototype, evidence, review, and
handoff. Git and real user data are not required. A completed execution does not mean the outcome is
verified.

The commands require Tramuri 0.5.0 or newer. Check `tramuri --version` after installation;
`uv tool install tramuri` installs the latest published release.

## 1. Install and set up

```console
uv tool install tramuri
tramuri --version
mkdir borrow-tools
cd borrow-tools
tramuri install --project --platform codex
```

The Codex integration preserves existing instructions. Its session hook checks database-file
presence only; the installed instructions and skills guide the workflow without reading Mission
data. A substantive natural-language request starts the exploratory Mission and initializes the
workspace automatically. Do not run `tramuri init` first or open an empty report.

## 2. Start with one sentence in Codex

After installing the integration, do not type `tramuri mission create`. Tell Codex what you want
to achieve in your own words. The assistant should start or resume the Mission before exploring files
or invoking another creation tool. It begins with a short Product framing update, then does useful
discovery before asking a question that would materially change the product:

> I have an idea for a new product: an app for neighbors to lend tools they rarely use. I want to
> start this project from scratch as a Product Engineer. Guide me from the idea and ask only what is
> essential.

The assistant should start the Mission with Tramuri, use the Product Engineering skill, and continue
without turning discovery into a questionnaire. It should separate user-provided facts, observations,
hypotheses, and unknowns. Tramuri preserves confirmed context; Codex helps turn the idea into software.
Answer in natural language and describe the work you did; you do not need to assemble commands. At
the end of the session, the assistant refreshes the local reports. It opens the HTML only if it has
not already opened it during that session; otherwise it refreshes without opening another window.

The following commands show the manual alternative and what gets recorded. Use them if you do not
have an assistant integration or prefer the terminal; you do not need to repeat this sequence in
Codex.

## Manual alternative: start with the goal and discover

The fictional goal is to create the first version of the app. Begin without criteria because the
product flow still needs discovery:

```console
tramuri start --goal "Build the first version of an app for neighbors to lend tools" --perspective product_engineer --json
```

Save the returned ID as `<MISSION_ID>` and record a known gap:

```console
tramuri mission workflow activity --mission <MISSION_ID> --activity discover
tramuri readiness assess --mission <MISSION_ID> --activity discover --state needs_review --gap "We have not learned which tools neighbors would be willing to lend" --actor "Product Engineer"
tramuri status
tramuri resume --mission <MISSION_ID>
```

After a fictional discovery conversation, record a decision and explicitly link it to the Mission:

```console
tramuri knowledge add --type decision --statement "Start with a tool catalog and lending requests; leave payments out of the first version" --human-input --json
tramuri mission link-knowledge --mission <MISSION_ID> --knowledge <KNOWLEDGE_ID> --actor "Product Engineer"
```

Use the first command's returned ID as `<KNOWLEDGE_ID>`. Tramuri does not infer associations.

## 3. Define the Success Contract

Create `success-contract.json` in the example directory:

```json
{
  "criteria": [
    {"description":"A person can browse available tools and request to borrow one","required":true,"evidence_expectation":"Local demonstration of the fictional request flow"},
    {"description":"A tool owner can approve or decline a request","required":true,"evidence_expectation":"Reviewed local demonstration of the response flow"}
  ]
}
```

Record the contract and inspect its current revision and criterion IDs:

```console
tramuri mission contract revise --mission <MISSION_ID> --criteria-file success-contract.json --json
tramuri mission show --mission <MISSION_ID> --json
```

Save `<CONTRACT_REVISION>`, `<CRITERION_1>`, and `<CRITERION_2>`. Reassess readiness after
revising the contract:

```console
tramuri readiness assess --mission <MISSION_ID> --activity prototype --state ready --criterion-applied "The initial request flow is defined" --criterion-applied "A local demo is selected for evaluation" --accept-uncertainty "The prototype has not been tried by real neighbors" --actor "Product Engineer"
```

`ready` requires a recorded check and cannot include unresolved gaps or blockers. Accepted
uncertainties remain visible; readiness is advisory and does not block work.

## 4. Record execution and evidence

After the synthetic experiment, record the execution and a fictional reference. Save the returned
IDs as `<EXECUTION_ID>` and `<EVIDENCE_ID>`:

```console
tramuri execution record --mission <MISSION_ID> --contract-revision <CONTRACT_REVISION> --actor-kind human --actor "Product Engineer" --work-perspective product_engineer --status completed --notes "Implemented and demonstrated the fictional local prototype" --json
tramuri evidence add --mission <MISSION_ID> --execution <EXECUTION_ID> --kind external_reference --locator "fictional-demo:borrow-tools-v1" --json
tramuri execution list --mission <MISSION_ID>
tramuri evidence list --mission <MISSION_ID>
```

The locator is a reference: Tramuri does not open URLs or read files to validate it. Assess criteria
only after reviewing their evidence:

```console
tramuri outcome assess --mission <MISSION_ID> --contract-revision <CONTRACT_REVISION> --criterion <CRITERION_1> --result met --evidence <EVIDENCE_ID> --verifier-kind human --verifier "Fictional prototype reviewer" --explanation "The flow was demonstrated locally with fabricated data"
tramuri outcome assess --mission <MISSION_ID> --contract-revision <CONTRACT_REVISION> --criterion <CRITERION_2> --result unverified --verifier-kind human --verifier "Fictional prototype reviewer" --explanation "The response to a request has not been demonstrated"
tramuri outcome show --mission <MISSION_ID>
```

The Outcome remains pending while a required criterion is unverified.

## 5. Handoff and report

In an independent session, ask another person to reconstruct the work without receiving this
simulation history:

```console
tramuri mission list
tramuri resume --mission <MISSION_ID>
tramuri status --open
```

Ask them to identify the product idea, linked decision, execution, evidence, and pending criterion. The
report is written to `.tramuri/reports/tramauri.html`, with Markdown and JSON versions beside it.
It does not change the Outcome. The `actor` label does not authenticate a person.
