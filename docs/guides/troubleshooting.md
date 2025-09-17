# 🔧 Troubleshooting Guide

*Comprehensive guide for diagnosing and resolving common issues in the Rust AI IDE*

## Overview

This guide covers the most common issues encountered when using or maintaining the Rust AI IDE, along with step-by-step solutions and preventive measures.

## Quick Diagnosis Tools

### System Health Check Script

Run this script to quickly diagnose common issues:

```bash
#!/bin/bash
# Comprehensive health check for Rust AI IDE

echo "=== Rust AI IDE Health Check ==="
echo "Timestamp: $(date)"
echo "Host: $(hostname)"
echo

# Check system resources
echo "📊 System Resources:"
echo "CPU Cores: $(nproc)"
echo "Memory: $(free -h | grep '^Mem' | awk '{print $2}') total, $(free -h | grep '^Mem' | awk '{print $3}') used"
echo "Disk Space: $(df -h / | tail -1 | awk '{print $4}') available"
echo

# Check Rust installation
echo "🦀 Rust Environment:"
if command -v rustc &> /dev/null; then
    echo "✅ Rust installed: $(rustc --version)"
    echo "✅ Cargo available: $(cargo --version)"
else
    echo "❌ Rust not found - install from https://rustup.rs/"
fi
echo

# Check Node.js installation
echo "📦 Node.js Environment:"
if command -v node &> /dev/null; then
    echo "✅ Node.js installed: $(node --version)"
    echo "✅ NPM available: $(npm --version)"
else
    echo "❌ Node.js not found - install from https://nodejs.org/"
fi
echo

# Check application status
echo "🏗️ Application Status:"
if pgrep -f "rust-ai-ide" > /dev/null; then
    echo "✅ Rust AI IDE is running"
else
    echo "❌ Rust AI IDE is not running"
fi
echo

# Check AI services
echo "🤖 AI Services:"
if pgrep -f "ai-service" > /dev/null; then
    echo "✅ AI service is running"
else
    echo "❌ AI service is not running"
fi
echo

# Check LSP services
echo "🔤 LSP Services:"
if netstat -tln 2>/dev/null | grep -q ":11311 "; then
    echo "✅ LSP service is listening on port 11311"
else
    echo "❌ LSP service not responding on port 11311"
fi
echo

# Check logs for errors
echo "📝 Recent Errors:"
if [ -f "/var/log/rust-ai-ide/error.log" ]; then
    echo "Last 5 error entries:"
    tail -5 /var/log/rust-ai-ide/error.log
else
    echo "No error log found at /var/log/rust-ai-ide/error.log"
fi

echo
echo "=== Health Check Complete ==="
```

## Installation Issues

### Issue: Build Fails with "nightly toolchain required"

**Symptoms:**
- `cargo build` fails with "toolchain 'nightly-2025-09-03' is not installed"
- Compiler errors about unstable features

**Solution:**
```bash
# Install the required nightly toolchain
rustup toolchain install nightly-2025-09-03
rustup default nightly-2025-09-03

# Install required components
rustup component add rust-src rustfmt clippy

# Verify installation
rustc --version  # Should show nightly-2025-09-03
cargo --version
```

**Prevention:**
- Always use the exact nightly version specified in `rust-toolchain.toml`
- Run `rustup show` to verify the active toolchain

### Issue: GTK/WebKit Libraries Missing (Linux)

**Symptoms:**
- Build fails with "webkit2gtk-4.1 not found"
- Compilation errors about missing GTK libraries

**Solution:**
```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libglib2.0-dev \
    libgdk-pixbuf2.0-dev \
    libpango1.0-dev \
    libatk1.0-dev \
    libsoup-3.0-dev \
    libssl-dev \
    build-essential \
    pkg-config

# Fedora/CentOS
sudo dnf install webkit2gtk4.1-devel \
    gtk3-devel \
    glib2-devel \
    openssl-devel

# Arch Linux
sudo pacman -S webkit2gtk gtk3 glib2 openssl
```

### Issue: Node.js Dependencies Installation Fails

**Symptoms:**
- `npm install` fails with permission errors
- `pnpm install` fails with EACCES errors

