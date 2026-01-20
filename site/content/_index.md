+++
title = "OxideKit"
description = "A Rust-native application platform"
template = "index.html"
+++

# Build Native Apps Without the Browser

OxideKit is a complete platform for building desktop applications in Rust. No Electron. No JavaScript. No bundlers.

## Why OxideKit?

- **Native Performance** — No embedded browser, no memory bloat
- **Instant Hot Reload** — State-preserving reloads that feel instant
- **Declarative UI** — Clean `.oui` syntax compiles to native widgets
- **Extension Ecosystem** — Add capabilities with `oxide add`
- **Theme Marketplace** — Professional UI kits ready to use

## Quick Start

```bash
# Install the CLI
cargo install oxide-cli

# Create a new project
oxide new my-app
cd my-app

# Start developing
oxide dev
```

## Sample Application

```oui
app MyApp {
    Column {
        align: center
        justify: center

        Text {
            content: "Hello OxideKit!"
            size: 48
        }

        Button {
            label: "Get Started"
            on click => app.navigate("/docs")
        }
    }
}
```

## Designed for Real Apps

OxideKit isn't a toy. It's built for production applications:

- **Wallets** — Secure, native crypto wallets
- **Dashboards** — Real-time monitoring and analytics
- **Productivity** — Note-taking, task management, editors
- **Enterprise** — Internal tools and admin panels

## Get Involved

- [Read the Docs](/docs/)
- [GitHub Repository](https://github.com/oxidekit/oxidekit-core)
- [Join Discord](https://discord.gg/oxidekit)
