# Troubleshooting Guide

Common issues and solutions for NRCS node.

## Node won't start

### Symptoms

```bash
docker-compose logs backend
# shows: "error: config file not found"
```

**Solution**:
- Ensure `config/default.toml` or `config/production.toml` exists
- If using Docker, volume mount config correctly:
  ```yaml
  volumes:
    - ./config:/etc/nrcs:ro
  ```

### Error: "Database connection failed"

**Checklist**:
1. Is PostgreSQL running? `docker-compose ps postgres`
2. Do credentials match? Verify `POSTGRES_PASSWORD` in `.env`
3. Can you connect manually?
   ```bash
   docker-compose exec postgres psql -U nrcs -d nrcs
   ```
4. Check `DATABASE_URL` in backend config

**Fix**:
```bash
# Reset database password if needed
docker-compose down
docker-compose up -d postgres
# Wait 10s for DB to be ready
docker-compose up -d
```

### "Port already in use"

**Solution**:
- Change ports in `docker-compose.yml` or stop conflicting service:
  ```bash
  sudo lsof -i :17976
  sudo kill <pid>
  ```

## High Memory Usage

### Symptoms

Node uses >4GB RAM, may get OOM killed.

**Diagnosis**:
```bash
docker stats nrcs-backend
```

**Causes**:
- `mempool.max_transactions` too high (default 10000 is fine)
- Too many peers (`p2p.max_connections` > 1000)
- Debug logging (`RUST_LOG=debug` uses more memory)

**Solutions**:
1. Reduce mempool size: `mempool.max_transactions = 1000`
2. Limit peers: `p2p.max_connections = 100`
3. Use `RUST_LOG=info` in production
4. Add swap if necessary:
   ```bash
   sudo fallocate -l 4G /swapfile
   sudo chmod 600 /swapfile
   sudo mkswap /swapfile
   sudo swapon /swapfile
   ```

## Low Block Production (PoS)

### Symptoms

- No new blocks for long periods
- `nrcs_peer_count` but `nrcs_block_height` not increasing

**Checks**:
1. Is your node forging? Check logs:
   ```bash
   docker-compose logs backend | grep "forging"
   ```
2. Does your account have enough balance? Need ≥ minimum_balance
3. Is your node connected to peers? Check `peer_count` metric
4. Are you in the forgers queue? Use `GET /api/v1/forgers`

**Solutions**:
- Increase balance (mint more tokens)
- Wait for your turn (forger selection is probabilistic)
- Ensure node clock is synchronized (NTP)
- Check if `lease` field is set (leasing out balance forfeits forging rights)

## Transaction Stuck in Mempool

### Symptoms

- Transaction submitted but never confirmed
- `GET /api/v1/transactions/:txid` shows `status: "PENDING"`

**Possible causes**:
1. **Low fee**: Fee below `minimumFeePerByte` (configurable)
2. **Deadline expired**: `deadline` timestamp passed
3. **Sender nonce too high**: Nonce must be sequential
4. **Insufficient balance**: Balance changed after submit
5. **Blockchain fork**: Transaction included in a different branch

**Actions**:
- Increase fee and resubmit (use `POST /api/v1/transactions` with higher fee)
- Cancel and replace: send another transaction with same `full_hash` but higher fee (replace-by-fee if enabled)
- Wait for mempool to clear (if network congested)

## Database Slow Queries

### Symptoms

- API responses slow (>1s)
- Prometheus shows `database_connection_wait_seconds` high

**Diagnosis**:
```sql
-- In PostgreSQL
SELECT query, calls, total_time, rows, mean_time
FROM pg_stat_statements
ORDER BY total_time DESC
LIMIT 10;
```

**Common slow queries**:
- `SELECT * FROM accounts WHERE id = ?` - ensure index on `id`
- `SELECT * FROM transactions BY sender_id` - need composite index

**Fix**:
```sql
-- Add missing indexes
CREATE INDEX idx_accounts_id ON accounts(id);
CREATE INDEX idx_transactions_sender ON transactions(sender_id);
CREATE INDEX idx_transactions_height ON transactions(height);
```

**Configure connection pool**:
```toml
[database]
max_connections = 50
```

## Frontend Can't Connect to API

### Symptoms

- Vue app loads but shows "Failed to load data"
- Browser console shows CORS errors

**Choices**:
1. **Different ports**: Frontend on 80, Backend on 17976 → need nginx proxy
2. **CORS not enabled**: Check `[server] enable_cors = true`
3. **Wrong API URL**: Check `VITE_API_BASE_URL` in frontend

**Fix**:
- For production, configure nginx reverse proxy (see deployment guide)
- For development, ensure frontend uses correct port:
  ```env
  VITE_API_BASE_URL=http://localhost:17976/api/v1
  ```

