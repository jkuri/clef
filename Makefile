# PNRS Makefile
# Provides convenient commands for building, testing, and running PNRS

.PHONY: help build test test-unit test-integration test-e2e test-e2e-quick test-all clean run dev install-deps check lint format

# Default target
help:
	@echo "PNRS - Private NPM Registry Server"
	@echo "=================================="
	@echo ""
	@echo "Available targets:"
	@echo "  build           Build the project"
	@echo "  run             Run PNRS server"
	@echo "  dev             Run PNRS server in development mode with debug logging"
	@echo ""
	@echo "Testing:"
	@echo "  test            Run all tests (unit + integration + e2e)"
	@echo "  test-unit       Run unit tests only"
	@echo "  test-integration Run integration tests only (fastest)"
	@echo "  test-e2e        Run all end-to-end tests"
	@echo "  test-e2e-quick  Run quick end-to-end tests (optimized)"
	@echo "  test-integration-fast Run integration tests via script"
	@echo ""
	@echo "Development:"
	@echo "  install-deps    Install development dependencies (pnpm, yarn)"
	@echo "  check           Run cargo check"
	@echo "  lint            Run clippy linter"
	@echo "  format          Format code with rustfmt"
	@echo "  clean           Clean build artifacts"
	@echo ""
	@echo "E2E Test Modules:"
	@echo "  test-e2e-package        Package management tests"
	@echo "  test-e2e-auth          Authentication tests"
	@echo "  test-e2e-publish       Publishing tests"
	@echo "  test-e2e-cache         Cache management tests"
	@echo "  test-e2e-analytics     Analytics tests (optimized - no recompilation)"
	@echo "  test-e2e-security      Security tests"
	@echo "  test-e2e-scoped        Scoped package tests"
	@echo "  test-e2e-compat        Cross-manager compatibility tests"
	@echo "  test-e2e-perf          Performance tests"

# Build targets
build:
	@echo "Building PNRS..."
	cargo build

build-release:
	@echo "Building PNRS (release mode)..."
	cargo build --release

# Run targets
run:
	@echo "Starting PNRS server..."
	cargo run

dev:
	@echo "Starting PNRS server in development mode..."
	RUST_LOG=debug cargo run

# Test targets
test: test-unit test-integration test-e2e-quick
	@echo "All tests completed!"

test-unit:
	@echo "Running unit tests..."
	cargo test --lib

test-integration:
	@echo "Running integration tests (fastest)..."
	cargo test --test integration_tests

test-integration-fast:
	@echo "Running integration tests with optimized script..."
	./scripts/run-e2e-tests.sh --integration

test-e2e:
	@echo "Running all end-to-end tests..."
	./scripts/run-e2e-tests.sh --all

test-e2e-quick:
	@echo "Running quick end-to-end tests (optimized - builds once)..."
	./scripts/run-e2e-tests.sh --quick

test-all: test-unit test-integration test-e2e
	@echo "All tests (including full e2e suite) completed!"

# Individual E2E test modules
test-e2e-package:
	./scripts/run-e2e-tests.sh --module package_management

test-e2e-auth:
	./scripts/run-e2e-tests.sh --module authentication

test-e2e-publish:
	./scripts/run-e2e-tests.sh --module publishing

test-e2e-cache:
	./scripts/run-e2e-tests.sh --module cache_management

test-e2e-analytics:
	@echo "Running analytics tests (optimized - builds once, reuses binary)..."
	./scripts/run-e2e-tests.sh --module analytics

test-e2e-security:
	./scripts/run-e2e-tests.sh --module security

test-e2e-scoped:
	./scripts/run-e2e-tests.sh --module scoped_packages

test-e2e-compat:
	./scripts/run-e2e-tests.sh --module compatibility

test-e2e-perf:
	./scripts/run-e2e-tests.sh --module performance

# Development targets
install-deps:
	@echo "Installing development dependencies..."
	@if command -v npm >/dev/null 2>&1; then \
		echo "Installing pnpm..."; \
		npm install -g pnpm || echo "Failed to install pnpm (may need sudo)"; \
		echo "Installing yarn..."; \
		npm install -g yarn || echo "Failed to install yarn (may need sudo)"; \
	else \
		echo "npm not found. Please install Node.js first."; \
	fi

check:
	@echo "Running cargo check..."
	cargo check

lint:
	@echo "Running clippy linter..."
	cargo clippy -- -D warnings

format:
	@echo "Formatting code..."
	cargo fmt

format-check:
	@echo "Checking code formatting..."
	cargo fmt -- --check

# Utility targets
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	@echo "Cleaning test cache directories..."
	rm -rf ./test_cache_*

# Docker targets
docker-build:
	@echo "Building Docker image..."
	docker build -t pnrs .

docker-run:
	@echo "Running PNRS in Docker..."
	docker run -p 8000:8000 pnrs

# Database targets
db-reset:
	@echo "Resetting database..."
	rm -f data/pnrs.db
	cargo run -- --reset-db || true

# Benchmark targets
benchmark:
	@echo "Running benchmarks..."
	./scripts/benchmark.sh

# CI targets
ci-test: check lint format-check test-unit test-integration test-e2e-quick
	@echo "CI tests completed!"

ci-test-full: check lint format-check test-all
	@echo "Full CI tests completed!"

# Help for specific commands
help-e2e:
	@echo "End-to-End Test Commands:"
	@echo "========================"
	@echo ""
	@echo "Quick tests (recommended for development):"
	@echo "  make test-integration        # Fastest (in-process)"
	@echo "  make test-integration-fast   # Fast (via script)"
	@echo "  make test-e2e-quick          # Quick E2E (optimized)"
	@echo ""
	@echo "Full test suite (comprehensive, optimized):"
	@echo "  make test-e2e                # All E2E tests (builds once)"
	@echo ""
	@echo "Individual test modules:"
	@echo "  make test-e2e-package    # Package management"
	@echo "  make test-e2e-auth       # Authentication"
	@echo "  make test-e2e-publish    # Publishing"
	@echo "  make test-e2e-cache      # Cache management"
	@echo "  make test-e2e-analytics  # Analytics (optimized)"
	@echo "  make test-e2e-security   # Security"
	@echo "  make test-e2e-scoped     # Scoped packages"
	@echo "  make test-e2e-compat     # Cross-manager compatibility"
	@echo "  make test-e2e-perf       # Performance"
	@echo ""
	@echo "Debug mode:"
	@echo "  RUST_LOG=debug make test-e2e-quick"

help-dev:
	@echo "Development Commands:"
	@echo "===================="
	@echo ""
	@echo "Setup:"
	@echo "  make install-deps    # Install pnpm and yarn"
	@echo ""
	@echo "Development cycle (fastest to slowest):"
	@echo "  make check               # Quick syntax check"
	@echo "  make test-unit           # Fast unit tests"
	@echo "  make test-integration    # Integration tests (fastest)"
	@echo "  make test-e2e-quick      # Core E2E tests (optimized)"
	@echo "  make dev                 # Run with debug logging"
	@echo ""
	@echo "Before committing:"
	@echo "  make lint           # Check code quality"
	@echo "  make format         # Format code"
	@echo "  make test           # Run all tests"
