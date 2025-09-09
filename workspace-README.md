# Rust AI IDE Workspace

This is a Cargo workspace containing multiple crates that together form the Rust AI IDE.

## Workspace Structure

```text
.
├── Cargo.toml              # Workspace root manifest
├── workspace-README.md     # This file
├── .cargo/
│   └── config.toml        # Cargo configuration
├── crates/                # Main workspace crates
│   ├── rust-ai-ide-*/     # Various IDE component crates
│   └── shared-test-utils/ # Shared testing utilities
├── scripts/               # Development and maintenance scripts
│   └── check-workspace-deps.sh  # Workspace dependency verification
└── workspace-hack/        # Workspace-hack crate for dependency unification
```

## Development Workflow

### Adding a New Crate

1. Create a new crate in the `crates/` directory:

   ```bash
   cd crates
   cargo new --lib rust-ai-ide-new-crate
   ```

2. Update the workspace `Cargo.toml` to include the new crate:

   ```toml
   [workspace]
   members = [
       # ... existing members ...
       "crates/rust-ai-ide-new-crate",
   ]
   ```

### Managing Dependencies

- **Workspace Dependencies**: Add common dependencies to the root `Cargo.toml` under `[workspace.dependencies]`
- **Crate-specific Dependencies**: Add to the crate's `Cargo.toml` if they're not shared

### Verifying Workspace Health

Run the workspace verification script to check for common issues:

```bash
./scripts/check-workspace-deps.sh
```

This script will:

1. Check for required tools (cargo-hakari, cargo-udeps)
2. Run `cargo check` on all crates
3. Check for unused dependencies with `cargo-udeps`
4. Verify dependency unification with `cargo-hakari`
5. Run security audits with `cargo audit`

## Best Practices

1. **Dependency Management**:
   - Prefer workspace dependencies for shared crates
   - Keep version requirements consistent across the workspace
   - Use the workspace-hack crate to unify feature flags

2. **Versioning**:
   - Use workspace inheritance for common package metadata
   - Bump versions in the root `Cargo.toml`

3. **Testing**:
   - Place unit tests in the same file as the code
   - Put integration tests in the `tests/` directory
   - Use the `shared-test-utils` crate for common test utilities

## Troubleshooting

### Dependency Resolution Issues

If you encounter dependency resolution issues:

1. Update the workspace-hack crate:

   ```bash
   cargo hakari generate
   ```

2. Check for duplicate dependencies:

   ```bash
   cargo tree -d
   ```

### Feature Unification Problems

If you see feature unification errors:

1. Check the workspace-hack crate's dependencies
2. Ensure consistent feature flags across crates
3. Run `cargo hakari verify` to check for issues

## CI/CD

The workspace includes GitHub Actions workflows in `.github/workflows/` that:

- Run tests on all platforms
- Check formatting and linting
- Generate documentation
- Perform security audits

## License

This workspace is licensed under either of:

- MIT license (see LICENSE-MIT)
- Apache License, Version 2.0 (see LICENSE-APACHE)

at your option.
