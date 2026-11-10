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

### How I defined the boundaries

I started with what the service should *not* do. It is one stateless HTTP
service: a request comes in, it computes, it answers. No storage, no calls to
other systems, no auth or rate limiting (that belongs in a gateway in front
of it). I chose the scope myself: the four basic operations plus power,
square root, modulo and percentage, one operation per request, and no
parsing of expressions like `2 + 3 * 4`, which would have eaten the timebox.
The PRD has a context diagram and an out-of-scope table with a reason for
each item. Inside the code there is a second boundary: `calc` is pure maths,
`api` is HTTP, and every input passes one validation function before it
reaches the maths.

### How I gathered the PRD

The agent interviewed me one question at a time (the `write-prd` skill,
built on superpowers `brainstorming`). I picked the scope and answered the
questions that change behaviour: results must be exact decimals
(`0.1 + 0.2 = 0.3`), modulo keeps the sign of the dividend, and `-0` is
returned as `0`. Every requirement had to have concrete input/output
examples, which later became the unit tests. I reviewed and merged the PRD
as a pull request before any architecture work started. When the
implementation showed the PRD was wrong about precision, the PRD was changed
first (amendments A1 and A2), then the code.

### When plans, tasks and PRD can be removed, and what follows

- **Tasks** can go as soon as they are all done; git history keeps the record.
- **The plan** can go once the code exists. The useful parts (diagrams,
  per-operation rules) should move into living docs first.
- **The PRD** stays until its behaviour is fully covered by tests and the API
  contract. After that, the source of truth is the tests named after
  requirements, the OpenAPI file whose examples run in the test suite, and the
  ADRs, which I would never delete because they explain *why*. New features
  start with a new, small PRD.

### How I review AI-generated documentation

The places I check first, because that is where AI goes wrong:

1. **Invented requirements**: anything I did not ask for gets removed.
2. **Vague acceptance criteria**: "handles errors gracefully" is not testable;
   `1 ÷ 0 → DIVISION_BY_ZERO, HTTP 422` is.
3. **Edge cases**: division by zero, overflow, precision, `-0`, bad input.
4. **Claims about libraries**: I had the agent test the decimal library before
   accepting the ADR. That caught three wrong assumptions (for example,
   `(-8) ^ 0.5` returned a number instead of an error).
5. **Consistency**: an ADR must not silently contradict the PRD, and every
   box in the plan must trace back to a requirement or decision.

Each review is written down in [docs/reviews/](docs/reviews/), and only I mark
a document as accepted.

### When team review should happen

Before code, when changing direction is still cheap: the PRD with the product
owner and an engineer, the ADRs with the engineers who will maintain the
system, and the plan optionally with one peer. Code review then gets shorter,
because it checks the code against decisions already agreed. In this repo
each step was its own pull request, so the review points are visible in the
history.

### Tests: what I added and why; legacy vs green-field

- **Unit tests** from every PRD example, named after the requirement
  (`fr4_divide_by_zero`).
- **Property-based tests** (`proptest`, 2000 random cases each) on the maths
  rules I wrote around the library: never panics, never `-0`, always in range.
  I checked they catch a planted bug.
- **API tests** through the real router, without a network port: status codes,
  error bodies, body limit.
- **Contract tests**: every example in the OpenAPI file is sent to the API and
  must match.

On a **green-field** system like this I prefer test-first unit and property
tests: the spec is fresh and the code is designed to be testable. On a
**legacy** system I would start from the outside: tests that record what the
system does today at the API level, then end-to-end tests on critical paths,
and unit tests only as code gets extracted into testable pieces.
