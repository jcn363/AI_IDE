# Architectural Decision Records (ADRs)

This directory contains Architectural Decision Records (ADRs) that document the key architectural decisions made during the development of the Rust AI IDE project.

## ADR Structure

Each ADR follows a consistent format:

```markdown
# ADR-XXX: Title of the Decision

## Status

- **Date**: YYYY-MM-DD
- **Status**: [Proposed | Accepted | Deprecated | Superseded]
- **Supersedes**: ADR-XXX (if applicable)

## Context

[Description of the context and forces that led to this decision]

## Decision

[Description of the decision made]

## Consequences

### Positive
- [List of positive consequences]

### Negative
- [List of negative consequences]

### Risks
- [List of risks and mitigation strategies]

## Alternatives Considered

- [Alternative 1]: [Reason why not chosen]
- [Alternative 2]: [Reason why not chosen]

## Implementation Notes

[Technical details, code references, or implementation specifics]

## Related ADRs

- [List of related ADRs]
```

## ADR Status Definitions

- **Proposed**: Decision is being considered
- **Accepted**: Decision has been approved and is in effect
- **Deprecated**: Decision is no longer recommended but may still be in use
- **Superseded**: Decision has been replaced by a newer ADR

## Current ADRs

| ADR | Title | Status | Date |
|-----|-------|--------|------|
| [ADR-001](adr-001-multi-crate-workspace-architecture.md) | Multi-Crate Workspace Architecture | Accepted | 2025-01-13 |
| [ADR-002](adr-002-nightly-rust-usage.md) | Nightly Rust Usage and Feature Adoption | Accepted | 2025-01-13 |
| [ADR-003](adr-003-tauri-integration-patterns.md) | Tauri Integration and Command Handling | Accepted | 2025-01-13 |
| [ADR-004](adr-004-ai-ml-service-architecture.md) | AI/ML Service Architecture and LSP Integration | Accepted | 2025-01-13 |
| [ADR-005](adr-005-security-framework.md) | Security Framework and Validation Patterns | Accepted | 2025-01-13 |
| [ADR-006](adr-006-async-concurrency-patterns.md) | Async Concurrency Patterns and State Management | Accepted | 2025-01-13 |

## Contributing

When creating a new ADR:

1. Use the next available number (ADR-XXX format)
2. Follow the established template
3. Include comprehensive context and rationale
4. Document alternatives considered
5. Update this README with the new ADR entry
6. Discuss the ADR with the team before marking as Accepted

## ADR Categories

ADRs are organized by category:

- **Architecture**: Core architectural patterns and decisions
- **Technology**: Technology stack and framework choices
- **Security**: Security-related architectural decisions
- **Performance**: Performance and scalability decisions
- **Integration**: Integration patterns and protocols

This ADR collection serves as a living documentation of our architectural evolution and provides context for future development decisions.