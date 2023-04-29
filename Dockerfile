# Use the official Rust image.
# https://hub.docker.com/_/rust
FROM lukemathwalker/cargo-chef:latest-rust-1-slim-bullseye AS chef
WORKDIR api

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /api/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN apt-get update && apt-get install -y ca-certificates pkg-config libssl-dev
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin api

# Run the web service on container startup.
FROM debian:bullseye-slim AS runtime
WORKDIR api
COPY --from=builder /api/target/release/api /usr/local/bin
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
ENTRYPOINT ["/usr/local/bin/api"]
