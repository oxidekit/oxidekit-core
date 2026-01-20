# OxideKit Licensing Architecture

> Last Updated: January 2026

This document defines the licensing strategy for the OxideKit ecosystem, providing a repo-by-repo mapping, rationale, and contribution guidelines.

---

## Executive Summary

OxideKit uses a **hybrid dual-license strategy** that balances:
- **Openness**: Enabling developers and enterprises to adopt freely
- **Protection**: Preventing competitive forks from fragmenting the ecosystem
- **Monetization**: Clear separation between open-source and commercial offerings

---

## License Types Used

| License | Purpose | Use Case |
|---------|---------|----------|
| **BSL 1.1** | Source-available with delayed open-source conversion | Core platform components |
| **Apache-2.0** | Permissive open-source | Extensions, SDKs, tooling |
| **MIT/Apache-2.0 Dual** | Maximum permissiveness | Utilities, examples |
| **Commercial EULA** | Proprietary offerings | Premium plugins, enterprise features |
| **Proprietary** | Service terms | Hosted services, attestation |

---

## Repo-by-Repo License Mapping

### Core Platform

| Repository | License | Change Date | Rationale | Contribution |
|------------|---------|-------------|-----------|--------------|
| `oxidekit-core` | BSL 1.1 | Apache-2.0 after 4 years | Protects core investment while ensuring eventual openness | DCO required |
| `oxidekit-runtime` | BSL 1.1 | Apache-2.0 after 4 years | Critical runtime component | DCO required |
| `oxidekit-renderer` | BSL 1.1 | Apache-2.0 after 4 years | Core rendering engine | DCO required |
| `oxidekit-ipc` | BSL 1.1 | Apache-2.0 after 4 years | Inter-process communication layer | DCO required |
| `oxidekit-windowing` | BSL 1.1 | Apache-2.0 after 4 years | Native windowing abstraction | DCO required |

### Extensions & SDKs

| Repository | License | Rationale | Contribution |
|------------|---------|-----------|--------------|
| `oxidekit-extensions` | Apache-2.0 | Encourages ecosystem growth | DCO required |
| `oxidekit-sdk-js` | Apache-2.0 | JavaScript bindings for adoption | DCO required |
| `oxidekit-sdk-python` | Apache-2.0 | Python bindings for adoption | DCO required |
| `oxidekit-sdk-go` | Apache-2.0 | Go bindings for adoption | DCO required |
| `oxidekit-cli` | Apache-2.0 | Developer tooling should be open | DCO required |

### Themes & Design

| Repository | License | Rationale | Contribution |
|------------|---------|-----------|--------------|
| `oxidekit-themes` (community) | Apache-2.0 | Enables community contributions | DCO required |
| `oxidekit-themes-premium` | Commercial EULA | Revenue stream for sustainability | License purchase |
| `oxidekit-design-tokens` | MIT | Maximum reusability | DCO optional |

### Services & Marketplace

| Repository | License | Rationale | Contribution |
|------------|---------|-----------|--------------|
| `oxidekit-market` | Proprietary | Core monetization infrastructure | Not open |
| `oxidekit-attest` | Proprietary | Verification service backend | Not open |
| `oxidekit-registry` | Proprietary | Package registry infrastructure | Not open |
| `oxidekit-cloud` | Proprietary | Hosted services | Not open |

### Documentation & Examples

| Repository | License | Rationale | Contribution |
|------------|---------|-----------|--------------|
| `oxidekit-docs` | CC-BY-4.0 | Open documentation | Open |
| `oxidekit-examples` | MIT/Apache-2.0 | Maximum reusability | Open |
| `oxidekit-templates` | MIT/Apache-2.0 | Starter templates | Open |

---

## BSL 1.1 Details

### What BSL 1.1 Means

The Business Source License 1.1 is a "source-available" license that:
1. **Allows**: Reading, modifying, and using the code
2. **Restricts**: Using the code for production commercial purposes that compete with OxideKit
3. **Converts**: Automatically becomes Apache-2.0 after the Change Date

### Change Date Policy

- **Initial Change Date**: 4 years from each major release
- **Rolling Releases**: Each major version has its own change date
- **Example**: v1.0 released Jan 2026 converts to Apache-2.0 on Jan 2030

