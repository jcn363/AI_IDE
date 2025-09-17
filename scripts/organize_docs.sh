#!/bin/bash

# Create necessary directories
mkdir -p docs/src/{overview,getting-started,user-guide,development,api,features,enterprise}

# Move existing documentation to the new structure
mv docs/overview/README.md docs/src/overview/
mv docs/getting-started/INSTALLATION.md docs/src/getting-started/
mv docs/features/ai-features.md docs/src/features/
mv docs/development/CONTRIBUTING.md docs/src/development/
mv docs/api/README.md docs/src/api/

# Create missing configuration files
cat > docs/src/overview/README.md << 'EOL'
# Rust AI IDE

[![Build Status](https://img.shields.io/badge/build-98%25+-brightgreen)](https://github.com/org/rust-ai-ide/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> **Version**: 3.3.0  
> **Status**: Active Development  
> **Last Updated**: 2025-09-16

## Overview
A high-performance, AI-powered IDE built with Rust, designed for modern software development.

## Key Features
- [AI-Powered Development](features/ai-features.md)
- [Enterprise-Grade Security](features/security.md)
- [Collaborative Tools](features/collaboration.md)
- [Extensible Architecture](development/plugins.md)

## Quick Links
- [Installation Guide](getting-started/installation.md)
- [User Guide](user-guide/README.md)
- [Contributing](development/CONTRIBUTING.md)
- [API Reference](api/README.md)
EOL

# Fix permissions
chmod -R 755 docs/src

# Build the documentation
cd docs && mdbook build

echo "Documentation organization complete! Access the documentation at: file://$(pwd)/book/index.html"
