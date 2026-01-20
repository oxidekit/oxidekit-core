# OxideKit Enterprise Offering

> Last Updated: January 2026

This document defines OxideKit's enterprise products and services, including support tiers, LTS releases, and the attestation service.

---

## Executive Summary

OxideKit offers enterprise customers:
- **Support packages** with SLAs and dedicated assistance
- **LTS releases** with extended maintenance windows
- **Attestation Service** for verified builds and compliance
- **Premium plugins** for enterprise-specific requirements
- **Professional services** for migration and custom development

---

## Support Packages

### Tier Overview

| Tier | Target | Annual Price | Response SLA | Features |
|------|--------|--------------|--------------|----------|
| **Community** | Individuals, startups | Free | Best effort | GitHub issues, Discord |
| **Standard** | Small teams | $5,000/year | 48 hours | Email support, quarterly reviews |
| **Premium** | Mid-market | $25,000/year | 4 hours | Priority support, dedicated contact |
| **Enterprise** | Large organizations | Custom | 1 hour | 24/7 support, on-call escalation |

### Community (Free)

**Included**:
- GitHub issue tracking
- Community Discord access
- Public documentation
- Community-contributed answers

**Not Included**:
- Guaranteed response times
- Private support channels
- Direct maintainer access

### Standard Support - $5,000/year

**Included**:
- Email support (business hours)
- 48-hour response SLA
- Quarterly support reviews
- Access to support knowledge base
- Priority issue triage
- 10 support tickets per month

**Best For**:
- Small teams (1-20 developers)
- Projects entering production
- Teams needing reliable issue resolution

### Premium Support - $25,000/year

**Included**:
- Everything in Standard, plus:
- 4-hour response SLA (business hours)
- 24-hour response SLA (after hours)
- Dedicated support contact
- Monthly architecture reviews
- Slack/Teams channel access
- Unlimited support tickets
- Early access to releases
- Roadmap preview and input

**Best For**:
- Mid-market companies (20-200 developers)
- Mission-critical applications
- Teams needing proactive guidance

### Enterprise Support - Custom Pricing

**Included**:
- Everything in Premium, plus:
- 1-hour response SLA (24/7)
- On-call escalation path
- Dedicated success manager
- Custom integration assistance
- Private training sessions
- Compliance documentation
- Executive business reviews
- Custom SLA terms

**Best For**:
- Large enterprises (200+ developers)
- Regulated industries
- Global deployments

---

## Long-Term Support (LTS) Releases

### LTS Policy

OxideKit provides Long-Term Support releases for enterprises requiring stability.

| Release Type | Support Duration | Update Frequency | Target Users |
|-------------|------------------|------------------|--------------|
| **Standard** | 12 months | Monthly | Most users |
| **LTS** | 36 months | Security only | Enterprises |
| **Extended LTS** | 60 months | Critical only | Regulated industries |

### LTS Release Schedule

| Version | Release Date | End of Support | Extended End |
|---------|--------------|----------------|--------------|
| 1.0 LTS | Q1 2026 | Q1 2029 | Q1 2031 |
| 2.0 LTS | Q1 2028 | Q1 2031 | Q1 2033 |
| (future) | Every 2 years | +3 years | +5 years |

### What LTS Includes

**Standard LTS (36 months)**:
- Security patches (critical and high)
- Bug fixes for severe issues
- Compatibility updates for OS/platform changes
- No breaking API changes

**Extended LTS (60 months)** - Additional fee:
- All Standard LTS features
- Extended security patch backporting
- Priority patch scheduling
- Custom build options
- Compliance certifications maintained

### LTS Pricing

| Package | Annual Fee | Notes |
|---------|------------|-------|
| LTS Access | Included with Standard+ support | Access to LTS releases |
| Extended LTS | +$10,000/year | 5-year support window |
| Custom LTS | Custom | Custom version/schedule |

---

## Verified Builds & Attestation Service

### Overview

The OxideKit Attestation Service provides cryptographic verification of application builds, capabilities, and security posture.

### What It Does

1. **Binary Scanning**: Analyzes compiled applications for known vulnerabilities
2. **SBOM Generation**: Creates Software Bill of Materials
3. **Capability Verification**: Validates declared vs. actual capabilities
4. **Network Analysis**: Verifies network access patterns
5. **Signature Generation**: Creates signed attestation documents
6. **Badge Issuance**: Issues "Verified by OxideKit" badge

### Service Tiers

| Tier | Target | Price | Features |
|------|--------|-------|----------|
| **Community** | OSS projects | Free | Basic local CLI tool |
| **Developer** | Individual developers | $29/month | 50 attestations/month, CI integration |
| **Team** | Development teams | $199/month | 500 attestations/month, team dashboard |
| **Enterprise** | Large organizations | Custom | Unlimited, self-hosted option |

### Community (Free)

**Included**:
- Local CLI attestation tool
- Basic capability analysis
- Self-signed attestations
- Community support

**Limitations**:
- No "Verified by OxideKit" badge
- No CI/CD integration
- No compliance reports

### Developer - $29/month

**Included**:
- 50 attestations per month
- CI/CD integration (GitHub Actions, GitLab CI)
- "Verified by OxideKit" badge
- Basic compliance reports
- API access

**Best For**:
- Independent developers
- Side projects going to production
- Open source maintainers

### Team - $199/month

