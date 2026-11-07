---
name: review-spec
description: Review AI-generated documentation (PRD, ADR, plan, tasks) before it is accepted. Use whenever a spec doc is drafted or changed, and before any code is generated from it.
---

# Review a spec document

AI-written docs fail in predictable ways: confident invented requirements,
vague acceptance criteria, missing edge cases, and plans that drift from the
decisions above them. Check these places first.

## PRD

- [ ] **Invented scope.** Every requirement traces to something the human said
      or an explicitly listed assumption. Delete anything else.
- [ ] **Boundary.** In scope / out of scope / actors are explicit. Out of scope
      is concrete (e.g. "no expression parsing", "no persistence").
- [ ] **Testability.** Each FR has an acceptance criterion with concrete inputs
      and outputs.
- [ ] **Edge cases.** Division by zero, overflow, precision/rounding,
      NaN/Infinity, malformed or missing input, huge numbers.
- [ ] **No solution leakage.** No frameworks or libraries in the PRD.

## ADR

- [ ] Real alternatives, not straw men; the simplest option was considered.
- [ ] Consequences include the costs, not only the benefits.
- [ ] The decision is linked to the FR/NFR that forces it.
- [ ] Nothing contradicts an earlier accepted ADR without superseding it.

## Plan and tasks

- [ ] FR → task coverage is complete in both directions.
- [ ] UML matches the ADRs (layers, dependencies, error path).
- [ ] Tasks are small, ordered, and each has a checkable *Done when*.
- [ ] Tests live inside tasks, not in a trailing "write tests" task.

## Output

Write findings as a list: location, problem, suggested fix, severity
(blocker / should / nit). Blockers must be resolved before the doc becomes
`Accepted`. A human makes the final call; the agent never marks its own doc
`Accepted`.
