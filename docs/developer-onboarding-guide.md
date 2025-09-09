# Developer Onboarding Guide - v2.4.0

## Welcome to Rust AI IDE Development! 🚀

This guide will help you get started with contributing to the Rust AI IDE, a complex project combining advanced Rust development tools with AI-powered assistance. Whether you're a Rust developer, AI/ML engineer, or full-stack developer, this guide will walk you through the project's architecture, development workflows, and contribution processes.

## 🏁 Quick Start (30 minutes)

### Prerequisites Check

**Required Software:**

```bash
# Check versions
rustc --version   # Should be 1.75+
cargo --version   # Latest stable
node --version    # 18+
pnpm --version    # 8.6+
```

**System Requirements:**

- **RAM**: 16GB minimum (32GB recommended for AI features)
- **Storage**: 20GB free space (for models and datasets)
- **OS**: Linux/macOS recommended, Windows supported

### Fast Setup

```bash
# 1. Clone the repository
git clone --recurse-submodules https://github.com/jcn363/rust-ai-ide.git
cd rust-ai-ide

# 2. Install dependencies
pnpm install

# 3. Setup Rust toolchain
rustup component add rust-analyzer clippy rustfmt

# 4. Build and run in development
pnpm tauri dev
```

> ✅ **Checkpoint**: You should see the IDE window with Monaco editor loaded.

## 🏗️ **System Architecture Overview**

### Core Project Structure

```text
RUST_AI_IDE/
├── crates/                 # Rust backend modules
│   ├── rust-ai-ide-core/   # Core types and utilities
│   ├── rust-ai-ide-ai/     # 🔑 AI services & model management
│   ├── rust-ai-ide-cargo/  # Cargo build system integration
│   ├── rust-ai-ide-lsp/    # Language Server Protocol interface
│   └── ...
├── web/                    # Frontend React/TypeScript application
│   └── src/features/       # Feature modules
│       ├── ai/            # AI-powered development tools
│       └── cargoToml/     # Cargo.toml editing features
├── src-tauri/             # Tauri native desktop application
└── docs/                  # 🆕 Comprehensive documentation
```

### Key Components You Need to Know

#### 🔑 **The AI Service Layer (rust-ai-ide-ai)**

```rust
// Most important crate - handles all AI functionality
use rust_ai_ide_ai::{
    model_loader::ModelRegistry,     // 🧠 Model management
    refactoring::{                    // 🔧 Code transformation
        RefactoringEngine,
        RangeNormalizer,              // Frontend 1-based ↔ Backend 0-based
        BackupManager
    },
    spec_generation::SpecificationParser
};
```

#### 🖥️ **Frontend Architecture**

```typescript
// React/TypeScript with modern tooling
import { useRefactoring } from '@/features/ai/hooks/useRefactoring'
import { RefactoringService } from '@/features/ai/services/RefactoringService'

// Key components:
interface RefactoringPanelProps {
  filePath: string
  codeRange: CodeRange
  onRefactoringComplete: (result: RefactoringResult) => void
}
```

#### 🏠 **The v2.4.0 Key Enhancement**

```rust
// Range Normalization - Critical for Frontend/Backend Communication
let range = RangeNormalizer::frontend_to_backend(&frontendRange);

// Backup Management - Enterprise-grade safety
let backup = BackupManager::execute_with_backup(
    || async_operation(),
    RefactoringType::ExtractFunction,
    &context,
    "path/to/file.rs"
).await;
```

## 🛠️ **Development Workflow**

### Daily Development Cycle

```bash
# 1. Pull latest changes
git pull --recurse-submodules

# 2. Run tests (quick feedback)
cargo test --workspace --lib

# 3. Build with optimizations off
cargo build --workspace

# 4. Run development server
pnpm tauri dev

# 5. Test your changes
cargo test --workspace
```

### Testing Strategy

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test integration_tests

# End-to-end tests with fixtures
cargo test --test e2e_tests

# Frontend tests
cd web && npm test
```

### Code Quality Tools

```bash
# Format code
cargo fmt --all

# Lint with clippy
cargo clippy --all

# Run integrations checks
cargo check --workspace

