# OxideKit Core

A Rust-native application platform for building desktop applications without shipping a browser.

## Overview

OxideKit is a complete replacement for Electron/Tauri + JavaScript/TypeScript stacks. It provides:

- **Native Runtime** - Windowing, input, accessibility, GPU rendering
- **Declarative UI** - Components, layout, styling, animation via `.oui` files
- **Hot Reload** - State-preserving hot reload for rapid iteration
- **Extension System** - Capability-based plugins with sandboxing
- **Theme Ecosystem** - Design tokens, themes, and UI kits

## Quick Start

```bash
# Install the CLI
cargo install oxide-cli

# Create a new project
oxide new my-app
cd my-app

# Run in development mode
oxide dev
```

## Project Structure

```
oxidekit-core/
├── crates/
│   ├── oxide-cli/        # CLI tool (oxide)
│   ├── oxide-compiler/   # .oui -> IR compiler
│   ├── oxide-runtime/    # Window, input, lifecycle
│   ├── oxide-layout/     # Flexbox layout engine
│   ├── oxide-text/       # Text shaping and rendering
│   ├── oxide-render/     # GPU renderer
│   └── oxide-devtools/   # Inspector, profiler
├── docs/
│   ├── rfcs/             # RFC process
│   └── specs/            # Format specifications
├── examples/
│   └── hello-oxidekit/   # Demo application
└── site/                 # Documentation site
```

## Documentation

- [Getting Started](https://oxidekit.com/docs/getting-started)
- [UI Language Guide](https://oxidekit.com/docs/guide/ui-language)
- [CLI Reference](https://oxidekit.com/docs/reference/cli)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.
