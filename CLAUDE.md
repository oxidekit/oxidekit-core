# CLAUDE.md

Guidelines for working with OxideKit codebase.

## Project Overview

OxideKit is a Rust-native application platform for building fast, secure, and portable desktop and mobile applications.

## Build Commands

```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Build specific crate
cargo build -p oxide-compiler

# Run with features
cargo build -p oxide-plugins --features full
```

## Commit Convention

Use conventional commit format for all commits:

| Prefix | Purpose |
|--------|---------|
| `feat:` | New feature |
| `fix:` | Bug fix |
| `docs:` | Documentation changes |
| `style:` | Formatting, no code change |
| `refactor:` | Code restructuring, no behavior change |
| `perf:` | Performance improvement |
| `test:` | Adding or updating tests |
| `chore:` | Build, CI, dependencies |
| `revert:` | Reverting a previous commit |

### Examples

```
feat: add --yes flag to oxide new command
fix: resolve theme tokens in static build
docs: update installation guide
style: format oxide-compiler with rustfmt
refactor: simplify parser state machine
perf: cache layout computations
test: add visual regression tests for buttons
chore: update wasmtime to 36.0
```

### Scope (Optional)

Add scope in parentheses for specificity:

```
feat(cli): add --yes flag to oxide new
fix(compiler): handle component imports
chore(deps): update wasmtime to 36.0
```

## Crate Structure

| Crate | Purpose |
|-------|---------|
| oxide-cli | Command-line interface |
| oxide-compiler | OUI DSL parser and compiler |
| oxide-components | UI components and design tokens |
| oxide-runtime | Application runtime |
| oxide-render | GPU rendering with wgpu |
| oxide-layout | Flexbox layout with taffy |
| oxide-text | Text rendering with cosmic-text |
| oxide-plugins | Plugin system with WASM sandbox |
| oxide-state | State management and persistence |

## Testing

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p oxide-compiler

# Run with output
cargo test -- --nocapture
```

## Key Files

| File | Purpose |
|------|---------|
| `Cargo.toml` | Workspace manifest |
| `crates/*/Cargo.toml` | Crate manifests |
| `.github/workflows/ci.yml` | CI pipeline |
| `.github/workflows/release.yml` | Release automation |
