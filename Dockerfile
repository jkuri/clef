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

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/clef /usr/local/bin/clef

# Set default environment variables
ENV CLEF_HOST=0.0.0.0
ENV CLEF_PORT=8000
ENV CLEF_UPSTREAM_REGISTRY=https://registry.npmjs.org
ENV RUST_LOG=info

# Expose the port
EXPOSE 8000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/ || exit 1

# Run the application
CMD ["clef"]