### Additional Use Grant

The BSL 1.1 license includes an Additional Use Grant allowing:
- Internal business use for any purpose
- Non-commercial use without restriction
- Development, testing, and evaluation
- Building applications distributed to end users

### What Is NOT Allowed

Without a commercial license:
- Offering OxideKit as a hosted/managed service
- Selling a product that embeds OxideKit as its primary value
- Creating a fork marketed as an alternative to OxideKit

---

## Apache-2.0 Details

### Why Apache-2.0 for Extensions

- **Patent protection**: Explicit patent grants protect contributors and users
- **Attribution**: Requires acknowledgment but not copyleft
- **Compatibility**: Works with most enterprise requirements
- **Ecosystem**: Encourages commercial plugins and integrations

### NOTICE File Requirements

All Apache-2.0 repositories must include:
1. `LICENSE` file with full Apache-2.0 text
2. `NOTICE` file acknowledging contributors and dependencies
3. SPDX headers in all source files

---

## Commercial Licensing

### When Commercial License Is Required

A commercial license is required when:
1. Offering OxideKit as a hosted platform service
2. Bundling OxideKit in a commercial product as core functionality
3. Distributing modified versions without source disclosure
4. Requiring extended support, SLAs, or indemnification

### Commercial License Tiers

| Tier | Target | Includes |
|------|--------|----------|
| **Startup** | <$5M ARR | BSL use rights, community support |
| **Business** | <$50M ARR | BSL use rights, email support, quarterly updates |
| **Enterprise** | >$50M ARR | Full rights, dedicated support, SLA, indemnification |

### Contact

Commercial licensing inquiries: licensing@oxidekit.com

---

## Contribution Requirements

### Developer Certificate of Origin (DCO)

All contributions to OxideKit repositories require DCO sign-off:

```
Signed-off-by: Your Name <your.email@example.com>
```

This certifies that:
1. You have the right to submit the contribution
2. You agree to the project's licensing terms
3. The contribution may be relicensed under the project's terms

### Why DCO Over CLA

- **Simpler**: No legal document to sign
- **Per-commit**: Each commit is individually certified
- **Git-native**: Uses git's existing sign-off mechanism
- **Reversible**: Contributors retain their copyright

### SPDX Headers

All source files should include SPDX license identifiers:

```rust
// SPDX-License-Identifier: BSL-1.1
// Copyright (c) 2026 OxideKit Contributors
```

---

## License Compatibility

### Dependency Guidelines

| OxideKit License | Compatible Dependencies |
|------------------|------------------------|
| BSL 1.1 | MIT, Apache-2.0, BSD, ISC |
| Apache-2.0 | MIT, Apache-2.0, BSD, ISC |
| Commercial | Any (within license terms) |

### Incompatible Licenses

The following licenses are NOT compatible with OxideKit core:
- GPL/LGPL (copyleft requirements)
- AGPL (network copyleft)
- SSPL (service copyleft)
- CC-BY-NC (non-commercial restriction)

---

## Enforcement Philosophy

### Light But Real

OxideKit's licensing enforcement:
1. **Education first**: Contact before legal action
2. **Clear violations only**: No ambiguous enforcement
3. **Community protection**: Prevent brand confusion
4. **Good faith assumed**: Work with unintentional violations

### Reporting Violations

Report suspected license violations to: legal@oxidekit.com

---

## FAQ

### Can I use OxideKit in my company's internal tools?
Yes. Internal business use is always permitted under the BSL 1.1 Additional Use Grant.

### Can I build and sell applications made with OxideKit?
Yes. Applications that use OxideKit as a framework (not as the primary product) are permitted.

### Can I offer OxideKit hosting as a service?
Not without a commercial license. This is the primary restricted use case.

### When will OxideKit become fully open source?
Each major version converts to Apache-2.0 four years after release.

### Can I fork OxideKit?
Yes, but you must:
1. Rename the fork (see TRADEMARK.md)
2. Comply with the BSL 1.1 terms
3. Not imply official OxideKit endorsement

---

## Document History

| Date | Version | Changes |
|------|---------|---------|
| 2026-01 | 1.0 | Initial licensing architecture |
