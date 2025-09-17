# Installation Guide

## Prerequisites

- Rust (latest stable version)
- Cargo
- Node.js (for web interface)
- Git

## Quick Start

1. Clone the repository:
   ```bash
   git clone https://github.com/rust-ai-ide/rust-ai-ide.git
   cd rust-ai-ide
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Start the IDE:
   ```bash
   cargo run --release
   ```

## Building from Source

1. Install dependencies:
   ```bash
   sudo apt-get update
   sudo apt-get install -y build-essential cmake pkg-config
   ```

2. Build with all features:
   ```bash
   cargo build --release --all-features
   ```

## System Requirements

- Minimum: 4GB RAM, 2 CPU cores, 2GB disk space
- Recommended: 8GB+ RAM, 4+ CPU cores, SSD storage

## Configuration

See [Configuration Guide](configuration.html) for detailed setup instructions.

## Troubleshooting

Common issues and solutions:

- **Build fails**: Ensure all dependencies are installed
- **Missing components**: Run `cargo update`
- **Permission errors**: Run with `sudo` if necessary

## Next Steps

- [Quick Start Guide](QUICKSTART.html)
- [Configuration Guide](configuration.html)
- [User Guide](../user-guide/README.html)