**Solution:**
```bash
# Fix npm permissions (Option 1 - Recommended)
mkdir ~/.npm-global
npm config set prefix '~/.npm-global'
export PATH=~/.npm-global/bin:$PATH

# Fix npm permissions (Option 2 - System-wide)
sudo chown -R $(whoami) ~/.npm
sudo chown -R $(whoami) /usr/local/lib/node_modules

# Clear npm cache and retry
npm cache clean --force
rm -rf node_modules package-lock.json
npm install

# Alternative: Use pnpm with proper permissions
pnpm install --frozen-lockfile
```

## Runtime Issues

### Issue: Application Won't Start

**Symptoms:**
- Double-clicking the app icon does nothing
- Terminal shows "command not found" or permission errors
- Application crashes immediately on startup

**Troubleshooting Steps:**
```bash
# 1. Check file permissions
ls -la /opt/rust-ai-ide/rust-ai-ide
# Should be executable: -rwxr-xr-x

# 2. Check dependencies
ldd /opt/rust-ai-ide/rust-ai-ide
# Look for "not found" entries

# 3. Check system logs
journalctl -u rust-ai-ide --since "1 hour ago"

# 4. Run from terminal for detailed errors
/opt/rust-ai-ide/rust-ai-ide --verbose

# 5. Check configuration files
cat ~/.config/rust-ai-ide/config.toml
```

**Common Solutions:**
```bash
# Fix permissions
sudo chmod +x /opt/rust-ai-ide/rust-ai-ide

# Install missing system libraries
sudo apt-get install libgtk-3-0 libwebkit2gtk-4.1-0

# Clear corrupted configuration
rm -rf ~/.config/rust-ai-ide/
mkdir ~/.config/rust-ai-ide/
```

### Issue: AI Models Won't Load

**Symptoms:**
- AI features are disabled or unavailable
- Error: "Failed to load AI model"
- Performance is degraded without AI assistance

**Diagnosis:**
```bash
# Check AI service status
systemctl status rust-ai-ide-ai

# Check model directory permissions
ls -la ~/.rust-ai-ide/models/

# Verify model files exist
find ~/.rust-ai-ide/models/ -name "*.bin" -o -name "*.gguf"

# Check available disk space
df -h ~/.rust-ai-ide/
```

**Solutions:**
```bash
# 1. Restart AI service
sudo systemctl restart rust-ai-ide-ai

# 2. Clear model cache
rm -rf ~/.rust-ai-ide/models/cache/

# 3. Re-download models
/opt/rust-ai-ide/rust-ai-ide --download-models

# 4. Check memory availability
free -h
# Ensure at least 8GB RAM available

# 5. Verify GPU access (if applicable)
nvidia-smi  # or lspci | grep -i nvidia
```

### Issue: High Memory Usage

**Symptoms:**
- Application uses excessive RAM (>4GB)
- System becomes slow or unresponsive
- Out of memory errors

**Investigation:**
```bash
# Monitor memory usage
ps aux --sort=-%mem | head -10

# Check for memory leaks
valgrind --tool=memcheck /opt/rust-ai-ide/rust-ai-ide 2>&1 | head -20

# Monitor AI model memory
nvidia-smi --query-gpu=memory.used,memory.total --format=csv
```

**Solutions:**
```bash
# 1. Enable memory profiling
export RUST_AI_IDE_MEMORY_PROFILE=1
/opt/rust-ai-ide/rust-ai-ide

# 2. Adjust AI model settings
# Edit ~/.config/rust-ai-ide/config.toml
ai:
  memory_limit_mb: 2048
  unload_unused_models: true
  model_cache_ttl_seconds: 3600

# 3. Restart with memory optimization
/opt/rust-ai-ide/rust-ai-ide --memory-efficient

# 4. Clear caches
rm -rf ~/.rust-ai-ide/cache/
rm -rf ~/.cargo/registry/cache/
```

### Issue: LSP Server Connection Failed

**Symptoms:**
- No code intelligence (autocomplete, diagnostics)
- Error: "Language server connection lost"
- Files not recognized or syntax highlighting missing

**Troubleshooting:**
```bash
# Check LSP service status
systemctl status rust-ai-ide-lsp

# Test LSP connection
telnet localhost 11311

# Check LSP logs
tail -f /var/log/rust-ai-ide/lsp.log

# Verify port configuration
netstat -tlnp | grep 11311
```

