# Contributing to OxideKit

Thank you for your interest in contributing to OxideKit! This document outlines how to contribute, our requirements, and the rationale behind our choices.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How to Contribute](#how-to-contribute)
- [Developer Certificate of Origin (DCO)](#developer-certificate-of-origin-dco)
- [Why DCO Over CLA](#why-dco-over-cla)
- [Contribution Workflow](#contribution-workflow)
- [Coding Standards](#coding-standards)
- [Review Process](#review-process)
- [Recognition](#recognition)

---

## Code of Conduct

OxideKit follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). We are committed to providing a welcoming and inclusive environment for everyone.

**Be respectful. Be constructive. Be collaborative.**

---

## How to Contribute

### Ways to Contribute

1. **Code**: Bug fixes, features, performance improvements
2. **Documentation**: Tutorials, guides, API docs
3. **Testing**: Test cases, fuzzing, benchmarks
4. **Design**: UI/UX improvements, accessibility
5. **Community**: Answering questions, reviewing PRs
6. **Translation**: Localizing documentation

### Finding Work

- **Good first issues**: [github.com/oxidekit/oxidekit-core/labels/good-first-issue](https://github.com/oxidekit/oxidekit-core/labels/good-first-issue)
- **Help wanted**: [github.com/oxidekit/oxidekit-core/labels/help-wanted](https://github.com/oxidekit/oxidekit-core/labels/help-wanted)
- **Roadmap items**: Check our public roadmap for larger initiatives

### Before You Start

1. Check if an issue already exists
2. For significant changes, open an issue first to discuss
3. Read the relevant documentation
4. Set up your development environment

---

## Developer Certificate of Origin (DCO)

### Requirement

All contributions to OxideKit require DCO sign-off. This is a lightweight alternative to a Contributor License Agreement (CLA).

### What It Means

By signing off on a commit, you certify that:

```
Developer Certificate of Origin
Version 1.1

Copyright (C) 2004, 2006 The Linux Foundation and its contributors.

Everyone is permitted to copy and distribute verbatim copies of this
license document, but changing it is not allowed.

Developer's Certificate of Origin 1.1

By making a contribution to this project, I certify that:

(a) The contribution was created in whole or in part by me and I
    have the right to submit it under the open source license
    indicated in the file; or

(b) The contribution is based upon previous work that, to the best
    of my knowledge, is covered under an appropriate open source
    license and I have the right under that license to submit that
    work with modifications, whether created in whole or in part
    by me, under the same open source license (unless I am
    permitted to submit under a different license), as indicated
    in the file; or

(c) The contribution was provided directly to me by some other
    person who certified (a), (b) or (c) and I have not modified
    it.

(d) I understand and agree that this project and the contribution
    are public and that a record of the contribution (including all
    personal information I submit with it, including my sign-off) is
    maintained indefinitely and may be redistributed consistent with
    this project or the open source license(s) involved.
```

### How to Sign Off

Add a `Signed-off-by` line to your commit message:

```
git commit -s -m "Your commit message"
```

This adds:
```
Signed-off-by: Your Name <your.email@example.com>
```

### Configuring Git

Set your name and email:

```bash
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"
```

### Fixing Missing Sign-off

If you forgot to sign off:

```bash
# For the most recent commit
git commit --amend -s

# For multiple commits
git rebase --signoff HEAD~N  # where N is the number of commits
```

---

## Why DCO Over CLA

### Our Decision

OxideKit uses the **Developer Certificate of Origin (DCO)** instead of a Contributor License Agreement (CLA). Here's why:

### Advantages of DCO

| Factor | DCO | CLA |
|--------|-----|-----|
| **Simplicity** | Per-commit sign-off | Legal document to sign |
| **Friction** | Minimal | May require legal review |
| **Reversibility** | Contributors retain copyright | Often assigns copyright |
| **Corporate acceptance** | Easy to approve | May require legal approval |
| **Transparency** | Git-native, auditable | Separate tracking system |

### Rationale

1. **Lower barrier to contribution**: No legal document means faster first contributions
2. **Corporate-friendly**: Many companies pre-approve DCO but require legal review for CLAs
3. **Linux kernel precedent**: Proven to scale to thousands of contributors
4. **Copyright retention**: Contributors keep their copyright while granting license
5. **Git-native**: Built into git's sign-off feature

### Trade-offs Acknowledged

- **Relicensing difficulty**: Without copyright assignment, relicensing requires contributor consent
- **Less explicit grants**: CLA can include explicit patent grants

### Mitigation

- BSL 1.1 to Apache-2.0 conversion is pre-defined in the license
- Apache-2.0 includes explicit patent grants
- Major changes will be announced with opportunity to object

---

## Contribution Workflow

### Step 1: Fork and Clone

```bash
git clone https://github.com/YOUR-USERNAME/oxidekit-core.git
cd oxidekit-core
git remote add upstream https://github.com/oxidekit/oxidekit-core.git
```

### Step 2: Create a Branch

```bash
git checkout -b feature/your-feature-name
```

Branch naming conventions:
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `test/` - Test additions/improvements

### Step 3: Make Changes

1. Write your code
2. Add tests for new functionality
3. Update documentation as needed
4. Run the test suite locally

```bash
cargo test
cargo clippy
cargo fmt --check
```

### Step 4: Commit with Sign-off

```bash
git add .
git commit -s -m "feat: add new widget rendering"
```

Commit message format:
```
<type>: <description>

[optional body]

[optional footer]

Signed-off-by: Your Name <email>
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

### Step 5: Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then open a Pull Request on GitHub.

### Step 6: Address Review Feedback

- Respond to comments
- Make requested changes
- Push additional commits (don't force push during review)

---

## Coding Standards

### Rust Style

- Follow `rustfmt` defaults
- Run `cargo clippy` with no warnings
- Use descriptive variable names
- Prefer explicit types in public APIs

### Documentation

- All public items must have doc comments
- Include examples in doc comments
- Use `///` for public docs, `//` for implementation comments

### Testing

- Write tests for all new functionality
- Maintain test coverage above 80%
- Include both unit and integration tests
- Test error cases, not just happy paths

### Commit Hygiene

- Each commit should be atomic and buildable
- Squash WIP commits before PR
- Use meaningful commit messages
- Reference issues where applicable

---

## Review Process

### What We Look For

1. **Correctness**: Does it work as intended?
2. **Tests**: Are there adequate tests?
3. **Documentation**: Is it documented?
4. **Style**: Does it follow our conventions?
5. **Performance**: Any performance implications?
6. **Security**: Any security implications?

### Review Timeline

- Initial response: Within 48 hours
- Full review: Within 1 week for small PRs
- Larger PRs may take longer

### Approval Requirements

- At least 1 maintainer approval
- CI passing (tests, lints, formatting)
- DCO sign-off verified
- No unresolved conversations

### After Merge

- Delete your feature branch
- Update your fork's main branch
- Celebrate your contribution!

---

## Recognition

### Contributors

All contributors are recognized in:
- CONTRIBUTORS file
- GitHub contributors page
- Release notes (for significant contributions)

### Maintainers

Active contributors may be invited to become maintainers, with:
- Merge permissions
- Roadmap input
- Recognition as a project maintainer

---

## Getting Help

- **Discord**: [discord.gg/oxidekit](https://discord.gg/oxidekit)
- **GitHub Discussions**: Ask questions and share ideas
- **Office Hours**: Weekly maintainer office hours (schedule on Discord)

---

## License

By contributing to OxideKit, you agree that your contributions will be licensed under the project's license (see [docs/licensing.md](docs/licensing.md)).

---

## Document History

| Date | Version | Changes |
|------|---------|---------|
| 2026-01 | 1.0 | Initial contributing guidelines |
