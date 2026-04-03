# Monitoring Guide

Comprehensive monitoring setup for NRCS node using Prometheus and Grafana.

## Metrics Overview

### Backend Metrics (Prometheus format)

All metrics exposed at `http://localhost:17976/metrics` when `[metrics] enable_prometheus = true`.

#### Blockchain metrics

| Metric | Type | Description |
|--------|------|-------------|
| `nrcs_block_height` | Gauge | Current blockchain height |
| `nrcs_block_interval_seconds` | Histogram | Time between blocks |
| `nrcs_transactions_per_second` | Gauge | TPS rate (1m rate) |
| `nrcs_mempool_size` | Gauge | Number of transactions in mempool |
| `nrcs_mempool_total_fee` | Gauge | Total fee in mempool (NQT) |
| `nrcs_peer_count` | Gauge | Connected P2P peers |
| `nrcs_transactions_validated_total` | Counter | Total validated transactions |
| `nrcs_transactions_executed_total` | Counter | Total executed transactions |
| `nrcs_transactions_failed_total` | Counter | Total failed transactions |

#### System metrics

| Metric | Type | Description |
|--------|------|-------------|
| `process_cpu_seconds_total` | Counter | CPU time used |
| `process_resident_memory_bytes` | Gauge | RSS memory |
| `process_virtual_memory_bytes` | Gauge | Virtual memory |
| `process_open_fds` | Gauge | Open file descriptors |
| `http_requests_total` | Counter | HTTP requests by status, endpoint |
| `http_request_duration_seconds` | Histogram | Request latency |
| `database_connection_pool_size` | Gauge | Active DB connections |
| `database_connection_wait_seconds` | Histogram | DB connection wait time |

### Frontend metrics (Optional)

If using frontend monitoring (e.g., Google Analytics, custom):

- Page load times
- API error rates
- User interactions

## Prometheus Configuration

Configure Prometheus to scrape metrics:

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'nrcs-backend'
    static_configs:
      - targets: ['backend:17976']
    metrics_path: '/metrics'
    scrape_interval: 15s

  - job_name: 'nrcs-frontend'
    static_configs:
      - targets: ['frontend:80']

  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres:5432']
    metrics_path: '/metrics'  # If using postgres exporter
```

## Grafana Dashboards

Import pre-configured dashboards:

1. Go to http://localhost:3000 (login: admin/admin)
2. Navigate to Dashboards → Import
3. Upload JSON dashboard file or paste ID

### Recommended dashboards

1. **NRCS Node Overview** - Quick health status
   - Block height trend
   - Peer count
   - TPS gauge
   - Mempool size
   - CPU/Memory usage

2. **Performance Metrics**
   - Request latency percentiles (P50, P95, P99)
   - Error rate by endpoint
   - TPS over time
   - Block interval distribution

3. **Infrastructure**
   - PostgreSQL metrics
   - Docker container resource usage
   - Disk I/O and space

Dashboard JSON files provided in `grafana/provisioning/dashboards/`.

## Alerting Rules

Configure alerts in Prometheus Alertmanager:

```yaml
alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:9093']

rule_files:
  - "alerts.yml"
```

```yaml
# alerts.yml
groups:
  - name: nrcs_node
    rules:
      - alert: NodeDown
        expr: up{job="nrcs-backend"} == 0
        for: 1m
        annotations:
          summary: "NRCS node is down"
          severity: critical

      - alert: StuckBlock
        expr: deriv(nrcs_block_height[5m]) == 0
        for: 5m
        annotations:
          summary: "Block height not increasing"
          severity: warning

      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m]) > 0.05
        for: 2m
        annotations:
          summary: "HTTP error rate > 5%"
          severity: warning

      - alert: LowTPS
        expr: nrcs_transactions_per_second < 10
        for: 10m
        annotations:
          summary: "TPS below 10 for 10 minutes"
          severity: warning

      - alert: HighMemory
        expr: process_resident_memory_bytes / 1024 / 1024 / 1024 > 4
        for: 5m
        annotations:
          summary: "Memory usage > 4GB"
          severity: warning

      - alert: FewPeers
        expr: nrcs_peer_count < 5
        for: 10m
        annotations:
          summary: "Connected to fewer than 5 peers"
          severity: info
```

Notification channels (email, Slack, Telegram) configured in Alertmanager.

## Log Aggregation

### Docker logging

Configure Docker daemon (`/etc/docker/daemon.json`):

```json
{
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "10m",
    "max-file": "3"
  }
}
```

### Centralized logging (Optional)

Use ELK stack or Loki:
- Fluentd/Fluent Bit to collect container logs
- Elasticsearch/Loki for storage
- Grafana Loki data source for querying

## Metrics Collection Best Practices

1. **Scrape interval**: 15s for real-time monitoring
2. **Retention**: Keep metrics for 30-90 days
3. **Cardinality**: Limit high-cardinality labels (e.g., avoid `txid` in metrics)
4. **Resource usage**: Monitor Prometheus itself; consider Thanos for large deployments
5. **Security**: Restrict access to metrics endpoint; use authentication if public

## Troubleshooting

### No metrics in Prometheus

1. Check if backend has `enable_prometheus = true` in config
2. Verify `/metrics` endpoint is accessible:
   ```bash
   curl http://backend:17976/metrics
   ```
3. Check Prometheus targets page (http://localhost:9090/targets)

### High cardinality warning

If you see "exemplar" or "too many series" warnings:
- Reduce label cardinality in application code
- Avoid using user-provided values (addresses, txids) as metric labels
- Use aggregated metrics instead

### Grafana panels showing "No data"

1. Verify Prometheus is scraping backend
2. Check time range in Grafana (e.g., last 1 hour)
3. Confirm metric names haven't changed
4. Inspect Prometheus query in "Explore" view

## Custom Metrics

To add custom metrics in Rust backend:

```rust
use prometheus::{IntGaugeVec, register_int_gauge_vec};

lazy_static! {
    static ref TRANSACTION_SIZE: IntGaugeVec = register_int_gauge_vec!(
        "nrcs_transaction_size_bytes",
        "Transaction size in bytes",
        &["type"]
    ).unwrap();
}

// In transaction processing:
TRANSACTION_SIZE.with_label_values(&[tx_type]).set(size as i64);
```

Recompile and restart; new metrics appear automatically.