**Solutions:**
```bash
# 1. Restart LSP service
sudo systemctl restart rust-ai-ide-lsp

# 2. Check firewall settings
sudo ufw allow 11311/tcp
# or
sudo iptables -A INPUT -p tcp --dport 11311 -j ACCEPT

# 3. Reconfigure LSP settings
# Edit ~/.config/rust-ai-ide/lsp.toml
lsp:
  port: 11311
  host: "127.0.0.1"
  timeout_seconds: 30
  retry_attempts: 3

# 4. Clear LSP cache
rm -rf ~/.rust-ai-ide/lsp-cache/
```

## Performance Issues

### Issue: Slow Startup Time

**Symptoms:**
- Application takes >30 seconds to start
- Loading screen appears for extended periods

**Performance Analysis:**
```bash
# Time application startup
time /opt/rust-ai-ide/rust-ai-ide --startup-benchmark

# Profile startup process
perf record -g /opt/rust-ai-ide/rust-ai-ide
perf report

# Check disk I/O performance
iostat -x 1 5
```

**Optimizations:**
```bash
# 1. Enable lazy loading
# Edit ~/.config/rust-ai-ide/config.toml
startup:
  lazy_load_ai: true
  deferred_initialization: true
  background_services: true

# 2. Optimize system settings
echo "vm.swappiness=10" | sudo tee -a /etc/sysctl.conf
sudo sysctl -p

# 3. Use faster storage
# Move application to SSD if possible
lsblk -o NAME,ROTA,MOUNTPOINT
# ROTA=0 indicates SSD

# 4. Preload libraries
echo "/opt/rust-ai-ide/lib/libwebkit2gtk-4.1.so" | sudo tee -a /etc/ld-musl-x86_64.path
```

### Issue: UI Responsiveness Problems

**Symptoms:**
- UI freezes or becomes unresponsive
- High CPU usage during normal operations
- Delayed response to user input

**Investigation:**
```bash
# Monitor CPU usage
top -p $(pgrep rust-ai-ide)

# Check for blocking operations
strace -p $(pgrep rust-ai-ide) -c

# Profile UI thread
perf record -g -p $(pgrep rust-ai-ide) -- sleep 10
perf report
```

**Solutions:**
```bash
# 1. Enable UI optimization
# Edit ~/.config/rust-ai-ide/config.toml
ui:
  enable_hardware_acceleration: true
  async_rendering: true
  background_processing: true

# 2. Adjust thread pool settings
export RUST_AI_IDE_THREAD_POOL_SIZE=4

# 3. Clear UI cache
rm -rf ~/.cache/rust-ai-ide/

# 4. Restart with clean state
pkill rust-ai-ide
sleep 2
/opt/rust-ai-ide/rust-ai-ide &
```

## Network and Connectivity Issues

### Issue: Proxy Configuration Problems

**Symptoms:**
- Unable to download dependencies or models
- Network timeouts during installation
- "Connection refused" errors

**Diagnosis:**
```bash
# Test proxy configuration
curl -I --proxy $http_proxy https://crates.io

# Check proxy settings
env | grep -i proxy

# Test direct connection
curl -I https://crates.io --noproxy
```

**Solutions:**
```bash
# 1. Configure proxy settings
export http_proxy="http://proxy.company.com:3128"
export https_proxy="http://proxy.company.com:3128"
export no_proxy="localhost,127.0.0.1,.local"

# 2. Configure Cargo proxy
mkdir -p ~/.cargo
cat > ~/.cargo/config.toml << EOF
[http]
proxy = "http://proxy.company.com:3128"

[https]
proxy = "http://proxy.company.com:3128"
EOF

# 3. Configure npm proxy
npm config set proxy http://proxy.company.com:3128
npm config set https-proxy http://proxy.company.com:3128

# 4. Test configuration
cargo search serde  # Should work through proxy
```

### Issue: WebAuthn Authentication Fails

**Symptoms:**
- Unable to register or use hardware security keys
- "WebAuthn not supported" error
- Biometric authentication not working

**Troubleshooting:**
```bash
# Check browser compatibility
# WebAuthn requires HTTPS or localhost
curl -I https://localhost:3000

# Test hardware key detection
lsusb | grep -i yubi  # YubiKey
lsusb | grep -i google  # Google Titan

# Check browser permissions
# Chrome: chrome://settings/content/securityKeys
# Firefox: about:config (security.webauthn.enable)

# Verify system libraries
dpkg -l | grep libu2f  # Linux U2F library
```

