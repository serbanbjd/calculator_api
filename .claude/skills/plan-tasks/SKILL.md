---
name: plan-tasks
description: Turn an accepted PRD and ADRs into an implementation plan, a task list and UML diagrams in docs/plan/. Use after the PRD and ADRs are accepted and before writing code.
---

# Plan and tasks

Builds on superpowers `writing-plans`. The plan must be detailed enough that an
agent with no context can execute it one task at a time.

## Inputs

`docs/prd/PRD.md` and every accepted ADR. If either is still `Draft`, stop and
say so.

## Outputs

- `docs/plan/PLAN.md`: approach, UML diagrams, milestones, risks.
- `docs/plan/TASKS.md`: the ordered task list.

## UML (Mermaid, inside PLAN.md)

Include at least:

1. **Component or class diagram**: the layers and their dependencies
   (HTTP layer → service/domain → nothing). Use `classDiagram` or `flowchart`.
2. **Sequence diagram**: one request end to end, including an error path
   (for example division by zero). Use `sequenceDiagram`.

Diagrams must match the ADRs; a box that no ADR or FR justifies is a smell.

## Task format (TASKS.md)

```markdown
| ID | Task | Implements | Depends on | Done when |
|----|------|------------|------------|-----------|
| T-01 | … | FR-1, ADR-0002 | — | test X passes; endpoint returns … |
```

Rules:

- Each task is small (≤ ~30 min of agent work) and ends in a green test run.
- Every FR maps to at least one task; every task maps to at least one FR, NFR
  or ADR. Add a coverage table FR → tasks at the bottom.
- Order tasks so the walking skeleton (one endpoint end to end with a test)
  comes first.
- Tests are part of the task, not a separate task at the end.
