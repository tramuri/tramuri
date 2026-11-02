# Product Engineering execution flow in Tramuri

This diagram shows how Tramuri organizes a Product Engineering Mission from intent through delivery
and learning. It separates local Tramuri operations from work performed by the host agent and its
tools.

```mermaid
flowchart TD
    A[User describes an objective] --> B{Tramuri integration active<br/>and native hook enabled?}
    B -- Yes --> C[Hook locks the gate at turn start]
    B -- No --> C2[Skill and instructions guide the flow<br/>without guaranteed enforcement]
    C --> D[Direct preflight: version, status, Mission list]
    C2 --> D
    D --> E{An open Mission clearly matches?}
    E -- Yes --> F[tramuri resume --mission ID<br/>preserve recorded perspective]
    E -- No --> G[Require explicit choice: Product Engineer, FDE, or mixed]
    E -- Ambiguous --> H[Ask which existing Mission to resume]
    H --> D
    G --> G1[TTY selector: arrows + Enter;<br/>agent asks in chat]
    G1 --> G2[tramuri start --goal objective<br/>--perspective selected-value]
    F --> I{CLI succeeded?}
    G2 --> I
    I -- No --> J[Gate remains locked; fix selection or clarify]
    J --> D
    I -- Yes --> K[Hookable local tools are unlocked for this turn]

    subgraph DEFINE[Start · deterministic DEFINE]
      K --> L{Product Engineer perspective?}
      L -- Yes --> M[Core initializes persistent five-step DEFINE]
      M --> N[Host calls mission define next]
      N --> O[Core authorizes one step and returns contract + skill]
      O --> P[Host loads skill and produces structured JSON]
      P --> Q[mission define submit validates structure and references]
      Q --> R{Accepted by Core?}
      R -- No --> S[Remain on current step; correct and resubmit]
      S --> N
      R -- Yes --> T{DEFINE complete?}
      T -- No --> N
      T -- Yes --> U[Product Brief is persisted;<br/>Mission Outcome remains separate]
      L -- FDE / other --> V[Existing Mission lifecycle;<br/>no mandatory PE workflow]
    end

    subgraph BUILD[Middle · build]
      U --> W{User requested build or another artifact?}
      W -- No --> X[Show Product Brief, assumptions,<br/>gaps and candidate criteria]
      W -- Yes --> Y[Host uses available tools<br/>to create a prototype or product]
      Y --> Z[Run appropriate tests<br/>in the project environment]
      Z --> AA[Record execution and observed results<br/>through Tramuri interfaces]
      AA --> AB[Attach evidence and provenance;<br/>do not claim success without observed results]
    end

    subgraph SHIP[End · ship, verify and learn]
      X --> AC{Enough criteria and evidence<br/>to assess the result?}
      AB --> AC
      AC -- No --> AD[Keep gaps explicit;<br/>record next step or handoff]
      AC -- Yes --> AE[Assess each criterion<br/>with its verifier and evidence]
      AE --> AF{Aggregate outcome verified?}
      AF -- No --> AD
      AF -- Yes --> AG[Record verified outcome<br/>with evidence limitations]
      AD --> AH[Update authorized focus and records]
      AG --> AH
      AH --> AI[Refresh tramauri.html, .md and .json<br/>without reopening an already-open browser]
      AI --> AJ[Session summary: changes, pending work,<br/>result and concrete next action]
      AJ --> AK[Later session: preflight and<br/>tramuri resume --mission ID]
      AK --> D
    end
```

## What the flow means

### Start

- The host checks existing Tramuri state and Missions before inspecting files or using other work
  tools when a supported, trusted native hook is active. Instructions are guidance when hooks are
  unavailable, disabled, or bypassed.
- `status` and `mission list` are preflight queries; they do not create/select a Mission or unlock
  the hook gate. Opening a supported older database may apply its existing additive migrations.
- A new Mission requires an explicit choice among Product Engineer, FDE, or mixed. A terminal start
  presents a three-option selector; an agent asks in chat and passes the selected value. Defaults and
  intent inference can suggest but cannot choose. A unique exact open Mission match resumes with its
  stored perspective. Product Engineer `tramuri start` also initializes persistent DEFINE in the Core.
  A complete Success Contract is not required.
- The host calls `tramuri mission define next --mission <id> --json`, loads the returned skill,
  produces structured JSON, and submits it. The Core rejects out-of-order steps and does not advance
  after rejection. CLI transitions refresh local reports best-effort without reopening the browser.
- The five steps are `objective_context`, `business_discovery`, `requirements`,
  `knowledge_synthesis`, and `product_brief`. The Core checks structure and artifact references; it
  cannot prove market claims are true or verify that the host actually used a skill.

### Build

- Skills guide the cognitive work; the Core owns sequence, structural acceptance, persistence, and
  progress. `status` and `next` restore persisted state.
- The CLI records work; it does not call Codex, Claude Code, Cursor, Gemini CLI, or Copilot, build a
  product, or run its tests.
- If a preferred host tool is unavailable, the host should report that and use a viable local
  alternative when available.

### Ship

- A test is reported as executed only after a tool runs it and the result is observed. Recording a
  reference does not validate an external system automatically.
- `Execution Completed`, evidence recorded, and `Outcome Verified` are distinct states.
- Completing DEFINE stores a Product Brief; it does not start BUILD, create code/tests, or verify a
  business outcome.
- Reports are projections and can be regenerated from SQLite. Direct Python SDK workflow calls do
  not refresh CLI reports automatically; run `tramuri status --refresh` afterward.

## Current limits

This combines deterministic Core/CLI state transitions with behavior performed by a host model.
The hook only gates supported local tools within each platform's documented boundary. It cannot
guarantee product quality, skill use, external research, tool availability, test execution, deploy,
or business validation. Those require real host-runtime tests and Product Engineer pilots.
