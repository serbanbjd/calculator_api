# Implementation plan: Calculator API

Status: Accepted (Serban, PR #4) · Date: 2026-11-07
Inputs: [PRD](../prd/PRD.md) (Accepted, with A1) · [ADR-0001 – 0005](../adr/README.md) (Accepted)
Tasks: [TASKS.md](TASKS.md)

## 1. Approach

Build inside out, test first (superpowers `test-driven-development`,
`implement-task` skill):

1. **Walking skeleton**: crates from the ADRs, library/binary split, `GET /health`
   served by `app()` and covered by an in-process API test.
2. **Domain (`calc`)**: pure functions for parsing operands and evaluating
   each operation, with the ADR-0003 rules. Unit tests from the PRD examples,
   then property tests.
3. **HTTP (`api`)**: `POST /v1/calculate`, request validation, error mapping,
   body limit, JSON fallbacks.
4. **Operations**: logging, OpenAPI contract, smoke test and run instructions.

Each task ends green on `cargo fmt --check`, `cargo clippy -- -D warnings`,
`cargo test`, and is one commit.

## 2. Components

Dependencies point one way: `main` → `api` → `calc` → `rust_decimal`.
`calc` knows nothing about HTTP or JSON (ADR-0004).

```mermaid
flowchart LR
    client([API client]) -->|HTTP JSON| main
    subgraph crate [calculator_api crate]
        main["main.rs<br/>PORT, tracing init, serve"] --> lib["lib.rs<br/>app(): Router"]
        lib --> api["api.rs<br/>handlers, request/response types,<br/>error → status mapping, fallbacks"]
        api --> calc["calc.rs<br/>Operation, CalcError,<br/>parse_operand, evaluate"]
    end
    api -.-> axum[(axum / tower-http / serde_json)]
    calc -.-> dec[(rust_decimal)]
    tests["tests/api.rs"] -.->|oneshot| lib
```

## 3. Types

```mermaid
classDiagram
    direction LR
    class Operation {
        <<enum>>
        Add
        Subtract
        Multiply
        Divide
        Power
        Sqrt
        Modulo
        Percentage
        +from_name(name: &str) Option~Operation~
        +arity() usize
    }
    class CalcError {
        <<enum>>
        InvalidOperand(reason)
        DivisionByZero
        UndefinedResult(reason)
        ResultOutOfRange
        +code() &str
    }
    class calc {
        <<module>>
        +parse_operand(text: &str) Result~Decimal, CalcError~
        +evaluate(op: Operation, operands: &[Decimal]) Result~Decimal, CalcError~
        -normalise(d: Decimal) Decimal
        -check_range(d: Decimal) Result~Decimal, CalcError~
        -round_significant(d: Decimal, digits: u32) Decimal
    }
    class CalculateRequest {
        <<serde, deny_unknown_fields>>
        operation: String
        operands: Vec~serde_json::Value~
    }
    class CalculateResponse {
        result: String
    }
    class ApiError {
        status: StatusCode
        code: &str
        message: String
        field: Option~String~
        +into_response() Response
    }
    class api {
        <<module>>
        +router() Router
        -calculate(body) Result~CalculateResponse, ApiError~
        -health() Json
        -not_found() ApiError
        -method_not_allowed() ApiError
    }
    calc ..> Operation
    calc ..> CalcError
    api ..> CalculateRequest
    api ..> CalculateResponse
    api ..> ApiError
    api ..> calc : calls
    ApiError ..> CalcError : From
```

`CalcError` → `ApiError` mapping (ADR-0002):

| `CalcError` | `code` | Status |
|-------------|--------|--------|
| `InvalidOperand` (plus api-side shape errors) | `INVALID_INPUT` | 400 (413 for body limit) |
| `DivisionByZero` | `DIVISION_BY_ZERO` | 422 |
| `UndefinedResult` | `UNDEFINED_RESULT` | 422 |
| `ResultOutOfRange` | `RESULT_OUT_OF_RANGE` | 422 |

## 4. Request flow

Happy path and the two kinds of failure: an input error stopped in `api`, and
a maths error raised by `calc`.

```mermaid
sequenceDiagram
    autonumber
    participant C as Client
    participant R as axum Router<br/>(body limit, TraceLayer)
    participant H as api::calculate
    participant D as calc

    C->>R: POST /v1/calculate {"operation":"divide","operands":[1,"3"]}
    R->>H: Json<CalculateRequest> (or rejection)
    alt malformed JSON, unknown field, body > 16 KB
        H-->>C: 400/413 {"error":{"code":"INVALID_INPUT",...}}
    end
    H->>H: Operation::from_name, check arity
    H->>D: parse_operand("1"), parse_operand("3")
    alt unknown operation, wrong arity or bad operand
        H-->>C: 400 {"error":{"code":"INVALID_INPUT","field":"operands[1]"}}
    end
    H->>D: evaluate(Divide, [1, 3])
    D->>D: domain check, checked_div, check_range, normalise
    alt divisor is 0
        D-->>H: Err(DivisionByZero)
        H-->>C: 422 {"error":{"code":"DIVISION_BY_ZERO",...}}
    else ok
        D-->>H: Ok(0.3333333333333333333333333333)
        H-->>C: 200 {"result":"0.3333333333333333333333333333"}
    end
    R->>R: log method, path, status, latency (no operands)
```

## 5. Evaluation rules per operation

From ADR-0003; each row is a unit-test group.

| Operation | Domain checks before computing | Computation | After |
|-----------|-------------------------------|-------------|-------|
| add, subtract, multiply | — | `checked_add/sub/mul` | range, normalise |
| divide | `b == 0` → `DivisionByZero` | `checked_div` | range, normalise |
| modulo | `b == 0` → `DivisionByZero` | `checked_rem` (sign of `a`) | range, normalise |
| percentage | — | `checked_mul(a, b) / 100`; on overflow `(a / 100) × b` | range, normalise |
| sqrt | `a < 0` → `UndefinedResult` | `sqrt` | round to 20 significant digits, normalise |
| power | `a == 0 && b < 0` → `UndefinedResult`; `a < 0 && b` not integer → `UndefinedResult` | `checked_powd`; if `b < 0` and it fails, check `a^|b|` overflows → `0` | integer `b`: range, normalise; otherwise round to 20 significant digits, normalise |

"range" = magnitude < 10^28 else `ResultOutOfRange`; "normalise" = strip
trailing zeros, `-0` → `0`.

## 6. Risks

| Risk | Mitigation |
|------|------------|
| `serde_json` `arbitrary_precision` changes number handling crate-wide | Only `api` touches `serde_json::Value`; operand text goes straight to `parse_operand` |
| axum rejections come back as plain text | Handler takes `Result<Json<_>, JsonRejection>` and maps every rejection; fallbacks for 404/405; API tests assert JSON bodies |
| `rust_decimal` `powd` precision varies with inputs | 20-digit rounding; property test compares `x^0.5` with `sqrt(x)` |
| Hand-written OpenAPI drifts from code | API tests use the contract's request/response examples (T-11) |
| Timebox | Tasks ordered by priority; `Could` items (FR-8) and polish last |

## 7. Definition of done

- Every task in TASKS.md ticked, each with its own commit.
- `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` green.
- Every PRD acceptance example appears in a test (coverage table in TASKS.md).
- Smoke test with `curl` against `cargo run` recorded in T-12.
