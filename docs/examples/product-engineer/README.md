# Example: Product Engineer

This tutorial takes a fictional idea to a local prototype of a SaaS online-course platform. The main
path is to copy one complete prompt into Codex. It gives the agent the product, users, MVP, and work
limits so you do not have to guide it through a string of questions. It also shows how to follow the
Mission, review the report, and hand the work off.

The example product is called **Aulae**. Its name, courses, people, and data are fictional. You do not
need Git, real interviews, payments, credentials, or deployment.

This tutorial requires Tramuri 0.10.0 or newer. Check `tramuri --version` after installation;
`uv tool install tramuri` installs the latest published version. If Tramuri is already installed,
use `uv tool upgrade tramuri` to update it.

## 1. Install and prepare the project

```console
uv tool install tramuri
tramuri --version
mkdir aulae-courses
cd aulae-courses
tramuri install --project --platform codex
```

The installer adds the instructions, skill, and Codex-supported hooks while preserving user-managed
files. The integration guides Codex to start or resume a Mission before using local project tools. It
does not run the product or store chat text.

Do not run `tramuri init` or `tramuri scan` for this empty example. When you send the prompt below,
Codex should initialize the local workspace along with the Mission. Tramuri works without Git.

## 2. Copy this prompt into Codex

Send the entire block as one message. It explicitly selects the **Product Engineer** perspective and
provides enough detail for the agent to frame and prototype the product. If the terminal still shows
the perspective selector, choose **Product Engineer** and press Enter. This choice describes the
work perspective; it does not identify who is performing the work.

```text
I want to build a fictional web product from scratch called Aulae: a multi-instructor SaaS platform
for online courses and students.

Work on this Mission from the Product Engineer perspective. Start or resume with Tramuri before using
other tools. Keep all work for this product in the same Mission; do not create another Mission for
the same product. Follow the Core's DEFINE workflow: request the next authorized step, load the skill
it returns, submit the result through the CLI, and advance only when it is accepted. Preserve what I
said and distinguish user-provided information, hypotheses, recommendations, approved decisions, and
open questions.

INITIAL PRODUCT CONTEXT

- Goal: let instructors publish courses and let students discover, subscribe to, and follow courses
  in one place.
- Problem to investigate: students may find it difficult to discover suitable courses and resume
  learning; instructors may rely on several tools to publish and track courses. Treat this as a
  hypothesis, not validated research.
- Initial working audience: adults in Brazil interested in practical and professional skills, and
  independent instructors who want to offer online courses. This is a prototype hypothesis, not a
  persona validated through interviews.
- Roles: student, instructor, and platform administrator.
- Student journey: discover and filter courses → view course details → simulate a subscription or
  enrollment → access lessons → mark a lesson complete and see progress.
- Instructor journey: create a draft course → add modules and lessons → submit it for review → see
  whether it was approved or needs changes.
- Administrator journey: review submitted courses and approve them or request changes. Keep this
  demonstrative; do not build real permissions or security.
- Working business model: recurring student access to a catalog with courses from multiple
  instructors. Use R$ 200/month only as an illustrative UI price, clearly labeled as fictional, not
  approved pricing. Instructor payouts and any commission remain undecided. Do not invent or record
  a financial decision.

LOCAL PROTOTYPE — SCOPE

1. Responsive course catalog in Brazilian Portuguese, with search, categories, and course cards.
2. Course details with a fictional instructor, description, level, duration, modules, and lessons.
3. Demonstration subscription/enrollment flow, with no checkout or charge, leading to the student
   area.
4. Student area with accessible courses, progress, and a button to continue a lesson.
5. Demonstration player with fictional content and an action to mark a lesson complete.
6. Simple instructor area to create/edit a fictional course and submit it for review.
7. Simple admin review queue to approve a course or request changes.
8. Use synthetic content, for example: “Excel for Small Businesses,” “Selling with WhatsApp,” and
   “UX Fundamentals.” Invent the names and instructors too.

OUT OF SCOPE

- Real payments, subscriptions, or instructor payouts.
- Real login, collection of personal data, or email delivery.
- Video hosting, external services, production backend, or deployment.
- Claims that real students or instructors validated the product.
- Final pricing, commission, or revenue split without my explicit decision.

SUCCESS CONTRACT FOR THIS TUTORIAL

- A person can search/filter the catalog and open a course.
- A simulated enrollment makes the course visible in the student area.
- A student can mark a fictional lesson complete and see progress change.
- An instructor can create a draft and submit it for review; the admin screen can change its
  demonstrative status.
- The interface works at common screen sizes and uses clear Brazilian Portuguese text.

For this exercise, I authorize these criteria as the Success Contract for the local demonstration.
They assess only the prototype; they do not prove demand, learning outcomes, or commercial success.
Do not change this contract without asking me or record the price or instructor payouts as approved
decisions.

HOW TO WORK

- Do the Product Engineering work available from this context before asking questions; do not turn
  the conversation into a questionnaire. During DEFINE, ask only if an answer would materially
  change product direction or prevent an important risk, and combine related questions. For
  reversible details, choose a clear hypothesis and continue. Do not repeat information already
  provided.
- Give short phase updates: what you are doing, what you produced, and what comes next. Do not reveal
  detailed private reasoning.
- Complete the Product Brief through Tramuri, then continue by building the local prototype in this
  directory; do not stop at the Brief or wait for me to ask what comes next.
- After the Brief is accepted, record the criteria above in the Mission's Success Contract. When the
  implementation is finished, record an Execution with its actual status and Evidence for the checks
  that ran. Do not claim an Outcome is verified just because code was written.
- Inspect existing files after starting the Mission and preserve them. This example starts empty, so
  do not scan without a reason. Use available creation tools; if a tool or MCP is unavailable, state
  the limitation and continue with a feasible local path without pretending the tool ran. Prefer the
  simplest available setup and do not add dependencies unnecessarily.
- Before finishing, run the app locally when possible, walk through the flows above, and run existing
  tests. Report exactly what was and was not verified. Do not deploy.
- As DEFINE steps are accepted, check Tramuri Status/report. At the end, tell me what was recorded,
  which files were created, which hypotheses remain open, which tests ran, and how to start the
  prototype.
```

