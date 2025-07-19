# End-to-End Tests for PNRS

This directory contains comprehensive end-to-end tests for the PNRS (Private NPM Registry Server) that test all features using npm, pnpm, and yarn package managers.

## Test Structure

The e2e tests are organized into the following modules:

### Core Functionality Tests

- **`package_management.rs`** - Tests package installation, metadata fetching, version resolution, and tarball downloads
- **`authentication.rs`** - Tests npm login, whoami, user registration, and authentication flows
- **`publishing.rs`** - Tests package publishing workflows including scoped packages and ownership validation
- **`cache_management.rs`** - Tests cache statistics, health checks, cache clearing, and cache behavior
- **`analytics.rs`** - Tests package analytics, popular packages, version tracking, and database features
- **`security.rs`** - Tests security audit endpoints, vulnerability scanning, and advisory proxying

### Advanced Feature Tests

- **`scoped_packages.rs`** - Tests scoped package handling (@scope/package) with all package managers
- **`compatibility.rs`** - Tests interoperability between npm, pnpm, and yarn using the same registry
- **`performance.rs`** - Tests concurrent operations, large package handling, and performance characteristics

## Prerequisites

Before running the e2e tests, ensure you have:

1. **Rust and Cargo** installed
2. **Node.js** installed (for npm)
3. **pnpm** installed (optional, but recommended):

   ```bash
   npm install -g pnpm
   ```

4. **Yarn** installed (optional, but recommended):

   ```bash
   npm install -g yarn
   ```

## Running the Tests

### Run All E2E Tests

```bash
# Run all e2e tests
cargo test --test e2e_tests

# Run with output
cargo test --test e2e_tests -- --nocapture

# Run with debug logging
RUST_LOG=debug cargo test --test e2e_tests -- --nocapture
```

### Run Specific Test Modules

```bash
# Run only package management tests
cargo test --test e2e_tests package_management

# Run only authentication tests
cargo test --test e2e_tests authentication

# Run only cache management tests
cargo test --test e2e_tests cache_management
```

### Run Individual Tests

```bash
# Run a specific test
cargo test --test e2e_tests test_package_installation_npm

# Run tests matching a pattern
cargo test --test e2e_tests scoped_package
```

## Test Configuration

The tests use the following configuration:

- **Test Server**: Each test starts a fresh PNRS server instance on a random port
- **Temporary Directories**: Each test uses isolated temporary directories for cache and database
- **Package Managers**: Tests automatically detect available package managers and skip tests if not available
- **Serial Execution**: Tests run serially to avoid port conflicts and ensure isolation

## Test Features

### Package Manager Support

The tests support all three major Node.js package managers:

- **npm** - The default Node.js package manager
- **pnpm** - Fast, disk space efficient package manager
- **yarn** - Alternative package manager with different features

Tests will automatically skip package manager-specific tests if the tool is not available.

### Test Infrastructure

- **TestServer**: Manages PNRS server lifecycle with random ports and isolated environments
- **TestProject**: Creates isolated npm projects with proper .npmrc configuration
- **ApiClient**: HTTP client for direct API testing with authentication support
- **PackageManager**: Abstraction over npm/pnpm/yarn commands

### Comprehensive Coverage

The tests cover:

- ✅ Package installation and resolution
- ✅ Metadata fetching and caching
- ✅ Tarball downloads and HEAD requests
- ✅ User authentication and authorization
- ✅ Package publishing and ownership
- ✅ Scoped package handling
- ✅ Cache management and statistics
- ✅ Analytics and database features
- ✅ Security audit endpoints
- ✅ Cross-package-manager compatibility
- ✅ Performance and load testing
- ✅ Error handling and edge cases

## Test Data

Tests use real npm packages for realistic scenarios:

- **lodash** - Popular utility library
- **express** - Web framework
- **react** - UI library
- **@types/node** - TypeScript definitions (scoped package)
- **@babel/core** - Babel compiler (scoped package)

## Debugging Tests

### Enable Debug Logging

```bash
RUST_LOG=debug cargo test --test e2e_tests -- --nocapture
```

### Run Single Test with Verbose Output

```bash
cargo test --test e2e_tests test_package_installation_npm -- --nocapture --exact
```

### Check Test Server Logs

The test infrastructure captures server output. Failed tests will display relevant logs.

## Common Issues

### Package Managers Not Available

If you see warnings about package managers not being available:

```bash
# Install pnpm
npm install -g pnpm

# Install yarn
npm install -g yarn
```

### Network Issues

Some tests may fail if:

- No internet connection (tests need to fetch real packages)
- Upstream npm registry is unavailable
- Corporate firewall blocks npm registry access

### Port Conflicts

Tests use random ports, but if you see port binding errors:

- Ensure no other services are using many ports
- Run tests serially (they already do this by default)

## Contributing

When adding new e2e tests:

1. Add tests to the appropriate module (or create a new one)
2. Use the `#[serial]` attribute for tests that need isolation
3. Use the test infrastructure (TestServer, TestProject, ApiClient)
4. Handle cases where package managers might not be available
5. Include both success and failure scenarios
6. Test with multiple package managers when relevant

## Performance Considerations

The e2e tests are comprehensive but can be slow because they:

- Start fresh server instances
- Download real packages from npm registry
- Test multiple package managers
- Include performance and load tests

For faster development cycles, run specific test modules rather than the full suite.
