# ADR-0006: Serve Swagger UI from the hand-written contract

Status: Proposed · Date: 2026-11-07
Drivers: FR-14, FR-12, ADR-0002

## Context

Serban wants a Swagger UI page on the deployed service (FR-14). ADR-0002 made
the OpenAPI contract a hand-written file, `docs/api/openapi.yaml`, whose
examples are checked by the test suite. We need to serve that file and a page
that renders it.

## Options considered

### Option A: `utoipa` + `utoipa-swagger-ui`

- Pros: spec generated from Rust types; Swagger UI assets embedded in the binary.
- Cons: replaces the hand-written, test-checked contract (reverses part of
  ADR-0002); two new crates, one of which downloads Swagger UI at build time;
  annotations on every type for two endpoints.

### Option B: embed the existing YAML, load Swagger UI from a CDN

- Pros: no new crate. `include_str!` puts the tested contract in the binary,
  so the served spec is exactly the one the tests check. The page is ~20 lines
  of HTML.
- Cons: the browser fetches Swagger UI JavaScript from a third-party CDN
  (unpkg). Mitigated by pinning the exact version and adding Subresource
  Integrity hashes, so a changed file is refused by the browser.

### Option C: commit the Swagger UI assets into the repo

- Pros: no CDN at runtime.
- Cons: ~1.5 MB of vendored minified JavaScript in the history for a docs page.

## Decision

Option B. `swagger-ui-dist` pinned to 5.17.14 (supports OpenAPI 3.1) from
unpkg, with SRI `sha384` hashes computed from the npm tarball.

- `GET /openapi.yaml` → the embedded file, `content-type: application/yaml`.
- `GET /docs` → HTML page that loads Swagger UI and points it at `/openapi.yaml`.
- `.dockerignore` stops excluding `docs/api/`, since the build now reads it.

## Consequences

- Positive: one source of truth for the contract, still checked by tests.
- Positive: no change to the Rust dependency tree.
- Negative: `/docs` needs the viewer's browser to reach unpkg.com; the API
  itself does not.
- Negative: upgrading Swagger UI means updating the version and both hashes.
