# 🏢 Corporate Environment Setup Guide

*Comprehensive guide for IT administrators deploying the Rust AI IDE in enterprise environments*

**⏱️ Estimated Setup Time: 2-4 hours | 📊 Difficulty: Intermediate | 🎯 Audience: Enterprise IT Admins**

---

## 📋 Overview

This guide covers complete enterprise deployment of the Rust AI IDE including:
- ✅ Corporate network configuration and proxy support
- ✅ Security hardening and compliance setup
- ✅ Group Policy integration for Windows environments
- ✅ Integration with enterprise identity providers
- ✅ Automated deployment and configuration management
- ✅ Monitoring and maintenance procedures

---

## 🔧 Pre-Deployment Assessment

### System Requirements Verification

#### Corporate Environment Assessment

**Network Requirements:**
```bash
# Test connectivity to required services
curl -I https://github.com                     # GitHub access
curl -I https://crates.io                       # Cargo registry
curl -I https://registry.npmjs.org              # NPM registry
curl -I https://static.rust-lang.org            # Rust toolchain

# Test proxy configuration
export http_proxy=http://proxy.company.com:3128
export https_proxy=http://proxy.company.com:3128
curl -I https://github.com                    # Should work through proxy
```

**Hardware Requirements Assessment:**
```bash
# Create hardware assessment report
cat > corporate_hardware_assessment.sh << 'EOF'
#!/bin/bash
echo "=== Corporate Hardware Assessment ==="
echo "Host: $(hostname)"
echo "OS: $(lsb_release -d | cut -f2)"
echo "CPU: $(nproc) cores"
echo "RAM: $(free -h | grep '^Mem' | awk '{print $2}')"
echo "Disk: $(df -h / | tail -1 | awk '{print $4}') free"
echo "Architecture: $(uname -m)"

# Check GPU if available
if command -v nvidia-smi &> /dev/null; then
    echo "GPU: NVIDIA $(nvidia-smi --query-gpu=name --format=csv,noheader)"
    echo "GPU Memory: $(nvidia-smi --query-gpu=memory.total --format=csv,noheader)"
else
    echo "GPU: None or not NVIDIA"
fi

echo "=== Assessment Complete ==="
EOF

chmod +x corporate_hardware_assessment.sh
./corporate_hardware_assessment.sh > assessment_$(hostname)_$(date +%Y%m%d).txt
```

---

## 🌐 Network Configuration

### Enterprise Proxy Setup

#### System-Wide Proxy Configuration

```bash
# Create corporate proxy configuration
sudo tee /etc/environment.d/proxy.conf > /dev/null << EOF
# Corporate Proxy Configuration
# Generated: $(date)

# HTTP Proxy
http_proxy=http://proxy.company.com:3128/
https_proxy=http://proxy.company.com:3128/
ftp_proxy=http://proxy.company.com:3128/
no_proxy=localhost,127.0.0.1,.local,.internal,company.com

# Rust/Cargo proxy configuration
export CARGO_HTTP_PROXY=http://proxy.company.com:3128/
export CARGO_HTTP_TIMEOUT=300

# NPM proxy configuration (if using npm instead of pnpm)
export npm_config_proxy=http://proxy.company.com:3128/
export npm_config_https_proxy=http://proxy.company.com:3128/

# Git proxy configuration
git config --global http.proxy http://proxy.company.com:3128
git config --global https.proxy http://proxy.company.com:3128
EOF

# Reload environment
sudo systemctl daemon-reload
source /etc/environment.d/proxy.conf
```

#### Certificate Authority Integration

```bash
# Install corporate CA certificates
sudo mkdir -p /usr/local/share/ca-certificates/company

# Copy corporate CA certificates (assuming you have them)
# This would typically be done by your security team
sudo cp /path/to/corporate/ca-chain.crt /usr/local/share/ca-certificates/company/

# Update certificate store
sudo update-ca-certificates

# Verify certificate installation
curl -I --cacert /etc/ssl/certs/ca-certificates.crt https://internal.company.com
```

