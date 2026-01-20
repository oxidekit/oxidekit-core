# OxideKit Guides

Welcome to the OxideKit learning resources. These guides teach you the mental model and practical skills needed to build applications with OxideKit.

## Learning Paths

### New to OxideKit?

Start here:

1. **[Quick Start](./05-quick-start.md)** - Build your first app in 5 minutes
2. **[Core Concepts](./01-core-concepts.md)** - Understand the OxideKit mental model

### Understanding the System

Deep dives into how OxideKit works:

3. **[Plugin Categories](./02-plugin-categories.md)** - UI, Service, Native, and Tooling plugins
4. **[Build Modes](./03-build-modes.md)** - Dev vs Release vs Diagnostics
5. **[AI Philosophy](./04-ai-philosophy.md)** - How AI assists without taking over

### Interactive Learning

```bash
oxide learn
```

The `oxide learn` command provides interactive, step-by-step tutorials in your terminal.

## Guide Overview

| Guide | What You'll Learn | Time |
|-------|-------------------|------|
| [Quick Start](./05-quick-start.md) | Get from zero to running app | 5 min |
| [Core Concepts](./01-core-concepts.md) | Tokens, Components, Packs, Design, Starters | 15 min |
| [Plugin Categories](./02-plugin-categories.md) | How plugins are organized and trusted | 10 min |
| [Build Modes](./03-build-modes.md) | Development, Release, and Diagnostics builds | 10 min |
| [AI Philosophy](./04-ai-philosophy.md) | AI-native design and validation | 10 min |

## Visual Diagrams

The `diagrams/` directory contains visual aids:

- **[architecture.svg](./diagrams/architecture.svg)** - Overall OxideKit architecture
- **[component-flow.svg](./diagrams/component-flow.svg)** - Data flow through components
- **[plugin-trust.svg](./diagrams/plugin-trust.svg)** - Plugin trust and permission model

## Key Concepts at a Glance

### The Conceptual Hierarchy

```
Tokens -> Components -> Packs -> Design -> Starters
```

Each layer builds on the previous one:

- **Tokens**: Design primitives (`$color.primary`, `$spacing.md`)
- **Components**: UI building blocks (`Button`, `Card`)
- **Packs**: Bundled functionality (`ui.tables`)
- **Design**: Visual systems (themes, typography)
- **Starters**: Project templates (`admin-panel`)

### Plugin Categories

| Category | Permissions | Examples |
|----------|-------------|----------|
| UI | None | `ui.tables`, `ui.charts` |
| Service | App-level | `service.auth`, `service.db` |
| Native | OS access | `native.fs`, `native.keychain` |
| Tooling | Dev-only | `tool.codegen`, `tool.lint` |

### Build Modes

| Mode | Command | Purpose |
|------|---------|---------|
| Dev | `oxide dev` | Hot reload, dev tools |
| Release | `oxide build --release` | Production, optimized |
| Diagnostics | `oxide build --release --features diagnostics` | Production with support |

### AI Integration

OxideKit is AI-friendly but human-controlled:

- Machine-readable component specs (`oxide.ai.json`)
- Validation prevents hallucination
- Recipes guide code generation
- Human always reviews and owns

## CLI Quick Reference

```bash
# Development
oxide dev                              # Start dev server
oxide build                            # Debug build

# Production
oxide build --release                  # Release build

# Learning
oxide learn                            # Interactive tutorials
oxide learn core-concepts              # Specific topic

# Project Setup
oxide new my-app                       # Create project
oxide new my-app --starter admin-panel # With starter

# Plugins
oxide add ui.tables                    # Add plugin
oxide starters list                    # List starters

# Diagnostics
oxide doctor                           # Check project health
oxide diagnostics export               # Export diagnostics
```

## Related Resources

- **[OxideKit Specifications](../specs/)** - Technical specifications
- **[RFC Process](../rfcs/)** - How changes are proposed
- **CLI Help**: `oxide --help`

## Getting Help

- Run `oxide learn` for interactive tutorials
- Run `oxide doctor` to diagnose issues
- Check the specifications in `docs/specs/`
- File issues on GitHub

---

Happy building with OxideKit!
