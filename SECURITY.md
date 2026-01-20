# Security Policy

> Last Updated: January 2026

The OxideKit project takes security seriously. This document outlines our security policy, vulnerability reporting process, and security practices.

---

## Supported Versions

| Version | Support Status | Security Updates |
|---------|---------------|------------------|
| 1.x (current) | Active | Yes |
| 0.x (pre-release) | Deprecated | Critical only |

LTS (Long-Term Support) versions receive security updates for extended periods. See [enterprise-offering.md](docs/enterprise-offering.md) for LTS details.

---

## Reporting a Vulnerability

### Where to Report

**Email**: security@oxidekit.com

**PGP Key**: Available at https://oxidekit.com/.well-known/security.txt

### What to Include

1. **Description**: Clear explanation of the vulnerability
2. **Impact**: What can an attacker do with this vulnerability?
3. **Reproduction**: Step-by-step instructions to reproduce
4. **Environment**: OS, OxideKit version, relevant configuration
5. **Proof of concept**: Code or screenshots (if applicable)
6. **Suggested fix**: Your recommendation (if any)

### What NOT to Do

- Do not disclose publicly before we've had time to address it
- Do not access or modify other users' data
- Do not perform denial-of-service attacks
- Do not social engineer OxideKit team members

---

## Response Process

### Timeline

| Stage | Target Time |
|-------|-------------|
| Acknowledgment | 24-48 hours |
| Initial assessment | 72 hours |
| Status update | Weekly |
| Fix development | Varies by severity |
| Disclosure coordination | Before public disclosure |

### Severity Classification

| Severity | Description | Target Resolution |
|----------|-------------|-------------------|
| **Critical** | Remote code execution, privilege escalation | 7 days |
| **High** | Data exposure, security bypass | 14 days |
| **Medium** | Limited impact vulnerabilities | 30 days |
| **Low** | Minor issues, hardening opportunities | 90 days |

### Process Flow

1. **Receipt**: We acknowledge your report
2. **Triage**: We assess severity and validity
3. **Investigation**: We reproduce and analyze
4. **Fix Development**: We create and test a patch
5. **Release Planning**: We coordinate timing
6. **Disclosure**: We publish advisory and credit

---

## Security Advisories

### Where We Publish

- **GitHub Security Advisories**: Primary channel
- **Website**: https://oxidekit.com/security/advisories
- **Mailing List**: security-announce@oxidekit.com

### Advisory Format

Each advisory includes:
- CVE identifier (when applicable)
- Affected versions
- Severity rating (CVSS)
- Description of vulnerability
- Recommended actions
- Credit to reporter

---

## Security Practices

### Code Security

- **Memory safety**: Rust's ownership model prevents common vulnerabilities
- **Dependency auditing**: Regular `cargo audit` checks
- **SAST**: Static analysis in CI pipeline
- **Code review**: All changes require review

### Build Security

- **Reproducible builds**: Deterministic compilation
- **Signed releases**: GPG-signed binaries
- **SBOM generation**: Software Bill of Materials for each release
- **Provenance**: Build provenance attestations

### Infrastructure Security

- **Minimal attack surface**: Limited external dependencies
- **Sandboxing**: Applications run in capability-restricted environments
- **Network isolation**: Explicit network permissions required
- **Update verification**: Signed update manifests

---

## Capability Security Model

OxideKit implements a capability-based security model:

### Default Deny

Applications have no permissions by default. All capabilities must be:
1. Declared in the application manifest
2. Approved by the user or administrator
3. Verified at runtime

### Capability Categories

| Category | Examples | Risk Level |
|----------|----------|------------|
| File System | Read/write specific paths | Medium |
| Network | HTTP, WebSocket, specific hosts | High |
| System | Clipboard, notifications | Low |
| Hardware | Camera, microphone, USB | High |
| IPC | Inter-process communication | Medium |

### Verification

The OxideKit Attestation Service verifies that applications:
- Only request declared capabilities
- Do not contain known malware signatures
- Have valid and traceable builds

---

## Dependency Policy

### Allowed Dependencies

- Rust crates with active maintenance
- Audited cryptographic libraries
- Dependencies with compatible licenses

### Prohibited Dependencies

- Unmaintained crates (>2 years inactive)
- Crates with known unpatched vulnerabilities
- Crates with restrictive or unclear licenses

### Audit Process

1. **Automated**: `cargo audit` runs on every PR
2. **Manual**: Periodic manual review of dependency tree
3. **Updates**: Security updates applied within 7 days

---

## Vulnerability Disclosure Policy

### Coordinated Disclosure

We follow coordinated disclosure:

1. **Private notification**: Reporter contacts us privately
2. **Fix development**: We develop and test a fix
3. **Advance notice**: We notify major users before public disclosure
4. **Public disclosure**: We publish advisory and fix
5. **Credit**: We credit the reporter (unless anonymity requested)

### Disclosure Timeline

- **Standard**: 90 days from report to disclosure
- **Critical**: May be shorter if active exploitation detected
- **Extended**: May be longer for complex issues requiring coordination

### Embargo

During the embargo period:
- We do not discuss the vulnerability publicly
- We do not commit fixes with revealing messages
- We coordinate with affected downstream projects

---

## Security Contact

### Primary Contact

**Email**: security@oxidekit.com

### Response Team

The OxideKit Security Response Team includes:
- Core maintainers with security expertise
- On-call rotation for critical issues
- Legal counsel for disclosure coordination

### Escalation

If you don't receive a response within 48 hours:
- Resend to security@oxidekit.com
- Open a GitHub issue (without details) asking for acknowledgment

---

## Recognition

### Security Hall of Fame

We maintain a hall of fame at https://oxidekit.com/security/thanks for researchers who:
- Report valid vulnerabilities
- Follow responsible disclosure
- Help improve OxideKit security

### What We Offer

- **Public credit**: Named recognition in advisories
- **Swag**: OxideKit merchandise for significant findings
- **Bounties**: Monetary rewards for critical vulnerabilities (program details TBD)

---

## Secure Development Guidelines

### For Contributors

1. Never commit secrets or credentials
2. Use `cargo audit` before submitting PRs
3. Follow the principle of least privilege
4. Validate and sanitize all inputs
5. Use safe Rust idioms (avoid `unsafe` unless necessary)

### For Application Developers

1. Request only necessary capabilities
2. Keep dependencies updated
3. Use the OxideKit Attestation Service
4. Implement proper error handling
5. Follow secure coding guidelines

---

## Compliance

### Standards

OxideKit aims to support compliance with:
- SOC 2 Type II
- ISO 27001
- GDPR (data protection)
- HIPAA (healthcare, where applicable)

### Enterprise Features

Enterprise customers have access to:
- Compliance documentation
- Audit logs
- Security questionnaire responses
- Penetration test reports (under NDA)

---

## Updates to This Policy

This policy may be updated. Significant changes will be announced via:
- GitHub releases
- Security mailing list
- Website changelog

---

## Document History

| Date | Version | Changes |
|------|---------|---------|
| 2026-01 | 1.0 | Initial security policy |
