# Build stage
FROM rust:1.91-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests and create dummy files to cache dependencies
COPY Cargo.toml Cargo.lock ./
COPY server/Cargo.toml ./server/Cargo.toml
COPY shared/Cargo.toml ./shared/Cargo.toml
COPY client/Cargo.toml ./client/Cargo.toml

RUN mkdir -p server/src shared/src client/src && \
    echo "fn main() {}" > server/src/main.rs && \
    echo "" > shared/src/lib.rs && \
    echo "fn main() {}" > client/src/main.rs

# Build dependencies only (this layer will be cached)
RUN cargo build --release --bin terma-server && rm -rf src server/src shared/src client/src

# Copy actual source code
COPY shared ./shared
COPY server ./server
COPY client ./client

# Touch to ensure rebuild
RUN touch server/src/main.rs

# Build the application
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
