# Security Policy

## Supported Versions

| Version | Supported          | Security Status                  |
| ------- | ------------------ | -------------------------------- |
| 3.2.x   | :white_check_mark: | Active support, security updates |
| 3.1.x   | :warning:          | Security fixes only              |
| < 3.1   | :x:                | No longer supported              |

## Reporting a Vulnerability

Please report (suspected) security vulnerabilities to [security team](security@rust-ai-ide.example.com). You will receive a response within 24 hours. If the issue is confirmed, we will release a patch as soon as possible.

## Security Measures (Updated 2025-09-14)

1. **Dependency Security**:
   - Automated security scanning using `cargo-audit`
   - Regular dependency updates with Dependabot
   - License compliance checks using `cargo-deny`
   - SBOM generation for all releases

2. **Code Quality & Security**:
   - Static analysis with Clippy (zero warnings enforced)
   - Automated fuzz testing for critical components
   - Regular third-party security audits (last: September 2025)
   - Memory-safe Rust codebase with `#![forbid(unsafe_code)]`

3. **Development Practices**:
   - Mandatory security reviews for all PRs
   - Security-focused pull request templates
   - Automated security testing in CI/CD pipeline
   - Dependency vulnerability scanning in CI

## Active Security Advisories

| Crate    | Version | Severity | Advisory          | Status                 |
| -------- | ------- | -------- | ----------------- | ---------------------- |
| glib     | 0.18.5  | Critical | RUSTSEC-2024-0429 | Mitigation in progress |
| failure  | 0.1.8   | Critical | RUSTSEC-2019-0036 | Patch pending          |
| image    | 0.22.5  | Medium   | RUSTSEC-2020-0073 | Under review           |
| lock_api | 0.3.4   | Medium   | RUSTSEC-2020-0070 | Monitoring             |

## Security Updates

Security updates are released as patch versions. We maintain a 30-day SLA for critical vulnerabilities and 90 days for medium/low severity issues.

### Recent Updates

- 2025-09-14: Fixed dependency resolution for `quick-xml`
- 2025-09-14: Updated security audit configuration (ignoring RUSTSEC-2020-0159)
- 2025-09-13: Added comprehensive security testing to CI pipeline

## Security Advisories

For detailed information about security advisories, please visit our [GitHub Security Advisories](https://github.com/jcn363/rust-ai-ide/security/advisories) page.

## Bug Bounty Program

We offer rewards for reporting security vulnerabilities. See our [Bug Bounty Program](https://github.com/jcn363/rust-ai-ide/security/policy) for details.
