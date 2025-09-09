#!/bin/bash

# Offline Package Download Script for Air-Gapped Deployments
# This script downloads all necessary dependencies for offline installations

set -e

# Configuration
OUTPUT_DIR="./offline-packages"
CARGO_REGISTRY="${CARGO_REGISTRY:-crates.io}"
NPM_REGISTRY="${NPM_REGISTRY:-registry.npmjs.org}"
DOCKER_REGISTRY="${DOCKER_REGISTRY:-registry-1.docker.io}"

# Create directory structure
mkdir -p "${OUTPUT_DIR}/cargo"
mkdir -p "${OUTPUT_DIR}/npm"
mkdir -p "${OUTPUT_DIR}/docker"
mkdir -p "${OUTPUT_DIR}/system"

echo "Starting offline package download..."

# 1. Download Cargo dependencies using cargo-vendor
echo "Downloading Cargo dependencies..."
if command -v cargo-vendor >/dev/null 2>&1; then
    cargo vendor "${OUTPUT_DIR}/cargo/vendor"
else
    echo "Warning: cargo-vendor not found. Please install it with:"
    echo "  cargo install cargo-vendor"
    echo "Manual Cargo dependency download..."
    # Alternative: Download Cargo.lock dependencies
    cargo fetch --locked
    cp -r ~/.cargo/registry "${OUTPUT_DIR}/cargo/"
fi

# 2. Download npm dependencies
echo "Downloading npm dependencies..."
if [ -f "web/package-lock.json" ]; then
    cd web
    npm ci --prefer-offline --no-audit
    # Copy node_modules for offline use
    cp -r node_modules "../../${OUTPUT_DIR}/npm/"
    cd ..
fi

# 3. Download Docker images
echo "Downloading Docker base images..."
IMAGES=(
    "debian:bullseye-slim"
    "postgres:14-alpine"
    "redis:7-alpine"
    "node:18-alpine"
    "nginx:alpine"
    "rust:1.91-slim"
)

for image in "${IMAGES[@]}"; do
    echo "Downloading $image..."
    docker pull "$image"
    docker save "$image" > "${OUTPUT_DIR}/docker/$(echo $image | tr '/' '_' | tr ':' '_').tar"
done

# 4. Download system packages for Debian/Ubuntu
echo "Downloading system packages..."
# Required packages based on Dockerfile dependencies
SYSTEM_PACKAGES=(
    "build-essential"
    "pkg-config"
    "libssl-dev"
    "libclang-dev"
    "git"
    "curl"
    "libgtk-3-dev"
    "libwebkit2gtk-4.1-dev"
    "libappindicator3-dev"
    "librsvg2-dev"
    "patchelf"
    "musl-tools"
    "musl-dev"
    "ca-certificates"
    "libssl1.1"
    "libgcc-s1"
    "wget"
    "nodejs"
    "python3"
    "python3-pip"
)

# Create a package list for offline installation
printf '%s\n' "${SYSTEM_PACKAGES[@]}" > "${OUTPUT_DIR}/system/deb-packages.list"

# Note: Actual package download would require apt or similar on target system
echo "System packages list saved. Download them manually with:"
echo "  apt download $(cat ${OUTPUT_DIR}/system/deb-packages.list)"

# 5. Download Rust toolchain for offline installation
echo "Downloading Rust toolchain..."
RUST_VERSION="1.91"
RUST_URL="https://static.rust-lang.org/dist/rust-${RUST_VERSION}-x86_64-unknown-linux-gnu.tar.gz"
NIGHTLY_DATE="2025-09-03"
NIGHTLY_URL="https://static.rust-lang.org/dist/${NIGHTLY_DATE}/rust-nightly-x86_64-unknown-linux-gnu.tar.gz"

wget -O "${OUTPUT_DIR}/system/rust-stable.tar.gz" "$RUST_URL"
wget -O "${OUTPUT_DIR}/system/rust-nightly.tar.gz" "$NIGHTLY_URL"

# 6. Create offline installation manifest
cat > "${OUTPUT_DIR}/manifest.md" << EOF
# Offline Package Manifest - Rust AI IDE
Generated on: $(date)
Target System: Linux x86_64

## Contents

### Cargo Dependencies
- Location: ${OUTPUT_DIR}/cargo/
- Usage: Copy to ~/.cargo/registry (create symlink or replace)
- Method: cargo-vendor (recommended) or cargo registry copy

### NPM Dependencies
- Location: ${OUTPUT_DIR}/npm/
- Usage: Copy to web/node_modules in project directory
- Method: Direct copy after npm ci completion

### Docker Images
- Location: ${OUTPUT_DIR}/docker/
- Usage: docker load < image.tar
- Images: $(printf '%s, ' "${IMAGES[@]}")

### System Packages
- Location: ${OUTPUT_DIR}/system/deb-packages.list
- Usage: apt-get download \$(cat deb-packages.list)
- Method: Cache in local apt repository

### Rust Toolchain
- Location: ${OUTPUT_DIR}/system/
- Files: rust-stable.tar.gz, rust-nightly.tar.gz
- Usage: tar -xzf rust-*.tar.gz && ./install.sh --prefix=/usr/local

## Installation Instructions

1. System packages (requires internet access initially):
   apt-get download \$(cat ${OUTPUT_DIR}/system/deb-packages.list)
   dpkg -i *.deb

2. Rust toolchain:
   cd ${OUTPUT_DIR}/system
   tar -xzf rust-nightly.tar.gz
   ./rust-nightly-x86_64-unknown-linux-gnu/install.sh --default-toolchain nightly --profile default

3. Cargo dependencies:
   mkdir -p ~/.cargo/registry
   cp -r ${OUTPUT_DIR}/cargo/* ~/.cargo/

4. NPM dependencies:
   cd [project]/web
   cp -r ${OUTPUT_DIR}/npm/* node_modules/

5. Docker images:
   for img in ${OUTPUT_DIR}/docker/*.tar; do docker load < "\$img"; done

## Notes
- Total size estimate: Check with 'du -sh ${OUTPUT_DIR}'
- Transfer via USB drive or secure file transfer
- Validate integrity with checksums if needed
EOF

# 7. Create checksums for verification
echo "Creating checksums..."
find "${OUTPUT_DIR}" -type f -exec sha256sum {} \; > "${OUTPUT_DIR}/checksums.sha256"

echo "Offline package download complete!"
echo "Manifest: ${OUTPUT_DIR}/manifest.md"
echo "Checksums: ${OUTPUT_DIR}/checksums.sha256"
echo "Total size: $(du -sh "${OUTPUT_DIR}" | cut -f1)"
echo ""
echo "Transfer ${OUTPUT_DIR} to your air-gapped environment."