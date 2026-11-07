# ADR-0004: Code structure

Status: Proposed · Date: 2026-11-07
Drivers: NFR-4, NFR-6, ADR-0001, ADR-0003, ADR-0005

## Context

The maths rules (ADR-0003) are the part most likely to be wrong and the part
we most want to test exhaustively. The HTTP layer (ADR-0001, ADR-0002) is
mostly framework configuration. Mixing them would make the maths testable
only through HTTP.

## Options considered

### Option A: everything in `main.rs`

- Pros: least ceremony.
- Cons: maths reachable only through HTTP; integration tests in `tests/` cannot
  import a binary crate.

### Option B: one crate, library + thin binary, two modules

- Pros: `src/lib.rs` exposes the router and the domain, so unit, property and
  API tests can all import them; `main.rs` only reads config and starts the
  server. Two modules make the dependency direction obvious.
- Cons: a little more structure than the smallest possible program.

### Option C: Cargo workspace with `calc-core` and `calc-api` crates

- Pros: the compiler enforces the layering.
- Cons: two `Cargo.toml`s and cross-crate plumbing for about 300 lines of code.
  Option B gets the same direction by convention plus review.

## Decision

Option B:

```text
src/
  main.rs        // bind address from PORT (default 3000), init tracing, serve
  lib.rs         // pub fn app() -> Router; pub mod calc; mod api;
  calc.rs        // domain: Operation, CalcError, parse_operand, evaluate. No axum, no serde_json.
  api.rs         // HTTP: request/response types, handlers, error -> status mapping, fallbacks
tests/
  api.rs         // API tests through app() in-process
docs/api/openapi.yaml
```

Rules:

- `calc` depends on `rust_decimal` only. It never imports `axum`, `serde_json`
  or `tracing`. It is synchronous and pure: same input, same output.
- `api` depends on `calc`, never the reverse. It does shape validation (JSON,
  field names, operand count) and maps `CalcError` to the error body and
  status from ADR-0002.
- No `unwrap`/`expect`/indexing that can panic on request data (NFR-4). Clippy
  runs with `-D warnings`; `unwrap_used` and `expect_used` are denied outside
  tests.
- Split a module into a folder only when it outgrows one screen of scrolling;
  not before.

## Consequences

- Positive: the maths can be tested without HTTP, which is where property
  tests are cheap (ADR-0005).
- Positive: `app()` lets API tests run the real router without a port.
- Negative: the layering is a convention, not compiler-enforced. A `use axum`
  in `calc.rs` is easy to spot in review and is listed in the review checklist.
