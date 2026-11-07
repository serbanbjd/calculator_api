# ADR-0005: Testing strategy

Status: Accepted (Serban, PR #3, 2026-11-07) · Date: 2026-11-07
Drivers: every FR acceptance criterion, NFR-3, NFR-4, ADR-0003, ADR-0004

## Context

The PRD gives concrete input/output examples for every FR, and ADR-0003 adds
our own rules on top of the decimal library (domain checks, rounding,
normalisation). The riskiest code is the maths edge cases and the error
mapping, not the happy path. Tests should prove the PRD, catch edge cases we
did not think of, and stay fast enough to run before every commit.

## Options considered

| Test type | What it would prove here | Cost | Verdict |
|-----------|--------------------------|------|---------|
| Unit tests (example-based) on `calc` | Each PRD acceptance example and each domain error | Low | **Yes** |
| Property-based tests (`proptest`) on `calc` | Invariants over thousands of generated inputs: no panic, never `-0`, no trailing zeros, `a + b - b == a` when in range, commutativity of `+` and `×`, every error is one of the four codes | Low–medium | **Yes**: finds edge cases examples miss |
| API tests, in-process (`tower::ServiceExt::oneshot` on `app()`) | Request parsing, status codes, error body shape, body limit, 404/405, `/health`; one test per OpenAPI example | Low | **Yes** |
| End-to-end over a real socket | Server binds and serves | Medium (ports, timing) | Minimal: one smoke check with `curl` in the plan, not in the suite |
| Contract tests against OpenAPI schema | Responses match `openapi.yaml` | Medium (extra tooling) | **No** for now; API tests reuse the contract's examples instead |
| Performance / load tests | NFR-2 (p99 < 10 ms) | Medium | **No** in the suite; a one-off measurement recorded in the plan |
| Mutation testing (`cargo-mutants`) | Tests actually fail when code is wrong | Medium, slow | Optional, run once by hand if time allows |

## Decision

Three automated layers, all run by `cargo test`:

1. **Unit tests** in `src/calc.rs` (`#[cfg(test)]`): table-driven, one row per
   PRD acceptance example, named after the FR (`fr4_divide_by_zero`).
2. **Property tests** in `src/calc.rs` with `proptest`: the invariants listed
   above, over operands generated inside and around the 10^28 / 28-place limits.
3. **API tests** in `tests/api.rs`: in-process through `app()`, one test per
   endpoint behaviour and per error code, using the same request examples as
   `docs/api/openapi.yaml`.

Dev-only crates: `proptest`, `tower` (`util`, for `oneshot`),
`http-body-util` (read response bodies).

Each PRD acceptance example must appear in at least one test; the plan's
traceability table maps FR → test names.

## Consequences

- Positive: the whole suite runs in seconds with no network or ports, so it
  runs before every commit (CLAUDE.md).
- Positive: property tests target exactly the code we wrote around the
  library (ADR-0003 rules 1–6).
- Negative: no automatic check that `openapi.yaml` matches responses; drift is
  caught only where tests reuse its examples.
- Negative: NFR-2 is measured once, not guarded continuously.
