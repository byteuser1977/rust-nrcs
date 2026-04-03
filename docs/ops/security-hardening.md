# Security Hardening Guide

Security best practices for deploying NRCS node in production.

## 1. Secrets Management

### Never commit secrets

- **Bad**: Hardcoding in config files
- **Good**: Environment variables or secrets manager

```bash
# .env file (DO NOT COMMIT)
POSTGRES_PASSWORD=super-secure-random-password
JWT_SECRET=64-byte-random-base64-encoded-secret
```

Add to `.gitignore`:
```
.env
config/production.toml
*.pem
*.key
```

### Generate strong secrets

```bash
# JWT secret (min 32 chars)
openssl rand -base64 64

# Database password
openssl rand -hex 32

# PostgreSQL password
pwgen -s 32 1
```

### Use Docker secrets (Swarm) or Kubernetes secrets

When using Docker Swarm:
```bash
echo "mysecret" | docker secret create postgres_password -
```

In `docker-compose.yml`:
```yaml
secrets:
  - postgres_password
```

## 2. Network Security

### Firewall configuration

```bash
# Only allow necessary ports
sudo ufw default deny incoming
sudo ufw allow 22/tcp     # SSH
sudo ufw allow 80/tcp     # HTTP (frontend)
sudo ufw allow 443/tcp    # HTTPS (frontend)
sudo ufw allow 17978/tcp  # P2P
sudo ufw enable
```

### TLS/SSL

**Always** enable TLS in production:

```toml
[security]
enable_tls = true
tls_cert_path = "/etc/nrcs/tls/cert.pem"
tls_key_path = "/etc/nrcs/tls/key.pem"
```

Get free certificates from Let's Encrypt:
```bash
# Using certbot with nginx
sudo certbot --nginx -d nrcs.yourdomain.com

# Or standalone
sudo certbot certonly --standalone -d nrcs.yourdomain.com
```

Mount certificates:
```yaml
volumes:
  - /etc/letsencrypt/live/nrcs.yourdomain.com:/etc/nrcs/tls:ro
```

### Rate limiting

Enable API rate limiting in config:

```toml
[api]
enable_rate_limit = true
rate_limit_requests_per_minute = 1000
```

And in nginx:
```nginx
location /api/ {
    limit_req zone=api burst=20 nodelay;
    proxy_pass http://backend:17976;
}
```

## 3. Container Security

### Use non-root users

Already configured in Dockerfiles:
- Backend: runs as `nrcs` (UID 1000)
- Frontend: runs as `nginxjs` (UID 1001)

Verify:
```bash
docker-compose exec backend id
# uid=1000(nrcs) gid=1000(nrcs)
```

### Read-only filesystem where possible

```yaml
services:
  backend:
    read_only: true
    tmpfs:
      - /tmp
      - /data  # if you need write
```

### No privileged mode

Ensure `privileged: false` (default) in compose file.

### Resource limits

Prevent DoS:
```yaml
services:
  backend:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G
```

## 4. Database Security

### Strong passwords

Use unique, random passwords for PostgreSQL:
```bash
# Generate
openssl rand -base64 32
```

Store in `.env` or secret manager.

### Separate database user

```sql
-- Create dedicated user
CREATE USER nrcs_app WITH PASSWORD 'secure-password';
GRANT CONNECT ON DATABASE nrcs TO nrcs_app;
GRANT USAGE ON SCHEMA public TO nrcs_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO nrcs_app;
```

### Enable pg_hba.conf restrictions

Edit `postgresql.conf` or use Docker environment:
```yaml
postgres:
  environment:
    POSTGRES_HOST_AUTH_METHOD: "scram-sha-256"
  command: ["postgres", "-c", "listen_addresses=localhost"]
```

Then configure `pg_hba.conf` to only allow localhost connections from Docker network.

### Regular backups with encryption

Already covered in backup guide. Ensure backups are encrypted at rest:
```bash
# Encrypt backup
gpg --batch --passphrase "$PASSPHRASE" -c backup.sql.gz
```

## 5. Application Security

### Keep dependencies updated

```bash
# Rust
cargo update

# Frontend
cd frontend && npm audit && npm update
```

Automate with Dependabot or Renovate.

### Input validation

All API endpoints validate input:
- Amounts checked for overflow
- Address formats validated
- Transaction signatures verified

Ensure custom endpoints also validate.

### SQL injection prevention

