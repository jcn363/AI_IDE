.PHONY: all build test generate-types watch generate-all docs clean

# Default target
all: build test

# Build the project
build:
	cargo build

# Run tests
test:
	cargo test

# Generate types for all platforms
generate-all:
	cargo run --bin type_generator -- --all-platforms --output generated

# Generate specific platform types
generate-ts:
	cargo run --bin type_generator -- --platform typescript --type-guards --strict-null

generate-py:
	cargo run --bin type_generator -- --platform python --python-format dataclass

generate-go:
	cargo run --bin type_generator -- --platform go --go-package models

generate-graphql:
	cargo run --bin type_generator -- --platform graphql --graphql-schema federation

generate-openapi:
	cargo run --bin type_generator -- --platform openapi

# Watch for changes and regenerate
watch:
	cargo watch -x 'run --bin type_generator -- --all-platforms --output generated'

# Quick generation (TypeScript only)
generate:
	cargo run --bin type_generator -- --platform typescript

# Generate documentation
docs:
	cargo doc --no-deps --open

# Clean generated files
clean:
	cargo clean
	rm -rf generated/
	rm -rf frontend/src/types/generated.ts
	rm -rf backend/api/models.*
	rm -rf docs/schema.graphql
	rm -rf docs/openapi.json

# Install development dependencies
setup:
	cargo add clap --features derive
	cargo add serde_json
	cargo install cargo-watch

# Show help
help:
	@echo "Available targets:"
	@echo "  build           - Build the project"
	@echo "  test            - Run tests"
	@echo "  generate-all    - Generate types for all platforms"
	@echo "  generate-ts     - Generate TypeScript types"
	@echo "  generate-py     - Generate Python types"
	@echo "  generate-go     - Generate Go types"
	@echo "  generate-graphql- Generate GraphQL schema"
	@echo "  generate-openapi- Generate OpenAPI spec"
	@echo "  generate        - Quick TypeScript generation"
	@echo "  watch           - Watch for changes and regenerate"
	@echo "  docs            - Generate documentation"
	@echo "  clean           - Clean generated files"
	@echo "  setup           - Install development dependencies"
	@echo "  help            - Show this help message"

# Production build with type generation
production: clean
	cargo build --release
	cargo run --release --bin type_generator -- --all-platforms --output dist

# Development workflow
dev: generate
	cargo build
	cargo test