# Base image with Rust toolchain for building
FROM rust:1.81 as builder

# Create a new empty shell project
WORKDIR /app

# Copy over the manifests
COPY Cargo.toml Cargo.lock ./
COPY MyHealthGuide-domain/Cargo.toml ./MyHealthGuide-domain/
COPY MyHealthGuide-data/Cargo.toml ./MyHealthGuide-data/
COPY MyHealthGuide-api/Cargo.toml ./MyHealthGuide-api/

# Create dummy source files to build dependencies
RUN mkdir -p MyHealthGuide-domain/src && \
    echo "fn main() {}" > MyHealthGuide-domain/src/lib.rs && \
    mkdir -p MyHealthGuide-data/src && \
    echo "fn main() {}" > MyHealthGuide-data/src/lib.rs && \
    mkdir -p MyHealthGuide-api/src/bin && \
    echo "fn main() {}" > MyHealthGuide-api/src/bin/main.rs && \
    mkdir -p MyHealthGuide-api/src/lib && \
    echo "pub fn dummy() {}" > MyHealthGuide-api/src/lib.rs && \
    cargo build --release

# Remove the dummy source files
RUN rm -rf MyHealthGuide-api/src MyHealthGuide-domain/src MyHealthGuide-data/src

# Copy the actual source code
COPY MyHealthGuide-domain/src ./MyHealthGuide-domain/src/
COPY MyHealthGuide-data/src ./MyHealthGuide-data/src/
COPY MyHealthGuide-api/src ./MyHealthGuide-api/src/

# Build the application
RUN cargo build --release

# Final stage - create a smaller image for runtime
FROM debian:bookworm-slim

# Install curl for healthcheck
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN adduser --disabled-password --gecos "" appuser

# Create data directory with proper permissions
WORKDIR /app
RUN mkdir -p /app/data && \
    chown -R appuser:appuser /app && \
    chmod 777 /app/data

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/MyHealthGuide-api /app/MyHealthGuide-api
RUN chown appuser:appuser /app/MyHealthGuide-api && chmod +x /app/MyHealthGuide-api

# Switch to non-root user
USER appuser

# Set environment variables with defaults
ENV RUST_LOG=info \
    PORT=3000 \
    DB_TYPE=sqlite \
    DB_SQLITE_PATH=/app/data/health_guide.db \
    DATA_DIR=/app/data

# Expose the API port
EXPOSE 3000

# Run the application
CMD ["/app/MyHealthGuide-api"] 