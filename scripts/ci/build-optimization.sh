#!/bin/bash

# Build Optimization Script for Rust AI IDE
# This script optimizes the build process for CI/CD pipelines
# Author: CI/CD Automation Team
# Version: 1.0.0

set -euo pipefail

# Constants
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
BUILD_LOG="${PROJECT_ROOT}/build-optimization.log"
BUILD_START_TIME=$(date +%s)

# Logging functions
log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*" | tee -a "${BUILD_LOG}"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "${BUILD_LOG}" >&2
}

log_warning() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $*" | tee -a "${BUILD_LOG}"
}

# Function to display usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Build optimization script for Rust AI IDE CI/CD pipelines.

OPTIONS:
    -h, --help              Show this help message
    -v, --verbose           Enable verbose output
    -t, --test              Run tests during optimization
    -c, --clean-cache       Clean build cache before optimization
    -p, --parallel-jobs N   Number of parallel build jobs (default: auto-detect)
    -o, --output DIR        Output directory for artifacts (default: target/optimized)
    --target-platform       Target platform architecture
    --features FEATURES     Cargo features to enable
    --profile PROFILE       Cargo build profile (default: release)

EXAMPLES:
    $0 --test --parallel-jobs 4
    $0 --clean-cache --target-platform x86_64-unknown-linux-gnu
    $0 --features ai-inference,gpu-support --profile release

EOF
}

# Parse command line arguments
VERBOSE=false
RUN_TESTS=false
CLEAN_CACHE=false
PARALLEL_JOBS=$(nproc 2>/dev/null || echo 4)
OUTPUT_DIR="${PROJECT_ROOT}/target/optimized"
TARGET_PLATFORM=""
FEATURES=""
PROFILE="release"

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            exit 0
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -t|--test)
            RUN_TESTS=true
            shift
            ;;
        -c|--clean-cache)
            CLEAN_CACHE=true
            shift
            ;;
        -p|--parallel-jobs)
            PARALLEL_JOBS="$2"
            shift 2
            ;;
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --target-platform)
            TARGET_PLATFORM="$2"
            shift 2
            ;;
        --features)
            FEATURES="$2"
            shift 2
            ;;
        --profile)
            PROFILE="$2"
            shift 2
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Function to check system requirements
check_requirements() {
    log_info "Checking system requirements..."

    # Check for required tools
    local required_tools=("cargo" "rustc" "pkg-config")
    for tool in "${required_tools[@]}"; do
        if ! command -v "${tool}" >/dev/null 2>&1; then
            log_error "Required tool '${tool}' not found"
            return 1
        fi
    done

    # Check Rust version
    local rust_version
    rust_version=$(rustc --version | grep -oP '\d+\.\d+\.\d+')
    log_info "Rust version: ${rust_version}"

    # Check available memory
    local available_memory
    available_memory=$(free -m | awk 'NR==2{printf "%.2f", $7/1024}')
    log_info "Available memory: ${available_memory} GB"

    if (( $(echo "${available_memory} < 2.0" | bc -l) )); then
        log_warning "Low memory detected (${available_memory} GB). Build may be slow."
    fi

    return 0
}

# Function to optimize cargo configuration
optimize_cargo_config() {
    log_info "Optimizing Cargo configuration..."

    local cargo_config="${PROJECT_ROOT}/.cargo/config.toml"

    # Create .cargo directory if it doesn't exist
    mkdir -p "${PROJECT_ROOT}/.cargo"

    cat > "${cargo_config}" << EOF
[build]
jobs = ${PARALLEL_JOBS}
rustc-wrapper = "sccache"

[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "target-cpu=native", "-C", "opt-level=3"]

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
debug = false

[profile.dev]
opt-level = 0
debug = true
EOF

    log_info "Cargo configuration optimized"
}

# Function to setup sccache for faster builds
setup_sccache() {
    log_info "Setting up sccache for build caching..."

    if ! command -v sccache >/dev/null 2>&1; then
        log_info "Installing sccache..."
        cargo install sccache
    fi

    export RUSTC_WRAPPER=sccache
    export SCCACHE_DIR="${PROJECT_ROOT}/.sccache"

    # Start sccache server
    sccache --start-server >/dev/null 2>&1 || true

    log_info "Sccache setup complete"
}

# Function to clean build cache if requested
clean_cache() {
    if [[ "${CLEAN_CACHE}" == true ]]; then
        log_info "Cleaning build cache..."
        cargo clean
        rm -rf "${PROJECT_ROOT}/.sccache" "${PROJECT_ROOT}/target"
        log_info "Build cache cleaned"
    fi
}

