# syntax=docker/dockerfile:1

FROM rust:1.91-bookworm AS chef
WORKDIR /app
RUN cargo install cargo-chef --locked

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY server/Cargo.toml server/Cargo.toml
COPY shared/Cargo.toml shared/Cargo.toml
# Create dummy lib.rs files for workspace members
RUN mkdir -p server/src shared/src && \
    echo "fn main() {}" > server/src/main.rs && \
    echo "" > shared/src/lib.rs
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libpq-dev \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin terma-server

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libpq5 \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /app

COPY --from=builder /app/target/release/terma-server /usr/local/bin/terma-server

ENV BIND_ADDR=0.0.0.0:8080
EXPOSE 8080

CMD ["terma-server"]