### Firewall and Security Policies

#### Corporate Firewall Configuration

```bash
# Allow required ports and services
sudo ufw enable

# Required for Rust AI IDE
sudo ufw allow ssh
sudo ufw allow 11311/tcp                # LSP (if remote)
sudo ufw allow 3000/tcp                 # Development server (optional)
sudo ufw allow 5173/tcp                 # Vite dev server (optional)

# Allow Cargo registry access
sudo ufw allow out to any port 80,443 proto tcp

# Block direct internet access, route through proxy
# (This would typically be handled by corporate proxy/firewall policies)
```

---

## 🚀 Installation & Deployment

### Automated Corporate Installation

#### Bash Installation Script for Linux

```bash
# Create enterprise installation script
cat > corporate_install_rust_ai_ide.sh << 'EOF'
#!/bin/bash
set -e

# Enterprise Rust AI IDE Installation Script
# Version: 2.4.0
# Date: 2025-09-02

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="/var/log/rust-ai-ide-install_$(date +%Y%m%d_%H%M%S).log"

# Logging functions
log() {
    echo "$(date +'%Y-%m-%d %H:%M:%S') - $*" | tee -a "$LOG_FILE"
}

error() {
    echo "$(date +'%Y-%m-%d %H:%M:%S') - ERROR: $*" >&2 | tee -a "$LOG_FILE"
    exit 1
}

# Pre-flight checks
preflight_checks() {
    log "Starting pre-flight checks..."

    # Check if running as root for system installation
    if [[ $EUID -eq 0 ]]; then
        INSTALL_TYPE="system"
        INSTALL_DIR="/opt/rust-ai-ide"
    else
        INSTALL_TYPE="user"
        INSTALL_DIR="$HOME/.rust-ai-ide"
    fi

    # Verify system requirements
    command -v curl >/dev/null 2>&1 || error "curl is required"
    command -v wget >/dev/null 2>&1 || error "wget is required"
    free -m | awk 'NR==2{printf "\nRAM: %.0fGB\n", $2/1024}'
    df -BG / | tail -1 | awk '{print "Disk space: " $4}'

    # Check network connectivity
    if curl -s --connect-timeout 10 https://github.com > /dev/null; then
        log "Network connectivity: OK"
    else
        error "Cannot connect to GitHub - check proxy settings"
    fi
}

# Install Rust toolchain
install_rust() {
    log "Installing Rust toolchain..."

    if ! command -v rustc >/dev/null 2>&1; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi

    # Install required components
    rustup component add rustfmt clippy rust-analyzer

    log "Rust version: $(rustc --version)"
}

# Install Node.js and pnpm
install_nodejs() {
    log "Installing Node.js and pnpm..."

    if ! command -v node >/dev/null 2>&1; then
        curl -fsSL https://deb.nodesource.com/setup_lts.x | bash -
        apt-get install -y nodejs
    fi

    if ! command -v pnpm >/dev/null 2>&1; then
        npm install -g pnpm
    fi

    log "Node.js version: $(node --version)"
    log "PNPM version: $(pnpm --version)"
}

# Install system dependencies
install_system_deps() {
    log "Installing system dependencies..."

    apt-get update

    # Core dependencies
    apt-get install -y \
        libwebkit2gtk-4.1-dev \
        libgtk-3-dev \
        libglib2.0-dev \
        libgdk-pixbuf2.0-dev \
        libpango1.0-dev \
        libatk1.0-dev \
        libsoup-3.0-dev \
        libssl-dev \
        build-essential \
        pkg-config

    # Optional: GPU support
    if lspci | grep -i nvidia > /dev/null; then
        log "Installing NVIDIA GPU support..."
        apt-get install -y nvidia-cuda-toolkit
    fi
}

# Clone and build Rust AI IDE
build_rust_ai_ide() {
    log "Building Rust AI IDE..."

    local build_dir="/tmp/rust-ai-ide-build-$(date +%s)"
    mkdir -p "$build_dir"
    cd "$build_dir"

    # Clone repository
    git clone https://github.com/jcn363/rust-ai-ide.git .

    # Install Node.js dependencies
    pnpm install

    # Configure for enterprise
    cp "$SCRIPT_DIR/enterprise-config.toml" ./src-tauri/tauri.conf.json

    # Build application
    pnpm tauri build

    # Install to target directory
    mkdir -p "$INSTALL_DIR"
    cp -r src-tauri/target/release/bundle/* "$INSTALL_DIR/"

    # Cleanup
    cd /
    rm -rf "$build_dir"

    log "Installation completed successfully"
    log "Installed to: $INSTALL_DIR"
}

# Main installation function
main() {
    log "=== Enterprise Rust AI IDE Installation Started ==="
    log "Installation type: $INSTALL_TYPE"
    log "Target directory: $INSTALL_DIR"

    preflight_checks
    install_rust
    install_nodejs
    install_system_deps
    build_rust_ai_ide

    log "=== Installation Completed Successfully ==="
    log "Installation log: $LOG_FILE"

    # Post-installation instructions
    cat << INSTALLEOF

Next steps:
1. Configure your corporate settings (see enterprise-config.toml)
2. Test the installation: $INSTALL_DIR/rust-ai-ide --version
3. Configure AI models if needed (see ai-model-configuration.md)
4. Set up user profiles for team members

Installation log available at: $LOG_FILE

INSTALLEOF
}

# Run main function
main "$@"
EOF

chmod +x corporate_install_rust_ai_ide.sh
```