# Function to optimize dependencies
optimize_dependencies() {
    log_info "Optimizing dependencies..."

    # Update Cargo.lock with minimal versions
    cargo update --package minreq --precise 2.4.2 || log_warning "Could not optimize minreq version"

    # Trim unused dependencies
    cargo +nightly udeps --workspace || log_warning "Unused dependencies check failed (nightly toolchain may not be available)"

    log_info "Dependencies optimized"
}

# Function to build with optimizations
build_optimized() {
    log_info "Starting optimized build..."

    local build_args=("build" "--profile" "${PROFILE}")

    if [[ -n "${FEATURES}" ]]; then
        build_args+=("--features" "${FEATURES}")
    fi

    if [[ -n "${TARGET_PLATFORM}" ]]; then
        build_args+=("--target" "${TARGET_PLATFORM}")
    fi

    # Set environment variables for optimized build
    export CARGO_BUILD_JOBS="${PARALLEL_JOBS}"
    export RUSTFLAGS="-C target-cpu=native -C opt-level=3 -C codegen-units=1"

    if [[ "${VERBOSE}" == true ]]; then
        build_args+=("-v")
    fi

    log_info "Running: cargo ${build_args[*]}"
    cargo "${build_args[@]}"

    log_info "Build completed successfully"
}

# Function to run optimized tests
run_optimized_tests() {
    if [[ "${RUN_TESTS}" != true ]]; then
        return 0
    fi

    log_info "Running optimized tests..."

    local test_args=("test" "--profile" "${PROFILE}")

    if [[ -n "${FEATURES}" ]]; then
        test_args+=("--features" "${FEATURES}")
    fi

    if [[ -n "${TARGET_PLATFORM}" ]]; then
        test_args+=("--target" "${TARGET_PLATFORM}")
    fi

    # Set test-specific optimizations
    export RUST_TEST_THREADS="${PARALLEL_JOBS}"
    export CARGO_TEST_TIMEOUT=300

    if [[ "${VERBOSE}" == true ]]; then
        test_args+=("-v")
    fi

    log_info "Running: cargo ${test_args[*]}"
    cargo "${test_args[@]}"

    log_info "Tests completed successfully"
}

# Function to collect build artifacts
collect_artifacts() {
    log_info "Collecting build artifacts..."

    # Create output directory
    mkdir -p "${OUTPUT_DIR}"

    # Copy optimized binaries
    local target_dir="${PROJECT_ROOT}/target"
    if [[ -n "${TARGET_PLATFORM}" ]]; then
        target_dir="${target_dir}/${TARGET_PLATFORM}"
    fi

    find "${target_dir}/${PROFILE}" -maxdepth 1 -type f -executable -exec cp {} "${OUTPUT_DIR}/" \;

    # Generate build report
    local build_end_time=$(date +%s)
    local build_duration=$((build_end_time - BUILD_START_TIME))

    cat > "${OUTPUT_DIR}/build-report.txt" << EOF
Build Report
============
Generated: $(date)
Duration: ${build_duration} seconds
Platform: ${TARGET_PLATFORM:-$(uname -m)}
Profile: ${PROFILE}
Features: ${FEATURES:-none}
Parallel Jobs: ${PARALLEL_JOBS}
Cache Used: $(sccache -s 2>/dev/null | grep -oP 'Cache hits:\s*\K\d+' || echo "N/A")

Artifacts:
$(ls -la "${OUTPUT_DIR}" | grep -v "^d" | awk '{print "  " $9 " (" $5 " bytes)"}')
EOF

    log_info "Artifacts collected in: ${OUTPUT_DIR}"
}

# Function to display build statistics
display_stats() {
    log_info "Build optimization completed!"

    local build_end_time=$(date +%s)
    local total_duration=$((build_end_time - BUILD_START_TIME))

    log_info "Total time: ${total_duration} seconds"

    # Display cache statistics
    if command -v sccache >/dev/null 2>&1; then
        log_info "Cache statistics:"
        sccache -s 2>/dev/null || log_warning "Could not retrieve cache statistics"
    fi

    # Display binary sizes
    log_info "Binary sizes:"
    find "${OUTPUT_DIR}" -type f -executable -exec ls -lh {} \; | awk '{print "  " $9 ": " $5}'
}

# Main function
main() {
    log_info "Starting build optimization script"
    log_info "Log file: ${BUILD_LOG}"

    # Trap to ensure cleanup on exit
    trap 'log_info "Build optimization script finished (exit code: $?)"; sccache --stop-server 2>/dev/null || true' EXIT

    check_requirements
    optimize_cargo_config
    setup_sccache
    clean_cache
    optimize_dependencies
    build_optimized
    run_optimized_tests
    collect_artifacts
    display_stats

    log_info "Build optimization completed successfully"
}

# Run main function
main "$@"