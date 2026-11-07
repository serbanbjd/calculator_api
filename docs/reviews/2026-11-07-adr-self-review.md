# Review: ADR-0001 – ADR-0005 (agent self-review)

Reviewed: `docs/adr/0001` – `0005` and PRD amendment A1 · Skill: `review-spec` · Date: 2026-11-07
Reviewer: AI agent. Final acceptance is Serban's, in the pull request.

## Checklist

| Check | Result |
|-------|--------|
| Real alternatives, simplest option considered | Pass. Each ADR includes the stdlib or "do less" option (std HTTP, `f64`, single file, fewer test types) |
| Consequences include costs | Pass. Each ADR lists negatives and what to watch |
| Each decision linked to the FR/NFR that forces it | Pass. `Drivers:` line on every ADR |
| No contradiction with earlier accepted docs | **Fail before fix**: ADR-0003 conflicts with accepted NFR-3; handled by amendment A1, not by silently diverging |
| Every library the code will use is named | Pass. Runtime crates in ADR-0001 and ADR-0003, dev crates in ADR-0005 |
| Edge cases of our own rules verified, not assumed | **Fail**, see F-1 – F-3 |

## Findings

A second spike run on edge cases the first one skipped:

| ID | Location | Problem | Fix | Severity |
|----|----------|---------|-----|----------|
| F-1 | ADR-0003, `percentage` | `percentage(5e27, 50)` = `2.5e27` is in range, but computing `a × b` first overflows, so the API would wrongly answer `RESULT_OUT_OF_RANGE` | Rule 5: retry as `(a ÷ 100) × b` when `a × b` overflows | Blocker |
| F-2 | ADR-0003, `power` | `2 ^ -100` returns `None` from the library (it computes `1 ÷ 2^100`, and `2^100` overflows). True result rounds to `0` | Rule 5: negative exponent whose positive counterpart overflows → `0`; PRD FR-5 example added | Blocker |
| F-3 | PRD FR-5 | `0 ^ 0` not specified; library returns `1` | Added Q-6 with `1` as the assumption, inside amendment A1 for Serban to confirm | Should |
| F-4 | ADR-0004 commit | ADR index listed ADR-0005 one commit before the file existed | No fix needed; noted so history readers are not confused | Nit |

All blocker and should findings are fixed in the following commit.
