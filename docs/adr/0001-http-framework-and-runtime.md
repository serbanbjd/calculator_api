# ADR-0001: HTTP framework, runtime and logging

Status: Accepted (Serban, PR #3, 2026-11-07) · Date: 2026-11-07
Drivers: Stack choice (Rust, by Serban), FR-9, FR-11, NFR-2, NFR-4, NFR-5, NFR-6, NFR-7

## Context

The language is Rust (Serban's choice). The service is a small stateless JSON
API (NFR-1) that must reject oversized bodies (NFR-5), never crash on input
(NFR-4), log each request without operands (NFR-7), and start with one
command (NFR-6). We need an HTTP server, JSON (de)serialisation and logging.

## Options considered

### Option A: std only (hand-written HTTP over `std::net`)

- Pros: zero dependencies; the ponytail default to beat.
- Cons: we would write HTTP parsing, keep-alive, body limits and malformed
  request handling ourselves. That is exactly the security-relevant code
  NFR-4 says must not fail, and it is not what this exercise is about.

### Option B: `axum` on `tokio`, with `serde`/`serde_json` and `tracing`

- Pros: the most widely used Rust web stack; built on `hyper` and `tower`, so
  body limits (`DefaultBodyLimit`), request tracing (`tower-http` `TraceLayer`)
  and in-process testing (`tower::ServiceExt::oneshot`) come ready-made.
  Typed extractors turn malformed JSON into a rejection we can map to
  `INVALID_INPUT`.
- Cons: async runtime is more than a calculator needs; larger dependency tree
  and compile time.

### Option C: `actix-web`

- Pros: mature and fast.
- Cons: its own actor/runtime model and test utilities; no advantage over B
  for this size, and a smaller ecosystem of `tower` middleware.

## Decision

Option B. Crates and why each is needed:

| Crate | Purpose |
|-------|---------|
| `axum` 0.8 | Routing, extractors, responses |
| `tokio` 1 (`macros`, `rt-multi-thread`, `net`) | Runtime required by axum |
| `serde` 1 (`derive`), `serde_json` 1 | JSON request/response types |
| `tower-http` (`trace`) | Per-request log span with method, path, status, latency (NFR-7) |
| `tracing`, `tracing-subscriber` (`env-filter`) | Log output, level set by `RUST_LOG` |

Number handling (`rust_decimal`, `serde_json` features) is decided in
ADR-0003; test-only crates in ADR-0005.

## Consequences

- Positive: body limits, malformed-JSON handling and logging are configuration,
  not code. Handlers stay small.
- Positive: the whole router can be tested in-process without opening a port.
- Negative: async adds noise to an otherwise synchronous computation; the maths
  itself stays sync (ADR-0004) so this does not leak into the domain.
- Negative: ~100 transitive crates and a longer first build.
- Watch: axum's default rejection bodies are plain text; ADR-0002 requires us
  to replace them with our JSON error format.
