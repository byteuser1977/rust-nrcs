# Backup and Restore Operations

## Backup Strategy

NRCS node data consists of:

1. **PostgreSQL database**: Accounts, transactions, blocks
2. **Node data directory**: Blockchain state, mempool, indexes
3. **Configuration files**: `config/`, TLS certificates
4. **Prometheus/Grafana data**: Metrics and dashboards

### Recommended backup schedule

| Item | Frequency | Retention | Method |
|------|-----------|-----------|--------|
| Database | Hourly | 30 days | pg_dump + compression |
| Node data | Daily | 90 days | Snapshot (rsync/btrfs) |
| Config files | On change | Indefinite | Git |
| Prometheus TSDB | Daily | 7 days | snapshots |
| Grafana dashboards | Daily | Indefinite | Export JSON |

## Database Backup

### Using pg_dump

```bash
#!/bin/bash
# backup-db.sh

BACKUP_DIR=/backups/nrcs/db
DATE=$(date +%Y%m%d_%H%M%S)
FILE="$BACKUP_DIR/nrcs_$DATE.sql.gz"

# Ensure backup directory exists
mkdir -p $BACKUP_DIR

# Dump and compress
docker-compose -f docker/docker-compose.yml exec -T postgres \
  pg_dump -U nrcs -d nrcs --no-owner --no-acl | gzip > $FILE

# Verify backup
if gzip -t $FILE 2>/dev/null; then
  echo "Backup successful: $FILE"
else
  echo "Backup corrupted!" >&2
  rm $FILE
  exit 1
fi

# Retention: delete older than 30 days
find $BACKUP_DIR -name "*.sql.gz" -mtime +30 -delete
```

### Using WAL archiving (Point-in-Time Recovery)

Configure PostgreSQL for continuous archiving:

```sql
-- In postgresql.conf
wal_level = replica
archive_mode = on
archive_command = 'cp %p /backups/nrcs/wal/%f'
```

Then you can restore to any point in time within the archived WALs.

### Restore database from backup

```bash
# Stop backend node
docker-compose stop backend

# Drop and recreate database
docker-compose exec -T postgres dropdb -U nrcs nrcs
docker-compose exec -T postgres createdb -U nrcs nrcs

# Restore from latest backup
LATEST=$(ls -t /backups/nrcs/db/*.sql.gz | head -1)
zcat $LATEST | docker-compose exec -T postgres psql -U nrcs nrcs

# Start backend
docker-compose start backend
```

## Node Data Backup

The node's data directory (`/data` in Docker) contains:

- `blocks/` - Block data files
- `txs/` - Transaction index
- `accounts/` - Account state
- `mempool.dat` - Current mempool (if persisted)

### Full backup (offline)

```bash
#!/bin/bash
# backup-node.sh

BACKUP_DIR=/backups/nrcs/node
DATE=$(date +%Y%m%d_%H%M%S)
FILE="$BACKUP_DIR/nrcs_node_$DATE.tar.gz"

# Stop node to ensure consistency
docker-compose stop backend

# Create tar archive
tar -czf $FILE -C /var/lib/docker/volumes/nrcs_backend_data/_data .

# Restart node
docker-compose start backend

# Keep only last 7 full backups
find $BACKUP_DIR -name "*.tar.gz" -mtime +7 -delete
```

### Hot backup (online)

If node supports online snapshot (e.g., `nrcs-node snapshot` command):

```bash
docker-compose exec backend nrcs-node snapshot --output /data/snapshot.bin
# Then copy /data/snapshot.bin to backup storage
```

## Configuration Backups

Configuration files should be version-controlled:

```bash
# In your git repo (config/ directory)
git add config/production.toml
git add docker-compose.yml
git commit -m "Update network settings"
git push origin main
```

Then backups are simply git clones.

## Restore from Complete Failure

Scenario: Server crash, data lost.

