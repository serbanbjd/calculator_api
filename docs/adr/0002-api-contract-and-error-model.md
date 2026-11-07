# ADR-0002: API contract and error model

Status: Accepted (Serban, PR #3, 2026-11-07) · Date: 2026-11-07
Drivers: FR-1 – FR-13, NFR-3, NFR-5, section 6 of the PRD

## Context

The PRD fixes the operations, the four error codes and that an error carries
`code`, `message` and, for input errors, `field`. It leaves open the endpoint
layout, how numbers travel in JSON without losing digits (NFR-3), and the
HTTP status codes.

## Options considered

### Endpoints

- **A1: one path per operation** (`POST /v1/add`, `POST /v1/sqrt`, …).
  RESTful-looking, but an unknown operation becomes a router 404 with no JSON
  body, which contradicts FR-9 (`INVALID_INPUT`), and adding an operation
  means a new route.
- **A2: one endpoint, operation in the body** (`POST /v1/calculate`).
  One handler, one request type, unknown operations fail validation like any
  other bad field. Less "resource-like", but a calculator call is an action,
  not a resource.

### Number encoding

- **B1: JSON numbers in and out.** Natural, but most JSON clients (JavaScript,
  many others) parse numbers as binary doubles, so `0.3333333333333333333333333333`
  arrives as `0.3333333333333333`. That breaks NFR-3 on the client side.
- **B2: JSON strings in and out** (`"0.1"`). Lossless for every client, but
  awkward for `curl` users typing `{"operands": [2, 3]}`.
- **B3: accept numbers or strings; always return the result as a string.**
  Input is lossless either way (the server reads the raw number text,
  ADR-0003); output is lossless for every client.

## Decision

A2 + B3.

```http
POST /v1/calculate
Content-Type: application/json

{ "operation": "divide", "operands": [1, "3"] }
```

```http
200 OK
{ "result": "0.3333333333333333333333333333" }
```

- `operation`: one of `add`, `subtract`, `multiply`, `divide`, `power`,
  `sqrt`, `modulo`, `percentage`.
- `operands`: array of JSON numbers or numeric strings; length 1 for `sqrt`,
  2 for everything else. For `percentage`, `[a, b]` means "a percent of b".
- Unknown fields are rejected (FR-9).

Errors share one body:

```json
{ "error": { "code": "INVALID_INPUT", "message": "operands[1] is not a number", "field": "operands[1]" } }
```

`field` is omitted when no single field is at fault.

| Code | HTTP status | Why this status |
|------|-------------|-----------------|
| `INVALID_INPUT` (malformed JSON, wrong shape, bad operand) | 400 Bad Request | The request itself is wrong |
| `INVALID_INPUT` (body over 16 KB, NFR-5) | 413 Content Too Large | Standard meaning; code stays `INVALID_INPUT` |
| `DIVISION_BY_ZERO` | 422 Unprocessable Content | Request is well formed; the maths is not defined |
| `UNDEFINED_RESULT` | 422 Unprocessable Content | Same |
| `RESULT_OUT_OF_RANGE` | 422 Unprocessable Content | Same |

Other endpoints:

- `GET /health` → `200 {"status":"ok"}` (FR-11). Unversioned: it belongs to
  operations, not to the public API.
- Unknown paths → 404 with an `INVALID_INPUT` error body; wrong method → 405
  with the same body. No plain-text responses anywhere.

The contract is written by hand as OpenAPI 3.1 in `docs/api/openapi.yaml`
(FR-12) and checked by the API tests (ADR-0005).

## Consequences

- Positive: one handler and one request type; adding an operation is an enum
  variant plus a function.
- Positive: no client can lose digits on the result.
- Negative: results are strings, so clients must parse them; documented in the
  OpenAPI contract.
- Negative: a hand-written OpenAPI file can drift from the code. Mitigation: API
  tests use the same examples as the contract. Generating it from code (e.g.
  `utoipa`) was rejected for now as another dependency for 2 endpoints.
- `/v1` lets a future breaking change ship as `/v2` beside it.
