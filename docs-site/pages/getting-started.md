# Getting Started with OxideKit

Build native desktop applications with Rust. No browser, no JavaScript runtime, no bundlers.

## What is OxideKit?

OxideKit is a **Rust-native application platform** designed to replace Electron, Tauri, and traditional web stacks. It provides a complete solution for building desktop applications with native performance.

### Key Advantages

| Feature | Electron | Tauri | OxideKit |
|---------|----------|-------|----------|
| Runtime | Chromium | WebView | Native GPU |
| Memory | 150MB+ | 50MB+ | 10-20MB |
| Startup | 2-5s | 1-2s | <500ms |
| Binary Size | 150MB+ | 10MB+ | 5-15MB |
| Language | JS/TS | JS/TS + Rust | OUI + Rust |

## Quick Example

Create a simple hello world application:

```oui
// ui/app.oui
app HelloWorld {
    Column {
        align: center
        justify: center
        width: fill
        height: fill
        background: "#030712"

        Text {
            content: "Hello OxideKit!"
            size: 48
            color: "#FFFFFF"
        }

        Button {
            label: "Get Started"
            variant: "primary"
            on_click: navigate("/docs")
        }
    }
}
```

## Core Features

### 1. Native Performance
GPU-accelerated rendering with wgpu. No browser overhead, no JavaScript runtime.

### 2. Hot Reload
State-preserving hot reload for rapid iteration. Edit your UI and see changes instantly.

### 3. Secure by Default
Permission system and network allowlists. UI code can't access privileged APIs without explicit permission.

### 4. Cross-Platform
Build once, deploy to macOS, Windows, Linux, and static HTML from a single codebase.

### 5. Design Tokens
Built-in token system for consistent, themeable styling across your application.

## Installation

Install the OxideKit CLI:

```bash
# Using Cargo
cargo install oxide-cli

# Verify installation
oxide --version
```

Create a new project:

```bash
oxide new my-app
cd my-app
oxide dev
```

## Project Structure

```
my-app/
  oxide.toml        # Project configuration
  ui/
    app.oui         # Main application file
    components/     # Reusable components
  assets/           # Images, fonts, etc.
  src/              # Rust backend code (optional)
```

## Next Steps

- [Installation Guide](/docs/installation) - Detailed setup instructions
- [Quick Start Tutorial](/docs/quick-start) - Build your first app
- [Components Reference](/docs/components) - Learn the UI components
- [Examples](/docs/examples) - See complete applications
