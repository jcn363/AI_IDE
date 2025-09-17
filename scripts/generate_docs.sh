#!/bin/bash
set -e

# Create required directories
mkdir -p docs/src/{getting-started,user-guide,development,api,features,enterprise}

# Generate basic documentation files
cat > docs/src/features/security.md << 'EOL'
# Security Features

## Overview

Rust AI IDE includes several security features to protect your code and development environment.

## Key Security Features

### Code Analysis
- Static code analysis to detect security vulnerabilities
- Dependency scanning for known vulnerabilities
- Secrets detection in code and configuration

### Authentication & Authorization
- Role-based access control (RBAC)
- Multi-factor authentication (MFA)
- OAuth 2.0 and OpenID Connect support

### Data Protection
- Encryption at rest and in transit
- Secure credential storage
- Audit logging for all sensitive operations

## Configuration

Security settings can be configured in `config/security.toml`:

```toml
[authentication]
enable_mfa = true
session_timeout = 3600

[encryption]
key_rotation_days = 90

[audit]
enable_logging = true
retention_days = 365
```

## Best Practices

1. Always enable MFA for production environments
2. Regularly rotate encryption keys
3. Review audit logs periodically
4. Keep dependencies up to date
5. Follow the principle of least privilege

## Troubleshooting

Common issues and solutions:

- **Authentication failures**: Check system time synchronization
- **Permission denied**: Verify user roles and permissions
- **TLS errors**: Ensure certificates are valid and trusted

## Reporting Security Issues

Please report security vulnerabilities to security@example.com.
EOL

cat > docs/src/features/performance.md << 'EOL'
# Performance Optimization

## Overview

This document covers performance optimization techniques and features in Rust AI IDE.

## Performance Features

### Code Analysis
- Incremental compilation
- Parallel dependency resolution
- Caching of analysis results

### Memory Management
- Memory usage optimization
- Leak detection
- Garbage collection tuning

### UI/UX Performance
- Virtual scrolling for large files
- Lazy loading of UI components
- Responsive design optimizations

## Configuration

Performance settings can be adjusted in `config/performance.toml`:

```toml
[compilation]
incremental = true
parallel = true

[memory]
max_heap_size = "4G"
cache_size = "2G"

[ui]
use_hardware_acceleration = true
animation_fps = 60
```

## Best Practices

1. Enable incremental compilation for faster builds
2. Monitor memory usage and adjust heap size as needed
3. Use the latest version of Rust and dependencies
4. Profile regularly to identify bottlenecks

## Troubleshooting

Common performance issues and solutions:

- **Slow compilation**: Enable incremental compilation
- **High memory usage**: Increase heap size or optimize code
- **UI lag**: Disable animations or reduce workspace size
EOL

cat > docs/src/features/collaboration.md << 'EOL'
# Collaboration Features

## Overview

Rust AI IDE provides powerful collaboration tools for team development.

## Key Features

### Real-time Collaboration
- Multi-user editing
- Cursor and selection sharing
- Presence indicators

### Code Review
- Inline comments
- Suggested changes
- Review workflows

### Team Management
- User roles and permissions
- Project sharing
- Activity feeds

## Getting Started

1. Invite team members from the project settings
2. Configure permissions for each member
3. Start collaborating in real-time

## Best Practices

1. Use @mentions to notify team members
2. Keep review feedback constructive
3. Resolve conversations when issues are addressed
4. Use branches for major changes

## Troubleshooting

- **Connection issues**: Check your internet connection
- **Permission errors**: Verify user roles and permissions
- **Sync conflicts**: Resolve conflicts using the built-in tools
EOL

# Create placeholder files for all referenced documentation
for dir in getting-started user-guide development api enterprise; do
  case $dir in
    getting-started)
      files=("CONFIGURATION.md")
      ;;
    user-guide)
      files=("BASIC_USAGE.md" "TROUBLESHOOTING.md")
      ;;
    development)
      files=("ARCHITECTURE.md" "CODING_STANDARDS.md" "TESTING.md")
      ;;
    api)
      files=("CORE_API.md" "AI_API.md" "PLUGIN_API.md")
      ;;
    enterprise)
      files=("DEPLOYMENT.md" "SCALING.md" "MONITORING.md")
      ;;
  esac
  
  for file in "${files[@]}"; do
    if [ ! -f "docs/src/$dir/$file" ]; then
      echo "# ${file%.*}" > "docs/src/$dir/$file"
      echo "\n*Documentation coming soon.*" >> "docs/src/$dir/$file"
    fi
  done
done

echo "Documentation files generated successfully!"
