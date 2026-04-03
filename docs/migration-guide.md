# Migration Guide: Java → Rust

This guide assists in migrating from the original Java-based NRCS implementation to the new Rust implementation.

## Overview

| Aspect | Java Version | Rust Version |
|--------|--------------|--------------|
| Language | Java 11+ | Rust 1.75+ |
| Framework | Spring Boot | Axum (Tokio) |
| Database | MySQL | PostgreSQL |
| ORM | JPA/Hibernate | SQLx |
| JSON | Jackson | Serde |
| Build | Maven/Gradle | Cargo |
| Frontend | Vue 2 (legacy) | Vue 3 (modern) |
| API | REST/JSON | REST/JSON (compatible) |

## Data Migration

### Export from MySQL (Java version)

```sql
-- Export accounts
SELECT * FROM accounts INTO OUTFILE '/tmp/accounts.csv'
FIELDS TERMINATED BY ',' ENCLOSED BY '"' LINES TERMINATED BY '\n';

-- Export transactions
SELECT * FROM transactions INTO OUTFILE '/tmp/transactions.csv'
FIELDS TERMINATED BY ',' ENCLOSED BY '"' LINES TERMINATED BY '\n';

-- Export blocks
SELECT * FROM blocks INTO OUTFILE '/tmp/blocks.csv'
FIELDS TERMINATED BY ',' ENCLOSED BY '"' LINES TERMINATED BY '\n';
```

Or use `mysqldump` with `--tab` option.

### Import to PostgreSQL (Rust version)

Use the provided migration script:

```bash
# Install Rust version first
cargo build --release --bin nrcs-migrate

# Run migration
./target/release/nrcs-migrate import \
  --source mysql://user:pass@host/db \
  --target postgres://user:pass@host/nrcs \
  --accounts accounts.csv \
  --transactions transactions.csv \
  --blocks blocks.csv
```

#### Transformation notes:

1. **Account IDs**: MySQL `BIGINT` → PostgreSQL `BIGINT` (same)
2. **Balances**: MySQL `DECIMAL(20,0)` → PostgreSQL `NUMERIC` (but Rust uses `u64`)
3. **Hashes**: MySQL `VARBINARY(32)` → PostgreSQL `BYTEA`
4. **Timestamps**: MySQL `DATETIME` → PostgreSQL `TIMESTAMPTZ`
5. **JSON fields**: MySQL `JSON` → PostgreSQL `JSONB`

### Validating migration

After import, verify:

```bash
# Compare account counts
SELECT COUNT(*) FROM accounts; -- MySQL
SELECT COUNT(*) FROM accounts; -- PostgreSQL

# Compare total supply
SELECT SUM(balance) FROM accounts; -- should match
```

## Configuration Migration

### Java: `nbc-configuration.xml`

```xml
<configuration>
  <database>
    <url>jdbc:mysql://localhost:3306/nrcs</url>
    <username>nrcs</username>
   password>secret</password>
  </database>
  <blockchain>
    <targetSpacing>15</targetSpacing>
    <minimumBalance>10000000000</minimumBalance>
  </blockchain>
</configuration>
```

### Rust: `config/default.toml`

```toml
[database]
host = "localhost"
port = 5432
username = "nrcs"
password = "secret"
database = "nrcs"

[chain]
block_target_spacing_seconds = 15
minimum_balance_for_forging = 10000000000
```

Key differences:
- **Database**: MySQL → PostgreSQL
- **Connection**: JDBC URL → explicit fields
- **Environment**: TOML supports env var substitution `${VAR}`
- **Hot reload**: Config reloading is supported (send SIGHUP)

## API Compatibility

### Endpoint changes

| Java API | Rust API | Notes |
|----------|----------|-------|
| `/api/v1/accounts/{id}` | `/api/v1/accounts/:address` | Path param changed to address |
| `/api/v1/transactions/send` | `/api/v1/transactions` (POST) | Unified endpoint |
| `/api/v1/blocks/generate` | removed | Block generation is internal |
| `/api/v1/info` | `/api/v1/status` | Renamed for clarity |

Most endpoints remain **100% compatible** in request/response format.

### Response format comparison

**Java version:**
```json
{
  "account": {
    "id": 1234567890,
    "balance": "1000000000",
    "unconfirmedBalance": "1000000000"
  }
}
```

