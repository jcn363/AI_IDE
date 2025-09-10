# 🚀 Installation and Configuration Guide

## Overview

This comprehensive guide covers the installation, configuration, and setup of the Rust AI IDE - a production-ready IDE built with Rust, featuring advanced AI capabilities, enterprise-grade security, and seamless collaboration tools.

> **Version**: 3.2.0-release
> **Status**: **🔧 Maintenance Phase - 98% Build Success with 2 Critical Bugs**
> **Architecture**: Modular Workspace (67 crates across 5 layers)

## Prerequisites

Before installing the Rust AI IDE, ensure your system meets the following requirements:

### System Requirements

#### Hardware Specifications
| Component | Minimum | Recommended | Enterprise |
|-----------|---------|-------------|------------|
| **OS** | Linux/macOS/Windows 10+ | Latest stable version | Enterprise Linux distros |
| **RAM** | 8GB (<2GB for workspaces up to 1M LOC) | 16GB+ | 64GB+ (Multi-user) |
| **CPU** | 4 cores, 3.0GHz | 8+ cores, 4.0GHz+ | 16+ cores, 4.5GHz+ |
| **GPU** | Integrated | NVIDIA/AMD 8GB+ VRAM | NVIDIA A-series/RTX 40+ |
| **Storage** | 10GB free | 50GB+ (Models + cache) | 100GB+ SSD (HA setup) |
| **Display** | 1366x768 | 1920x1080+ | Multiple monitors |
| **Network** | 10Mbps | 100Mbps+ (Cloud features) | 1Gbps+ (Real-time collab) |

### Software Dependencies

#### Rust Nightly Toolchain
- **Required**: Rust nightly 2025-09-03 with components: rust-src, rustfmt, clippy
- **Installation**: Use rustup to install the nightly toolchain
- **Note**: Stable Rust (1.91.0+) is the minimum floor, but nightly is required for unstable features

#### Node.js and npm
- **Required**: Latest LTS version with npm
- **Purpose**: Web frontend build and development tooling

#### SQLite Development Libraries
- **Required**: System SQLite development libraries (bundled via libsqlite3-sys)
- **Purpose**: Database functionality with version enforcement

#### Additional Dependencies
- **Git**: For version control and dependency management
- **Build Tools**: CMake, pkg-config (system-specific)

## Setup

### 1. Rust Nightly Toolchain Installation

Install and configure the required nightly Rust toolchain:

```bash
# Install rustup if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add to your shell's configuration
source $HOME/.cargo/env

# Install nightly toolchain with required components
rustup install nightly-2025-09-03
rustup component add rust-src rustfmt clippy --toolchain nightly-2025-09-03

# Set nightly as default (optional, or use +nightly in commands)
rustup default nightly-2025-09-03

# Verify installation
rustc --version  # Should show nightly-2025-09-03
cargo --version
```

### 2. Node.js Installation

Install Node.js and npm for web frontend development:

```bash
# Using Node Version Manager (recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
source ~/.bashrc
nvm install --lts
nvm use --lts

# Verify installation
node --version
npm --version
```

### 3. System Dependencies

#### Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libsqlite3-dev cmake
```

#### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Homebrew (if not installed)
# /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install cmake sqlite
```

#### Windows
```bash
# Install Visual Studio Build Tools
# Download: https://visualstudio.microsoft.com/visual-cpp-build-tools/
# Select "Desktop development with C++" workload

# Install SQLite development libraries
# Download: https://www.sqlite.org/download.html
# Or via Chocolatey: choco install sqlite
```

### 4. Security Tools Installation

Install cargo-deny for dependency security and license compliance:

```bash
cargo install cargo-deny
cargo deny --version
```

## Installation Methods

### Method 1: Source Build (Recommended for Development)

Clone and build from source for the latest features and development capabilities:

```bash
# Clone the repository
git clone https://github.com/jcn363/rust-ai-ide.git
cd rust-ai-ide

# Install dependencies and build workspace
cargo +nightly build --workspace

# Build web frontend
cd web
npm install
npm run build
cd ..

# Run tests
cargo +nightly test --workspace

# Launch the application
cargo +nightly run --release
```

### Method 2: Pre-built Binaries

Download pre-compiled binaries for your platform:

```bash
# Download latest release
# Visit: https://github.com/jcn363/rust-ai-ide/releases/latest

# Linux
wget https://github.com/jcn363/rust-ai-ide/releases/latest/download/rust-ai-ide-linux.tar.gz
tar -xzf rust-ai-ide-linux.tar.gz
cd rust-ai-ide
./install.sh

# macOS
curl -L -o rust-ai-ide-macos.dmg https://github.com/jcn363/rust-ai-ide/releases/latest/download/rust-ai-ide-macos.dmg
open rust-ai-ide-macos.dmg
# Follow on-screen instructions

# Windows
# Visit releases page and download rust-ai-ide-windows.exe
# Run installer and follow prompts
```

