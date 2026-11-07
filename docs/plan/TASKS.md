# Tasks: Calculator API

Status: Draft · Date: 2026-11-07 · Plan: [PLAN.md](PLAN.md)

Rules (from `plan-tasks` and `implement-task`): one task = one commit
`feat(T-nn): …`; test first; done only when fmt, clippy and tests are green.
Tick the box in the same commit.

| Done | ID | Task | Implements | Depends on | Done when |
|------|----|------|------------|------------|-----------|
| [x] | T-01 | **Walking skeleton.** Add runtime crates from ADR-0001/0003 and dev crates from ADR-0005. Split into `lib.rs` (`pub fn app() -> Router`) and `main.rs`. `GET /health` → `200 {"status":"ok"}`. Deny `clippy::unwrap_used` and `clippy::expect_used` in `Cargo.toml` `[lints]` (allowed in tests). | FR-11, NFR-1, NFR-6, ADR-0001, ADR-0004 | — | `tests/api.rs::health_returns_ok` passes through `app()` in-process; `cargo run` serves it on `0.0.0.0:$PORT` (default 8080, matching the Dockerfile from PR #2); Railway health check points at `/health` |
| [x] | T-02 | **Operation and errors.** `Operation` enum with `from_name` and `arity`; `CalcError` with `code()`. | FR-9, ADR-0002, ADR-0004 | T-01 | Unit tests: all 8 names parse, unknown name → `None`, arity 1 for `sqrt` and 2 otherwise, each error maps to its PRD code |
| [x] | T-03 | **Operand parsing.** `parse_operand(&str)`: exact decimal and scientific notation; reject non-numbers, > 28 decimal places, magnitude ≥ 10^28. | FR-9, NFR-3, ADR-0003 rule 1 | T-02 | Unit tests: `"0.1"` exact, `"1e3"` → 1000, `"abc"`, `"1e400"`, `"1e-40"`, `"1e28"` → `InvalidOperand`; `"9999999999999999999999999999"` accepted |
| [x] | T-04 | **Arithmetic core.** `evaluate` for add, subtract, multiply, divide; `check_range`; `normalise`. | FR-1, FR-2, FR-3, FR-4, FR-10, FR-13, ADR-0003 rules 2, 3, 6 | T-03 | Unit tests for every FR-1 – FR-4 example incl. `0.1 + 0.2 = 0.3` and `1 ÷ 3`; `1 ÷ 0` → `DivisionByZero`; `1e20 × 1e10` → `ResultOutOfRange`; `-3 × 0` and `0 ÷ -5` → `0`; `2.50` style results have no trailing zeros |
| [x] | T-05 | **Modulo and percentage.** Including the percentage overflow retry. | FR-7, FR-8, ADR-0003 rule 5 | T-04 | Unit tests for every FR-7 and FR-8 example; `percentage(5e27, 50)` → `2.5e27`; `modulo(1, 0)` → `DivisionByZero` |
| [x] | T-06 | **Power and square root.** Domain checks, 20-digit rounding, negative-exponent underflow → `0`. | FR-5, FR-6, ADR-0003 rules 3, 4, 5 | T-04 | Unit tests for every FR-5 and FR-6 example incl. `4 ^ 0.5 = 2`, `√2 = 1.4142135623730950488`, `0 ^ 0 = 1`, `2 ^ -100 = 0`, `-8 ^ 0.5` and `0 ^ -1` → `UndefinedResult`, `10 ^ 30` → `ResultOutOfRange` |
| [x] | T-07 | **Property tests** for `calc` with `proptest`. | NFR-3, NFR-4, FR-10, FR-13, ADR-0005 | T-05, T-06 | Properties pass on 1 000+ cases each: `evaluate` never panics; results never `-0`, never trailing zeros, always < 10^28; `a + b == b + a`; `a × b == b × a`; `(a + b) - b == a` for operands with ≤ 10 decimal places and magnitude < 10^15 (outside that, rounding to 28 places is expected); `√(x)` equals `x ^ 0.5` for `x ≥ 0` |
| [x] | T-08 | **Calculate endpoint.** `POST /v1/calculate`: request type with `deny_unknown_fields`, operation lookup, arity check, operand extraction (JSON number via raw text or numeric string), call `evaluate`, `{"result": "<string>"}`. | FR-1 – FR-9, NFR-3, ADR-0002 | T-05, T-06 | API tests: one success per operation; `{"operands":[0.1,0.2]}` add → `"0.3"`; unknown operation, wrong arity, `true` / `null` operand, extra field → 400 `INVALID_INPUT` with the right `field` |
| [x] | T-09 | **Error mapping and hardening.** `ApiError` → JSON body + status; map every `JsonRejection`; 16 KB body limit → 413; JSON 404 and 405 fallbacks. | FR-9, NFR-4, NFR-5, ADR-0002 | T-08 | API tests: malformed JSON → 400; wrong `Content-Type` → 400; 17 KB body → 413 `INVALID_INPUT`; `GET /nope` → 404 JSON; `GET /v1/calculate` → 405 JSON; divide by zero → 422 `DIVISION_BY_ZERO`; every error body has `code` and `message` |
| [ ] | T-10 | **Logging.** `tracing-subscriber` with `RUST_LOG` (default `info`); `TraceLayer` logs method, path, status, latency. | NFR-7, ADR-0001 | T-09 | Running the server and sending a request prints one line with status and latency and no operand values (checked by hand, noted in the commit) |
| [ ] | T-11 | **OpenAPI contract.** `docs/api/openapi.yaml` (3.1) for both endpoints, all four error codes, with examples. | FR-12, ADR-0002 | T-09 | File parses as YAML and declares OpenAPI 3.1; every request/response example in it has a matching API test (an external linter is not allowed in this sandbox, so validation is by tests and review) |
| [ ] | T-12 | **Smoke test and run docs.** `curl` every operation against `cargo run`; measure latency of 1 000 requests; README "Run" section. | NFR-2, NFR-6 | T-10, T-11 | Smoke output and p99 latency recorded in the commit message; README shows build, test and run commands |

## Coverage: requirements → tasks

| Requirement | Tasks | | Requirement | Tasks |
|-------------|-------|-|-------------|-------|
| FR-1 add | T-04, T-08 | | FR-11 health | T-01 |
| FR-2 subtract | T-04, T-08 | | FR-12 API docs | T-11 |
| FR-3 multiply | T-04, T-08 | | FR-13 no `-0` | T-04, T-07 |
| FR-4 divide | T-04, T-08 | | NFR-1 stateless | T-01 (no state anywhere; checked in review) |
| FR-5 power | T-06, T-08 | | NFR-2 fast | T-12 |
| FR-6 sqrt | T-06, T-08 | | NFR-3 exact decimals | T-03, T-07, T-08 |
| FR-7 modulo | T-05, T-08 | | NFR-4 robust | T-07, T-09 |
| FR-8 percentage | T-05, T-08 | | NFR-5 body limit | T-09 |
| FR-9 invalid input | T-02, T-03, T-08, T-09 | | NFR-6 easy to run | T-01, T-12 |
| FR-10 out of range | T-04, T-07 | | NFR-7 logging | T-10 |

## Coverage: tasks → requirements

Every task row above lists at least one FR, NFR or ADR in *Implements*.
