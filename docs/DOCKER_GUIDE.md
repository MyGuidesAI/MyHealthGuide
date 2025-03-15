# Docker Guide for MyHealthGuide API

This guide explains how to use Docker with the MyHealthGuide API for both development and production environments.

## Container Options

We provide multiple Docker configurations:

1. **Development Container** (`Dockerfile.dev`) - Optimized for local development with hot reloading
2. **Production Container** (`Dockerfile`) - Multi-stage build optimized for deployment
3. **Helper Scripts** - Simplified commands for common operations

## Quick Start

The easiest way to start is using our helper script:

```bash
# Make the script executable (only needed once)
chmod +x docker-compose-up.sh

# Start development environment (default)
./docker-compose-up.sh

# Start production environment
./docker-compose-up.sh prod

# Start both environments
./docker-compose-up.sh both

# Show help
./docker-compose-up.sh --help
```

### Manual Commands

If you prefer using Docker Compose directly:

#### Development

```bash
# Build and start the development container
docker compose up dev

# Or build and start in detached mode
docker compose up -d dev

# View logs when running in detached mode
docker compose logs -f dev
```

#### Production

```bash
# Build and start the production container
docker compose up prod

# Or build and start in detached mode
docker compose up -d prod

# View logs when running in detached mode
docker compose logs -f prod
```

## Configuration

### Environment Variables

The container supports these environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Log level (debug, info, warn, error) | `info` in prod, `debug` in dev |
| `PORT` | Port to run the API on | `3000` |
| `DB_TYPE` | Database type (sqlite, mysql, postgres) | `sqlite` |
| `DB_SQLITE_PATH` | Path to SQLite database file | `/app/data/health_guide.db` |
| `DATA_DIR` | Directory for storing data | `/app/data` |
| `OIDC_CLIENT_ID` | OIDC client ID | `your_client_id` |
| `OIDC_CLIENT_SECRET` | OIDC client secret | `your_client_secret` |
| `OIDC_ISSUER_URL` | OIDC issuer URL | `https://accounts.google.com` |
| `OIDC_REDIRECT_URL` | OIDC redirect URL | `http://localhost:3000/api/v1/auth/oidc/callback` |
| `OIDC_SESSION_TIMEOUT` | OIDC session timeout in seconds | `600` |

### Port Configuration

- Development environment: `http://localhost:3000`
- Production environment: `http://localhost:3001`

## Development vs Production Builds

### Development Container (`Dockerfile.dev`)

- Uses Rust 1.76 with the full development toolchain
- Mounts source code as a volume for hot reloading
- Compiles code on the fly with `cargo run`
- Includes development tools like Vim for debugging
- Uses debug log level by default

### Production Container (`Dockerfile`)

- Uses multi-stage build to minimize image size
- First stage: Builds the application with Rust 1.76
- Second stage: Uses a minimal Debian base image
- Pre-compiles the application for faster startup
- Includes only the executable and necessary runtime dependencies
- Uses info log level by default

## Volume Persistence

Data is persisted using Docker volumes:

- Development: `myhealth-api-data-dev`
- Production: `myhealth-api-data-prod`
- Cargo cache: `myhealth-api-cargo-cache`

## Performance Optimization

### For Development

- Volume mounts use the `:delegated` flag for better performance on macOS
- Cargo registry is cached using a persistent volume
- Hot reloading for faster development iterations

### For Production

- Multi-stage builds minimize image size
- Dependencies are pre-compiled for faster subsequent builds
- Only the compiled binary and runtime dependencies are included in the final image

## Debugging

To access container for debugging:

```bash
# Development container
docker compose exec dev /bin/bash

# Production container
docker compose exec prod /bin/bash
```

## Health Checks

Both containers include health checks at:
- `http://localhost:3000/api/v1/health` (Development)
- `http://localhost:3001/api/v1/health` (Production)

Monitor container health with:
```bash
docker compose ps
```

## Common Issues

### Permission Problems

If you encounter permission issues with mounted volumes:
```bash
# Fix permissions for data volume
sudo chown -R 1000:1000 ./data
```

### Build Failures on Apple Silicon

On Apple Silicon (M1/M2), you might need to force amd64 platform:
```bash
# Force amd64 platform build
DOCKER_DEFAULT_PLATFORM=linux/amd64 docker compose build
```

### Slow Builds

The first build may be slow as it needs to download and compile all dependencies. Subsequent builds will be faster due to cargo caching.

## Recommended Practices

1. Use the development container for coding and testing
2. Use the production container for staging and production
3. Keep volumes backed up for data persistence
4. Regularly update dependencies and base images
5. Use the helper script for common operations 