### Method 3: Development Environment Setup

For contributors and advanced users:

```bash
# Clone with submodules if any
git clone --recursive https://github.com/jcn363/rust-ai-ide.git
cd rust-ai-ide

# Setup development environment
./scripts/init-workspace-config.sh

# Build with development optimizations
cargo +nightly build --workspace --profile dev

# Run with debug features
RUST_LOG=debug cargo +nightly run
```

## Configuration

### Basic Configuration

Create a `.env` file in your project root:

```env
# AI/ML Configuration
AI_MODEL=rustcoder-7b
AI_ENDPOINT=http://localhost:11434
AI_TEMPERATURE=0.7
AI_MAX_TOKENS=2048

# Editor Settings
THEME=dark
FONT_SIZE=14
TAB_SIZE=4

# Security Settings
ENABLE_AUDIT_LOGGING=true
SECURITY_LEVEL=enterprise
```

### Service Initialization

Certain features require specific service initialization:

#### AI/ML Services
- **LSP Service**: Automatically initializes for AI-powered features
- **Model Loading**: Pre-download models for offline operation
- **Hyperparameter Tuning**: Requires specific ai-learning pipelines

```bash
# Initialize AI services
cargo +nightly run -- --init-ai-services

# Pre-download models for offline use
cargo +nightly run -- --download-models
```

#### Webhook System
- **Port Configuration**: Default webhook port is 3000
- **Cloud Integrations**: Initialize webhook system for external API connections

```bash
# Configure webhook port
export WEBHOOK_PORT=3000

# Initialize webhook system
cargo +nightly run -- --init-webhooks
```

#### Database Services
- **SQLite Connection**: Automatic pooling with version enforcement
- **Migration Scripts**: Forward-only migrations (no rollbacks)

```bash
# Initialize database
cargo +nightly run -- --init-database

# Run migrations
cargo +nightly run -- --migrate
```

### Security Configuration

Configure security policies and compliance:

```toml
# deny.toml (Security & License Compliance)
[advisories]
yanked = "warn"

[bans]
multiple-versions = "warn"
deny = [
    { name = "openssl", reason = "Use rustls instead" },
    { name = "md5", reason = "MD5 cryptographically broken" },
    { name = "ring", reason = "Low-level crypto code" },
    { name = "quick-js", reason = "Experimental JS engine" },
]

[licenses]
allow = [
    "MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause",
    "ISC", "Unicode-DFS-2016"
]
deny = []
exceptions = [
    { allow = ["GPL-2.0", "GPL-3.0"], name = "git2", version = "*" }
]
```

### Advanced Configuration

For detailed configuration options, see the [Configuration Guide](docs/configuration.md).

## Build Configuration

### Workspace Build Management

The project uses a modular workspace with 67+ specialized crates:

```bash
# Build entire workspace
cargo +nightly build --workspace

# Build specific crate
cargo +nightly build -p rust-ai-ide-core

# Build with optimizations
cargo +nightly build --workspace --release

# Build with debug symbols
cargo +nightly build --workspace --profile dev
```

### Web Frontend Build

```bash
cd web

# Install dependencies
npm install

# Build with type generation
npm run build

# Development server
npm run dev

# Type checking
npm run type-check

# Testing
npm run test
```

### Type Generation

Type generation runs automatically through Cargo:

```bash
# Generate TypeScript types from Rust
npm run build  # This runs cargo bin to generate types

# Manual type generation
cd web
npm run generate-types
```

### Development Workflow

```bash
# Lint code
cargo +nightly clippy --workspace

# Format code
cargo +nightly fmt --workspace

# Run tests
cargo +nightly test --workspace

# Run specific test
cargo +nightly test -p <crate_name> -- --test <test_function_name>

# License compliance check
cargo deny check

# Performance testing
cargo +nightly run --bin performance_test
```

## Troubleshooting

### Build & Installation Issues

#### Nightly Rust Toolchain Problems
- **Symptom**: Compilation errors related to unstable features
- **Solution**: Ensure correct nightly version (2025-09-03) with required components
```bash
rustup install nightly-2025-09-03
rustup component add rust-src rustfmt clippy --toolchain nightly-2025-09-03
```

#### SQLite Library Issues
- **Symptom**: SQLite compilation errors
- **Solution**: Install system SQLite development libraries
```bash
# Ubuntu/Debian
sudo apt install libsqlite3-dev

# macOS
brew install sqlite

# Windows: Install SQLite SDK
```

#### Node.js Dependency Problems
- **Symptom**: Web build failures
- **Solution**: Ensure Node.js LTS and npm are installed
```bash
nvm install --lts
nvm use --lts
cd web && npm install
```

### Service Initialization Problems

#### AI Features Not Working
- **Symptom**: AI suggestions not appearing
- **Cause**: AI LSP service requires async initialization
- **Solution**: Wait for service startup before using AI features
```bash
# Check service status
cargo +nightly run -- --service-status

# Reinitialize AI services
cargo +nightly run -- --init-ai-services
```