**Solutions:**
```bash
# 1. Enable HTTPS for local development
# Edit src-tauri/tauri.conf.json
{
  "security": {
    "csp": "default-src 'self' https:; script-src 'self' 'unsafe-eval'"
  },
  "windows": [{
    "url": "https://localhost:3000"
  }]
}

# 2. Install U2F system libraries
sudo apt-get install libu2f-udev

# 3. Add udev rules for hardware keys
sudo tee /etc/udev/rules.d/70-u2f.rules > /dev/null << EOF
# YubiKey
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="1050", ATTRS{idProduct}=="0113|0114|0115|0116|0120|0200|0402|0403|0406|0407|0410", TAG+="uaccess"
# Google Titan
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="18d1", ATTRS{idProduct}=="5026", TAG+="uaccess"
EOF

sudo udevadm control --reload-rules
sudo udevadm trigger

# 4. Restart application
pkill rust-ai-ide
sleep 2
/opt/rust-ai-ide/rust-ai-ide &
```

## Security Issues

### Issue: Certificate Validation Errors

**Symptoms:**
- "SSL certificate problem" errors
- Unable to connect to secure endpoints
- Certificate verification fails

**Resolution:**
```bash
# 1. Update CA certificates
sudo apt-get install ca-certificates
sudo update-ca-certificates

# 2. Add corporate CA certificates
sudo cp /path/to/corporate-ca.crt /usr/local/share/ca-certificates/
sudo update-ca-certificates

# 3. Verify certificate chain
openssl s_client -connect api.rust-ai-ide.dev:443 -servername api.rust-ai-ide.dev

# 4. Configure custom CA path
export SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
```

### Issue: Permission Denied Errors

**Symptoms:**
- "Permission denied" when accessing files or directories
- Unable to save files or create directories
- Application cannot access user data

**Diagnosis:**
```bash
# Check file permissions
ls -la ~/.rust-ai-ide/
ls -la ~/.config/rust-ai-ide/

# Check user groups
groups $USER

# Check SELinux/AppArmor status
sestatus  # SELinux
sudo apparmor_status  # AppArmor
```

**Solutions:**
```bash
# 1. Fix ownership
sudo chown -R $USER:$USER ~/.rust-ai-ide/
sudo chown -R $USER:$USER ~/.config/rust-ai-ide/

# 2. Set proper permissions
chmod 755 ~/.rust-ai-ide/
chmod 644 ~/.config/rust-ai-ide/config.toml

# 3. Disable SELinux for troubleshooting
sudo setenforce 0  # Temporary - not recommended for production

# 4. Configure AppArmor profile
sudo tee /etc/apparmor.d/rust-ai-ide > /dev/null << EOF
#include <tunables/global>

/opt/rust-ai-ide/rust-ai-ide {
  #include <abstractions/base>
  #include <abstractions/nameservice>
  #include <abstractions/user-tmp>

  /opt/rust-ai-ide/ r,
  /opt/rust-ai-ide/** r,
  /home/*/ mr,
  /tmp/ mrw,
}
EOF

sudo apparmor_parser -r /etc/apparmor.d/rust-ai-ide
```

## Database Issues

### Issue: SQLite Database Corruption

**Symptoms:**
- Application fails to start with database errors
- "Database disk image is malformed" errors
- Inconsistent data or missing records

**Recovery:**
```bash
# 1. Backup corrupted database
cp ~/.rust-ai-ide/database.db ~/.rust-ai-ide/database.db.backup

# 2. Run SQLite integrity check
sqlite3 ~/.rust-ai-ide/database.db "PRAGMA integrity_check;"

# 3. Export data if possible
sqlite3 ~/.rust-ai-ide/database.db ".dump" > database_dump.sql

# 4. Recreate database from backup
sqlite3 ~/.rust-ai-ide/database.db < database_dump.sql

# 5. Rebuild indexes
sqlite3 ~/.rust-ai-ide/database.db "REINDEX;"

# 6. Verify recovery
sqlite3 ~/.rust-ai-ide/database.db "SELECT COUNT(*) FROM projects;"
```

### Issue: Database Performance Degradation

**Symptoms:**
- Slow application response times
- Database queries taking >1 second
- High disk I/O when accessing database

