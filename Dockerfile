# Build stage
FROM rust:1.91-bookworm AS chef
WORKDIR /app
RUN cargo install cargo-chef

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY server/Cargo.toml ./server/Cargo.toml
COPY shared/Cargo.toml ./shared/Cargo.toml
COPY client/Cargo.toml ./client/Cargo.toml
COPY server/src ./server/src
COPY shared/src ./shared/src
COPY client/src ./client/src
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin terma-server

# Runtime stage
FROM debian:bookworm-slim

# Install required runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/terma-server /app/terma-server

# Expose port
EXPOSE 8080

# Set default bind address
ENV BIND_ADDR=0.0.0.0:8080

# Run the binary
CMD ["/app/terma-server"]