### Enterprise Configuration Template

```toml
# Corporate Configuration Template
# /etc/rust-ai-ide/corporate-config.toml

[enterprise]
# Company information
company_name = "Your Company Inc."
department = "Development"
contact_email = "it-support@company.com"

[security]
# Security settings
force_https = true
certificate_validation = "strict"
audit_logging = true
session_timeout = 480  # 8 hours

[network]
# Network configuration
proxy_url = "http://proxy.company.com:3128"
timeout_seconds = 300
retry_attempts = 3

[ai]
# AI service configuration
provider = "azure"  # or "openai", "local"
endpoint = "https://your-company.openai.azure.com/"
rate_limiting = true
max_tokens_per_minute = 100000

[compliance]
# Compliance settings
data_retention_days = 2555  # 7 years for SOX compliance
gdpr_compliance = true
audit_trail = true
encryption_at_rest = true

[telemetry]
# Enterprise telemetry (minimal required for support)
enabled = false  # Disabled by default for privacy
crash_reporting = true
performance_metrics = false
usage_statistics = false
```

---

## 🔐 Security Configuration

### Corporate Security Policies

```bash
# Create secure application directory
sudo mkdir -p /opt/rust-ai-ide/data
sudo chown root:rust-ai-ide-users /opt/rust-ai-ide/data
sudo chmod 750 /opt/rust-ai-ide/data

# Configure SELinux/AppArmor policies if applicable
sudo setsebool -P allow_execstack 0
sudo setsebool -P allow_execmod 0

# Network security
sudo iptables -A INPUT -p tcp --dport 11311 -s your-corporate-network/24 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 11311 -j DROP
```

### Encryption and Data Protection

```bash
# Setup encryption for sensitive data
cat > setup_encryption.sh << 'EOF'
#!/bin/bash

# Generate encryption keys
openssl genrsa -out /etc/rust-ai-ide/private.key 4096
openssl rsa -in /etc/rust-ai-ide/private.key -pubout -out /etc/rust-ai-ide/public.key

# Create encrypted configuration
ansible-vault encrypt enterprise-secrets.yml --vault-password-file vault-password.txt

echo "Encryption setup complete"
echo "Keys created in /etc/rust-ai-ide/"
echo "Encrypted secrets in enterprise-secrets.yml"
EOF
```

### Identity Provider Integration

