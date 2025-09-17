#!/bin/bash
set -e

# Create necessary directories
mkdir -p docs/src/{overview,getting-started,user-guide,development,api,features,enterprise}

# Move existing markdown files to src directory
find docs -maxdepth 1 -name "*.md" -exec mv {} docs/src/ \;

# Create basic documentation files if they don't exist
[ ! -f docs/src/overview/README.md ] && cat > docs/src/overview/README.md << 'EOL'
# Rust AI IDE

[![Build Status](https://img.shields.io/badge/build-98%25+-brightgreen)](https://github.com/org/rust-ai-ide/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> **Version**: 3.3.0  
> **Status**: Active Development  
> **Last Updated**: 2025-09-16

## Overview
A high-performance, AI-powered IDE built with Rust, designed for modern software development.
EOL

# Create book.toml if it doesn't exist
[ ! -f docs/book.toml ] && cat > docs/book.toml << 'EOL'
[book]
title = "Rust AI IDE Documentation"
authors = ["Rust AI IDE Team"]
language = "en"
multilingual = false
src = "src"

[output.html]
default-theme = "rust"
preferred-dark-theme = "navy"
EOL

# Create SUMMARY.md if it doesn't exist
[ ! -f docs/src/SUMMARY.md ] && cat > docs/src/SUMMARY.md << 'EOL'
# Summary

- [Introduction](overview/README.md)
- [Getting Started](getting-started/INSTALLATION.md)
- [Features](features/ai-features.md)
- [Development](development/CONTRIBUTING.md)
- [API Reference](api/README.md)
EOL

# Set proper permissions
chmod -R 755 docs/src

# Build the documentation
cd docs
if ! command -v mdbook &> /dev/null; then
    echo "mdBook not found. Installing..."
    cargo install mdbook
fi

mdbook build

echo "Documentation build complete! Access the documentation at: file://$(pwd)/book/index.html"
