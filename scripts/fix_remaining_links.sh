#!/bin/bash
set -e

# Create missing files
touch docs/src/getting-started/QUICKSTART.md
cat > docs/src/getting-started/QUICKSTART.md << 'EOL'
# Quick Start Guide

## First Steps

1. **Install** the IDE following the [Installation Guide](INSTALLATION.md)
2. **Launch** the application
3. **Open** a project folder
4. **Start coding!** 🚀

## Common Tasks

### Opening a Project
1. Click "File" > "Open Folder"
2. Select your project directory
3. Start editing files

### Running Code
- Press `F5` to start debugging
- Use `Ctrl+F5` to run without debugging

### Using the Terminal
- Open integrated terminal with `` Ctrl+` ``
- Run build commands directly in the IDE

## Next Steps
- [Learn about features](../features/ai-features.md)
- [Configure your environment](../getting-started/CONFIGURATION.md)
- [Read the full documentation](../user-guide/README.md)
EOL

# Fix PLUGINS.md reference
sed -i 's/PLUGINS\.md/PLUGINS.html/g' docs/src/overview/README.md

# Create a simple CONTRIBUTING.md if it doesn't exist
if [ ! -f "docs/src/development/CONTRIBUTING.md" ]; then
    cat > docs/src/development/CONTRIBUTING.md << 'EOL'
# Contributing to Rust AI IDE

Thank you for your interest in contributing! We welcome all contributions.

## Getting Started

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## Code Style

Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/) format:

```
type(scope): brief description

Detailed description

BREAKING CHANGE: details about breaking changes
```

## Pull Requests

1. Update the README.md with details of changes
2. Add tests for new features
3. Ensure all tests pass

## License

By contributing, you agree that your contributions will be licensed under the project's license.
EOL
fi

echo "Fixed remaining documentation issues!"
echo "Run './scripts/update_docs.sh' to rebuild the documentation."
