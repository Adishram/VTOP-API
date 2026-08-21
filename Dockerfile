# Stage 1: Build the Rust application
FROM rust:1.80-slim-bookworm AS builder

# Install build dependencies (pkg-config and libssl-dev are required for reqwest/openssl)
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy dependency manifests first to leverage Docker layer caching
COPY Cargo.toml Cargo.lock ./
# Create a dummy src/main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Now copy the actual source code and rebuild
COPY src ./src
# We need to touch main.rs to force cargo to recompile it instead of using the dummy cached version
RUN touch src/main.rs && cargo build --release

# Stage 2: Create the minimal runtime image
FROM debian:bookworm-slim

# Install runtime dependencies (OpenSSL is needed for reqwest's TLS)
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/vtop-api /usr/local/bin/vtop-api

# Create a non-root user for security
RUN useradd -m appuser && chown -R appuser /app
USER appuser

EXPOSE 8080

ENTRYPOINT ["vtop-api"]
