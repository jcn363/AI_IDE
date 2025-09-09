# Workspace Maintenance Scripts

This directory contains scripts to help maintain and verify the health of the Rust AI IDE workspace.

## Available Scripts

### `check-workspace-consistency.sh`

A comprehensive script that checks various aspects of workspace health, including:

- Workspace-hack dependency management
- Unused dependencies
- Outdated dependencies
- Duplicate dependencies
- Workspace member verification

**Usage:**

```bash
./scripts/check-workspace-consistency.sh
```

### `check-workspace-deps.sh`

Verifies dependency consistency across the workspace using various Cargo tools.

**Usage:**

```bash
./scripts/check-workspace-deps.sh
```

## Prerequisites

These scripts require the following tools to be installed:

- `cargo-edit` - For version management
- `cargo-hakari` - For workspace-hack management
- `cargo-udeps` - For finding unused dependencies
- `jq` - For JSON processing

Install them using:

```bash
cargo install cargo-edit cargo-hakari
cargo install cargo-udeps --locked
```

## Best Practices

1. Run these scripts before creating pull requests
2. Fix all warnings and errors before merging to main
3. Keep the workspace-hack crate up to date
4. Use consistent versions across all crates

## Troubleshooting

If you encounter issues:

1. Run `cargo clean` and try again
2. Update your Rust toolchain: `rustup update`
3. Ensure all required tools are installed
4. Check for network connectivity issues

## License

This project is licensed under either of:

- MIT license (see [LICENSE-MIT](../LICENSE-MIT))
- Apache License, Version 2.0 (see [LICENSE-APACHE](../LICENSE-APACHE))

at your option.
