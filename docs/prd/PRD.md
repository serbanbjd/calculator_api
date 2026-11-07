# PRD: Calculator API

Status: **Draft** · Last reviewed: 2026-11-07 · Owner: Serban

## 1. Problem and goal

Client applications need a small, reliable service that performs arithmetic
and reports errors in a predictable, machine-readable way. The goal is a
stateless HTTP JSON API that does a well-defined set of calculations correctly,
rejects bad input clearly, and never returns a silently wrong number.

A secondary goal: the service is the subject of a spec-driven development
exercise, so every behaviour here must be specific enough to test.

## 2. Users and use cases

| Actor | Use case |
|-------|----------|
| API client (another service, script or developer with `curl`) | Send two numbers and an operation, get the result back |
| API client | Get a clear error when input is invalid or the operation is undefined |
| Operator | Check that the service is up |

## 3. Scope

The system boundary is one stateless HTTP service. It receives a request,
computes, and answers. It stores nothing and calls no other system.

```mermaid
flowchart LR
    client([API client]) -- "HTTP + JSON request" --> api[Calculator API]
    api -- "JSON result or error" --> client
    ops([Operator]) -- "health check" --> api
```

### In scope

- Arithmetic on real numbers: add, subtract, multiply, divide, power, square
  root, modulo, percentage.
- Input validation and a consistent error format.
- A health check.
- Machine-readable API documentation of the endpoints.

### Out of scope (and why)

| Item | Why |
|------|-----|
| Parsing expressions like `2 + 3 * 4` | Needs a grammar and parser; doubles the work and risk for little extra value in the timebox |
| History, memory, persistence | Keeps the service stateless; no storage to design or secure |
| Authentication, users, rate limiting | Belongs to an API gateway in front of the service, not to the calculator |
| Arbitrary-precision or big-integer maths | Standard 64-bit floating point covers the use cases; see Q-1 |
| Complex numbers, trigonometry, logarithms, units, currency | Not requested; can be added later as new FRs |
| Batch or chained operations in one request | One request = one operation keeps the contract simple |
| User interface | API only |

## 4. Functional requirements

All operations take finite numbers as JSON input and return a single finite
number. "Error" means the error response from section 6.

| ID | Requirement | Acceptance criteria | Priority |
|----|-------------|---------------------|----------|
| FR-1 | Add two numbers `a + b` | `2, 3` → `5`; `-1.5, 0.5` → `-1` | Must |
| FR-2 | Subtract `a - b` | `5, 3` → `2`; `3, 5` → `-2` | Must |
| FR-3 | Multiply `a × b` | `4, 2.5` → `10`; `-3, 0` → `0` | Must |
| FR-4 | Divide `a ÷ b` | `10, 4` → `2.5`; `1, 0` → error `DIVISION_BY_ZERO` | Must |
| FR-5 | Power `a ^ b` | `2, 10` → `1024`; `4, 0.5` → `2`; `2, -1` → `0.5`; `0, -1` → error `UNDEFINED_RESULT`; `-8, 0.5` → error `UNDEFINED_RESULT` (not a real number) | Should |
| FR-6 | Square root `√a` (one operand) | `9` → `3`; `2` → `1.4142135623730951`; `-1` → error `UNDEFINED_RESULT` | Should |
| FR-7 | Modulo: remainder of `a ÷ b`, sign follows the dividend `a` | `10, 3` → `1`; `-10, 3` → `-1`; `5.5, 2` → `1.5`; `1, 0` → error `DIVISION_BY_ZERO` | Should |
| FR-8 | Percentage: `a` percent of `b`, i.e. `a × b ÷ 100` | `20, 50` → `10`; `150, 40` → `60` | Could |
| FR-9 | Reject invalid input | Missing operand, extra unknown fields, non-numeric operand, wrong operand count for the operation, malformed JSON, unknown operation → error `INVALID_INPUT` naming the problem field | Must |
| FR-10 | Never return a non-finite number | Any result that would be ±Infinity or NaN (e.g. `1e308 × 10`) → error `RESULT_OUT_OF_RANGE` instead | Must |
| FR-11 | Health check | A health endpoint answers success with no calculation when the service is running | Must |
| FR-12 | API documentation | Every endpoint, request and response shape, including errors, is described in a machine-readable contract kept in the repo | Should |

## 5. Non-functional requirements

| ID | Requirement | Measure |
|----|-------------|---------|
| NFR-1 | Stateless | No data survives a request; any instance can serve any request |
| NFR-2 | Fast | p99 under 10 ms per request on a developer laptop, excluding network |
| NFR-3 | Predictable numerics | Results follow IEEE-754 double precision, so `0.1 + 0.2` returns `0.30000000000000004` (see Q-1) |
| NFR-4 | Robust | No input, however malformed or large, crashes the service or returns a server error |
| NFR-5 | Bounded input | Request bodies over 16 KB are rejected with `INVALID_INPUT` |
| NFR-6 | Easy to run | One command builds, one command tests, one command starts the service |
| NFR-7 | Observable | Each request logs the operation, outcome and duration; operands are not logged |

## 6. Error behaviour

Every error has the same JSON shape: a stable machine-readable `code`, a
human-readable `message`, and, for input errors, the `field` at fault. The
exact HTTP status mapping and body layout are decided in an ADR.

| Code | When | Category |
|------|------|----------|
| `INVALID_INPUT` | Bad request shape, types, operand count, unknown operation, oversized body (FR-9, NFR-5) | Client error |
| `DIVISION_BY_ZERO` | Divide or modulo by zero (FR-4, FR-7) | Client error |
| `UNDEFINED_RESULT` | Mathematically undefined in real numbers: `√` of a negative, `0 ^ negative`, negative base with fractional exponent (FR-5, FR-6) | Client error |
| `RESULT_OUT_OF_RANGE` | Result overflows to ±Infinity or becomes NaN (FR-10) | Client error |

No input causes a server error (NFR-4).

## 7. Open questions and assumptions

| ID | Question | Current assumption | Status |
|----|----------|--------------------|--------|
| Q-1 | Must `0.1 + 0.2` return exactly `0.3` (decimal arithmetic)? | No: IEEE-754 doubles, documented in NFR-3. Decimal would rule out a simple `√` and fractional power | Open |
| Q-2 | Which operations are in scope? | Core four plus power, square root, modulo, percentage; no expression parsing | Answered by Serban, 2026-11-07 |
| Q-3 | Modulo sign convention for negatives? | Sign follows the dividend (truncated remainder, as in C, Rust, Java) | Open |
| Q-4 | Is `-0` a valid result to return? | Normalise `-0` to `0` in responses | Open |
| Q-5 | Auth and rate limiting? | Out of scope, handled by a gateway | Assumed |

## 8. Glossary

- **Operand**: an input number (`a`, `b`).
- **Finite number**: a real number that is not ±Infinity or NaN.
- **Error code**: the stable identifier in an error response that clients match on.
