# Contributing to Rust AI IDE

Thank you for your interest in contributing! We welcome all contributions, from bug reports to documentation improvements to code contributions.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/your-username/rust-ai-ide.git`
3. Create a new branch: `git checkout -b feature/your-feature`
4. Make your changes
5. Run tests: `cargo test --all`
6. Submit a pull request

## Code Style

### Rust Code
- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Run `cargo fmt` before committing
- Run `cargo clippy -- -D warnings` to catch common mistakes

### Documentation
- Document all public APIs
- Include examples where appropriate
- Update the relevant documentation when making changes

## Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Formatting, missing semicolons, etc.
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `perf`: Performance improvements
- `test`: Adding or modifying tests
- `chore`: Changes to the build process or auxiliary tools

## Pull Requests

1. Keep pull requests focused on a single feature or fix
2. Include tests for new features
3. Update documentation as needed
4. Ensure all tests pass
5. Request reviews from appropriate team members