# Generate documentation
cargo doc --workspace --open
```

## 🌟 **Contributing Areas**

### For Rust Developers

**Perfect for you if:**

- You love Rust's type system
- Enjoy systems programming
- Interested in compiler tooling
- Want to work with AST transformations

**Recommended First Contributions:**

1. **Refactoring Engine**: Add new refactoring operations
2. **LSP Integration**: Enhance language server features
3. **Cargo Integration**: Improve build system support
4. **Performance Optimization**: Memory and CPU optimization

### For AI/ML Developers

**Perfect for you if:**

- You want to integrate LLMs
- Interested in code generation
- Focus on machine learning pipelines
- Build intelligent code analysis

**Recommended First Contributions:**

1. **Model Management**: Add new model types or loaders
2. **Code Generation**: Enhance AI-powered code suggestions
3. **Specification Parser**: Improve natural language parsing
4. **Analysis Engine**: Add new code quality metrics

### For Frontend Developers

**Perfect for you if:**

- Experienced with React/TypeScript
- Enjoy UI/UX development
- Want to build developer tools
- Interested in desktop application development

**Recommended First Contributions:**

1. **UI Components**: Enhance refactoring panels
2. **User Experience**: Improve editor integration
3. **Charts/Data Viz**: Build dependency graphs
4. **Accessibility**: Improve keyboard navigation

## 🎯 **Your First Contribution in 4 Steps**

### Step 1: Explore the Codebase

```bash
# Find areas of interest
find crates/ -name "*.rs" -type f | head -20

# Check recent changes
git log --oneline -10

# Look at open issues labeled "good first issue"
git log --grep="good first issue" --oneline
```

### Step 2: Set Up Your Development Environment

```bash
# Configure git
git config user.name "Your Name"
git config user.email "your.email@example.com"

# Create feature branch
git checkout -b feature/your-first-contribution

# Enable git hooks (pre-commit formatting)
chmod +x .git/hooks/pre-commit
```

### Step 3: Find Your Starting Point

#### Rust Developer Path

```bash
# Look at the refactoring engine
code crates/rust-ai-ide-ai/src/refactoring/mod.rs

# Run refactoring tests
cargo test refactoring_tests

# Add a simple refactoring operation
```

#### AI/ML Developer Path

```bash
# Explore model management
code crates/rust-ai-ide-ai/src/model_loader/mod.rs

# Check integration tests
cargo test integration_tests

# Add model loader support
```

#### Frontend Developer Path

```typescript
// Look at existing components
code web/src/features/ai/components/RefactoringPanel.tsx

// Run frontend
pnpm dev

// Enhance a UI component
```

### Step 4: Make Your First Changes

#### Style Guide Compliance

```rust
// ✅ Good: Clear naming, documentation
/// Analyzes code patterns for refactoring opportunities
fn analyze_refactoring_patterns(code: &str) -> Vec<RefactoringSuggestion> {
    // Implementation
}

// ❌ Bad: Unclear naming, no docs
fn analyzepat(code: &str) -> Vec<Sugg> {
    // Implementation
}
```

```typescript
// ✅ Good: TypeScript with proper typing
interface RefactoringProps {
  code: string
  selections: CodeSelection[]
  onApply: (changes: CodeChange[]) => void
}

const CodeRefactoring = ({ code, selections, onApply }: RefactoringProps) => {
  // Implementation with proper TypeScript
}
```

## 🚨 **Critical Concepts to Master**

### 1. **Range Normalization (Critical!)**

```rust
// Frontend sends 1-based line indices
let frontendRange = CodeRange {
    start_line: 1,      // Line 1 in editor
    start_character: 0, // Character 0
    end_line: 2,        // Line 2
    end_character: 5    // Character 5
};

// Backend expects 0-based indices
let backendRange = RangeNormalizer::frontend_to_backend(&frontendRange);
// Result: start_line: 0, end_line: 1
```

### 2. **Async/Await Patterns**

```rust
// AI services are always async
#[tokio::test]
async fn test_ai_service() {
    let result = model_registry.generate_code(spec).await?;
    assert!(result.success);
}
```

### 3. **Error Handling Convention**

```rust
// Use thiserror for error types
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RefactoringError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Invalid range: {details}")]
    InvalidRange { details: String },

    #[error("Backend error: {source}")]
    BackendError { #[from] source: BackendError }
}
```

### 4. **Safety Patterns**

```rust
// Always validate file existence before operations
let file_path = context.file_path.clone();
if !std::fs::metadata(&file_path)?.is_file() {
    return Err(RefactoringError::FileNotFound { path: file_path });
}
```

## 🧪 **Quality Assurance Checklist**

Before submitting your first PR:

### Code Quality

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy` has no warnings
- [ ] `cargo test --workspace` passes
- [ ] `cargo check --workspace` passes

### Documentation

