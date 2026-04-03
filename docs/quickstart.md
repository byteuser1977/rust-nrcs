# Quick Start Guide

This guide helps you get a local NRCS node running in minutes.

## Docker Method (Recommended)

### 1. Clone and start

```bash
git clone https://github.com/yourorg/rust-nrcs.git
cd rust-nrcs

# Start all services
docker-compose -f docker/docker-compose.yml up -d
```

### 2. Verify startup

```bash
# Check all services are running
docker-compose -f docker/docker-compose.yml ps

# View logs
docker-compose -f docker/docker-compose.yml logs -f

# Check backend health
curl http://localhost:17976/health
# Expected: OK
```

### 3. Access the application

- **Frontend**: http://localhost
- **Backend API**: http://localhost:17976
- **API Docs**: http://localhost:17976/api/docs
- **Database**: localhost:5432 (user: nrcs, password from .env)
- **Metrics**: http://localhost:9090 (Prometheus)
- **Dashboards**: http://localhost:3000 (Grafana, admin/admin)

## Native Build (Development)

### Backend

#### Prerequisites

- Rust 1.75+ (with cargo)
- PostgreSQL 16+
- libssl-dev (Debian/Ubuntu) or equivalent

#### Build and run

```bash
# Clone and enter project
cd rust-nrcs/crates

# Build all crates
cargo build --release

# Run database migrations (if any)
# sqlx migrate run (if using sqlx CLI)

# Start node
cargo run --bin nrcs-node -- --config ../config/default.toml
```

### Frontend

#### Prerequisites

- Node.js 20+
- npm or yarn

#### Build and run

```bash
cd frontend
npm install

# Development server with hot reload
npm run dev

# Or build for production
npm run build
npm run preview
```

## Configuration

### Environment variables

Create a `.env` file in project root:

```bash
POSTGRES_PASSWORD=your-secure-password
JWT_SECRET=$(openssl rand -base64 64)
RUST_LOG=info
```

### Configuration file

Edit `config/default.toml` for local development or `config/production.toml` for production.

Key settings:

```toml
[database]
host = "localhost"
port = 5432
username = "nrcs"
password = "your-password"

[server]
http_listen = "0.0.0.0:17976"

[security]
jwt_secret = "your-64-char-secret"
```

## Testing

### Unit tests (Rust)

```bash
cargo test --all-features --workspace
```

### Unit tests (Vue)

```bash
cd frontend
npm run test

# With UI
npm run test:ui

# With coverage
npm run test:coverage
```

### Integration tests

```bash
# Ensure services are running
docker-compose -f docker/docker-compose.yml up -d

cd tests/integration
npm install  # if using Playwright/Cypress
npx playwright test
```

## Next Steps

- Configure your network parameters in `config/default.toml`
- Set up seed nodes for P2P networking
- Enable TLS for production
- Configure monitoring (Prometheus + Grafana)
- Read the [Architecture Guide](../docs/architecture.md)

## Common Issues

| Problem | Solution |
|---------|----------|
| Database connection refused | Ensure PostgreSQL is running and credentials match |
| Port 17976 already in use | Change `http_listen` to another port |
| Permission denied on `/data` | Ensure user has write access to data directory |
| Out of memory | Increase system RAM or swap; Rust binaries need ~2GB+ |

## Getting Help

- [Documentation](../docs/)
- [Troubleshooting](../docs/ops/troubleshooting.md)
- [Community Forum](https://community.nrcs.network)
