# Project Documentation Rules (Non-Obvious Only)

## Documentation Organization
- "src-tauri/" contains Tauri Rust code, web/ contains React frontend (clear naming but different tech stacks)
- AI/ML modules heavily use LSP service initialization - function docs missing this prerequisite
- Cloud integrations lack initialization order documentation - webhook system must start on port 3000 first
- Extension vs webview features not clearly separated - some features non-functional due to isolation

## Missing Documentation Traps
- Many Tauri command implementations are placeholders returning dummy data
- Function signatures mismatch documentation when implementations not completed
- AI service initialization documented but not enforced - causes silent feature failures
- Database migration scripts documented as bidirectional but actually forward-only

## Source Code Reading Required
- Real command behaviors found in src-tauri/src/commands/ not project docs
- AI/ML pipeline configurations scattered across LSP, ai-analysis, and core-ai crates
- Security policies enforce banned crates but licenses aren't obviously documented
- Performance patterns use non-standard caching (Moka TTL) without explanation

## Configuration Gotchas
- Rust-toolchain.toml requires nightly but Cargo.toml has stable rust-version
- Package.json scripts generate types from Rust but don't document the generator binary
- Web build scripts depend on Cargo but no documentation about the interdependency
- Monorepo workspaces have circular dependency warnings documented but intended

## Component Relationships
- LSP service integrates with Tauri state but no clear command mapping
- Webview ES modules isolated from extension but documentation suggests unified API
- AI model loading uses LSP channels but development docs show direct access patterns
- Security crate audit logging integrated but not documented in main workflows

## Reference Sources
- Command implementations in src-tauri/src/commands/ provide actual capabilities
- Integration tests in integration-tests/ demonstrate real use cases not in docs
- Architecture diagrams in docs/Architecture-Diagrams.md can't be automatically generated
- Ethics framework documented but not automated in AI/ML pipelines