**Included**:
- 500 attestations per month
- Everything in Developer, plus:
- Team dashboard
- Role-based access control
- Audit logs
- SOC 2 compliance reports
- Slack/email notifications
- Priority scanning queue

**Best For**:
- Development teams (5-25 developers)
- Companies with multiple products
- B2B software vendors

### Enterprise - Custom Pricing

**Included**:
- Unlimited attestations
- Everything in Team, plus:
- Self-hosted deployment option
- Custom compliance frameworks
- Advanced audit logging
- SSO/SAML integration
- Dedicated infrastructure
- Custom integration support
- SLA guarantees

**Best For**:
- Large enterprises
- Regulated industries
- Air-gapped environments

---

## Attestation Output

### What You Get

Each attestation produces:

```json
{
  "attestation": {
    "id": "attest-xxxx-xxxx-xxxx",
    "timestamp": "2026-01-15T10:30:00Z",
    "version": "1.0",
    "subject": {
      "name": "my-application",
      "version": "2.1.0",
      "hash": "sha256:abc123..."
    },
    "verification": {
      "capabilities": {
        "declared": ["network:https", "filesystem:read"],
        "verified": ["network:https", "filesystem:read"],
        "status": "MATCH"
      },
      "security": {
        "vulnerabilities": {
          "critical": 0,
          "high": 0,
          "medium": 2,
          "low": 5
        },
        "malware": "CLEAN",
        "status": "PASS"
      },
      "sbom": {
        "format": "CycloneDX",
        "components": 127,
        "licenses": ["Apache-2.0", "MIT"]
      }
    },
    "badge": {
      "eligible": true,
      "url": "https://verify.oxidekit.com/attest-xxxx",
      "expires": "2026-04-15T10:30:00Z"
    }
  },
  "signature": "-----BEGIN SIGNATURE-----..."
}
```

### Badge Display

Verified applications can display:

```
[Verified by OxideKit]
ID: attest-xxxx
Verify: oxidekit.com/v/attest-xxxx
```

---

## Premium Enterprise Plugins

### Available Plugins

| Plugin | Purpose | Included In |
|--------|---------|-------------|
| **SAML/SSO** | Enterprise identity integration | Enterprise support |
| **SCIM** | User provisioning | Enterprise support |
| **Advanced Audit** | Detailed audit logging | Premium+, Team+ attestation |
| **Observability Export** | DataDog, Splunk, etc. integration | Premium+, add-on |
| **Policy Engine** | Advanced capability policies | Enterprise, add-on |
| **Release Automation** | Enterprise release workflow | Enterprise, add-on |

### SAML/SSO Plugin

**Features**:
- SAML 2.0 support
- OIDC support
- Integration with major IdPs (Okta, Azure AD, etc.)
- Just-in-time provisioning
- Group mapping

**Pricing**: Included with Enterprise support

### SCIM Plugin

**Features**:
- SCIM 2.0 support
- Automatic user provisioning/deprovisioning
- Group synchronization
- Audit trail

**Pricing**: Included with Enterprise support

### Advanced Observability

**Features**:
- OpenTelemetry export
- Custom metric definitions
- Application performance monitoring
- Integration with DataDog, New Relic, Splunk

**Pricing**: $500/month add-on or included with Enterprise

### Policy Engine

**Features**:
- Custom capability policies
- Organization-wide rules
- Exception workflows
- Compliance reporting

**Pricing**: $1,000/month add-on or included with Enterprise

---

## Professional Services

### Migration Assistance

| Service | Description | Price |
|---------|-------------|-------|
| **Assessment** | Evaluate current stack, migration plan | $5,000 |
| **Pilot Migration** | Migrate one application end-to-end | $15,000 |
| **Full Migration** | Complete migration assistance | Custom |

### Custom Development

| Service | Description | Price |
|---------|-------------|-------|
| **Plugin Development** | Custom plugin for your needs | $10,000+ |
| **Integration Development** | Connect OxideKit to your systems | $15,000+ |
| **Feature Sponsorship** | Fund roadmap features | Custom |

### Training

| Service | Description | Price |
|---------|-------------|-------|
| **Developer Workshop** | 1-day hands-on training | $5,000 |
| **Admin Training** | Deployment and operations | $3,000 |
| **Custom Training** | Tailored to your needs | Custom |

---

## Why Trust OxideKit for Enterprise?

### License Clarity

- Clear BSL 1.1 with defined conversion to Apache-2.0
- Enterprise use explicitly permitted
- No surprise license changes
- Commercial licenses available

### Support Commitment

- SLAs with financial backing
- Dedicated support channels
- Long-term maintenance guaranteed
- Roadmap visibility

### Verification & Auditability

- Attestation service for security verification
- SBOM generation for compliance
- Reproducible builds
- Signed releases

### Security Posture

- Capability-based security model
- Regular security audits
- Responsible disclosure program
- CVE tracking and rapid patching

### Compatibility Guarantees

- Semantic versioning strictly followed
- LTS releases for stability
- Migration guides for major versions
- Backward compatibility commitment

---

## Getting Started

### Contact Sales

**Email**: enterprise@oxidekit.com
**Phone**: +1 (555) OXK-ENTP

### Request a Demo

Schedule a personalized demo: https://oxidekit.com/enterprise/demo

### Proof of Concept

We offer 30-day POC programs for enterprise evaluation.

---

## Document History

| Date | Version | Changes |
|------|---------|---------|
| 2026-01 | 1.0 | Initial enterprise offering |
