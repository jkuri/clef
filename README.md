# Clef

> Unlock private NPM registry, sealed in Rust.

[![API](https://github.com/jkuri/clef/actions/workflows/clef.yml/badge.svg)](https://github.com/jkuri/clef/actions/workflows/clef.yml)

A high-performance private npm registry built with Rust and Rocket. Clef provides secure package hosting with intelligent upstream proxying, authentication, and caching—delivering the reliability and speed your development workflow demands.

## Features

- 🚀 **High Performance** - Built with Rust for maximum speed and memory safety
- 🔒 **Secure Authentication** - Token-based auth with user management
- 📦 **Package Publishing** - Full npm publish/install workflow support
- 🌐 **Upstream Proxying** - Seamless fallback to public registries
- ⚡ **Smart Caching** - Intelligent metadata and tarball caching
- 🎯 **Scoped Packages** - Complete support for @scope/package naming
- 🔄 **Multi-Client Support** - Works with npm, yarn, pnpm

## Quick Start

### Installation

Make sure you have rust and Node.JS installed on your system.

```bash
# Clone and build
git clone https://github.com/jkuri/clef.git
cd clef
make build-release

# Run with default settings
RUST_LOG=info ./target/release/clef
```

### Configuration

Set environment variables or use defaults:

```bash
export CLEF_HOST=127.0.0.1          # Default: 127.0.0.1
export CLEF_PORT=8000               # Default: 8000
export CLEF_UPSTREAM_REGISTRY=https://registry.npmjs.org  # Default
export CLEF_DATABASE_URL=./data/clef.db  # Default
```

### Docker

Run with Docker.

```sh
docker run -it --rm -p 8000:8000 -v ./data:/app/data jkuri/clef:latest
```

## Usage Examples

### npm

```bash
# Configure registry
npm config set registry http://localhost:8000/registry

# Login (creates account if needed)
npm login --registry http://localhost:8000/registry

# Publish package
npm publish

# Install packages
npm install my-private-package
npm install @myorg/my-scoped-package
```

### yarn

```bash
# Configure registry
yarn config set registry http://localhost:8000/registry

# Login
yarn login --registry http://localhost:8000/registry

# Publish
yarn publish

# Install
yarn add my-private-package
yarn add @myorg/my-scoped-package
```

### pnpm

```bash
# Configure registry
pnpm config set registry http://localhost:8000/registry

# Login
pnpm login --registry http://localhost:8000/registry

# Publish
pnpm publish

# Install
pnpm add my-private-package
pnpm add @myorg/my-scoped-package
```

### Per-Project Configuration

Create `.npmrc` in your project root:

```ini
registry=http://localhost:8000/registry
```

## Development

```bash
# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run

# Database migrations
diesel migration run
```

## Preview

![Screenshot](https://github-production-user-asset-6210df.s3.amazonaws.com/1796022/468427688-b60eccdd-bdc8-4779-be0d-c30638d94263.png?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAVCODYLSA53PQK4ZA%2F20250721%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20250721T000955Z&X-Amz-Expires=300&X-Amz-Signature=cf2f5f3f33d60a2426b82650b6df0233cd26cf23e67f5b924575c5d24c321be8&X-Amz-SignedHeaders=host)

![Screenshot](https://github-production-user-asset-6210df.s3.amazonaws.com/1796022/468427722-82a59819-b845-457f-bd4f-90852703ff23.png?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAVCODYLSA53PQK4ZA%2F20250721%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20250721T001011Z&X-Amz-Expires=300&X-Amz-Signature=75af8a48a3309a986ad13e126d39875c1c6e58ba04207d13ebdb3056b1c12aa6&X-Amz-SignedHeaders=host)

![Screenshot](https://github-production-user-asset-6210df.s3.amazonaws.com/1796022/468427735-a5dc275f-3f3c-4666-9a9c-7eec04f7ed55.png?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAVCODYLSA53PQK4ZA%2F20250721%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20250721T001022Z&X-Amz-Expires=300&X-Amz-Signature=fefbcbda462ea4d32a9165a651081582d120781465d2f501c290ad91d17ddf3e&X-Amz-SignedHeaders=host)

## License

MIT License - see [LICENSE](LICENSE) for details.
