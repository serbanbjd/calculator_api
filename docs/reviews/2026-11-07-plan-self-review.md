# Review: PLAN.md and TASKS.md (agent self-review)

Reviewed: `docs/plan/PLAN.md`, `docs/plan/TASKS.md` · Skill: `review-spec` · Date: 2026-11-07
Reviewer: AI agent. Final acceptance is Serban's, in the pull request.

## Checklist

| Check | Result |
|-------|--------|
| FR → task coverage complete both ways | Pass. All 13 FRs and 7 NFRs map to tasks; every task names an FR, NFR or ADR |
| UML matches ADRs (layers, dependencies, error path) | Pass. `calc` has no axum/serde edge; sequence shows input-error and maths-error paths with ADR-0002 statuses |
| Tasks small, ordered, checkable *Done when* | Pass with fixes F-2, F-4 |
| Tests inside tasks, not a trailing "write tests" task | Pass. T-07 is extra property tests on top of per-task unit tests |
| Every ADR decision has a task | **Fail**, see F-1 |

## Findings

| ID | Location | Problem | Fix | Severity |
|----|----------|---------|-----|----------|
| F-1 | TASKS.md | ADR-0004 requires denying `unwrap`/`expect` outside tests (NFR-4), but no task configured it | Added to T-01 | Blocker |
| F-2 | T-07 | Property `(a + b) - b == a` is false in general: with 28 decimal places near 10^27 the sum must round. The test would fail for a correct implementation | Restricted the generator to ≤ 10 places and magnitude < 10^15, with the reason | Blocker |
| F-3 | T-11 | *Done when* relied on `npx @redocly/cli`, but running downloaded tools is blocked in this environment (also blocked a Mermaid render check) | Validation is by API tests reusing the contract's examples, plus review | Should |
| F-4 | T-08 | Depended on T-07 (property tests) for no reason, serialising the plan | Now depends on T-05, T-06 | Nit |
| F-5 | PLAN.md class diagram | Nested generics (`Result~Json~X~~`) render unreliably in Mermaid; could not verify by rendering (see F-3) | Simplified the signature | Nit |

All findings are fixed in the following commit.