#### Webhook Connection Failures
- **Symptom**: Cloud integrations failing
- **Cause**: Webhook system not initialized on port 3000
- **Solution**: Initialize webhook system before use
```bash
export WEBHOOK_PORT=3000
cargo +nightly run -- --init-webhooks
```

#### Database Connection Errors
- **Symptom**: Database operations failing
- **Cause**: SQLite version conflicts or connection pooling issues
- **Solution**: Check SQLite versions and ConnectionPool configuration
```bash
# Check SQLite version
sqlite3 --version

# Reinitialize database
cargo +nightly run -- --init-database
```

### Placeholder Implementations

#### Commands Returning Dummy Data
- **Symptom**: Commands return `{"status": "ok"}`
- **Cause**: Many Tauri commands have placeholder implementations
- **Solution**: Check if real implementation exists or wait for service initialization

#### Missing Features
- **Symptom**: Enterprise features showing placeholder responses
- **Cause**: Features requiring background services need proper startup sequence
- **Solution**: Ensure all services are initialized before use

### Performance Issues

#### High Memory Usage
- **Symptom**: Memory consumption >2GB
- **Cause**: Large workspaces require virtual memory management
- **Solution**: Enable virtual memory management for workspaces >1M LOC
```bash
export RUST_AI_IDE_VM_MANAGEMENT=true
cargo +nightly run -- --enable-vm
```

#### Slow Startup
- **Symptom**: Cold start >500ms, warm start >100ms
- **Cause**: Service initialization delays
- **Solution**: Check for service initialization issues and optimize startup sequence

### Configuration Problems

#### Path Validation Errors
- **Symptom**: File operations failing with path errors
- **Cause**: Path traversal attacks blocked by validation
- **Solution**: Use `validate_secure_path()` for all file operations

#### Command Injection Prevention
- **Symptom**: Commands failing with injection errors
- **Cause**: Input sanitization requirements
- **Solution**: Use sanitized command args from TauriInputSanitizer

#### License Compliance Issues
- **Symptom**: Build failing with license errors
- **Cause**: cargo-deny blocking forbidden dependencies
- **Solution**: Ensure all dependencies use permitted licenses (MIT/Apache-2.0/BSD)

### Common Development Issues

#### Workspace Build Conflicts
- **Symptom**: Circular dependency errors
- **Cause**: Multi-crate workspace with intentional circular dependencies in types
- **Solution**: Use workspace-wide cargo operations, avoid individual crate builds

#### Webview Restrictions
- **Symptom**: localStorage/Cookies/web APIs not working
- **Cause**: Webview isolation prevents external state libraries
- **Solution**: Use IPC channels for communication between webview and extension

#### Async Initialization Requirements
- **Symptom**: Features not working immediately after startup
- **Cause**: Services require async initialization (AI LSP, webhooks)
- **Solution**: Wait for service initialization or check service status

## Verifying Installation

### Basic Verification

```bash
# Check version
rust-ai-ide --version

# Run self-diagnostics
rust-ai-ide doctor

# Check service status
cargo +nightly run -- --service-status
```

### Advanced Diagnostics

```bash
# Performance diagnostics
cargo +nightly run -- --performance-test

# Security audit
cargo deny check

# Dependency analysis
cargo +nightly build --workspace --message-format=json | jq '.'
```

## Updating

### From Source
```bash
git pull origin main
cargo +nightly build --workspace
cd web && npm install && npm run build
```

### Via Cargo (if applicable)
```bash
cargo install --force rust-ai-ide
```

## Uninstallation

```bash
# Remove binary
cargo uninstall rust-ai-ide

# Remove configuration (optional)
# Linux/macOS: ~/.config/rust-ai-ide/
# Windows: %APPDATA%\rust-ai-ide/

# Remove source (if built from source)
rm -rf rust-ai-ide/
```

## Security Compliance

### Key Policies
- **Banned Packages**: openssl, md5, ring, quick-js (security reasons)
- **Allowed Licenses**: MIT, Apache-2.0, BSD variants only
- **Registry Restrictions**: Only crates.io registry allowed
- **Git Dependencies**: Specific GitHub organizations only

### Audit Logging
All sensitive operations require audit logging via the security crate.

## Support and Community

### Getting Help
1. Check this installation guide
2. Review [RUST_AI_IDE_PLAN.md](RUST_AI_IDE_PLAN.md) for technical details
3. Search [GitHub Issues](https://github.com/jcn363/rust-ai-ide/issues)
4. Join the [Discord Community](https://discord.gg/rust-ai-ide)

### Contributing
- Follow the [AGENTS.md](AGENTS.md) coding standards
- Use nightly Rust toolchain for development
- Ensure all tests pass: `cargo +nightly test --workspace`
- Format code: `cargo +nightly fmt --workspace`

---

Built with ❤️ by the Rust AI IDE Team