Using SQLx ensures compile-time query verification. Never use string concatenation for queries.

### JWT security

```toml
[security]
jwt_secret = "64+ random characters or 32-64 bytes hex"
jwt_expiry_hours = 24
```

Rotate JWT secret periodically (requires re-login for all users).

### CSP headers

Add Content Security Policy in nginx:
```nginx
add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline';" always;
```

## 6. Logging and Auditing

### Sensitive data redaction

Ensure passwords, keys, tokens never appear in logs:

```rust
// Bad
log::info!("Received password: {}", password);

// Good
log::debug!("Received auth request for user: {}", username);
// Never log password
```

### Centralized logging

Send logs tosyslog or external service:

```yaml
# docker-compose.yml
logging:
  driver: "syslog"
  options:
    syslog-address: "tcp://logserver:514"
    tag: "nrcs/backend"
```

### Log retention

Keep logs for at least 90 days for audit. Use logrotate if writing to files:
```bash
/var/log/nrcs/*.log {
  daily
  rotate 30
  compress
  delaycompress
  missingok
  notifempty
}
```

## 7. Identity and Access Control

### Docker access control

- Limit who can run `docker-compose up`
- Use separate Linux user for deployment
- Configure Docker daemon with `authorization-plugins` if multi-tenant

### API authentication

All state-changing operations require valid JWT. Ensure:
- `Authorization: Bearer <token>` header present
- Token not expired
- User has required role/permissions

Implement RBAC in future if needed.

## 8. Vulnerability Scanning

### Container scanning

```bash
# Scan with Trivy
trivy image rust-nrcs-backend:latest

# Integrate into CI
# GitHub Actions: aquasecurity/trivy-action
```

### Dependency scanning

Rust: `cargo audit`
```bash
cargo install cargo-audit
cargo audit
```

JavaScript: `npm audit`

### OS security updates

```bash
# On host
sudo apt-get update && sudo apt-get upgrade -y
# Rebuild containers to get latest base images
docker-compose build
docker-compose up -d
```

## 9. Penetration Testing Checklist

Before going live:

- [ ] Run OWASP ZAP baseline scan against API
- [ ] Test rate limiting effectiveness
- [ ] Verify TLS configuration (SSL Labs test)
- [ ] Check for information leakage in error messages
- [ ] Attempt SQL injection on all endpoints
- [ ] Test JWT token replay attacks
- [ ] VerifyDoS via large requests (file upload limits)
- [ ] Test P2P port for amplification attacks

## 10. Compliance

If operating in regulated environment (financial):

- **KYC/AML**: Implement identity verification for account creation
- **Data retention**: Configure logs to retain X years
- **Privacy**: GDPR right-to-be-forgotten requires account deletion capability
- **Audit trails**: Ensure all state changes are logged with user ID
- **Pen testing**: Annual third-party security audit

## Emergency Response

If compromised:

1. **Isolate**: Disconnect server from network
2. **Preserve**: Take disk image for forensic analysis
3. **Assess**: Determine breach scope (data leaked? funds stolen?)
4. **Notify**: Inform stakeholders, users if sensitive data exposed
5. **Remediate**: Rotate all secrets, patch vulnerabilities
6. **Restore**: From clean backup, change all passwords
7. **Document**: Root cause analysis and prevent recurrence

## Security Checklist

Pre-launch:

- [ ] All secrets rotated from defaults
- [ ] TLS enabled with valid certificate
- [ ] Firewall configured
- [ ] Rate limiting enabled
- [ ] Latest security patches applied
- [ ] Database user has minimal privileges
- [ ] Logs collected and monitored
- [ ] Intrusion detection (Fail2ban, auditd) configured
- [ ] Backup encryption enabled
- [ ] Penetration test completed

## Security Monitoring

Set up alerts for:

- Multiple failed login attempts
- Unusual spike in outbound connections
- Sudden increase in block reorgs (possible chain attack)
- New admin account creation
- Database backup failures

Example alertmanager rule:
```yaml
- alert: SuspiciousLoginAttempts
  expr: rate(http_requests_total{path="/api/v1/auth/login", status="401"}[5m]) > 10
  for: 2m
```

## References

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [CIS Docker Bench](https://www.cisecurity.org/benchmark/docker/)
- [PostgreSQL Security](https://www.postgresql.org/docs/current/security.html)
- [Rust Security](https://rust-lang.github.io/rfcs/2505-sneak.html)
