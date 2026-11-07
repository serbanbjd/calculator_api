# Review: PRD draft (agent self-review)

Reviewed: `docs/prd/PRD.md` at commit 33f0308 · Skill: `review-spec` · Date: 2026-11-07
Reviewer: AI agent. Final acceptance is Serban's, in the pull request.

## Checklist

| Check | Result |
|-------|--------|
| Invented scope: every FR traces to the brief or an answered/listed question | Pass. Operation set from Q-2; health check and API docs are standard operability needs, listed as Must/Should |
| Boundary explicit: in, out, actors | Pass. Context diagram plus an out-of-scope table with reasons |
| Testability: each FR has concrete inputs and outputs | Pass with one nit (F-3) |
| Edge cases: ÷0, overflow, precision, NaN/Inf, malformed, huge numbers | **Fail**, see F-1 and F-2 |
| No solution leakage | Pass. HTTP and JSON are the product interface; status codes and body layout are deferred to an ADR |

## Findings

| ID | Location | Problem | Fix | Severity |
|----|----------|---------|-----|----------|
| F-1 | FR-9 | A JSON number such as `1e400` is outside double range. Most parsers turn it into Infinity, so it would enter the calculation instead of being rejected | Add: operands that are not finite doubles → `INVALID_INPUT` | Blocker |
| F-2 | FR-3 vs Q-4 | IEEE-754 gives `-3 × 0 = -0`, so the acceptance example `→ 0` silently depends on the open answer to Q-4 | Reference Q-4 from FR-3 and make the `-0` rule a requirement (FR-13) so it gets a test | Should |
| F-3 | FR-11 | "answers success" does not say what a test checks | State: responds with a success status and a body reporting `ok` | Nit |

All findings are fixed in the following commit.