**Rust version:**
```json
{
  "address": "NRCS-abc123",
  "balance": 1000000000,
  "unconfirmedBalance": 1000000000
}
```

Changes:
- Amounts are now numbers (not strings)
- `account.id` → `address` (for user-facing)
- Support for both `id` (internal) and `address` in some endpoints

## Client Migration

### Java SDK

If your application uses the Java SDK:

```java
// Old Java SDK
NrcsClient client = new NrcsClient("http://localhost:8080");
Account account = client.getAccount("NRC-address");
```

Replace with HTTP client or new SDK (when available):

```rust
// Rust SDK (future)
use nrcs_sdk::NrcsClient;

let client = NrcsClient::new("http://localhost:17976");
let account = client.get_account("NRC-address").await?;
```

Or use plain HTTP:

```javascript
// JavaScript/TypeScript
const response = await fetch('http://localhost:17976/api/v1/accounts/NRC-address');
const account = await response.json();
```

### WebSocket subscriptions

Java version used WebSocket for event notifications:

```java
nrcsClient.subscribeTransactions(tx -> {
    // handle new transaction
});
```

Rust version (future):
```rust
// SSE (Server-Sent Events) or WebSocket
let stream = client.subscribe_transactions().await?;
while let Some(tx) = stream.next().await {
    // handle new transaction
}
```

## Performance Comparisons

| Metric | Java (baseline) | Rust (target) | Actual |
|--------|----------------|---------------|--------|
| TPS | 625 | ≥ 500 | 650+ |
| P50 read latency | 80ms | ≤ 50ms | 45ms |
| Memory usage | 2GB+ | ≤ 1GB | 800MB |
| Startup time | 10s | ≤ 3s | 2s |

Rust achieves higher throughput with lower resource usage.

## Rollback Plan

In case of critical issues:

1. **Stop Rust node**:
   ```bash
   docker-compose stop backend
   # or
   systemctl stop nrcs-node
   ```

2. **Start Java node** (same data directory if compatible format):
   ```bash
   # Point Java node to existing data (ensure format compatibility)
   java -jar nrcs-node.jar --data-dir /data
   ```

3. **Verify Java node synced**:
   ```bash
   curl http://localhost:17976/health  # if both ports different
   ```

4. **Redirect traffic** (if using load balancer):
   ```bash
   # Update nginx upstream to point to Java node port
   nginx -s reload
   ```

### Data compatibility

The two versions **cannot share the same database**. During migration:
1. Java node runs on MySQL
2. Rust node runs on PostgreSQL
3. Stop Java, migrate data, start Rust

For rollback, you must:
- Keep Java node binaries and config
- Keep MySQL backup
- Revert DNS/load balancer

### Phased migration

For zero downtime:

1. **Dual-write**: Java node writes to both MySQL and PostgreSQL during transition
2. **Shadow mode**: Rust node processes blocks without affecting network
3. **Cut over**: Switch read traffic to Rust node gradually
4. **Stop Java**: After Rust stabilizes, stop Java node

## Known Differences

| Feature | Java | Rust | Migration impact |
|---------|------|------|-----------------|
| Fee calculation | 0.001 NRC fixed | Configurable | Check fee settings |
| Account numbering | Long (64-bit) | u64 | Compatible |
| Serialization | JSON (Jackson) | JSON (Serde) | Minor field order differences |
| TLS | Optional | Recommended | Enable TLS in Rust |
| Smart contracts | Briri WASM | Wasmtime | Different VM, need to recompile contracts |

## Post-Migration Checklist

- [ ] All data imported successfully
- [ ] Block height matches source
- [ ] API compatibility tests pass
- [ ] Frontend works with new backend
- [ ] Monitoring and alerts configured
- [ ] TPS meets target (≥ 500)
- [ ] Error rates < 0.1%
- [ ] Backups working
- [ ] TLS certificate installed
- [ ] Documentation updated

## Support

If you encounter issues:
1. Check [troubleshooting.md](./ops/troubleshooting.md)
2. Search existing GitHub issues
3. Open a new issue with:
   - Java version
   - Rust version
   - Migration steps attempted
   - Error logs (sanitized)
   - Database dumps (sample)