## Peer Disconnections

### Symptoms

- `nrcs_peer_count` low or dropping
- Logs: "peer disconnected: timeout"

**Checks**:
1. Is port 17978 open in firewall?
2. Are outbound connections blocked? Node needs to connect to seeds
3. Is `p2p_peers` list configured?
4. Check logs for specific errors:
   ```bash
   docker-compose logs backend | grep -i peer
   ```

**Solutions**:
- Add seed nodes to `p2p_peers` in config
- Ensure `p2p_listen` address is bindable (not 127.0.0.1 if peers need external)
- Increase `p2p_handshake_timeout_seconds`
- Check for clock skew (>30s difference causes disconnect)

## "Invalid block" Errors

### Symptoms

Logs show:
```
"block validation failed: block signature invalid"
```

**Causes**:
1. Wrong signing key (not the expected forger)
2. Block corrupted during network transmission
3. Forger not eligible (balance too low, lease issues)

**Action**:
- Enable `RUST_LOG=debug` to see more details
- Check forger's balance: `GET /api/v1/accounts/:forger_id`
- Check if forger is leasing out balance
- If isolated, the node may be on a fork - restart to re-sync

## Docker Compose services restart loop

### Symptoms

```bash
docker-compose ps
# backend    restarting   (exit 1)
```

**Diagnosis**:
```bash
docker-compose logs backend --tail 50
docker-compose logs postgres
```

**Common fixes**:
1. Database not ready: Add `depends_on` with healthcheck (already in docker-compose.yml)
2. Config mount wrong: `docker-compose config` to validate
3. Permissions: Ensure user `nrcs` can read config files

**Fix permissions**:
```bash
sudo chown -R 1000:1000 config/
```

## Metrics not appearing in Prometheus

### Symptoms

Prometheus shows "down" or no metrics.

**Checklist**:
1. `[metrics] enable_prometheus = true` in config
2. Backend is running: `curl http://localhost:17976/metrics`
3. Prometheus config has correct target: `backend:17976`
4. No firewall blocking port 17976 from prometheus container
5. Prometheus logs show "scrape successful"

**Test**:
```bash
# From prometheus container
docker-compose exec prometheus wget -qO- http://backend:17976/metrics | head
```

## Performance Degradation

### Symptoms

- TPS dropping
- Latency increasing
- CPU/Memory spikes

**Diagnostic steps**:

1. **Check database**:
   ```sql
   SELECT * FROM pg_stat_activity WHERE state != 'idle';
   SELECT * FROM pg_stat_user_tables;
   ```
   Look for long-running queries or table bloat.

2. **Check disk I/O**:
   ```bash
   iostat -x 1 5
   ```
   High `await` or `%util` indicates disk bottleneck.

3. **Check network**:
   ```bash
   iftop -P -n
   ```
   Too many peers may saturate bandwidth.

**Solutions**:
- Increase `max_connections` and restart PostgreSQL
- Add connection pooler (PgBouncer) in transaction pooling mode
- Upgrade to faster SSD
- Reduce `p2p.max_connections`
- Enable query caching (application-level)

## TLS Certificate Issues

### Symptoms

- HTTPS errors
- Certificate expired

**Renew** (Let's Encrypt):
```bash
docker-compose exec backend certbot renew
docker-compose exec nginx nginx -s reload
```

For self-signed, generate new:
```bash
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout /etc/nrcs/tls/key.pem \
  -out /etc/nrcs/tls/cert.pem
```

## Frontend showing stale data

### Symptoms

- Data doesn't update after transaction
- Block height lags

**Cause**: Frontend caching or WebSocket not updating.

**Fix**:
1. Ensure `VITE_API_BASE_URL` is correct
2. Disable service worker (if enabled) or update cache logic
3. Check CORS headers: `Access-Control-Allow-Credentials: true`
4. Verify WebSocket connection (if used) is live

## Log Locations

- **Docker**: `docker-compose logs <service>`
- **Systemd**: `journalctl -u nrcs-node -f`
- **File**: If configured: `/var/log/nrcs/node.log` (mounted volume)

## Getting Help

1. Check [documentation](./)
2. Search [GitHub issues](https://github.com/yourorg/rust-nrcs/issues)
3. Join [community chat](https://nrcs.network/chat)
4. File a new issue with:
   - Docker compose version (`docker-compose version`)
   - OS and architecture
   - Full error logs
   - Steps to reproduce
   - Configuration (sanitized)

## Emergency Contacts

- **Security**: security@nrcs.network
- **Critical outage**: [Slack/SMS alerts configured via Prometheus Alertmanager]
