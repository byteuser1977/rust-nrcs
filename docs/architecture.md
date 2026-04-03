# Architecture

High-level architecture of the NRCS Rust implementation.

## System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                         Users                               │
├─────────────────────────────────────────────────────────────┤
│  Web Frontend (Vue 3)         CLI/SDK         Third-party  │
└─────────────────────────────┬───────────────────────────────┘
                              │ HTTPS/REST API
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Load Balancer (optional)                │
└─────────────────────────────┬───────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    NRCS Node (Rust)                        │
│  ┌──────────────────────────────────────────────────────┐  │
│  │                API Layer (Axum)                      │  │
│  │  • HTTP endpoints                                   │  │
│  │  • JWT authentication                              │  │
│  │  • Request validation                              │  │
│  │  • Rate limiting                                   │  │
│  └───────────────────────────┬──────────────────────────┘  │
│                              │                              │
│  ┌───────────────────────────▼──────────────────────────┐  │
│  │              Business Logic Layer                    │  │
│  │  • Account Manager                                  │  │
│  │  • Transaction Processor                            │  │
│  │  • Mempool Management                               │  │
│  │  • Consensus Engine (PoS/PoW)                       │  │
│  └───────────────────────────┬──────────────────────────┘  │
│                              │                              │
│  ┌───────────────────────────▼──────────────────────────┐  │
│  │            Data Access Layer (ORM/SQLx)              │  │
│  │  • Repositories (Account, Transaction, Block)       │  │
│  │  • Database connection pool                         │  │
│  └───────────────────────────┬──────────────────────────┘  │
│                              │                              │
│  ┌───────────────────────────▼──────────────────────────┐  │
│  │              PostgreSQL Database                     │  │
│  │  • Accounts table                                   │  │
│  │  • Transactions table                               │  │
│  │  • Blocks table                                     │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                 P2P Network (libp2p?)                     │
│  • Block propagation                                     │
│  • Transaction gossiping                                 │
│  • Peer discovery                                        │
│  • Consensus participation                              │
└─────────────────────────────────────────────────────────────┘
```

## Components

### Blockchain Core (`crates/blockchain-types`)

Core data structures that define the blockchain state:

- `Block`: Block header and transactions
- `Transaction`: Individual operations
- `Account`: Account balances and assets
- `Hash256`, `Hash512`: Fixed-size hash types

All types are `serde`-serializable and use deterministic binary formats (bincode) for network transmission.

### Cryptography (`crates/crypto`)

Cryptographic primitives:

- Ed25519 digital signatures
- SHA-256, SHA-512, BLAKE3 hashing
- Account ID derivation (public key → 64-bit ID)
- Base58Check address encoding

### Consensus (`crates/consensus`)

Pluggable consensus engines:

- **PoS (Proof of Stake)**: Forger selection based on effective balance and `generation_signature`.
- **PoW (Proof of Work)**: Traditional hash-based difficulty adjustment.

Consensus trait:
```rust
pub trait ConsensusEngine: Send + Sync {
    fn verify_difficulty(&self, block: &Block) -> Result<()>;
    fn calculate_next_difficulty(&self, recent_blocks: &[Block]) -> u64;
    fn verify_timestamp(&self, block: &Block, current_time: Timestamp) -> Result<()>;
}
```

### Transaction Engine (`crates/tx-engine`)

Transaction lifecycle:

1. **Validation**: Signature, balance, nonce, deadline checks
2. **Execution**: Apply state changes atomically
3. **Mempool**: In-memory queue with fee-based sorting
4. **Receipt**: Generate execution receipt (success/failure, logs)

Key types:
- `TransactionProcessor`: Trait for pluggable processors
- `Mempool`: Transaction pool with priority ordering
- `TxReceipt`: Execution result

### Account Management (`crates/account`)

Account operations:

- Account creation (keypair generation)
- Balance queries and transfers
- Nonce management (anti-replay)
- Asset holdings tracking
- Leasing management

### ORM Layer (`crates/orm`)

Database abstraction using SQLx:

- Compile-time SQL query verification
- Async/await support with Tokio
- PostgreSQL only (with potential MySQL compatibility layer)
- Repository pattern for all entities

### HTTP API (`crates/http-api`)

REST API server using Axum:

- JWT-based authentication
- JSON request/response format
- CORS support
- Swagger/OpenAPI documentation
- Prometheus metrics middleware

Routes:
```
GET    /api/v1/accounts/:address
POST   /api/v1/transactions
GET    /api/v1/blocks/latest
GET    /api/v1/blocks/:height
GET    /api/v1/transactions/:txid
POST   /api/v1/auth/login
GET    /health
```

### Full Node (`crates/node`)

Orchestrates all components:

- Initializes database connection pool
- Loads configuration
- Starts P2P network listener
- Launches API server
- Runs consensus loop (block production)
- Handles graceful shutdown

### Frontend (`frontend/`)

Vue 3 single-page application:

- Vue Router for navigation
- Pinia for global state (account, transactions)
- Element Plus UI components
- Axios for API calls
- i18n for internationalization

Pages:
- Login
- Dashboard (account overview)
- Send Transaction
- Transaction History
- Block Explorer
- Settings

## Data Model

### Accounts Table

| Column | Type | Description |
|--------|------|-------------|
| id | BIGINT (PK) | Account ID (derived from pubkey) |
| address | VARCHAR | Base58Check address (denormalized) |
| balance | NUMERIC | Confirmed balance (NQT) |
| unconfirmed_balance | NUMERIC | Unconfirmed balance |
| reserved_balance | NUMERIC | Reserved/Locked balance |
| assets | JSONB | Asset holdings {asset_id: amount} |
| properties | JSONB | User-defined metadata |
| lease | JSONB | Lease info if leasing out |
| created_at | TIMESTAMPTZ | Creation timestamp |
| updated_at | TIMESTAMPTZ | Last modification |

### Transactions Table

| Column | Type | Description |
|--------|------|-------------|
| id | BIGSERIAL (PK) | Internal DB ID |
| transaction_id | VARCHAR | Full hash (unique) |
| version | SMALLINT | Transaction version |
| type_id | SMALLINT | Transaction type enum |
| sender_id | BIGINT | Sender account ID |
| recipient_id | BIGINT (nullable) | Recipient account ID |
| amount | NUMERIC | Transfer amount |
| fee | NUMERIC | Transaction fee |
| timestamp | TIMESTAMPTZ | Creation time |
| deadline | INTEGER | Block height deadline |
| signature | BYTEA | Ed25519 signature |
| attachment_bytes | BYTEA | Optional attachment |
| height | INTEGER | Block height (0 if unconfirmed) |
| created_at | TIMESTAMPTZ | DB insertion time |

### Blocks Table

| Column | Type | Description |
|--------|------|-------------|
| id | BIGSERIAL (PK) | Internal DB ID |
| height | INTEGER (unique) | Block height |
| hash | VARCHAR | Block hash |
| previous_hash | VARCHAR | Previous block hash |
| payload_hash | VARCHAR | Merkle root of transactions |
| generator_id | BIGINT | Forger account ID |
| timestamp | TIMESTAMPTZ | Block timestamp |
| base_target | BIGINT | Difficulty target |
| total_amount | NUMERIC | Sum of transaction amounts |
| total_fee | NUMERIC | Sum of transaction fees |
| version | INTEGER | Block version |
| created_at | TIMESTAMPTZ | DB insertion time |

## Data Flow

### Transaction Submission

```
Frontend → HTTP POST /api/v1/transactions
  ↓
