# PNRS - Private NPM Registry Server

A high-performance npm registry proxy server built with Rust and Rocket.rs, providing an alternative to Verdaccio for proxying npm packages.

## Features

- **Package Metadata Proxy**: Fetch package information from upstream npm registry
- **Version-Specific Metadata**: Get metadata for specific package versions
- **Tarball Download Proxy**: Download package tarballs through the proxy
- **Intelligent Caching**: Local filesystem cache for tarballs with **PERMANENT STORAGE** (never deleted)
- **SQLite Database**: Rich metadata storage with Diesel ORM for package tracking and analytics
- **Cache Management**: Built-in cache statistics, health monitoring, and comprehensive analytics
- **Package Analytics**: Popular packages, download counts, version tracking, and usage patterns
- **HEAD Request Support**: Check tarball availability without downloading
- **Configurable Upstream**: Support for custom upstream registries
- **Request Logging**: Comprehensive logging of all proxy requests
- **CORS Support**: Cross-origin resource sharing enabled
- **Environment Configuration**: Configure via environment variables
- **Health Check Endpoint**: Monitor server status

## Architecture

PNRS uses a hybrid storage approach:

- **Filesystem Storage**: Organized directory structure (`data/packages/{package}/{filename}.tgz`)
- **SQLite Database**: Metadata, analytics, and package information (`data/pnrs.db`)
- **Permanent Storage**: Packages are never deleted, ensuring fast access to all previously downloaded packages
- **Rich Metadata**: ETags, file paths, timestamps, access counts, and version tracking

## Quick Start

### Prerequisites

- Rust 1.70+ (with Cargo)
- Internet connection for upstream registry access

### Installation

1. Clone the repository:

```bash
git clone <repository-url>
cd pnrs
```

2. Build the project:

```bash
cargo build --release
```

3. Run the server:

```bash
cargo run
```

The server will start on `http://127.0.0.1:8000` by default.

## Configuration

Configure PNRS using environment variables:

| Variable                 | Default                      | Description                          |
| ------------------------ | ---------------------------- | ------------------------------------ |
| `PNRS_UPSTREAM_REGISTRY` | `https://registry.npmjs.org` | Upstream npm registry URL            |
| `PNRS_HOST`              | `127.0.0.1`                  | Server bind address                  |
| `PNRS_PORT`              | `8000`                       | Server port                          |
| `PNRS_CACHE_ENABLED`     | `true`                       | Enable/disable tarball caching       |
| `PNRS_CACHE_DIR`         | `./data`                     | Cache directory path                 |
| `PNRS_CACHE_TTL_HOURS`   | `24`                         | Cache TTL in hours                   |
| `RUST_LOG`               | -                            | Log level (info, debug, warn, error) |

### Example Configuration

```bash
export PNRS_UPSTREAM_REGISTRY="https://registry.npmjs.org"
export PNRS_HOST="0.0.0.0"
export PNRS_PORT="8080"
export RUST_LOG="info"
cargo run
```

## API Endpoints

### Health Check

```
GET /
```

Returns server status message.

### Package Metadata

```
GET /registry/{package}
```

Fetch complete metadata for a package from upstream registry.

**Example:**

```bash
curl http://localhost:8000/registry/express
```

### Package Version Metadata

```
GET /registry/{package}/{version}
```

Fetch metadata for a specific package version.

**Example:**

```bash
curl http://localhost:8000/registry/express/4.18.2
```

### Package Tarball Download

```
GET /registry/{package}/-/{filename}
HEAD /registry/{package}/-/{filename}
```

Download or check availability of package tarballs.

**Example:**

```bash
# Download tarball
curl http://localhost:8000/registry/lodash/-/lodash-4.17.21.tgz -o lodash.tgz

# Check availability
curl -I http://localhost:8000/registry/lodash/-/lodash-4.17.21.tgz
```

### Cache Management

```
GET /cache/stats
```

Get cache statistics including hit rate, size, and configuration.

**Example:**

```bash
curl http://localhost:8000/cache/stats
```

```
GET /cache/health
```

Check cache health status and usage.

**Example:**

```bash
curl http://localhost:8000/cache/health
```

```
DELETE /cache
```

Clear all cached tarballs.

**Example:**

