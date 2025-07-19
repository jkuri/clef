# Build stage
FROM rust:1.88-alpine AS builder

RUN apk add --no-cache musl-dev sqlite-dev sqlite-static openssl-dev openssl-libs-static


WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY diesel.toml ./
COPY migrations/ ./migrations/

# Build the application
RUN cargo build --release

# Runtime stage
FROM alpine:latest

# Install CA certificates for HTTPS requests
RUN apk add --no-cache sqlite ca-certificates

# Create a non-root user
RUN adduser -D pnrs

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
