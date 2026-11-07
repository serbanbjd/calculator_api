# Calculator API

A small calculator HTTP API built with **AI spec-driven development**: every line of
code traces back to a written spec (PRD → ADR → plan/tasks) that an AI agent drafted
and a human reviewed.

> Status: specs accepted, code complete (see [docs/PROGRESS.md](docs/PROGRESS.md)).

## Run

Needs Rust stable (pinned in `rust-toolchain.toml`).

```sh
cargo test                       # unit, property and API tests (~1 s)
cargo run                        # serves on 0.0.0.0:8080, or $PORT
RUST_LOG=debug cargo run         # more logging
docker build -t calculator-api . # container image, see docs/DEPLOYMENT.md
```

```sh
curl -s localhost:8080/v1/calculate -H 'content-type: application/json' \
  -d '{"operation": "divide", "operands": [1, "3"]}'
# {"result":"0.3333333333333333333333333333"}

curl -s localhost:8080/v1/calculate -H 'content-type: application/json' \
  -d '{"operation": "divide", "operands": [1, 0]}'
# {"error":{"code":"DIVISION_BY_ZERO","message":"division by zero"}}   (HTTP 422)
```

Operations: `add`, `subtract`, `multiply`, `divide`, `power`, `modulo`,
`percentage` (two operands) and `sqrt` (one). Full contract:
[docs/api/openapi.yaml](docs/api/openapi.yaml), also served at `/openapi.yaml`, with
Swagger UI at [`/docs`](https://calculator-api-production-b82f.up.railway.app/docs).

## Repository layout

| Path | What lives there |
|------|------------------|
| `CLAUDE.md` / `AGENTS.md` | How the AI agent must work in this repo |
| `.claude/settings.json` | Agent plugins (superpowers, ponytail) |
| `.claude/skills/` | Project-specific agent skills for the spec-driven workflow |
| `docs/prd/` | Product requirements: *what* the calculator does |
| `docs/adr/` | Architecture decision records: *how* and *why* |
| `docs/plan/` | Implementation plan, task list, UML diagrams |
| `docs/api/` | OpenAPI contract with executable examples |
| `docs/reviews/` | Agent self-reviews of each spec, with findings and fixes |
| `src/calc.rs` | Pure maths domain (exact decimals) |
| `src/api.rs` | HTTP layer (axum) |
| `tests/api.rs` | API tests through the real router |
| `docs/DEPLOYMENT.md`, `Dockerfile`, `railway.toml` | Container build and Railway hosting |

## Workflow

1. **PRD** — what the system does and, just as important, what it does not.
2. **ADR** — architecture decisions with context, options and consequences.
3. **Plan & tasks** — small, testable tasks with a UML diagram of the design.
4. **Code** — generated task by task, test first, against the docs above.

Each step was a pull request that Serban reviewed and merged before the next
one started (PRs #1, #3, #4, #5), and each spec has an agent self-review in
[docs/reviews/](docs/reviews/). Progress is tracked in
[docs/PROGRESS.md](docs/PROGRESS.md).

## Questions from the brief

### How I defined the boundaries of the system

I started from what the service must *not* do. The PRD
([section 3](docs/prd/PRD.md)) draws the boundary as one stateless HTTP service:
it receives a request, computes, answers, stores nothing and calls nothing. A
context diagram shows the only actors (API client, operator), and an
out-of-scope table lists what was left out *and why*: expression parsing,
history, auth and rate limiting (a gateway's job), arbitrary precision, UI.

The second boundary is inside the code (ADR-0004): `calc` is pure maths with no
HTTP or JSON, `api` translates HTTP into `calc` calls. The rule is that every
input crosses one validation point (`parse_operand`), so nothing unchecked
reaches the maths.

### How I gathered the PRD

With the `write-prd` skill, which runs superpowers `brainstorming`: the agent
asks one question at a time, prefers multiple choice, and records every answer
or assumption in the PRD's open-questions table instead of inventing it. In
practice: scope was a three-option decision (core four / core four + extras /
expression parsing), and three follow-up questions settled the details that
change behaviour: exact decimals (`0.1 + 0.2 = 0.3`), modulo sign, and `-0`.
Every requirement had to come with concrete input/output examples, which later
became the unit tests one-to-one.

The PRD was not frozen at acceptance. Twice the implementation showed it could
not be met as written (amendments A1 and A2 in the PRD change log); both times
the document was changed first, reviewed in a PR, then the code followed.

### Plans, tasks and PRD: when they can go, and what follows

- **Tasks** (`TASKS.md`) are done when the last box is ticked. Their value then
  is history, which git already holds, so they can be deleted (or archived)
  right after the feature ships.
- **The plan** (`PLAN.md`) is useful until the code exists. Its lasting parts are
  the diagrams and rules; those should move into living documentation (module
  docs, an architecture page) and the plan can then go.
- **The PRD** lives longer: it is the reference for "is this a bug or a
  feature?" until the behaviour is fully pinned by tests and the API contract.
  After that, what follows it is the **executable spec**: the tests named after
  requirements (`fr4_divide_by_zero`), the OpenAPI file whose examples are run
  by the test suite, and the ADRs, which are never deleted because they explain
  *why*. New work starts a new, small PRD for the change, not an edit of a
  giant one.

### How I review AI-generated documentation

The `review-spec` skill is the checklist; the places I look at first, because
that is where AI docs go wrong:

1. **Invented requirements.** Anything that does not trace to something a
   human said or to a listed assumption gets deleted.
2. **Untestable criteria.** "Handles errors gracefully" is not a requirement;
   `1 ÷ 0 → DIVISION_BY_ZERO, HTTP 422` is.
3. **Edge cases and limits.** Division by zero, overflow, precision, `-0`,
   malformed and huge input.
4. **Claims about libraries.** AI states what a library does with confidence.
   I made the agent run a spike against the real crate before ADR-0003; it
   found three behaviours the draft got wrong (`4 ^ 0.5 = 2.0…0003`,
   `(-8) ^ 0.5` returning a number, the 28-decimal-place limit).
5. **Consistency across documents.** A plan box that no ADR justifies, or an
   ADR that silently contradicts the accepted PRD.
6. **Costs in ADRs.** Every option must list its negatives, including the
   chosen one.

Every review is written down with findings and severities
([docs/reviews/](docs/reviews/)); the agent never marks its own document
`Accepted`.

### When team review should happen

At the cheapest point to change direction, which is before code:

- **PRD**: product owner and one engineer, before any ADR. Scope mistakes are
  the most expensive ones.
- **ADRs**: the engineers who will maintain it, before the plan. This is where
  dependencies and contracts get fixed.
- **Plan**: optional, a quick look by one peer; it is derived from the two
  above.
- **Code**: normal PR review, but it is now shorter, because the reviewer
  checks the code against already-agreed docs instead of debating design in
  the diff.

Here each of those was a separate PR merged by the reviewer, so the review gate
is visible in the history.

### Tests: what I added and why; legacy vs green-field

| Type | Where | Why |
|------|-------|-----|
| Unit, example-based | `src/calc.rs` | One test per PRD acceptance example, named after the requirement, so the PRD is executable |
| Property-based (`proptest`) | `src/calc.rs` | Our own rules around the decimal library are where bugs hide; 2000 random cases per property check "never panics, never -0, always in range" etc. Verified it bites: removing one `normalize()` call makes it fail with a shrunk counterexample |
| API, in-process | `tests/api.rs` | Status codes, error bodies, body limit, 404/405, through the real router without opening a port |
| Contract examples | `tests/api.rs` + `docs/api/openapi.yaml` | Every example in the OpenAPI file is sent and must match exactly, so docs cannot drift from code |
| Smoke + latency, by hand | T-12 commit message | One real-socket run and a p99 measurement; not worth automating at this size |

**Green-field** (this project): test-first unit tests from the spec plus
property tests, because the spec is fresh and precise and the code is designed
to be testable (pure domain). Fast feedback matters more than broad coverage.

**Legacy**: start from the outside. Characterisation (golden-master) tests at
the API or contract level first, recording what the system does today before
touching it; then end-to-end tests on the critical paths. Unit tests come
later, as code is extracted into testable pieces. Property tests are still
useful on legacy code to find the edge cases nobody documented.

