# NRCS (Neo Rapid BlockChain EcoSystem)

[![CI](https://github.com/yourorg/rust-nrcs/actions/workflows/ci.yml/badge.svg)](https://github.com/yourorg/rust-nrcs/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/yourorg/rust-nrcs/branch/main/graph/badge.svg)](https://codecov.io/gh/yourorg/rust-nrcs)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

A high-performance blockchain platform rewritten in Rust with a Vue 3 frontend.

## Features

- **High Performance**: Rust backend with async runtime (Tokio) achieving 500+ TPS
- **Modern Tech Stack**: Rust + Vue 3 + TypeScript + Pinia + Vite
- **Comprehensive Consensus**: Supports both PoW and PoS consensus algorithms
- **Smart Contracts**: WASM-based smart contract support
- **Asset Management**: Native asset issuance and transfer
- **Developer Friendly**: Complete API documentation and SDK support
- **Docker Ready**: One-command deployment with Docker Compose

## Quick Start

### Prerequisites

- Docker & Docker Compose
- Node.js 18+ (for development)
- Rust 1.75+ (for development)
- PostgreSQL 16+ (if running without Docker)

### Production Deployment (Docker)

```bash
# Clone repository
git clone https://github.com/yourorg/rust-nrcs.git
cd rust-nrcs

# Copy environment configuration
cp config/production.toml.example config/production.toml
# Edit config/production.tol and set JWT_SECRET, POSTGRES_PASSWORD

# Start all services
docker-compose -f docker/docker-compose.yml up -d

# Check status
docker-compose -f docker/docker-compose.yml ps
docker-compose -f docker/docker-compose.yml logs -f

# Access services
# - Frontend: http://localhost
# - Backend API: http://localhost:17976
# - API Docs: http://localhost:17976/api/docs
# - Prometheus: http://localhost:9090
# - Grafana: http://localhost:3000 (admin/admin)
```

### Development Setup

#### Backend

```bash
cd crates
cargo check
cargo test --all-features
cargo run --bin nrcs-node -- --config ../config/default.toml
```

#### Frontend

```bash
cd frontend
npm install
npm run dev
```

Open http://localhost:5173 in your browser.

## Project Structure

```
rust-nrcs/
├── crates/                 # Rust workspace members
│   ├── blockchain-types/  # Core data structures (blocks, txs, accounts)
│   ├── crypto/            # Cryptographic primitives (Ed25519, hashing)
│   ├── consensus/         # PoW and PoS consensus engines
│   ├── orm/               # SQLx ORM layer and models
│   ├── tx-engine/         # Transaction validation and execution
│   ├── account/           # Account management and balance operations
│   ├── http-api/          # REST API server (Axum)
│   └── node/              # Full node implementation
├── frontend/              # Vue 3 application
│   ├── src/
│   │   ├── views/         # Pages (Login, Dashboard, Transactions)
│   │   ├── stores/        # Pinia stores (account, transaction)
│   │   ├── components/    # Reusable UI components
│   │   └── api/           # API client (axios)
│   └── tests/             # Unit and E2E tests
├── docker/                # Docker configuration
│   ├── backend.Dockerfile
│   ├── frontend.Dockerfile
│   ├── docker-compose.yml
│   └── nginx.conf
├── config/                # Configuration files
│   ├── default.toml
│   ├── production.toml
│   └── production.toml.example
├── docs/                  # Documentation
│   ├── quickstart.md
│   ├── deployment.md
│   ├── monitoring.md
│   ├── migration-guide.md
│   └── ops/
├── tests/                 # Integration and performance tests
│   ├── integration/
│   └── performance/
└── .github/workflows/     # CI/CD pipelines
```

## API Reference

### Authentication

All protected endpoints require a Bearer token in Authorization header:

```
Authorization: Bearer <jwt_token>
```

### Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/auth/login` | User login |
| GET | `/api/v1/accounts/:address` | Get account balance |
| POST | `/api/v1/transactions` | Submit transaction |
| GET | `/api/v1/blocks/latest` | Get latest block |
| GET | `/api/v1/blocks/:height` | Get block by height |
| GET | `/api/v1/transactions/:txid` | Get transaction details |
| GET | `/api/v1/health` | Health check |

Full API documentation available at `/api/docs` when Swagger is enabled.

## Testing

### Run all tests

```bash
# Backend
cargo test --all-features --workspace

# Frontend
cd frontend
npm run test

# Coverage
cargo tarpaulin --workspace --outXml
cd frontend && npm run test:coverage
```

### Integration tests

```bash
# Ensure docker-compose is running
docker-compose -f docker/docker-compose.yml up -d

cd tests/integration
# Run with your test runner (e.g., Playwright, Cypress)
```

### Performance tests

```bash
# Using k6
k6 run --vus 100 --duration 60s tests/performance/load_test.k6.js
```

## Docker Deployment

### One-command startup

```bash
docker-compose -f docker/docker-compose.yml up -d
```

### Environment variables

Set these in `.env` file or shell environment:

- `POSTGRES_PASSWORD` - Database password
- `JWT_SECRET` - JWT signing secret (min 32 chars)
- `RUST_LOG` - Log level (info, debug, trace)
- `P2P_SEEDS` - Comma-separated seed nodes

### Image tags

- `ghcr.io/yourorg/rust-nrcs/backend:latest`
- `ghcr.io/yourorg/rust-nrcs/frontend:latest`
- `ghcr.io/yourorg/rust-nrcs/backend:v0.1.0`
- `ghcr.io/yourorg/rust-nrcs/frontend:v0.1.0`

## Monitoring

- **Prometheus metrics** exposed at `/metrics`
- **Grafana dashboards** available at http://localhost:3000

Key metrics:
- `nrcs_transactions_per_second`
- `nrcs_block_height`
- `nrcs_peer_count`
- `nrcs_mempool_size`
- `http_requests_total` (by endpoint, status)

## Performance

Benchmark targets (tested on 8-core VM):

- **Transaction throughput**: ≥ 500 TPS
- **P95 read latency**: < 200ms
- **P99 read latency**: < 500ms
- **Block propagation**: < 1 second

Performance test scripts in `tests/performance/`.

## Troubleshooting

See [docs/ops/troubleshooting.md](./docs/ops/troubleshooting.md) for common issues.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make changes and add tests
4. Ensure CI passes
5. Submit a Pull Request

## License

Apache-2.0

## Support

- Documentation: https://docs.nrcs.network
- Community: https://community.nrcs.network
- Issues: https://github.com/yourorg/rust-nrcs/issues
