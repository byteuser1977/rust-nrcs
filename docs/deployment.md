# Deployment Guide

This guide covers production deployment of NRCS node.

## Requirements

- **CPU**: 2+ cores (4+ recommended)
- **Memory**: 4GB RAM minimum (8GB+ recommended)
- **Storage**: 100GB+ SSD (blockchain grows over time)
- **Network**: Public IP with port 17978 (P2P) open
- **OS**: Linux (Ubuntu 22.04, Debian 12, or similar)

## Pre-deployment Checklist

- [ ] Set up domain name (optional)
- [ ] Obtain TLS certificate
- [ ] Configure firewall rules
- [ ] Set up monitoring
- [ ] Plan backup strategy
- [ ] Configure log aggregation

## 1. Prepare the Server

```bash
# Update system
sudo apt-get update && sudo apt-get upgrade -y

# Install Docker and Docker Compose
curl -fsSL https://get.docker.com | sh
sudo systemctl enable docker
sudo usermod -aG docker $USER

# Install Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose
```

## 2. Deploy Application

```bash
# Pull or clone repository
git clone https://github.com/yourorg/rust-nrcs.git
cd rust-nrcs

# Create configuration
cp config/production.toml.example config/production.toml
# Edit config/production.toml with your settings

# Set environment file
cat > .env <<EOF
POSTGRES_PASSWORD=your-secure-db-password-here
JWT_SECRET=$(openssl rand -base64 64)
GRAFANA_PASSWORD=your-grafana-password
EOF

# (Optional) Mount TLS certs
sudo mkdir -p /etc/nrcs/tls
sudo cp your-cert.pem /etc/nrcs/tls/cert.pem
sudo cp your-key.pem /etc/nrcs/tls/key.pem
sudo chmod 600 /etc/nrcs/tls/key.pem

# Start services
docker-compose -f docker/docker-compose.yml up -d

# Check status
docker-compose -f docker/docker-compose.yml ps
```

## 3. Configure Reverse Proxy (Nginx)

If exposing frontend on port 80/443, configure nginx:

```nginx
# /etc/nginx/sites-available/nrcs
server {
    listen 80;
    server_name nrcs.yourdomain.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name nrcs.yourdomain.com;

    ssl_certificate /etc/letsencrypt/live/nrcs.yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/nrcs.yourdomain.com/privkey.pem;

    location / {
        proxy_pass http://localhost:80;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }

    location /api/ {
        proxy_pass http://localhost:17976/api/;
        # ... same headers
    }
}
```

```bash
sudo ln -s /etc/nginx/sites-available/nrcs /etc/nginx/sites-enabled/
sudo nginx -t && sudo systemctl reload nginx
```

## 4. Configure Firewall

```bash
# Allow SSH, HTTP, HTTPS, and P2P port
sudo ufw allow 22/tcp
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw allow 17978/tcp
sudo ufw enable
```

## 5. Backup Strategy

### Database backup

```bash
# Daily backup script
#!/bin/bash
BACKUP_DIR=/backups/nrcs
DATE=$(date +%Y%m%d_%H%M%S)
docker-compose -f docker/docker-compose.yml exec -T postgres pg_dump -U nrcs nrcs > $BACKUP_DIR/nrcs_$DATE.sql

# Keep only last 30 days
find $BACKUP_DIR -name "*.sql" -mtime +30 -delete
```

Add to crontab:

```bash
0 2 * * * /path/to/backup.sh
```

### Node data backup

The node's data directory (`/data` in Docker) contains:
- Blockchain data
- Mempool (if persisted)
- Indexes

Backup strategy:
- Full backup weekly (snapshot)
- Incremental daily (rsync)

## 6. Monitoring

### Prometheus

Already included in docker-compose. Access at http://your-server:9090.

Key metrics:
- `nrcs_block_height`
- `nrcs_peer_count`
- `nrcs_transactions_per_second`
- `nrcs_mempool_size`
- `process_cpu_seconds_total`
- `process_resident_memory_bytes`

### Grafana

Import pre-configured dashboards from `grafana/provisioning/dashboards/`.

### Alerts

Configure Prometheus Alertmanager to send notifications for:
- Node down (health check fails)
- Block height not increasing (stuck)
- Disk space < 10%
- CPU > 90% for 5m
- Memory > 90% for 5m

## 7. Scaling

### Vertical scaling

- Increase CPU/RAM on server
- Use faster SSD (NVMe)
- Increase max PostgreSQL connections in config

### Horizontal scaling

- Deploy multiple full nodes behind a load balancer for API
- Use read replicas for database queries
- Consider sharding in future versions

## 8. Upgrade Procedure

1. **Schedule maintenance window** and notify users
2. **Stop services**: `docker-compose down`
3. **Pull latest images**: `docker-compose pull`
4. **Run migrations** (if any):
   ```bash
   docker-compose run --rm backend nrcs-node migrate
   ```
5. **Start services**: `docker-compose up -d`
6. **Verify**: Check logs, health endpoint, block height
7. **Resume normal operations**

For zero-downtime upgrades, use rolling updates with multiple nodes.

## 9. Security Hardening

See [Security Hardening](./ops/security-hardening.md) for detailed checklist.

Critical steps:
- Change default passwords
- Use strong JWT secret
- Enable TLS for all communication
- Restrict database access
- Keep OS and Docker updated
- Use non-root user in containers (already configured)
- Audit logs regularly

## 10. Performance Tuning

### PostgreSQL

```sql
-- Increase shared_buffers (e.g., to 25% of RAM)
ALTER SYSTEM SET shared_buffers = '4GB';
SELECT pg_reload_conf();

-- Enable WAL archiving for point-in-time recovery
ALTER SYSTEM SET wal_level = 'replica';
```

### Node configuration

In `config/production.toml`:

```toml
[database]
max_connections = 50  # Match connection pool size

[mempool]
max_transactions = 50000  # Increase if you have high tx volume

[consensus.pos]
target_spacing_seconds = 15  # Faster blocks if network supports

[api]
enable_rate_limit = true
rate_limit_requests_per_minute = 10000
```

## Support

- [Troubleshooting Guide](./ops/troubleshooting.md)
- [Monitoring Guide](./monitoring.md)
- [Backup & Restore](./ops/backup-restore.md)
