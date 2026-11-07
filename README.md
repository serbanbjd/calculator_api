# Calculator API

A small calculator HTTP API built with **AI spec-driven development**: every line of
code traces back to a written spec (PRD → ADR → plan/tasks) that an AI agent drafted
and a human reviewed.

> Status: project setup. Specs and code land in the following steps.

## Repository layout

| Path | What lives there |
|------|------------------|
| `CLAUDE.md` / `AGENTS.md` | How the AI agent must work in this repo |
| `.claude/settings.json` | Agent plugins (superpowers, ponytail) |
| `.claude/skills/` | Project-specific agent skills for the spec-driven workflow |
| `docs/prd/` | Product requirements: *what* the calculator does |
| `docs/adr/` | Architecture decision records: *how* and *why* |
| `docs/plan/` | Implementation plan, task list, UML diagrams |

## Workflow

1. **PRD** — what the system does and, just as important, what it does not.
2. **ADR** — architecture decisions with context, options and consequences.
3. **Plan & tasks** — small, testable tasks with a UML diagram of the design.
4. **Code** — generated task by task, test first, against the docs above.

The questions from the brief (boundaries, PRD gathering, doc lifecycle, reviewing
AI docs, team review timing, test strategy) are answered at the end of this README
once the work is done.
