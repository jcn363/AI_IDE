#!/bin/bash

# Make script executable on creation
if [[ ! -x "$0" ]]; then
    chmod +x "$0"
fi

# Workspace Configuration Template Generator
# Version: 1.0.0
# Date: 2025
# Description: Automated workspace configuration template generator

set -euo pipefail

# ========================================
# Configuration & Colors
# ========================================

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m'

# Template configurations
WORKSPACE_VERSION="0.1.0"
WORKSPACE_AUTHORS="[\"Rust AI IDE Team\"]"
WORKSPACE_LICENSE="MIT OR Apache-2.0"
REQUIRED_RUST_VERSION="1.91.0"

# ========================================
# Utility Functions
# ========================================

print_header() {
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${WHITE}🛠️  Workspace Configuration Template Generator v1.0${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# ========================================
# Cargo.toml Template Generator
# ========================================

generate_crate_cargo_toml() {
    local crate_name="$1"
    local crate_description="$2"
    local crate_path="$3"

    mkdir -p "$crate_path"

    cat > "$crate_path/Cargo.toml" << EOF
[package]
name = "$crate_name"
version = { workspace = true }
edition = { workspace = true }
authors = { workspace = true }
license = { workspace = true }
rust-version = { workspace = true }
publish = { workspace = true }
repository = { workspace = true }
homepage = { workspace = true }
description = "$crate_description"

[dependencies]
# Workspace hack for dependency deduplication
workspace-hack = { path = "../../workspace-hack" }

# Core workspace dependencies
tokio = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
serde = { workspace = true }

# Add crate-specific dependencies here

[dev-dependencies]
tokio-test = { workspace = true }

[features]
default = ["std"]
std = []

# Platform-specific features
windows = []
unix = []
macos = []

[package.metadata]
category = "$(get_crate_category "$crate_name")"
maturity = "alpha"
maintainer = "Rust AI IDE Team"
EOF

    print_success "Generated Cargo.toml for $crate_name"
}

generate_workspace_cargo_toml() {
    local workspace_name="$1"
    local members="$2"

    cat > "Cargo.toml" << EOF
[workspace]
resolver = "2"

members = [
$members
    "workspace-hack"
]

[workspace.package]
version = "$WORKSPACE_VERSION"
edition = "2021"
rust-version = "$REQUIRED_RUST_VERSION"
authors = $WORKSPACE_AUTHORS
license = "$WORKSPACE_LICENSE"
publish = false
repository = "https://github.com/rust-ai-ide/rust-ai-ide"
homepage = "https://github.com/rust-ai-ide/rust-ai-ide"
readme = "README.md"

# Explicitly specify SQLite-related dependencies to avoid conflicts
[workspace.dependencies]
# Force all crates to use the same SQLite version
libsqlite3-sys = { version = "0.35.0", default-features = false, features = [
  "bundled",
] }
rusqlite = { version = "^0.37", default-features = false, features = [
  "bundled",
] }

$(generate_common_dependencies)

# Workspace hack for dependency deduplication

$(generate_profiles)
EOF

    print_success "Generated workspace Cargo.toml"
}

generate_common_dependencies() {
    cat << 'EOF'
# Common dependencies shared across crates (using caret for flexible versioning)
anyhow = { version = "^1.0.99" }
bytes = "1.10.1"
chrono = { version = "0.4.42", default-features = false, features = [
  "serde",
  "clock",
] }
dirs-next = { version = "^2.0.0" }
filetime = { version = "^0.2.26" }
hex = "0.4.3"
hmac = "0.12.1"
sha2 = "0.10.9"
futures = { version = "^0.3.31" }
governor = { version = "0.10.1" }
lazy_static = "^1.5.0"
log = "^0.4.28"
lru = "0.16.0"
once_cell = "^1.21.3"
petgraph = "^0.8.2"
semver = { version = "^1.0.26", default-features = false, features = ["serde"] }
serde = { version = "^1.0.219", default-features = false, features = [
  "derive",
] }
serde_json = "^1.0.143"
thiserror = { version = "2.0.16" }
tracing = { version = "0.1.41" }
uuid = { version = "^1.18.1", default-features = false, features = [
  "v4",
  "serde",
] }

# Tauri and system dependencies
blake3 = "^1.8.2"
dashmap = "6.1.0"
lsp-server = { version = "^0.7.9" }
lsp-types = { version = "^0.97.0" }
notify = { version = "^8.2.0", default-features = false, features = ["serde"] }
proc-macro2 = { version = "1.0.101", default-features = false, features = [] }
quote = { version = "1.0.40" }
regex = { version = "1.11.2" }
reqwest = { version = "^0.12.23", default-features = false, features = [
  "json",
  "rustls-tls",
] }
serde_yaml = { version = "0.9.34" }
syn = { version = "2.0.106", default-features = false, features = [
  "parsing",
  "printing",
  "visit",
  "visit-mut",
] }
tauri = { version = "2.8.5", features = [] }
tauri-build = { version = "2.4.1" }
tauri-runtime = "2.8.0"
tempfile = "^3.21.0"
terminal_size = "^0.4.3"
tokio = { version = "1.47.1", default-features = false, features = [
  "full",
] }
tokio-test = { version = "0.4.4" }
tokio-util = { version = "0.7.16" }
toml = "^0.9.5"
tower-lsp = { version = "^0.20" }
validator = { version = "^0.20", default-features = false, features = [
  "derive",
] }
walkdir = { version = "2.5.0" }

# File system and system utilities
cargo_metadata = { version = "^0.22.0" }
dunce = { version = "^1.0.5" }
fs-err = { version = "^3.1.1" }
path-absolutize = { version = "^3.1.1" }

# Tauri plugins
tauri-plugin-dialog = { version = "^2.4.0" }
tauri-plugin-fs = { version = "^2.4.2" }
tauri-plugin-log = { version = "^2.7", default-features = false, features = [
  "tracing",
] }

# AI/ML dependencies
candle-core = { version = "0.9.1" }
candle-nn = { version = "0.9.1" }
candle-transformers = { version = "0.9.1" }
hf-hub = { version = "0.4.3" }
tokenizers = { version = "0.22.0" }
safetensors = { version = "0.6.2" }

# Security and quality tools
cargo-deny = { version = "0.18.4" }
cargo-geiger = { version = "0.13.0" }

# Additional utilities
async-trait = { version = "^0.1.89" }
color-eyre = { version = "0.6.5" }
criterion = { version = "0.7.0" }
fancy-regex = { version = "0.16.1" }
handlebars = { version = "6.3.2" }
ignore = { version = "^0.4.23" }
moka = { version = "0.12.10", default-features = false, features = ["future"] }
num_cpus = { version = "^1.17.0" }
proc-macro-error = { version = "1.0.4" }
rayon = { version = "^1.11.0" }
rust-ai-ide-errors = { path = "./crates/rust-ai-ide-errors" }
shared-test-utils = { path = "./crates/shared-test-utils" }
rust-ai-ide-types = { path = "./crates/rust-ai-ide-types" }
rust-ai-ide-common = { path = "crates/rust-ai-ide-common", features = [
  "tauri-integrated",
] }
sysinfo = { version = "0.37.0" }
tokio-stream = { version = "0.1.17" }
tree-sitter = { version = "0.25.9" }
tree-sitter-rust = { version = "0.24.0" }
tree-sitter-javascript = { version = "0.25.0" }
tree-sitter-typescript = { version = "0.23.2" }
tree-sitter-python = { version = "0.23.6" }
tree-sitter-go = { version = "0.25.0" }
tree-sitter-java = { version = "0.23.5" }
tree-sitter-cpp = { version = "0.23.4" }
typetag = { version = "0.2.20" }

# Build-time utilities
heck = { version = "^0.5.0" }
rand = { version = "^0.9.2" }
strsim = { version = "^0.11" }
parking_lot = { version = "^0.12.4" }
tracing-subscriber = { version = "^0.3.20", default-features = false, features = [
  "env-filter",
] }
nu-ansi-term = { version = "^0.50.1" }
env_logger = { version = "^0.11.8" }
url = { version = "^2.5", features = ["serde"] }
serde_repr = { version = "^0.1.20" }
pathdiff = { version = "^0.2.3" }
which = { version = "^8.0" }
clap = { version = "^4.5", default-features = false, features = ["derive"] }
glob = { version = "^0.3.3" }
futures-util = { version = "^0.3.31", default-features = false, features = [
  "std",
] }
fs-err = { version = "^3.1.1" }

# AI/ML service dependencies
ort = { version = "2.0", default-features = false, features = [
  "cuda",
  "tensorrt",
  "openvino",
] }
ndarray = { version = "^0.15", default-features = false }
tantivy = { version = "0.21", default-features = false }
hnsw = { version = "0.8", default-features = false }
memmap2 = { version = "^0.9", default-features = false }
lz4 = { version = "^1.24" }
EOF
}

generate_profiles() {
    cat << 'EOF'
[profile.dev]
opt-level = 0
debug = true
debug-assertions = true
overflow-checks = true
strip = false
panic = "unwind"
incremental = true
codegen-units = 256

[profile.release]
opt-level = 3
debug = false
debug-assertions = false
overflow-checks = false
strip = true
panic = "unwind"
incremental = false
codegen-units = 1
lto = true

[profile.bench]
inherits = "release"
debug = true

# Performance analysis profile
[profile.perf]
inherits = "release"
debug = true
strip = false
EOF
}

# ========================================
# Library/Module Template Generator
# ========================================

generate_lib_rs() {
    local crate_name="$1"
    local crate_path="$2"

    mkdir -p "$crate_path/src"

    cat > "$crate_path/src/lib.rs" << EOF
//! $crate_name - $(get_crate_description "$crate_name")
//!
//! A core component of the Rust AI IDE providing specialized functionality
//! for $(get_crate_category "$crate_name") operations.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]

/// Module version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Error types specific to $crate_name
pub mod error;

/// Core functionality for $crate_name
pub mod core;

/// Public API interface
pub mod api;

/// Internal utilities
mod utils;

/// Configuration management
pub mod config;

/// Testing utilities (only available in tests)
#[cfg(test)]
pub mod test_utils;

pub use api::*;
pub use error::*;
EOF

    print_success "Generated lib.rs for $crate_name"
}

generate_error_rs() {
    local crate_name="$1"
    local crate_path="$2"

    cat > "$crate_path/src/error.rs" << EOF
//! Error types and handling for $crate_name

use std::fmt;
use thiserror::Error;

/// Errors that can occur in $crate_name operations
#[derive(Debug, Error)]
pub enum ${crate_name}Error {
    /// IO operation failed
    #[error("IO operation failed: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    /// Operation timeout
    #[error("Operation timed out after {0}s")]
    TimeoutError(u64),

    /// Generic error
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result type alias for $crate_name operations
pub type ${crate_name}Result<T> = Result<T, ${crate_name}Error>;

/// Error handling utilities
pub struct ErrorHandler;

impl ErrorHandler {
    /// Convert any error to ${crate_name}Error
    pub fn handle<E: std::error::Error>(error: E) -> ${crate_name}Error {
        ${crate_name}Error::InternalError(error.to_string())
    }

    /// Convert to ${crate_name}Error with context
    pub fn with_context<E: std::error::Error>(error: E, context: &str) -> ${crate_name}Error {
        ${crate_name}Error::InternalError(format!("{}: {}", context, error))
    }
}
EOF

    print_success "Generated error.rs for $crate_name"
}

generate_readme_md() {
    local crate_name="$1"
    local crate_path="$2"

    cat > "$crate_path/README.md" << EOF
# $crate_name

A specialized crate in the Rust AI IDE workspace providing $(get_crate_description "$crate_name").

## Overview

$crate_name is part of the Rust AI IDE's $(get_crate_category "$crate_name") layer, focusing on $(get_crate_purpose "$crate_name").

## Features

- 🚀 High-performance implementation
- 🔧 Configurable through workspace settings
- 📊 Comprehensive error handling
- 🧪 Thoroughly tested
- 📚 Well-documented API

## Usage

Add this to your \`Cargo.toml\`:

\`\`\`toml
[dependencies]
$crate_name = { path = "../$crate_name" }
\`\`\`

## Architecture

This crate fits into the broader Rust AI IDE architecture as part of the $(get_layer_name "$crate_name") layer.

## Contributing

See the main [CONTRIBUTING.md](../../../CONTRIBUTING.md) for workspace-wide contribution guidelines.

## License

Licensed under $(grep '^license =' ../../../Cargo.toml | cut -d'=' -f2 | tr -d '"' | tr -d ' ' | head -1)
EOF

    print_success "Generated README.md for $crate_name"
}

# ========================================
# Category Detection
# ========================================

get_crate_category() {
    local crate_name="$1"

    if [[ "$crate_name" == *"ai"* ]] || [[ "$crate_name" == *"ml"* ]]; then
        echo "ai/ml"
    elif [[ "$crate_name" == *"core"* ]]; then
        echo "core-infrastructure"
    elif [[ "$crate_name" == *"lsp"* ]] || [[ "$crate_name" == *"cargo"* ]]; then
        echo "system-integration"
    elif [[ "$crate_name" == *"advanced"* ]] || [[ "$crate_name" == *"performance"* ]]; then
        echo "advanced-services"
    elif [[ "$crate_name" == *"shared"* ]] || [[ "$crate_name" == *"common"* ]]; then
        echo "shared-utilities"
    else
        echo "specialized"
    fi
}

get_crate_description() {
    local crate_name="$1"

    case "$crate_name" in
        *"advanced-memory") echo "Advanced memory management and optimization capabilities" ;;
        *"advanced-refactoring") echo "Sophisticated code refactoring and transformation tools" ;;
        *"ai-analysis") echo "Intelligent code analysis and quality assessment" ;;
        *"ai-codegen") echo "AI-powered code generation and synthesis" ;;
        *"core") echo "Core functionality and system foundation" ;;
        *"lsp") echo "Language Server Protocol implementation" ;;
        *"cargo") echo "Cargo integration and build system management" ;;
        *"common") echo "Common utilities and shared functionality" ;;
        *) echo "Specialized functionality for $(get_crate_category "$crate_name") operations" ;;
    esac
}

get_crate_purpose() {
    local crate_name="$1"

    case "$crate_name" in
        *"advanced-memory") echo "optimizing memory usage and performance" ;;
        *"advanced-refactoring") echo "automating complex code transformations" ;;
        *"ai-analysis") echo "providing intelligent code insights" ;;
        *"ai-codegen") echo "generating high-quality code automatically" ;;
        *"core") echo "serving as the central coordination hub" ;;
        *"lsp") echo "enabling multi-language IDE support" ;;
        *"cargo") echo "managing Rust project build and dependencies" ;;
        *"common") echo "standardizing common operations across crates" ;;
        *) echo "delivering specialized $(get_crate_category "$crate_name") capabilities" ;;
    esac
}

get_layer_name() {
    local crate_name="$1"
    local category=$(get_crate_category "$crate_name")

    case "$category" in
        "core-infrastructure") echo "Foundation" ;;
        "ai/ml") echo "AI/ML Specialization" ;;
        "system-integration") echo "System Integration" ;;
        "advanced-services") echo "Advanced Services" ;;
        "shared-utilities") echo "Shared Services" ;;
        *) echo "Specialized Components" ;;
    esac
}

# ========================================
# Main Execution
# ========================================

create_new_crate() {
    local crate_name="$1"
    local crate_type="${2:-library}"

    print_info "Creating new crate: $crate_name"

    # Create crate directory structure
    local crate_path="crates/$crate_name"

    if [[ -d "$crate_path" ]]; then
        read -p "Crate directory already exists. Continue? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_info "Operation cancelled"
            exit 0
        fi
    else
        mkdir -p "$crate_path/src" "$crate_path/tests"
    fi

    # Get description
    local description=$(get_crate_description "$crate_name")

    # Generate configuration files
    generate_crate_cargo_toml "$crate_name" "$description" "$crate_path"
    generate_lib_rs "$crate_name" "$crate_path"
    generate_error_rs "$crate_name" "$crate_path"
    generate_readme_md "$crate_name" "$crate_path"

    # Create basic test file
    cat > "$crate_path/tests/basic.rs" << EOF
//! Basic integration tests for $crate_name

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        // TODO: Implement basic functionality tests
        assert!(true);
    }

    #[test]
    fn test_error_handling() {
        // TODO: Implement error handling tests
        assert!(true);
    }
}
EOF

    # Create .gitignore
    cat > "$crate_path/.gitignore" << 'EOF'
# Rust target directories
target/

# IDE files
.vscode/
.idea/

# OS generated files
.DS_Store
Thumbs.db

# Log files
*.log

# Build artifacts
Cargo.lock

# Test coverage
*.profraw
EOF

    print_success "Crate $crate_name created successfully!"
    print_info "Next steps:"
    echo "1. Edit $crate_path/src/lib.rs with your implementation"
    echo "2. Add the crate to workspace Cargo.toml members list"
    echo "3. Run 'cargo build --workspace' to build with new crate"
    echo "4. Add comprehensive tests in $crate_path/tests/"
    echo "5. Update this crate's README.md with specific usage examples"
}

update_workspace_members() {
    print_info "Updating workspace members in Cargo.toml"

    local workspace_members=""

    # Collect all crate directories
    for crate_dir in crates/rust-*/; do
        if [[ -d "$crate_dir" && -f "${crate_dir}Cargo.toml" ]]; then
            local crate_name=$(basename "$crate_dir")
            workspace_members+="    \"crates/$crate_name\",\n"
        fi
    done

    # Add additional members
    workspace_members+="    \"integration-tests\",\n"
    workspace_members+="    \"src-tauri\",\n"
    workspace_members+="    \"test-performance-analyzer\",\n"
    workspace_members+="    \"test-performance-project\",\n"

    # Update Cargo.toml with new member list
    sed -i.bak '/^members = \[$/,/^\]$/c\members = [\n'"$workspace_members"']' Cargo.toml

    print_success "Updated workspace members in Cargo.toml"
}

validate_workspace_config() {
    print_info "Validating workspace configuration against standards"

    # Check for required files
    local required_files=("Cargo.toml" "README.md")
    for file in "${required_files[@]}"; do
        if [[ ! -f "$file" ]]; then
            print_error "Missing required file: $file"
            exit 1
        fi
    done

    # Validate Cargo.toml structure
    if ! grep -q "\[workspace\]" Cargo.toml; then
        print_error "Cargo.toml missing [workspace] table"
        exit 1
    fi

    print_success "Workspace configuration validation passed"
}

# ========================================
# Script Entry Point
# ========================================

main() {
    print_header

    case "${1:-help}" in
        "create")
            if [[ $# -lt 2 ]]; then
                print_error "Usage: $0 create <crate_name> [library|binary]"
                exit 1
            fi
            create_new_crate "$2" "${3:-library}"
            ;;
        "update-members")
            update_workspace_members
            ;;
        "validate")
            validate_workspace_config
            ;;
        "help"|*)
            echo "Workspace Configuration Template Generator v1.0"
            echo ""
            echo "Usage:"
            echo "  $0 create <crate_name>         Create new crate with standard template"
            echo "  $0 update-members               Update workspace members list"
            echo "  $0 validate                     Validate current workspace configuration"
            echo "  $0 help                         Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0 create rust-ai-ide-new-feature"
            echo "  $0 update-members"
            ;;
    esac
}

# Execute main function
main "$@"