```bash
curl -X DELETE http://localhost:8000/cache
```

### Package Analytics & Database Features

```
GET /packages
```

List all cached packages with metadata from the database.

**Example:**

```bash
curl http://localhost:8000/packages
```

```
GET /packages/{name}
```

Get all cached versions of a specific package.

**Example:**

```bash
curl http://localhost:8000/packages/express
```

```
GET /packages/popular?limit={n}
```

Get most popular packages by download count.

**Example:**

```bash
curl http://localhost:8000/packages/popular?limit=5
```

```
GET /analytics
```

Get comprehensive cache analytics including popular packages, recent downloads, and hit rates.

**Example:**

```bash
curl http://localhost:8000/analytics
```

## Using with npm

Configure npm to use PNRS as a registry:

```bash
# Set registry for current project
npm config set registry http://localhost:8000/registry

# Or use with specific commands
npm install --registry http://localhost:8000/registry

# Reset to default registry
npm config set registry https://registry.npmjs.org
```

## Development

### Running Tests

PNRS includes comprehensive test suites:

```bash
# Run all tests (unit + integration + e2e quick)
make test

# Run unit tests only
make test-unit

# Run integration tests only
make test-integration

# Run end-to-end tests
make test-e2e-quick    # Quick e2e tests (recommended)
make test-e2e          # Full e2e test suite

# Run specific e2e test modules
make test-e2e-package  # Package management tests
make test-e2e-auth     # Authentication tests
make test-e2e-cache    # Cache management tests
# ... see 'make help' for all modules

# Alternative: use the test script directly
./scripts/run-e2e-tests.sh --quick
./scripts/run-e2e-tests.sh --module package_management
./scripts/run-e2e-tests.sh --help
```

#### End-to-End Tests

The e2e tests comprehensively test PNRS with real npm, pnpm, and yarn package managers:

- ✅ Package installation and resolution
- ✅ Authentication and publishing
- ✅ Cache management and analytics
- ✅ Scoped package handling
- ✅ Security audit endpoints
- ✅ Cross-package-manager compatibility
- ✅ Performance and load testing

See [tests/e2e/README.md](tests/e2e/README.md) for detailed documentation.

### Development Mode

```bash
# Run with debug logging
RUST_LOG=debug cargo run

# Run on different port
PNRS_PORT=3000 cargo run
```

## Architecture

PNRS is built with:

- **Rocket.rs**: Web framework for handling HTTP requests
- **Reqwest**: HTTP client for upstream registry communication
- **Tokio**: Async runtime for concurrent request handling
- **Serde**: JSON serialization/deserialization
- **Env Logger**: Configurable logging

### Request Flow

1. Client makes request to PNRS
2. PNRS logs the request
3. PNRS forwards request to upstream registry
4. PNRS streams response back to client
5. PNRS logs the result

## Performance

PNRS is designed for high performance:

- **Async/Await**: Non-blocking I/O for concurrent requests
- **Streaming**: Large tarballs are streamed without buffering
- **Connection Pooling**: Reuses HTTP connections to upstream
- **Minimal Overhead**: Direct proxy with minimal processing

## Error Handling

PNRS provides detailed error responses:

- **404 Not Found**: Package or version doesn't exist upstream
- **400 Bad Request**: Invalid request or upstream error
- **500 Internal Server Error**: Server-side issues

All errors include descriptive messages for debugging.

## Logging

PNRS logs all requests and responses:

```
[INFO] GET /express curl/8.7.1
[INFO] Fetching metadata for package: express
[INFO] Successfully proxied metadata for package: express
```

Configure log level with `RUST_LOG` environment variable.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Run `cargo test` to ensure tests pass
6. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Comparison with Verdaccio

| Feature          | PNRS                  | Verdaccio     |
| ---------------- | --------------------- | ------------- |
| Language         | Rust                  | Node.js       |
| Memory Usage     | Low                   | Higher        |
| Startup Time     | Fast                  | Slower        |
| Configuration    | Environment Variables | Config Files  |
| Plugin System    | No                    | Yes           |
| Private Packages | Proxy Only            | Full Registry |
| Performance      | High                  | Good          |

PNRS is ideal for simple proxy use cases where high performance and low resource usage are priorities.
