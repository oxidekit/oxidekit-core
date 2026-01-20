# OxideKit RFC Process

This document describes the RFC (Request for Comments) process for OxideKit.

## What is an RFC?

RFCs are design documents that describe proposed changes to OxideKit. They provide a consistent and controlled path for new features and changes to enter the platform.

## When to Write an RFC

RFCs are **required** for:

- New CLI commands or subcommands
- Changes to manifest formats (`oxide.toml`, `extensions.lock`)
- New crates or major crate restructuring
- UI language (`.oui`) syntax changes
- Extension API changes
- Breaking changes to public APIs
- Security-sensitive features

RFCs are **NOT required** for:

- Bug fixes
- Documentation improvements
- Internal refactoring that doesn't affect public APIs
- Performance optimizations (unless they change APIs)
- Minor feature additions with obvious design

## Before Writing an RFC

1. **Check existing RFCs** - Your idea might already be proposed or implemented
2. **Open a discussion** - Use GitHub Discussions to gauge interest and get early feedback
3. **Gather requirements** - Understand the problem space thoroughly

## RFC Process

### 1. Fork & Copy

```bash
# Fork the repository, then:
git clone https://github.com/YOUR_USERNAME/oxidekit-core.git
cd oxidekit-core
cp docs/rfcs/0000-template.md docs/rfcs/0000-my-feature.md
```

### 2. Write the RFC

Fill in all sections of the template:

- **Summary** - One paragraph overview
- **Motivation** - Why is this needed?
- **Guide-level explanation** - How would you teach this?
- **Reference-level explanation** - Technical details
- **Drawbacks** - Why should we NOT do this?
- **Rationale and alternatives** - Why this design?
- **Prior art** - How do others solve this?
- **Unresolved questions** - What needs more discussion?

### 3. Submit Pull Request

```bash
git checkout -b rfc/my-feature
git add docs/rfcs/0000-my-feature.md
git commit -m "RFC: My feature proposal"
git push origin rfc/my-feature
```

Open a pull request with:
- Title: `RFC: <feature name>`
- Description: Brief summary and motivation

### 4. Discussion Period

- Minimum 10 calendar days for discussion
- Address feedback by updating the RFC
- Major changes restart the discussion period

### 5. Final Comment Period (FCP)

When discussion has settled:
- Core team member proposes FCP
- 7 calendar days for final comments
- Must have at least 2 core team approvals

### 6. Decision

After FCP, the core team decides:

- **Accept** - RFC is approved
  - Assign RFC number
  - Merge the PR
  - Create tracking issue
- **Reject** - RFC is not accepted
  - Document reasons
  - Close the PR
- **Postpone** - Good idea, but not now
  - Add `postponed` label
  - Keep PR open

### 7. Implementation

After acceptance:
1. Tracking issue is created
2. Anyone can submit implementation PRs
3. Implementation may reveal issues requiring RFC amendments

## RFC Lifecycle States

| State | Description |
|-------|-------------|
| **Draft** | Initial submission, open for feedback |
| **Active** | Under serious consideration |
| **FCP** | Final Comment Period |
| **Accepted** | Approved, awaiting implementation |
| **Implemented** | Feature shipped |
| **Rejected** | Not accepted |
| **Postponed** | Good idea, wrong time |
| **Withdrawn** | Author withdrew the RFC |

## RFC Numbering

- Accepted RFCs are numbered sequentially: 0001, 0002, ...
- Numbers are assigned when entering FCP
- Draft RFCs use 0000

## File Organization

```
docs/rfcs/
├── README.md           # This file
├── 0000-template.md    # RFC template
└── text/               # Accepted RFCs
    ├── 0001-manifest-format.md
    ├── 0002-extension-api.md
    └── ...
```

## Questions?

- Open a GitHub Discussion for meta-discussion about the RFC process
- Tag issues with `rfc-process` for process-related questions
