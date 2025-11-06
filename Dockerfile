# syntax=docker/dockerfile:1

FROM rust:1.91-slim-bullseye AS chef
WORKDIR /app
RUN cargo install cargo-chef --locked

COPY Cargo.toml Cargo.lock ./
COPY server/Cargo.toml server/Cargo.toml
COPY shared/Cargo.toml shared/Cargo.toml
COPY server/src server/src
COPY server/migrations server/migrations
COPY shared/src shared/src

RUN cargo chef prepare --recipe-path recipe.json

FROM rust:1.91-slim-bullseye AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --locked

COPY --from=chef /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin terma-server

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app

COPY --from=builder /app/target/release/terma-server /usr/local/bin/terma-server
COPY server/migrations server/migrations

ENV BIND_ADDR=0.0.0.0:8080
EXPOSE 8080

CMD ["terma-server"]
