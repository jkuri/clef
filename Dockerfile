# Build stage
FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install CA certificates for HTTPS requests
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -r -s /bin/false pnrs

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/pnrs /usr/local/bin/pnrs

# Change ownership to pnrs user
RUN chown pnrs:pnrs /usr/local/bin/pnrs

# Switch to non-root user
USER pnrs

# Set default environment variables
ENV PNRS_HOST=0.0.0.0
ENV PNRS_PORT=8000
ENV PNRS_UPSTREAM_REGISTRY=https://registry.npmjs.org
ENV RUST_LOG=info

# Expose the port
EXPOSE 8000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/ || exit 1

# Run the application
CMD ["pnrs"]