## 3. Follow progress without directing every step

After sending the prompt:

1. If the perspective selector appears, choose **Product Engineer**.
2. Let Codex start/resume the Mission and run the authorized DEFINE steps. Core validates order and
   submitted structure; it does not validate the market, interviews, or truth of the hypotheses.
3. If the agent asks about something already in the prompt or that can be decided provisionally,
   reply:

   > Use a reversible hypothesis, label it as a hypothesis, and continue. Ask again only if it would
   > materially change the product or involve an important risk.

4. After the Brief, Codex should continue to the local prototype using available tools or a feasible
   local alternative. If a site-building integration fails, that alone does not prove a Tramuri bug;
   check whether the agent can continue using local resources.

You do not need to type `tramuri mission create`, assemble each step's JSON, or ask “what should I do
now?”. Codex requests each step contract through the CLI and submits its result to Tramuri. A step
counts as complete only after Core accepts it. A validated DEFINE means the Product Brief was
structured; it does not mean the SaaS was built or its business outcome verified.

## 4. Check the prototype criteria

After the Product Brief is accepted, check that its Success Contract contains the five criteria you
authorized in the prompt. The agent should record the contract revision before assessing results;
this explicit change creates a revision and preserves history.

When the prototype is ready, exercise the criteria yourself. Record only what you actually tested.
For example, if you searched for a course and opened it, you may assess that criterion as `met`; if
you did not check the admin review flow, leave it `unverified`. The reference below is only a local
label—Tramuri does not open or automatically validate it.

An Execution describes work performed; Evidence points to what was observed; Outcome is an explicit
assessment of the criteria. If evidence is insufficient, leave the result unverified.

## 5. Check the Mission and report

Codex should update the state as work progresses. To inspect it in a terminal:

```console
tramuri status
tramuri mission list
tramuri resume --mission <MISSION_ID>
```

Reports are stored at:

```text
.tramuri/reports/tramauri.html
.tramuri/reports/tramauri.md
.tramuri/reports/tramauri.json
```

Accepted CLI writes refresh report files on a best-effort basis. An already-open HTML tab is a static
page; use its refresh control or refresh the browser page to load the latest state. To open the
report in a browser at the end, run `tramuri status --open` if it has not already been opened in this
session. You do not need to reopen the window at every step.

## 6. Test the prototype and hand off

