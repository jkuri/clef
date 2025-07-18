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
| `PNRS_CACHE_DIR`         | `./cache`                    | Cache directory path                 |
| `PNRS_CACHE_MAX_SIZE_MB` | `1024`                       | Maximum cache size in MB             |
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
GET /{package}
```

Fetch complete metadata for a package from upstream registry.

**Example:**

```bash
curl http://localhost:8000/express
```

### Package Version Metadata

```
GET /{package}/{version}
```

Fetch metadata for a specific package version.

**Example:**

```bash
curl http://localhost:8000/express/4.18.2
```

### Package Tarball Download

```
GET /{package}/-/{filename}
HEAD /{package}/-/{filename}
```

Download or check availability of package tarballs.

**Example:**

```bash
# Download tarball
curl http://localhost:8000/lodash/-/lodash-4.17.21.tgz -o lodash.tgz

# Check availability
curl -I http://localhost:8000/lodash/-/lodash-4.17.21.tgz
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
npm config set registry http://localhost:8000

# Or use with specific commands
npm install --registry http://localhost:8000

# Reset to default registry
npm config set registry https://registry.npmjs.org
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_health_check
```

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