```yaml
# Azure AD Configuration Example
auth:
  provider: azure_ad
  tenant_id: "your-tenant-id"
  client_id: "your-client-id"
  authority: "https://login.microsoftonline.com/your-tenant-id"
  scope: "api://your-api-id/access"
  redirect_uri: "rust-ai-ide://auth/callback"

# LDAP Configuration Example
ldap:
  server: "ldap.company.com"
  port: 636
  ssl: true
  base_dn: "dc=company,dc=com"
  user_search: "(sAMAccountName={username})"
  bind_dn: "cn=service-account,ou=service-accounts,dc=company,dc=com"
```

---

## ⚙️ Group Policy & Centralized Management

### Windows Group Policy Configuration

#### GPO Template for Windows Deployment

```powershell
# Create GPO for Rust AI IDE

# 1. Computer Configuration → Preferences → Windows Settings → Registry

# Registry key: HKLM\SOFTWARE\Rust AI IDE
# Value: InstallPath, REG_SZ, "C:\Program Files\Rust AI IDE"

# Registry key: HKLM\SOFTWARE\Rust AI IDE
# Value: ConfigPath, REG_SZ, "\\server\share\rust-ai-ide\config"

# 2. User Configuration → Preferences → Windows Settings → Files

# Source: \\server\share\rust-ai-ide\user-config.toml
# Destination: C:\Users\%USERNAME%\.rust-ai-ide\config.toml
# Action: Update

# 3. Computer Configuration → Policies → Administrative Templates → System → Group Policy

# Enable "Specify intranet Microsoft update service location"
# Set intranet update service: http://wsus.company.com
# Set intranet statistics server: http://wsus.company.com

# PowerShell script for installation:
$installerPath = "\\server\share\software\rust-ai-ide\coprporate_install.ps1"
& $installerPath -Silent -CompanyConfig "\\server\share\rust-ai-ide\company-config.toml"
```

### Linux Configuration Management

```yaml
# Ansible playbook for corporate deployment
---
- name: Deploy Rust AI IDE to corporate workstations
  hosts: developer_workstations
  become: yes
  vars:
    install_dir: /opt/rust-ai-ide
    config_dir: /etc/rust-ai-ide
    data_dir: /var/lib/rust-ai-ide

  tasks:
    - name: Install system dependencies
      package:
        name:
          - libwebkit2gtk-4.1-dev
          - libgtk-3-dev
          - build-essential
          - nodejs
          - npm
        state: present

    - name: Create application directories
      file:
        path: "{{ item }}"
        state: directory
        mode: '0755'
        owner: root
        group: developers
      loop:
        - "{{ install_dir }}"
        - "{{ config_dir }}"
        - "{{ data_dir }}"

    - name: Deploy corporate configuration
      copy:
        src: corporate-config.toml
        dest: "{{ config_dir }}/corporate-config.toml"
        mode: '0644'
        owner: root
        group: developers

    - name: Run corporate installation script
      script: corporate_install_rust_ai_ide.sh
      args:
        creates: "{{ install_dir }}/rust-ai-ide"

    - name: Configure systemd service
      template:
        src: rust-ai-ide.service.j2
        dest: /etc/systemd/system/rust-ai-ide.service
        mode: '0644'

    - name: Start and enable service
      systemd:
        name: rust-ai-ide
        state: started
        enabled: yes

    - name: Validate installation
      command: "{{ install_dir }}/rust-ai-ide --version"
      register: version_output
      failed_when: version_output.rc != 0

    - name: Log installation completion
      debug:
        msg: "Rust AI IDE {{ version_output.stdout }} installed successfully"
```

---

## 📊 Monitoring & Maintenance

### Corporate Monitoring Setup

