# 🚀 Installation Guide

## System Requirements

| Component       | Minimum                  | Recommended               | Enterprise              |
|----------------|--------------------------|---------------------------|-------------------------|
| **OS**        | Linux, macOS, Windows 10+| Latest stable version     | Enterprise Linux distros|
| **RAM**       | 16GB (Basic features)   | 32GB+ (AI features)      | 64GB+ (Multi-user)     |
| **CPU**       | 4 cores, 3.0GHz         | 8+ cores, 4.0GHz+        | 16+ cores, 4.5GHz+     |
| **GPU**       | Integrated              | NVIDIA/AMD 8GB+ VRAM     | NVIDIA A-series/RTX 40+ |
| **Storage**   | 20GB free               | 50GB+ (Models + cache)   | 100GB+ SSD (HA setup)   |
| **Display**   | 1366x768                | 1920x1080+ (Full IDE)    | Multiple monitors       |
| **Network**   | 10Mbps                  | 100Mbps+ (Cloud features)| 1Gbps+ (Real-time collab)|

## Prerequisites

### 1. Install Rust Toolchain

```bash
# Install Rust using rustup (recommended)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add to your shell's configuration
source $HOME/.cargo/env

# Verify installation
rustc --version  # Should be 1.75 or higher
cargo --version
```

### 2. Install System Dependencies

#### Linux (Debian/Ubuntu)

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev
```

#### macOS

```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Homebrew (if not installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install cmake
```

#### Windows

- Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- Select "Desktop development with C++" workload
- Install [LLVM](https://releases.llvm.org/download.html) and add it to PATH

## Installation Methods

### Method 1: Install from Source (Recommended for Development)

```bash
# Clone the repository
git clone https://github.com/jcn363/rust-ai-ide.git
cd rust-ai-ide

# Install dependencies and build
cargo build --release

# Run the application
cargo run --release
```

### Method 2: Using Cargo (Stable Release)

```bash
cargo install rust-ai-ide
rust-ai-ide
```

### Method 3: Download Pre-built Binaries

Visit our [Releases](https://github.com/jcn363/rust-ai-ide/releases) page to download pre-built binaries for your platform.

## Post-Installation

1. **First Run Configuration**
   - The IDE will guide you through initial setup
   - Select your preferred theme and keybindings
   - Configure AI model preferences if using AI features

2. **Install Extensions** (Optional)
   - Open the command palette (`Ctrl+Shift+P` or `Cmd+Shift+P`)
   - Search for "Install Extension"
   - Browse and install additional language support or tools

## Verifying Installation

```bash
# Check version
rust-ai-ide --version

# Run self-diagnostics
rust-ai-ide doctor
```

## Troubleshooting

### Common Issues

1. **Missing Dependencies**
   - Ensure all system dependencies are installed
   - Check the [Troubleshooting Guide](docs/troubleshooting-guide.md) for your OS

2. **Build Failures**
   - Update Rust: `rustup update`
   - Clean build: `cargo clean && cargo build --release`
   - Check [open issues](https://github.com/jcn363/rust-ai-ide/issues) for similar problems

3. **Performance Issues**
   - Disable unnecessary extensions
   - Increase system resources if possible
   - Check `rust-ai-ide --help` for performance-related flags

## Updating

```bash
# If installed via cargo
cargo install --force rust-ai-ide

# If built from source
git pull origin main
cargo build --release
```

## Uninstallation

```bash
# Remove the binary
cargo uninstall rust-ai-ide

# Remove configuration files (optional)
# Linux/macOS: ~/.config/rust-ai-ide
# Windows: %APPDATA%\rust-ai-ide
```

## Need Help?

If you encounter any issues during installation, please:

1. Check the [Troubleshooting Guide](docs/troubleshooting-guide.md)
2. Search our [GitHub Issues](https://github.com/jcn363/rust-ai-ide/issues)
3. Join our [Discord Community](https://discord.gg/rust-ai-ide) for support

> [🔝 Back to Top](#rust-ai-ide---installation-guide)
