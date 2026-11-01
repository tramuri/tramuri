<!-- tramuri:managed-reference:v1 -->

# Missions and verified outcomes

A Mission states the objective and its Success Contract. It may concern Product Engineering,
Forward Deployed Engineering, product work, or another outcome, and does not require a repository.

Keep these concepts distinct:

1. **Mission state** describes the work's overall lifecycle.
2. **Execution state** records whether a person or tool carried out an attempt.
3. **Evidence** records supporting observations and references, with provenance.
4. **Criterion assessment** evaluates one Success Contract criterion against evidence. Record the
   assessment result, evidence, assessor or verification mechanism, and date when supported.
5. **Verified Outcome** is the result of evaluating the contract's criteria; it is not implied by a
   completed execution.

Do not claim business success solely because a command, deployment, or execution completed. Distinguish
technical checks from acceptance criteria and human or external validation. Report missing, stale, or
conflicting evidence and leave the outcome unverified when appropriate.

Use only operations supported by the installed Tramuri version. Do not invent a generic rule engine
or automatic orchestration behavior.

## Create and review a Mission

Missions require an initialized Tramuri workspace, but do not require Git metadata or a discovery
scan. Define at least one required criterion in the Success Contract:

```json
{
  "criteria": [
    {
      "description": "The customer can complete onboarding",
      "required": true,
      "evidence_expectation": "Customer acceptance or observed completion"
    }
  ]
}
```

Create the Mission from the CLI:

```text
tramuri init .
tramuri mission create --objective "Validate customer onboarding" --criteria-file criteria.json --json
tramuri mission show --mission <mission-id> --json
tramuri execution record --mission <mission-id> --contract-revision 1 --actor-kind human --status completed --json
tramuri evidence add --mission <mission-id> --execution <execution-id> --kind human_attestation --locator customer-approval-record --json
tramuri outcome assess --mission <mission-id> --contract-revision 1 --criterion <criterion-id> --result met --evidence <evidence-id> --verifier-kind human --verifier product-owner --json
tramuri outcome show --mission <mission-id> --json
```

Or create it through the Python SDK:

```python
from pathlib import Path

import tramuri

workspace = Path(".")
tramuri.init(workspace)
mission = tramuri.mission_create(
    workspace,
    objective="Validate customer onboarding",
    criteria=[
        {
            "description": "The customer can complete onboarding",
            "required": True,
            "evidence_expectation": "Customer acceptance or observed completion",
        }
    ],
)
```

An execution and its evidence are separate records. For example, recording `completed` does not
assess the Success Contract:

```python
execution = tramuri.execution_record(
    workspace,
    mission_id=mission["id"],
    contract_revision=mission["success_contract"]["revision"],
    actor_kind="human",
    state="completed",
)
evidence = tramuri.evidence_add(
    workspace,
    mission_id=mission["id"],
    execution_id=execution["id"],
    kind="human_attestation",
    locator="customer-approval-record",
)
criterion_id = mission["success_contract"]["criteria"][0]["id"]
tramuri.outcome_assess(
    workspace,
    mission_id=mission["id"],
    contract_revision=mission["success_contract"]["revision"],
    criterion_id=criterion_id,
    result="met",
    evidence_ids=[evidence["id"]],
    verifier_kind="human",
    verifier_reference="product-owner",
)
outcome = tramuri.outcome_get(workspace, mission_id=mission["id"])
```

Only assess a criterion as `met` when its evidence supports the acceptance criterion and the
identified person or verification mechanism has actually reviewed it. Use the CLI equivalents
`tramuri execution record --help`, `tramuri evidence add --help`, `tramuri outcome assess --help`,
and `tramuri outcome show --help` for the installed version's exact flags. Evidence locators are
references: Tramuri does not fetch URLs or copy file contents.
