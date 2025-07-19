# Clef

> Unlock private NPM registry, sealed in Rust.

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

```bash
# Clone and build
git clone https://github.com/jkuri/clef.git
cd clef
cargo build --release

# Run with default settings
./target/release/clef
```

### Configuration

Set environment variables or use defaults:

```bash
export CLEF_HOST=127.0.0.1          # Default: 127.0.0.1
export CLEF_PORT=8000               # Default: 8000
export CLEF_UPSTREAM_REGISTRY=https://registry.npmjs.org  # Default
export CLEF_DATABASE_URL=./data/clef.db  # Default
```

## Usage Examples

### npm

```bash
# Configure registry
npm config set registry http://localhost:8000/registry

# Login (creates account if needed)
npm login --registry http://localhost:8000/registry

# For scoped packages
npm login --registry http://localhost:8000/registry --scope=@myorg

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
@myorg:registry=http://localhost:8000/registry
//localhost:8000/registry/:_authToken=${NPM_TOKEN}
```

## API Endpoints

| Method   | Endpoint                                     | Description          |
| -------- | -------------------------------------------- | -------------------- |
| `PUT`    | `/registry/-/user/org.couchdb.user:username` | Login/Register       |
| `GET`    | `/registry/-/whoami`                         | Get current user     |
| `DELETE` | `/registry/-/user/token/{token}`             | Logout               |
| `PUT`    | `/registry/{package}`                        | Publish package      |
| `GET`    | `/registry/{package}`                        | Get package metadata |
| `GET`    | `/registry/{package}/-/{filename}`           | Download tarball     |

## Development

```bash
# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run

# Database migrations
diesel migration run
```

## License

MIT License - see [LICENSE](LICENSE) for details.
