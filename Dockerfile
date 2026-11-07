# Multi-stage build for the calculator API (see docs/DEPLOYMENT.md).
# Generic for a single Cargo binary: no HTTP framework assumed. The app must
# listen on 0.0.0.0:$PORT (Railway injects PORT; defaults to 8080 here).

# --- build ---
FROM rust:1-slim-bookworm AS build
WORKDIR /app

# Cache dependencies: build once with a stub main so later source edits
# don't recompile every crate.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build --release --locked \
    && rm -rf src

COPY src ./src
# touch so cargo rebuilds the real main.rs instead of reusing the stub
RUN touch src/main.rs && cargo build --release --locked

# --- runtime ---
FROM debian:bookworm-slim
RUN useradd --system --uid 10001 app
COPY --from=build /app/target/release/calculator_api /usr/local/bin/calculator_api
USER app
ENV PORT=8080
EXPOSE 8080
CMD ["calculator_api"]
