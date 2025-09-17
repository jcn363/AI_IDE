#!/bin/bash

# Configuration
VERSION="3.3.0"
DOCS_DIR="$(pwd)/docs"

# Create required directories if they don't exist
mkdir -p "${DOCS_DIR}/src" "${DOCS_DIR}/book"

# Function to safely update version in markdown files
update_version() {
    find "${DOCS_DIR}/src" -type f -name "*.md" -exec sed -i.bak -E "s/Version: .*/Version: ${VERSION}/g" {} \;
    find "${DOCS_DIR}/src" -type f -name "*.md.bak" -delete
}

# Function to check for required tools
check_requirements() {
    local missing=0
    
    if ! command -v mdbook &> /dev/null; then
        echo "Error: mdbook is required but not installed."
        echo "Install with: cargo install mdbook"
        missing=1
    fi
    
    if ! command -v cargo &> /dev/null; then
        echo "Error: cargo is required but not installed."
        echo "Install Rust from: https://rustup.rs/"
        missing=1
    fi
    
    [ $missing -eq 1 ] && exit 1
}

# Main function
main() {
    echo "Updating documentation to version ${VERSION}..."
    
    # Check requirements
    check_requirements
    
    # Update version in documentation
    echo "Updating version numbers..."
    update_version
    
    # Generate API documentation
    echo "Generating API documentation..."
    if [ -f "Cargo.toml" ]; then
        cargo doc --no-deps --document-private-items || echo "Warning: Failed to generate API docs"
    else
        echo "No Cargo.toml found, skipping API doc generation"
    fi
    
    # Build the documentation
    echo "Building documentation with mdBook..."
    if [ -f "${DOCS_DIR}/book.toml" ]; then
        (cd "${DOCS_DIR}" && mdbook build)
    else
        echo "Error: ${DOCS_DIR}/book.toml not found"
        exit 1
    fi
    
    # Check for broken links if markdown-link-check is available
    if command -v markdown-link-check &> /dev/null; then
        echo "Checking for broken links..."
        find "${DOCS_DIR}/book" -type f -name "*.html" | xargs -I{} markdown-link-check {} || true
    else
        echo "markdown-link-check not found. Install with: npm install -g markdown-link-check"
    fi
    
    echo "Documentation update complete!"
    echo "You can view the documentation at: file://${DOCS_DIR}/book/index.html"
}

# Run the main function
main
