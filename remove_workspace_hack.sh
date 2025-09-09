#!/bin/bash

# Find all Cargo.toml files in the crates directory and remove workspace-hack dependency
find crates -name "Cargo.toml" -type f -exec sed -i '/workspace-hack/d' {} \;

# Also remove from src-tauri/Cargo.toml
sed -i '/workspace-hack/d' src-tauri/Cargo.toml

echo "Workspace-hack dependency has been removed from all Cargo.toml files"