- [ ] Added or updated doc comments
- [ ] Updated relevant documentation
- [ ] Added tests for new functionality

### Testing

- [ ] Added unit tests for new code
- [ ] Added integration tests if needed
- [ ] Verified edge cases are handled
- [ ] Performance impact assessed

## 📚 **Resources by Role**

### Rust Developer Resources

- **Book**: [The Rust Programming Language](https://doc.rust-lang.org/book/)
- **API**: [Standard Library Documentation](https://doc.rust-lang.org/std/)
- **Tools**: [rust-analyzer User's Manual](https://rust-analyzer.github.io/manual.html)
- **Project**: [Cargo Book](https://doc.rust-lang.org/cargo/index.html)

### AI/ML Developer Resources

- **Model**: [CodeLlama Documentation](https://github.com/facebookresearch/codellama)
- **Tools**: [Hugging Face Rust](https://github.com/huggingface/tokenizers)
- **Performance**: [Rust NN Book](https://russellwinder.github.io/neural-network-book/)
- **Integration**: [Tokio Async Book](https://tokio.rs/tokio/tutorial)

### Frontend Developer Resources

- **Framework**: [React Documentation](https://react.dev/)
- **Language**: [TypeScript Handbook](https://www.typescriptlang.org/docs/)
- **Tools**: [Monaco Editor API](https://microsoft.github.io/monaco-editor/)
- **Styling**: [Emotion Documentation](https://emotion.sh/docs/introduction)

## 🔄 **Weekly Learning Path**

### Week 1: Getting Comfortable

- [ ] Complete fast setup
- [ ] Explore all crates once
- [ ] Run all test suites
- [ ] Submit tiny documentation fix

### Week 2: Understanding the System

- [ ] Read system architecture overview
- [ ] Understand range normalization
- [ ] Run E2E tests manually
- [ ] Fix small refactoring bug

### Week 3: Active Contribution

- [ ] Add small feature enhancement
- [ ] Write comprehensive tests
- [ ] Update documentation
- [ ] Submit meaningful PR

### Week 4: Independent Development

- [ ] Manage your own feature branch
- [ ] Handle merge conflicts
- [ ] Review other contributors' PRs
- [ ] Mentor newcomer if capable

## 🆘 **Getting Help**

### Communication Channels

1. **GitHub Issues**: [Bug reports & feature requests](https://github.com/jcn363/rust-ai-ide/issues)
2. **Discord**: Join `#rust-ai-ide` community
3. **Documentation**: Check `docs/` directory first
4. **Code**: Look at existing implementations

### When You're Stuck

```bash
# Search for similar implementations
git grep "similar_pattern"

# Look for examples
find . -name "*.rs" -exec grep -l "example" {} \;

# Check recent changes
git log --grep="similar_feature" --oneline
```

### Health Check Questions

- ✅ Do you understand the project structure?
- ✅ Can you run `cargo test` successfully?
- ✅ Do you know where the important files are?
- ✅ Can you create a basic test?
- ✅ Have you looked at the existing documentation?

## 🎉 **Celebration Milestones**

- **First Green CI**: Your code builds and tests pass
- **First Merged PR**: Welcome to the contributor community
- **First Feature**: You helped shape the IDE's future
- **Mentor Status**: You can now help others onboard

## ⚡ **Quick Reference Commands**

```bash
# Development
pnpm tauri dev              # Start development
pnpm dev                    # Frontend only
cargo build                 # Backend build

# Quality checks
cargo fmt --all            # Format code
cargo clippy --all         # Lint code
cargo test --workspace     # Run tests
cargo doc --open           # View docs

# CI/CD simulation
cargo check --workspace    # Compiler check
cargo test --all-targets   # Full test coverage

# Useful tools
cargo tree                 # Dependency graph
cargo audit                # Security audit
cargo outdated             # Update check
```

## 🏆 **Next Steps**

1. **Start Small**: Don't try to solve everything at once
2. **Ask Questions**: The community is welcoming and helpful
3. **Share Progress**: Let others know what you're learning
4. **Teach Others**: Explain concepts you mastered to help newcomers
5. **Stay Curious**: Rust AI IDE is at the cutting edge of developer tooling

---

**Remember**: Every expert was once a beginner. Your first contribution is the hardest - every one after gets easier. You have the power to shape the future of how developers use AI in their daily workflow!

**Happy Coding!** 🚀⚡

---

**Version**: v2.4.0
**Last Updated**: September 2, 2025
**Mentors Available**: Check `#contributors` on Discord