```bash
# Install monitoring stack
cat > setup_monitoring.sh << 'EOF'
#!/bin/bash

# Create monitoring directories
sudo mkdir -p /var/log/rust-ai-ide/monitoring
sudo chown rust-ai-ide:rust-ai-ide /var/log/rust-ai-ide/monitoring

# Configure systemd journal gateway
sudo mkdir -p /etc/systemd/journald.conf.d
cat > /etc/systemd/journald.conf.d/rust-ai-ide.conf << EOF
[Journal]
Storage=persistent
SystemMaxUse=100M
RuntimeMaxUse=50M
ForwardToSyslog=yes
EOF

# Setup log aggregation (example with rsyslog)
cat > /etc/rsyslog.d/rust-ai-ide.conf << EOF
# Forward Rust AI IDE logs to central server
*.* @logserver.company.com:514
EOF

sudo systemctl restart rsyslog

echo "Monitoring setup complete"
echo "Logs will be forwarded to: logserver.company.com"
EOF
```

### Maintenance Procedures

```bash
# Create corporate maintenance script
cat > corporate_maintenance.sh << 'EOF'
#!/bin/bash

SCRIPT_VERSION="1.0.0"
LOG_FILE="/var/log/rust-ai-ide/maintenance_$(date +%Y%m%d_%H%M%S).log"

log() { echo "$(date +'%Y-%m-%d %H:%M:%S') - $*" | tee -a "$LOG_FILE"; }

# Daily maintenance
daily_maintenance() {
    log "Starting daily maintenance..."

    # Rotate logs
    /usr/sbin/logrotate /etc/logrotate.d/rust-ai-ide

    # Update antivirus definitions
    if command -v freshclam &> /dev/null; then
        freshclam
    fi

    # Check disk space
    DISK_USAGE=$(df / | tail -1 | awk '{print $5}' | sed 's/%//')
    if [ "$DISK_USAGE" -gt 90 ]; then
        log "WARNING: Disk usage is at ${DISK_USAGE}%"
    fi

    # Verify service status
    if systemctl is-active --quiet rust-ai-ide; then
        log "Rust AI IDE service: RUNNING"
    else
        log "ERROR: Rust AI IDE service is not running"
        systemctl start rust-ai-ide
    fi
}

# Weekly maintenance
weekly_maintenance() {
    log "Starting weekly maintenance..."

    # Security updates
    apt-get update
    apt-get upgrade -y --security

    # Clear old cache
    /opt/rust-ai-ide/bin/rust-ai-ide --clear-cache

    # Verify configuration files
    if [ -f /etc/rust-ai-ide/corporate-config.toml ]; then
        log "Corporate configuration: PRESENT"
    else
        log "ERROR: Corporate configuration missing"
    fi
}

# Usage
case "${1:-daily}" in
    daily)
        daily_maintenance
        ;;
    weekly)
        weekly_maintenance
        ;;
    *)
        echo "Usage: $0 [daily|weekly]"
        exit 1
        ;;
esac

log "Maintenance completed successfully"
EOF

chmod +x corporate_maintenance.sh

# Setup cron jobs
echo "0 2 * * * root /opt/rust-ai-ide/corporate_maintenance.sh daily" > /etc/cron.d/rust-ai-ide-maintenance
echo "0 3 * * 1 root /opt/rust-ai-ide/corporate_maintenance.sh weekly" >> /etc/cron.d/rust-ai-ide-maintenance
```

---

## 🧪 Testing & Validation

### Corporate Deployment Testing

