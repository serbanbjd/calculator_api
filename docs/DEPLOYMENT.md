# Deployment

Status: Draft

How the calculator API is packaged and hosted. This is an operational note, not a
spec: the PRD and ADRs decide what the service does and how it is built. Anything
here that becomes a requirement must be moved into them.

## Packaging: `Dockerfile`

A two-stage build that is generic for a single Cargo binary, so it does not depend
on the HTTP framework the ADR will pick.

1. **build** (`rust:1-slim-bookworm`): compiles dependencies first against a stub
   `main.rs` so they stay cached, then builds the real crate with
   `cargo build --release --locked`.
2. **runtime** (`debian:bookworm-slim`): only the `calculator_api` binary, run as a
   non-root user.

`.dockerignore` keeps `target/`, `.git/`, docs and agent config out of the build
context. `rust-toolchain.toml` is deliberately not copied: the image already pins a
stable toolchain, and copying it would make rustup download rustfmt and clippy,
which a release build does not need.

Build and run locally:

```sh
docker build -t calculator_api .
docker run --rm -p 8080:8080 calculator_api
```

## Contract the app must honour

These are inputs for the PRD (NFRs) and the HTTP ADR:

- Listen on `0.0.0.0:$PORT`. Railway injects `PORT`; the image defaults it to `8080`.
- Log to stdout/stderr; the platform collects it.
- A cheap health endpoint (for example `GET /health`) is recommended so Railway can
  run a healthcheck. Once it exists, add `healthcheckPath` to `railway.toml`.

## Hosting: Railway

`railway.toml` tells Railway to build with the `Dockerfile` and restart the service
on failure.

One-time setup (dashboard):

1. railway.com → **New Project** → **Deploy from GitHub repo** →
   `serbanbjd/calculator_api`, branch `main`.
2. Service → **Settings** → **Networking** → **Generate Domain** for a public URL.

Or with the CLI from the repo root:

```sh
railway login
railway init            # creates the project
railway up              # builds the Dockerfile and deploys
railway domain          # generates a public URL
```

Every push to `main` then redeploys automatically (dashboard setup).
