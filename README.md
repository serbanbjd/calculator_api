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
[docs/api/openapi.yaml](docs/api/openapi.yaml).

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

The questions from the brief (boundaries, PRD gathering, doc lifecycle, reviewing
AI docs, team review timing, test strategy) are answered at the end of this README
once the work is done.