```bash
# Create comprehensive test suite
cat > corporate_deployment_test.sh << 'EOF'
#!/bin/bash

FAILED_TESTS=0

# Test functions
test_basic_installation() {
    echo "Testing basic installation..."
    if /opt/rust-ai-ide/rust-ai-ide --version > /dev/null 2>&1; then
        echo "✓ Basic installation test PASSED"
        return 0
    else
        echo "✗ Basic installation test FAILED"
        ((FAILED_TESTS++))
        return 1
    fi
}

test_network_connectivity() {
    echo "Testing network connectivity..."
    if timeout 10 curl -I https://company-registry.company.com > /dev/null 2>&1; then
        echo "✓ Network connectivity test PASSED"
        return 0
    else
        echo "✗ Network connectivity test FAILED"
        ((FAILED_TESTS++))
        return 1
    fi
}

test_ai_service() {
    echo "Testing AI service connectivity..."
    if systemd is-active --quiet rust-ai-ide-ai; then
        echo "✓ AI service test PASSED"
        return 0
    else
        echo "✗ AI service test FAILED - attempting restart"
        systemctl restart rust-ai-ide-ai
        sleep 5
        if systemd is-active --quiet rust-ai-ide-ai; then
            echo "✓ AI service restart test PASSED"
            return 0
        else
            echo "✗ AI service restart test FAILED"
            ((FAILED_TESTS++))
            return 1
        fi
    fi
}

test_security_compliance() {
    echo "Testing security compliance..."
    # Check file permissions
    if [[ $(stat -c "%a" /etc/rust-ai-ide/corporate-config.toml 2>/dev/null) == "600" ]]; then
        echo "✓ File permissions test PASSED"
    else
        echo "✗ File permissions test FAILED"
        ((FAILED_TESTS++))
    fi

    # Check audit logging
    if journalctl -u rust-ai-ide --since "1 hour ago" | grep -q "audit"; then
        echo "✓ Audit logging test PASSED"
    else
        echo "✗ Audit logging test FAILED - audit logs not found"
        ((FAILED_TESTS++))
    fi
}

# Run all tests
main() {
    echo "=== Corporate Deployment Test Suite ==="
    echo "Started: $(date)"
    echo

    test_basic_installation
    test_network_connectivity
    test_ai_service
    test_security_compliance

    echo
    echo "=== Test Results ==="
    echo "Failed tests: $FAILED_TESTS"

    if [[ $FAILED_TESTS -eq 0 ]]; then
        echo "🎉 All tests PASSED - Deployment successful!"
        exit 0
    else
        echo "❌ $FAILED_TESTS test(s) FAILED - Please review configuration"
        exit 1
    fi
}

main "$@"
EOF

chmod +x corporate_deployment_test.sh
```

---

## 📞 Support & Documentation

### Enterprise Support Contact

- **Enterprise Support Team**: enterprise@rust-ai-ide.dev
- **Security Issues**: security@rust-ai-ide.dev
- **Emergency Support**: +1-555-ENTERPRISE (24/7)

### Additional Resources

- **[Security Best Practices](../security/best-practices.md)** - Enterprise security configuration
- **[GDPR Compliance Guide](../security/gdpr-compliance.md)** - Privacy compliance setup
- **[Troubleshooting Guide](../../troubleshooting/common-issues.md)** - Common deployment issues
- **[API Documentation](../../api/core-system.md)** - Technical integration

---

## 📋 Deployment Checklist

### Pre-Deployment
- [ ] Hardware assessment completed
- [ ] Network connectivity verified
- [ ] Corporate proxy configured
- [ ] Security policies reviewed
- [ ] User permissions planned

### During Deployment
- [ ] System dependencies installed
- [ ] Rust toolchain configured
- [ ] Node.js and pnpm installed
- [ ] Corporate configuration applied
- [ ] Security policies implemented
- [ ] Network policies configured

### Post-Deployment
- [ ] Installation verified
- [ ] AI services configured
- [ ] User access tested
- [ ] Monitoring configured
- [ ] Documentation provided to users
- [ ] Support procedures established

### Ongoing Maintenance
- [ ] Automated monitoring configured
- [ ] Security updates scheduled
- [ ] Backup procedures implemented
- [ ] Performance monitoring established
- [ ] User training completed

---

**🎯 Enterprise Deployment Success Criteria:**

1. **Security**: Corporate security policies fully implemented
2. **Compliance**: GDPR/SOX/PCI compliance requirements met
3. **Scalability**: Architecture supports 1000+ concurrent users
4. **Reliability**: 99.9% uptime with automated failover
5. **Supportability**: Clear monitoring and troubleshooting procedures
6. **User Adoption**: Intuitive interface with comprehensive training

---

*Enterprise-ready deployment ensures your organization can leverage AI-powered development securely and at scale.*