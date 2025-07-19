# Using PNRS with NPM Clients

This document shows how to use PNRS with various npm clients and tools.

## NPM

### Global Configuration

```bash
# Set PNRS as default registry
npm config set registry http://localhost:8000/registry

# Verify configuration
npm config get registry

# Reset to default
npm config set registry https://registry.npmjs.org
```

### Per-Project Configuration

```bash
# Create .npmrc in your project
echo "registry=http://localhost:8000/registry" > .npmrc

# Or use npm config
npm config set registry http://localhost:8000/registry --location=project
```

### One-time Usage

```bash
# Install with custom registry
npm install express --registry http://localhost:8000/registry

# Publish (will fail - PNRS is proxy-only)
npm publish --registry http://localhost:8000/registry
```

## Yarn

### Global Configuration

```bash
# Set PNRS as default registry
yarn config set registry http://localhost:8000/registry

# Verify configuration
yarn config get registry

# Reset to default
yarn config set registry https://registry.yarnpkg.com
```

### Per-Project Configuration

```bash
# Create .yarnrc.yml for Yarn 2+
echo 'npmRegistryServer: "http://localhost:8000/registry"' > .yarnrc.yml

# Or .yarnrc for Yarn 1.x
echo 'registry "http://localhost:8000/registry"' > .yarnrc
```

### One-time Usage

```bash
# Install with custom registry
yarn add express --registry http://localhost:8000/registry
```

## PNPM

### Global Configuration

```bash
# Set PNRS as default registry
pnpm config set registry http://localhost:8000/registry

# Verify configuration
pnpm config get registry

# Reset to default
pnpm config set registry https://registry.npmjs.org
```

### Per-Project Configuration

```bash
# Create .npmrc in your project
echo "registry=http://localhost:8000/registry" > .npmrc
```

### One-time Usage

```bash
# Install with custom registry
pnpm add express --registry http://localhost:8000/registry
```

## Docker Usage

### Using PNRS in Docker builds

```dockerfile
FROM node:18-alpine

# Set npm registry to use PNRS
RUN npm config set registry http://pnrs:8000/registry

WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production

COPY . .
CMD ["npm", "start"]
```

### Docker Compose with PNRS

```yaml
version: "3.8"
services:
  pnrs:
    build: .
    ports:
      - "8000:8000"

  app:
    image: node:18-alpine
    depends_on:
      - pnrs
    environment:
      - npm_config_registry=http://pnrs:8000/registry
    volumes:
      - ./app:/app
    working_dir: /app
    command: sh -c "npm install && npm start"
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Build with PNRS
on: [push]

jobs:
  build:
    runs-on: ubuntu-latest

    services:
      pnrs:
        image: pnrs:latest
        ports:
          - 8000:8000

    steps:
      - uses: actions/checkout@v3

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: "18"
          registry-url: "http://localhost:8000/registry"

      - name: Install dependencies
        run: npm ci
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

### GitLab CI

```yaml
image: node:18

services:
  - name: pnrs:latest
    alias: pnrs

variables:
  npm_config_registry: "http://pnrs:8000"

before_script:
  - npm config set registry http://pnrs:8000

test:
  script:
    - npm ci
    - npm test
```

## Advanced Usage

### Scoped Packages

```bash
# Configure scoped packages to use PNRS
npm config set @mycompany:registry http://localhost:8000

# Install scoped package
npm install @mycompany/my-package
```

### Authentication (Pass-through)

```bash
# Set auth token (passed through to upstream)
npm config set //localhost:8000/:_authToken $NPM_TOKEN

# Or use .npmrc
echo "//localhost:8000/:_authToken=${NPM_TOKEN}" >> .npmrc
```

### Multiple Registries

```bash
# Use different registries for different scopes
npm config set registry http://localhost:8000
npm config set @private:registry https://private-registry.com
npm config set @public:registry https://registry.npmjs.org
```

## Troubleshooting

### Common Issues

1. **Connection refused**

   ```bash
   # Check if PNRS is running
   curl http://localhost:8000/

   # Check PNRS logs
   RUST_LOG=debug cargo run
   ```

2. **SSL/TLS errors**

   ```bash
   # Disable SSL verification (not recommended for production)
   npm config set strict-ssl false
   ```

3. **Timeout issues**
   ```bash
   # Increase timeout
   npm config set timeout 60000
   ```

### Health Check

```bash
# Test PNRS health
curl http://localhost:8000/

# Test package resolution
curl http://localhost:8000/registry/express | jq '.name'

# Test tarball download
curl -I http://localhost:8000/registry/express/-/express-4.18.2.tgz
```

## Performance Tips

1. **Use HTTP/2**: Configure a reverse proxy with HTTP/2 support
2. **Enable compression**: Use gzip compression in reverse proxy
3. **Cache headers**: PNRS passes through cache headers from upstream
4. **Connection pooling**: PNRS automatically pools connections to upstream

## Security Considerations

1. **Network access**: Ensure PNRS can reach upstream registry
2. **Firewall rules**: Open only necessary ports
3. **Authentication**: PNRS passes through authentication to upstream
4. **HTTPS**: Use HTTPS in production with reverse proxy