**Optimization:**
```bash
# 1. Analyze database performance
sqlite3 ~/.rust-ai-ide/database.db "ANALYZE;"

# 2. Check table statistics
sqlite3 ~/.rust-ai-ide/database.db "SELECT * FROM sqlite_stat1;"

# 3. Rebuild with better settings
sqlite3 ~/.rust-ai-ide/database.db << EOF
PRAGMA journal_mode=WAL;
PRAGMA synchronous=NORMAL;
PRAGMA cache_size=-64000;
PRAGMA temp_store=MEMORY;
VACUUM;
EOF

# 4. Monitor database size
ls -lh ~/.rust-ai-ide/database.db*
du -sh ~/.rust-ai-ide/
```

## Logging and Monitoring

### Issue: Excessive Log File Sizes

**Symptoms:**
- Disk space running low due to large log files
- Application performance degraded by logging overhead

**Log Management:**
```bash
# 1. Check log sizes
du -sh /var/log/rust-ai-ide/
find /var/log/rust-ai-ide/ -name "*.log" -size +100M

# 2. Configure log rotation
sudo tee /etc/logrotate.d/rust-ai-ide > /dev/null << EOF
/var/log/rust-ai-ide/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 644 root root
    postrotate
        systemctl reload rust-ai-ide
    endscript
}
EOF

# 3. Adjust logging levels
# Edit ~/.config/rust-ai-ide/logging.toml
logging:
  level: "INFO"  # Change from DEBUG to INFO
  max_file_size_mb: 100
  max_files: 5

# 4. Compress old logs
find /var/log/rust-ai-ide/ -name "*.log.1" -exec gzip {} \;
```

### Issue: Missing or Incomplete Logs

**Symptoms:**
- No logs generated for troubleshooting
- Log files are empty or contain no useful information
- Unable to diagnose application issues

**Log Configuration:**
```bash
# 1. Enable detailed logging
export RUST_LOG=rust_ai_ide=debug,ai_service=debug,lsp=info
export RUST_BACKTRACE=full

# 2. Check log file permissions
ls -la /var/log/rust-ai-ide/
sudo chown rust-ai-ide:rust-ai-ide /var/log/rust-ai-ide/

# 3. Configure structured logging
# Edit ~/.config/rust-ai-ide/logging.toml
logging:
  format: "json"  # or "text"
  output: "file"  # or "stdout", "syslog"
  level: "DEBUG"
  include_timestamps: true
  include_request_ids: true

# 4. Test logging configuration
/opt/rust-ai-ide/rust-ai-ide --test-logging
tail -f /var/log/rust-ai-ide/app.log
```

## Advanced Troubleshooting

### Memory Leak Investigation

```bash
# 1. Install memory profiling tools
cargo install heaptrack

# 2. Profile memory usage
heaptrack /opt/rust-ai-ide/rust-ai-ide

# 3. Analyze heap dump
heaptrack --analyze heaptrack.rust-ai-ide.*

# 4. Alternative: Use valgrind
valgrind --tool=massif /opt/rust-ai-ide/rust-ai-ide
ms_print massif.out.*
```

### Performance Profiling

```bash
# 1. CPU profiling
perf record -g /opt/rust-ai-ide/rust-ai-ide -- sleep 30
perf report

# 2. Memory profiling
valgrind --tool=cachegrind /opt/rust-ai-ide/rust-ai-ide
cg_annotate cachegrind.out.*

# 3. SystemTap/DTrace (Linux)
stap -e 'probe process("/opt/rust-ai-ide/rust-ai-ide").function("*") { println($$parms) }'
```

### Network Debugging

```bash
# 1. Monitor network traffic
tcpdump -i any port 11311 -w lsp_traffic.pcap

# 2. Check DNS resolution
dig api.rust-ai-ide.dev
nslookup api.rust-ai-ide.dev

# 3. Test connectivity with specific protocols
openssl s_client -connect api.rust-ai-ide.dev:443 -tls1_3
curl -v https://api.rust-ai-ide.dev/health
```

## Emergency Procedures

### Complete System Reset

**⚠️ WARNING: This will remove all user data and settings**