1. **Provision new server** with same OS and Docker
2. **Install Docker & Docker Compose**
3. **Clone repository**:
   ```bash
   git clone https://github.com/yourorg/rust-nrcs.git
   cd rust-nrcs
   ```
4. **Restore database**:
   ```bash
   gunzip -c /backups/nrcs/db/nrcs_20260101_020000.sql.gz | \
     docker-compose exec -T postgres psql -U nrcs nrcs
   ```
5. **Restore node data**:
   ```bash
   tar -xzf /backups/nrcs/node/nrcs_node_20260101_020000.tar.gz -C \
     /var/lib/docker/volumes/nrcs_backend_data/_data
   ```
6. **Restore config**:
   ```bash
   cp -r config/ /etc/nrcs/
   # Ensure production.toml has correct passwords
   ```
7. **Start services**:
   ```bash
   docker-compose up -d
   ```
8. **Verify**:
   ```bash
   curl http://localhost:17976/health
   curl http://localhost:17976/api/v1/blocks/latest
   ```
   Check block height matches expected.

## Disaster Recovery Testing

Test your restore procedure quarterly:

1. **Spin up test server** (different from production)
2. **Restore from latest backup**
3. **Verify chain syncs** to latest height
4. **Run validation checks**:
   - Account total supply matches expected
   - No orphaned blocks
   - Transactions execute correctly

Document recovery time (RTO) and data loss (RPO).

## Cloud Backups

Offsite backups are critical:

```bash
# Upload to S3
aws s3 cp /backups/nrcs/db/nrcs_20260101_020000.sql.gz s3://nrcs-backups/db/

# Or to Backblaze B2
b2 upload-file nrcs-backups /backups/nrcs/db/... sql.gz

# With lifecycle policy to expire after 90 days
```

Use encrypted backups:

```bash
gpg --batch --passphrase "$BACKUP_PASSPHRASE" \
  -c /backups/nrcs/db/nrcs_*.sql.gz
```

## Monitoring Backups

Set up alerts for:

- Backup job failures (non-zero exit code)
- Disk space usage > 80%
- Oldest backup older than 24h
- Corrupted backups (gzip test fails)

Example Prometheus alert:

```yaml
- alert: BackupFailed
  expr: backup_last_success_timestamp < time() - 3600
  for: 5m
```

## Security

- Encrypt sensitive backups at rest
- Store backups in separate location from production
- Use IAM roles with least privilege
- Rotate backup encryption keys annually
- Audit backup access logs

## Retention Policy

| Backup type | Frequency | Retention |
|-------------|-----------|-----------|
| Hourly DB dumps | Every hour | 7 days |
| Daily DB dumps | Midnight | 30 days |
| Weekly full node data | Sunday | 12 weeks |
| Monthly archive | 1st of month | Indefinitely (archival) |

Automatic cleanup:

```bash
# In backup script
find $BACKUP_DIR -name "*.sql.gz" -mtime +7 -delete  # hourly
find $BACKUP_DIR -name "*_daily.sql.gz" -mtime +30 -delete  # daily
```

## Cost Considerations

Estimate backup storage:

- **DB hourly dump**: 1GB compressed → 168GB/month
- **Node data**: 10GB/day → 300GB/month
- **Total**: ~500GB/month

Using S3 Standard-IA: ~$2.50/TB/month = ~$1.25/month (negligible).

## Appendix: Commands Reference

```bash
# List all backups
ls -lh /backups/nrcs/db/

# Test a specific backup
gzip -t /backups/nrcs/db/nrcs_20260101_020000.sql.gz

# List database sizes
docker-compose exec postgres psql -U nrcs -c "\l+"

# Check disk usage
docker-compose exec postgres df -h /var/lib/postgresql/data

# Verify backup integrity by test restore
gunzip -c /backups/nrcs/db/latest.sql.gz | \
  docker-compose exec -T postgres psql -U nrcs -d postgres -c "SELECT 1"
```
