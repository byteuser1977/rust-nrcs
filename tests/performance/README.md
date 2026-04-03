# Performance Testing

This directory contains performance testing tools for NRCS backend.

## Tools

### k6 (Load Testing)

[k6](https://k6.io/) is used for load testing the API.

Install:
```bash
# macOS
brew install k6

# Linux
sudo apt-get install k6

# Or download from https://k6.io/docs/getting-started/installation/
```

Run tests:
```bash
# Basic load test
k6 run --vus 100 --duration 60s load_test.k6.js

# Ramp-up test
k6 run --stage 10s:50 --stage 30s:100 --duration 60s load_test.k6.js

# With threshold checks (from config)
k6 run --config benchmark_config.json load_test.k6.js
```

### vegeta (HTTP Load Testing)

Alternative: [vegeta](https://github.com/tsenart/vegeta)

```bash
# Install
go install github.com/tsenart/vegeta/v6@latest

# Create targets file (targets.txt)
echo "POST http://localhost:17976/api/v1/transactions" > targets.txt
echo "Content-Type: application/json" >> targets.txt
echo "" >> targets.txt
echo "{"sender":"test","recipient":"test","amount":1000,"fee":10}" >> targets.txt

# Run attack
echo "POST http://localhost:17976/api/v1/transactions" | \
  vegeta attack -duration=30s -rate=100 | \
  vegeta report

# Plot results
echo "POST http://localhost:17976/api/v1/transactions" | \
  vegeta attack -duration=30s -rate=100 | \
  vegeta plot > report.html
```

## Benchmarks

### Targets

See `benchmark_config.json` for performance targets:

- **TPS**: ≥ 500 (target), ≥ 400 (threshold)
- **P50 latency**: ≤ 50ms
- **P95 latency**: ≤ 200ms
- **P99 latency**: ≤ 500ms

### Running benchmarks

```bash
# 1. Ensure node is running and warmed up
docker-compose up -d
sleep 10  # wait for startup

# 2. Run k6 test with summary
k6 run --out json=results.json load_test.k6.js

# 3. Generate report
k6 report results.json

# 4. Compare against thresholds
node scripts/check-benchmarks.js results.json
```

## Performance Profiling (Rust)

### CPU profiling with perf

```bash
# Install perf (Linux)
sudo apt-get install linux-tools-common linux-tools-generic

# Run node with performance settings
RUST_LOG=info cargo run --release --bin nrcs-node -- -f

# In another terminal, attach perf
sudo perf record -g --pid=$(pgrep nrcs-node)
sudo perf report
```

### Memory profiling with heaptrack

```bash
# Install heaptrack
sudo apt-get install heaptrack

# Run
heaptrack cargo run --release --bin nrcs-node
heaptrack_gui heaptrack.nrcs-node.*.gz
```

### Flamegraph with flamegraph

```bash
cargo install flamegraph
cargo flamegraph --bin nrcs-node -- --config config/default.toml
# Opens flamegraph.svg in browser
```

## Database Performance

### PostgreSQL tuning

Check current settings:
```sql
SHOW shared_buffers;
SHOW work_mem;
SHOW maintenance_work_mem;
SHOW max_connections;
```

Recommended for 8GB RAM:
```ini
shared_buffers = 2GB
work_mem = 16MB
maintenance_work_mem = 256MB
effective_cache_size = 6GB
```

### Query analysis

```sql
-- Enable pg_stat_statements (if not already)
CREATE EXTENSION pg_stat_statements;

-- Find slowest queries
SELECT query, calls, total_time, rows, mean_time
FROM pg_stat_statements
ORDER BY total_time DESC
LIMIT 10;

-- Find missing indexes
SELECT * FROM pg_stat_user_indexes
WHERE idx_scan = 0;
```

## Stress Testing Scenarios

### Read-heavy (account queries)

```bash
k6 run -i 30s -u 200 -r 50 tests/read_heavy.js
```

### Write-heavy (transaction submission)

```bash
k6 run -i 30s -u 100 -r 200 tests/write_heavy.js
```

### Mixed workload

```bash
k6 run -i 30s -u 300 --rps 500 tests/mixed.js
```

## Continuous Benchmarking

Integrate into CI:

```yaml
# .github/workflows/benchmark.yml
- name: Run benchmarks
  run: |
    cargo build --release
    docker-compose up -d
    sleep 30
    k6 run --out json=bench.json tests/performance/load_test.k6.js
    node scripts/compare.js bench.json
```

## Reporting

Generate benchmark report:

```bash
node scripts/generate-report.js results.json > report.md
```

Contents:
- TPS achieved
- Latency percentiles (P50/P95/P99)
- Error rates
- System metrics (CPU, memory)
- Comparison to baseline

## Performance Regression Testing

Store baseline metrics in `baseline.json`:

```json
{
  "tps": 600,
  "latency_p50_ms": 45,
  "latency_p95_ms": 180,
  "latency_p99_ms": 400
}
```

Compare:
```bash
node scripts/regression.js results.json baseline.json
# Outputs: ✓ PASS or ✗ FAIL with delta
```

## Interpreting Results

- **TPS < 400**: Investigate database bottlenecks, network, or thread contention
- **P95 > 200ms**: Check slow queries, missing indexes, connection pool exhaustion
- **High error rate**: Look for timeout errors, database deadlocks, panic crashes
- **Memory growth**: Possible memory leak (run valgrind or heaptrack)

## Optimization Checklist

- [ ] Database queries use indexes
- [ ] No full table scans (EXPLAIN ANALYZE)
- [ ] Connection pool size matches `max_connections`
- [ ] Block production not delayed (check consensus logs)
- [ ] Garbage collection pauses minimal (Rust has none, but check allocations)
- [ ] CPU not saturated (check context switches)
- [ ] Network bandwidth sufficient (check for packet drops)

## Contact

For performance questions, open an issue with:
- Hardware specs (CPU, RAM, disk type)
- k6 output (results.json)
- PostgreSQL version and tuning parameters
- Docker compose version (`docker-compose version`)