Axum router → authenticate JWT
  ↓
Deserialize request → validate fields
  ↓
TransactionProcessor.validate()
  • Signature verification
  • Balance check
  • Nonce check
  • Fee sufficiency
  ↓
Mempool.add_transaction() (if valid)
  ↓
Return 201 Created { transactionId }
```

### Block Production (PoS)

```
Consensus loop (every target_spacing seconds):
  ↓
Load current blockchain state (accounts, height)
  ↓
PoSEngine.select_forger()
  • Filter accounts with balance ≥ minimum
  • Use generation_signature + timestamp to pick forger
  ↓
BlockProducer creates block:
  • Collect transactions from mempool (by fee priority)
  • Build Merkle root
  • Forger signs block
  ↓
TransactionProcessor.execute_all()
  • Apply balance changes atomically
  • Generate receipts
  ↓
Save block + transactions to DB
  ↓
Broadcast to P2P peers
  ↓
Mempool.prune(transactions_in_block)
```

### Peer-to-Peer Sync

```
Inbound connection → handshake (version, genesis hash)
  ↓
If chain shorter:
  Request blocks from peer (getBlocks message)
  ↓
Validate each block (signature, difficulty, transactions)
  ↓
Apply to local DB (fork choice: longest valid chain)
  ↓
Broadcast verified blocks to other peers
```

## Configuration

Configuration is loaded from TOML file with environment variable substitution.

See `config/default.toml` for all options.

Key sections:

- `[database]`: Connection settings
- `[server]`: HTTP and P2P listeners
- `[chain]`: Blockchain parameters
- `[consensus]`: Algorithm and tuning
- `[logging]`: Log level and format

## Security Model

- **Authentication**: JWT tokens issued via `/api/v1/auth/login`
- **Authorization**: All state-changing operations require valid token
- **Input validation**: All API inputs validated (type, range, format)
- **Signature verification**: Every transaction must be signed
- **SQL injection**: Prevented by SQLx compile-time checks
- **Replay attacks**: Prevented by nonce and deadline
- **Double spend**: Prevented by atomic balance updates in DB transactions

## Scalability Considerations

### Current limitations

- Single node: all requests handled by one process
- PostgreSQL: single primary (read/write)
- P2P: limited to ~100 peers (configurable)

### Horizontal scaling path

1. **Read replicas**: Add read replicas for account/block queries
2. **API load balancer**: Multiple API nodes behind HAProxy/Nginx
3. **Database sharding**: Future work (by account range or territory)
4. **Caching layer**: Redis for hot account lookups
5. **Microservices**: Split API, miner, and full node roles

## Performance Optimizations Implemented

1. **Connection pooling**: r2d2/BB8 pool for PostgreSQL
2. **Concurrent validation**: Transactions validated in parallel
3. **Batch DB writes**: Transactions in a block committed together
4. **Indexing**: All query paths have appropriate indexes
5. **Zero-copy deserialization**: Using bincode with references where possible
6. **Async I/O**: All network and DB operations non-blocking
7. **Memory-mapped files**: For blockchain data (future)

## Monitoring and Observability

- **Metrics**: Prometheus at `/metrics`
- **Tracing**: OpenTelemetry (planned)
- **Logging**: Structured JSON logs to stdout
- **Health**: `/health` endpoint for liveness probes

Key metrics:
- `nrcs_block_height`
- `nrcs_transactions_per_second`
- `http_request_duration_seconds`
- `database_connection_wait_seconds`

## Technology Choices

| Decision | Alternatives | Rationale |
|----------|--------------|-----------|
| **Rust** | Go, Java, C++ | Memory safety + performance + async |
| **Axum** | Actix, Warp | Built on tower, good ecosystem |
| **SQLx** | Diesel, SeaORM | Compile-time query checking, async |
| **PostgreSQL** | MySQL, SQLite | JSONB, reliability, features |
| **Vue 3** | React, Svelte | Composition API, smaller bundle |
| **Pinia** | Vuex 4 | TypeScript-friendly, simpler API |
| **Vite** | Webpack, Rollup | Fast HMR, modern tooling |
| **Docker** | Podman, bare metal | Reproducible deployments |

## Future Enhancements

- [ ] Light client protocol (SPV)
- [ ] zk-SNARKs for privacy
- [ ] Cross-chain bridges
- [ ] Decentralized storage integration (IPFS)
- [ ] Smart contract language (Rust/WASM)
- [ ] Mobile SDKs (iOS/Android)
- [ ] Hardware wallet integration

---

For detailed implementation questions, refer to inline code documentation and `docs/`.
