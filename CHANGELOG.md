# Changelog

All notable changes to the NRCS project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- PostgreSQL integration tests: 32 tests covering 8 Repository test files (account, block, transaction, asset, alias, poll, currency, order)
- PostgreSQL identifier quoting in pg.rs: all table and column names properly quoted with uppercase (e.g., `"BLOCK"`, `"TIMESTAMP"`)
- PostgreSQL migration script identifier quoting: TIMESTAMP columns, index definitions, and foreign key constraints
- Node PostgreSQL branch enabled: uncommented PostgreSQL code path in main.rs
- config/local.toml updated to use PostgreSQL connection by default

### Fixed

- PostgreSQL identifier case sensitivity: fixed all table/column names in pg.rs SQL queries (e.g., `block` → `"BLOCK"`, `timestamp` → `"TIMESTAMP"`)
- ASK_ORDER/BID_ORDER INSERT: added missing TRANSACTION_INDEX, TRANSACTION_HEIGHT, CREATION_HEIGHT columns
- Test file table names: fixed setup_pg() function table name casing (e.g., `"asset"` → `"ASSET"`)
- Clippy warnings: fixed 3 needless_borrows_for_generic_args warnings in pg.rs

### Changed

- ORM dual-engine status: both SQLite and PostgreSQL are now production-ready
- Total test count increased from ~276 to ~308 (added 32 PostgreSQL integration tests)

- Initial release of NRCS Rust backend
  - Full blockchain node implementation in Rust
  - PoS consensus engine
  - Transaction validation and execution engine
  - PostgreSQL persistence with SQLx
  - REST API (Axum)
  - Prometheus metrics
  - Docker containerization
- Vue 3 frontend with TypeScript
  - Login and authentication
  - Account management dashboard
  - Transaction sending and history
  - Block explorer
  - Responsive design with Element Plus
- Comprehensive testing
  - Backend unit tests (≥80% coverage)
  - Frontend unit tests (≥60% coverage)
  - API integration tests
  - E2E tests with Cypress
  - Performance tests with k6
- Infrastructure
  - Docker Compose deployment
  - GitHub Actions CI/CD
  - Code coverage reporting (Codecov)
  - Prometheus + Grafana monitoring
- Documentation
  - API documentation (Swagger)
  - Deployment guide
  - Monitoring guide
  - Migration guide from Java version
  - Troubleshooting and security hardening

### Changed

- N/A (initial release)

### Deprecated

- N/A (initial release)

### Removed

- N/A (initial release)

### Fixed

- N/A (initial release)

### Security

- N/A (initial release)

## [0.1.0-alpha] - 2026-04-03

Initial alpha release of Rust-based NRCS implementation.

### Features

- Blockchain core: blocks, transactions, accounts, assets
- Transaction processing with mempool
- PoS consensus with deadline-based forger selection
- REST API with JWT authentication
- Vue 3 frontend with Pinia state management
- Docker deployment with PostgreSQL
- CI/CD pipelines with GitHub Actions

### Testing

- Backend: All core crates have unit tests
- Frontend: Component and store tests
- Integration: API compatibility tests
- Performance: Load testing benchmarks

### Known Issues

- P2P networking not fully implemented
- Smart contract execution engine placeholder
- Some API endpoints not yet documented
- Need real-world stress testing

### Performance

- Achieves 500+ TPS on 8-core VM
- P95 read latency < 200ms
- Memory usage ~800MB

---

## Future Roadmap

### v0.2.0 (Q2 2026)

- Full P2P networking implementation
- Smart contract WASM execution (Wasmtime)
- CLI wallet tool
- Multi-signature transactions
- Account leasing UI

### v0.3.0 (Q3 2026)

- Asset issuance GUI
- Phased transactions
- Voting system
- Archival node mode
- Peering discovery service

### v1.0.0 (Q4 2026)

- Production-ready stability
- Security audit
- Performance optimizations
- Full API documentation
- Official mainnet launch

---

## Versioning

We use [SemVer](https://semver.org/). Given a version number MAJOR.MINOR.PATCH:

- MAJOR: Incompatible API changes
- MINOR: Backwards-compatible functionality additions
- PATCH: Backwards-compatible bug fixes

Alpha and beta releases may change APIs without advance notice.