```bash
# 1. Stop all services
sudo systemctl stop rust-ai-ide rust-ai-ide-ai rust-ai-ide-lsp

# 2. Backup important data (optional)
tar -czf ~/rust-ai-ide-backup-$(date +%Y%m%d).tar.gz ~/.rust-ai-ide/

# 3. Remove all data and configurations
rm -rf ~/.rust-ai-ide/
rm -rf ~/.config/rust-ai-ide/
sudo rm -rf /var/lib/rust-ai-ide/
sudo rm -rf /var/log/rust-ai-ide/

# 4. Clean system caches
rm -rf ~/.cargo/registry/cache/
rm -rf ~/.npm/_cacache/

# 5. Reinstall application
./corporate_install_rust_ai_ide.sh

# 6. Restore from backup if needed
# tar -xzf ~/rust-ai-ide-backup-*.tar.gz -C ~/
```

### Emergency Shutdown Procedure

```bash
# 1. Graceful shutdown attempt
sudo systemctl stop rust-ai-ide

# 2. Force kill if necessary
pkill -9 rust-ai-ide
pkill -9 ai-service
pkill -9 lsp-service

# 3. Clean up shared memory
ipcs -m | grep rust | awk '{print $2}' | xargs -r ipcrm -m
ipcs -s | grep rust | awk '{print $2}' | xargs -r ipcrm -s

# 4. Reset system resources
echo 3 > /proc/sys/vm/drop_caches  # Clear page cache
```

## Prevention Best Practices

### Regular Maintenance Schedule

```bash
# Daily maintenance
@daily /opt/rust-ai-ide/maintenance.sh --daily

# Weekly maintenance
@weekly /opt/rust-ai-ide/maintenance.sh --weekly

# Monthly maintenance
@monthly /opt/rust-ai-ide/maintenance.sh --monthly
```

### Monitoring Setup

```bash
# Install monitoring stack
sudo apt-get install prometheus prometheus-node-exporter grafana

# Configure alerts
cat > /etc/prometheus/alert_rules.yml << EOF
groups:
  - name: rust-ai-ide
    rules:
      - alert: RustAIIDE_Down
        expr: up{job="rust-ai-ide"} == 0
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Rust AI IDE is down"
EOF
```

### Backup Strategy

```bash
# Automated backup script
cat > /opt/rust-ai-ide/backup.sh << 'EOF'
#!/bin/bash
BACKUP_DIR="/var/backups/rust-ai-ide"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup directory
mkdir -p $BACKUP_DIR

# Backup database
sqlite3 ~/.rust-ai-ide/database.db ".backup '$BACKUP_DIR/database_$DATE.db'"

# Backup configuration
tar -czf $BACKUP_DIR/config_$DATE.tar.gz ~/.config/rust-ai-ide/

# Backup models (if needed)
tar -czf $BACKUP_DIR/models_$DATE.tar.gz ~/.rust-ai-ide/models/

# Rotate old backups (keep last 7 days)
find $BACKUP_DIR -name "*.db" -mtime +7 -delete
find $BACKUP_DIR -name "*.tar.gz" -mtime +7 -delete

echo "Backup completed: $BACKUP_DIR"
EOF

# Setup cron job
echo "0 2 * * * root /opt/rust-ai-ide/backup.sh" > /etc/cron.d/rust-ai-ide-backup
```

## Support Resources

### Getting Help

1. **Check Application Logs:**
   ```bash
   tail -f /var/log/rust-ai-ide/app.log
   journalctl -u rust-ai-ide --follow
   ```

2. **Run Diagnostics:**
   ```bash
   /opt/rust-ai-ide/diagnostics.sh
   ```

3. **Check System Health:**
   ```bash
   /opt/rust-ai-ide/health-check.sh
   ```

4. **Collect Debug Information:**
   ```bash
   /opt/rust-ai-ide/collect-debug-info.sh
   ```

### Support Contacts

- **Enterprise Support**: enterprise@rust-ai-ide.dev
- **Technical Issues**: support@rust-ai-ide.dev
- **Security Issues**: security@rust-ai-ide.dev
- **Documentation**: docs@rust-ai-ide.dev

### Community Resources

- **GitHub Issues**: https://github.com/jcn363/rust-ai-ide/issues
- **Community Forums**: https://community.rust-ai-ide.dev
- **Knowledge Base**: https://kb.rust-ai-ide.dev
- **Status Page**: https://status.rust-ai-ide.dev