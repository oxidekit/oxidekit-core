# Building Native Desktop Apps

OxideKit builds true native desktop applications with GPU-accelerated rendering. Your app runs directly on the metal, without a browser or webview.

## Quick Start

```bash
# Development with hot reload
oxide dev

# Build debug binary
oxide build

# Build release binary
oxide build --release

# Run the app
oxide run
```

## Platform Support

| Platform | Status | Min Version |
|----------|--------|-------------|
| macOS | Stable | 10.15+ |
| Windows | Stable | Windows 10+ |
| Linux | Stable | Ubuntu 20.04+ |

## Development Mode

Start the dev server with hot reload:

```bash
# Basic dev mode
oxide dev

# With inspector panel
oxide dev --inspector

# Specific theme
oxide dev --theme light

# Custom port
oxide dev --port 3001
```

Development features:
- **Hot Reload**: UI changes apply instantly without restart
- **State Preservation**: State persists across reloads
- **Inspector**: Debug panel for component tree, state, and performance
- **Error Overlay**: Compile errors shown in-app

## Building

### Debug Build

```bash
oxide build

# Output: build/debug/my-app (or target/debug/my-app)
```

Debug builds include:
- Debug symbols for stack traces
- Verbose logging
- Development assertions
- Faster compile times

### Release Build

```bash
oxide build --release

# Output: build/release/my-app
```

Release optimizations:
- Full compiler optimizations
- Link-time optimization (LTO)
- Dead code elimination
- Stripped debug symbols
- Smaller binary size

## Configuration

Configure builds in `oxide.toml`:

```toml
[app]
id = "com.company.myapp"
name = "My Application"
version = "1.0.0"

[window]
title = "My Application"
width = 1200
height = 800
min_width = 800
min_height = 600
resizable = true
decorations = true

[build]
target = ["desktop"]

# Optimization settings
release_mode = "optimized"    # optimized, size, debug
lto = true                    # Link-time optimization
strip = true                  # Strip debug symbols

# Cargo features
features = ["analytics"]
```

## Cross-Compilation

Build for different platforms:

```bash
# macOS (Intel)
oxide build --release --target x86_64-apple-darwin

# macOS (Apple Silicon)
oxide build --release --target aarch64-apple-darwin

# Windows
oxide build --release --target x86_64-pc-windows-gnu

# Linux
oxide build --release --target x86_64-unknown-linux-gnu
```

### Setting up Cross-Compilation

```bash
# Install cross-compilation targets
rustup target add x86_64-pc-windows-gnu
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-apple-darwin

# For Windows (on macOS/Linux), install mingw
brew install mingw-w64          # macOS
sudo apt install mingw-w64      # Ubuntu
```

## Application Bundles

### macOS (.app)

Create a macOS application bundle:

```bash
oxide build --release --bundle
# Output: build/release/My Application.app
```

Configure the bundle:

```toml
[bundle]
identifier = "com.company.myapp"
icon = "assets/icon.icns"
category = "public.app-category.developer-tools"
copyright = "Copyright 2024 Company"

[bundle.macos]
minimum_version = "10.15"
entitlements = "entitlements.plist"
frameworks = ["Metal.framework"]

# Code signing
signing_identity = "Developer ID Application: Company Name"
hardened_runtime = true
```

### Windows (.exe with resources)

```toml
[bundle.windows]
icon = "assets/icon.ico"
file_description = "My Application"
product_name = "My Application"
company_name = "Company Name"

# Installer options
installer = true
installer_type = "msi"    # msi, nsis
```

### Linux

```toml
[bundle.linux]
icon = "assets/icon.png"
desktop_file = "my-app.desktop"
categories = ["Development", "Utility"]

# AppImage
appimage = true

# Deb package
deb = true
deb_depends = ["libssl-dev"]

# RPM package
rpm = true
```

## Window Configuration

```toml
[window]
title = "My Application"
width = 1200
height = 800
min_width = 800
min_height = 600
max_width = 1920
max_height = 1080
resizable = true
decorations = true           # Window frame
transparent = false          # Transparent background
always_on_top = false
fullscreen = false
visible = true
focused = true

# Position
x = 100
y = 100
centered = true              # Override x/y

# macOS specific
[window.macos]
titlebar_transparent = true
title_hidden = true
fullsize_content_view = true
```

## Programmatic Window Control

```oui
app MyApp {
    // Window manipulation
    Button {
        label: "Minimize"
        on_click: window.minimize()
    }

    Button {
        label: "Maximize"
        on_click: window.maximize()
    }

    Button {
        label: "Close"
        on_click: window.close()
    }

    Button {
        label: "Toggle Fullscreen"
        on_click: window.toggle_fullscreen()
    }

    // Window info
    Text {
        content: "Size: {window.width}x{window.height}"
    }
}
```

## Native Features

### System Tray

```toml
[tray]
enabled = true
icon = "assets/tray-icon.png"
tooltip = "My Application"
```

```oui
// Tray menu
TrayMenu {
    MenuItem { label: "Show" on_click: window.show() }
    MenuItem { label: "Hide" on_click: window.hide() }
    Separator {}
    MenuItem { label: "Quit" on_click: app.quit() }
}
```

### Native Menus

```oui
// macOS menu bar
MenuBar {
    Menu {
        title: "File"

        MenuItem { label: "New" shortcut: "Cmd+N" on_click: new_file }
        MenuItem { label: "Open" shortcut: "Cmd+O" on_click: open_file }
        Separator {}
        MenuItem { label: "Save" shortcut: "Cmd+S" on_click: save_file }
    }

    Menu {
        title: "Edit"

        MenuItem { label: "Undo" shortcut: "Cmd+Z" on_click: undo }
        MenuItem { label: "Redo" shortcut: "Cmd+Shift+Z" on_click: redo }
    }
}
```

### File Dialogs

```oui
Button {
    label: "Open File"
    on_click: {
        let path = dialog.open_file({
            title: "Select File",
            filters: [
                { name: "Images", extensions: ["png", "jpg"] },
                { name: "All Files", extensions: ["*"] }
            ]
        })

        if path != null {
            load_file(path)
        }
    }
}
```

### Notifications

```oui
Button {
    label: "Notify"
    on_click: {
        notification.show({
            title: "Task Complete",
            body: "Your export has finished",
            icon: "success"
        })
    }
}
```

## Performance

### GPU Rendering

OxideKit uses wgpu for GPU-accelerated rendering:

- Direct Metal/Vulkan/DX12 rendering
- 60fps animations
- Hardware-accelerated compositing
- Efficient texture atlasing

### Memory Usage

Typical memory footprint:

| App Type | Memory |
|----------|--------|
| Simple | 10-20MB |
| Medium | 20-50MB |
| Complex | 50-100MB |

Compare to Electron (150-300MB+).

### Startup Time

- Cold start: < 500ms
- Warm start: < 200ms

## CI/CD

### GitHub Actions

```yaml
name: Build

on: [push, pull_request]

jobs:
  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install OxideKit
        run: cargo install oxide-cli

      - name: Build
        run: oxide build --release

      - name: Upload Artifact
        uses: actions/upload-artifact@v4
        with:
          name: my-app-${{ matrix.os }}
          path: build/release/*
```

### Release Workflow

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: windows-latest
            target: x86_64-pc-windows-msvc

    steps:
      - uses: actions/checkout@v4

      - name: Build Release
        run: oxide build --release --target ${{ matrix.target }} --bundle

      - name: Upload Release
        uses: softprops/action-gh-release@v1
        with:
          files: build/release/*
```