Follow the run instructions Codex leaves at the end. Manually test the catalog, course details,
simulated enrollment, lesson progress, and course review. Record as evidence only what you actually
observed. If you did not exercise a flow, leave its criterion unverified. Tramuri does not
automatically validate a demo reference.

To simulate a handoff, start another Codex session or ask another person to continue without giving
them this chat history. Ask them to resume the Mission with Tramuri and identify the objective,
decisions, hypotheses, open items, files, and tests. They can run:

```console
tramuri mission list
tramuri resume --mission <MISSION_ID>
tramuri status --open
```

The `Product Engineer` perspective label does not say who performed the work. `actor` is a declared
label, not an authenticated identity.

## 7. Manual alternative: record the contract, execution, and evidence

The main path asks Codex to create these records after the criteria are authorized and checks have
run. If you are operating manually, create `success-contract.json` with only the criteria you
approved:

```json
{
  "criteria": [
    {"description":"A person can search the catalog and open a course detail page","required":true,"evidence_expectation":"Flow exercised in the local prototype using fictional data"},
    {"description":"A simulated enrollment makes a course visible in the student area","required":true,"evidence_expectation":"Local demonstration of fictional enrollment"},
    {"description":"Completing a lesson updates student progress","required":true,"evidence_expectation":"Local demonstration of updated progress"},
    {"description":"An instructor can submit a draft course for admin review","required":true,"evidence_expectation":"Local demonstration of submission and status change"},
    {"description":"The interface works at common screen sizes and uses clear Brazilian Portuguese","required":true,"evidence_expectation":"Manual review at representative viewport sizes"}
  ]
}
```

Record the contract and inspect its revision and criterion IDs. Use those values in the following
commands:

```console
tramuri mission contract revise --mission <MISSION_ID> --criteria-file success-contract.json --json
tramuri mission show --mission <MISSION_ID> --json
tramuri execution record --mission <MISSION_ID> --contract-revision <CONTRACT_REVISION> --actor-kind <ACTOR_KIND> --actor "<actual executor label>" --work-perspective product_engineer --status completed --notes "Implemented the Aulae prototype and exercised the local flows"
tramuri evidence add --mission <MISSION_ID> --execution <EXECUTION_ID> --kind external_reference --locator "demo-local:aulae-v1" --json
```

Choose `ACTOR_KIND` and the label to match who actually performed the work; the label is not
authenticated. Use `completed` only if the Execution finished. Do not record a test as observed if
you did not run it. Assess each criterion with an ID returned by `mission show`:

```console
tramuri outcome assess --mission <MISSION_ID> --contract-revision <CONTRACT_REVISION> --criterion <CRITERION_ID> --result met --evidence <EVIDENCE_ID> --verifier-kind human --verifier "<human reviewer label>" --explanation "Describe the flow that was actually tested"
tramuri outcome show --mission <MISSION_ID>
```

`met` is only an example for a criterion you actually verified; do not copy it for every criterion
without testing. Use `unverified` for anything not proven. A recorded approval with a local reference
does not mean Tramuri validated the application, the evidence, or commercial success.

## 8. Manual alternative: inspect the workflow through the CLI

Use this if you are not using Codex. The CLI persists and validates the workflow, but does not
perform product analysis or generate LLM content on its own. For each step, prepare JSON according
to the contract returned by `next` and submit it; the technical reference explains the full flow.

```console
tramuri start --goal "Create a SaaS online-course platform for students and instructors" --perspective product_engineer --context "Fictional Aulae example; catalog, course details, simulated enrollment, student area, progress, and instructor course review; no real payments or deployment" --json
tramuri mission define status --mission <MISSION_ID> --json
tramuri mission define next --mission <MISSION_ID> --json
```

After the required steps are accepted, set prototyping as the activity and record the Discovery
uncertainty:

```console
tramuri mission workflow activity --mission <MISSION_ID> --activity prototype
tramuri readiness assess --mission <MISSION_ID> --activity prototype --state needs_review --gap "No validation with real students or instructors" --accept-uncertainty "R$ 200/month is only illustrative" --actor "Product Engineer"
tramuri status
tramuri resume --mission <MISSION_ID>
```

For the remaining Execution, Evidence, and Outcome operations, follow the examples in the previous
section and record only results you observed. Finishing code is not proof of commercial success.

See the [Product Engineering technical flow](../../reference/product-engineering-execution-flow.md)
for DEFINE contracts, step reviews, and persistence.
