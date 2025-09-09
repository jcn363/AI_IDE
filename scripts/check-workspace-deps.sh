#!/bin/bash
set -euo pipefail

# Check for cargo-hakari
if ! command -v cargo-hakari &> /dev/null; then
    echo "Installing cargo-hakari..."
    cargo install cargo-hakari
fi

# Check for cargo-udeps
if ! command -v cargo-udeps &> /dev/null; then
    echo "Installing cargo-udeps..."
    cargo install cargo-udeps --locked
fi

echo "Running cargo check..."
cargo check --all-targets --workspace

echo "Running cargo udeps..."
cargo +nightly udeps --workspace

echo "Running cargo hakari verify..."
cargo hakari verify

echo "Running cargo deny check..."
cargo deny check

echo "Running cargo audit..."
cargo audit

echo "Workspace verification complete!"
