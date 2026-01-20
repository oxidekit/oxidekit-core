# oxide.toml Specification

**Version:** 0.1.0
**Status:** Draft

## Overview

`oxide.toml` is the project manifest file for OxideKit applications. It defines application metadata, window configuration, extensions, and build settings.

## Location

The manifest must be located at the root of an OxideKit project:

```
my-app/
├── oxide.toml      # Project manifest (required)
├── src/
├── ui/
└── assets/
```

## Full Schema

```toml
[app]
# Required: Reverse-domain identifier
id = "com.example.myapp"
# Required: Human-readable name
name = "My Application"
# Required: Semantic version
version = "0.1.0"
# Optional: Description
description = "A sample OxideKit application"
# Optional: Authors
authors = ["Your Name <you@example.com>"]
# Optional: License identifier (SPDX)
license = "MIT"

[core]
# Required: OxideKit core version constraint (semver)
requires = ">=0.1.0,<0.2.0"

[window]
# Optional: Window title (default: app name)
title = "My App"
# Optional: Initial width in logical pixels (default: 1280)
width = 1280
# Optional: Initial height in logical pixels (default: 720)
height = 720
# Optional: Minimum width
min_width = 800
# Optional: Minimum height
min_height = 600
# Optional: Allow resizing (default: true)
resizable = true
# Optional: Show window decorations (default: true)
decorations = true

[extensions]
# Optional: List of allowed extensions
allow = [
    "ui.tables",
    "native.filesystem",
    "theme.wallet.dark",
]

[permissions]
# Optional: Per-extension permission grants
"native.filesystem" = ["filesystem.read", "filesystem.write"]
"native.keychain" = ["keychain.access"]

[build]
# Optional: Target platforms
target = ["windows", "macos", "linux"]
# Optional: Enable optimizations
optimize = true
# Optional: Asset glob patterns to include
assets = ["assets/*"]

[dev]
# Optional: Enable hot reload (default: true in dev)
hot_reload = true
# Optional: Enable inspector overlay
inspector = true
```

## Sections

### `[app]` - Application Metadata

| Field | Required | Type | Description |
|-------|----------|------|-------------|
| `id` | Yes | String | Reverse-domain identifier (e.g., `com.example.myapp`) |
| `name` | Yes | String | Human-readable application name |
| `version` | Yes | String | Semantic version (e.g., `0.1.0`) |
| `description` | No | String | Brief description |
| `authors` | No | Array | List of authors with optional email |
| `license` | No | String | SPDX license identifier |

**Validation Rules:**
- `id` must match: `^[a-z][a-z0-9]*(\.[a-z][a-z0-9]*)+$`
- `version` must be valid semver

### `[core]` - Core Version Constraint

| Field | Required | Type | Description |
|-------|----------|------|-------------|
| `requires` | Yes | String | Semver version range for OxideKit core |

**Examples:**
- `">=0.1.0"` - Any version 0.1.0 or later
- `">=0.1.0,<0.2.0"` - 0.1.x only
- `"0.1.5"` - Exactly 0.1.5

### `[window]` - Window Configuration

| Field | Default | Type | Description |
|-------|---------|------|-------------|
| `title` | App name | String | Window title |
| `width` | 1280 | u32 | Initial width (logical pixels) |
| `height` | 720 | u32 | Initial height (logical pixels) |
| `min_width` | None | u32 | Minimum width |
| `min_height` | None | u32 | Minimum height |
| `resizable` | true | bool | Allow window resizing |
| `decorations` | true | bool | Show window decorations |

### `[extensions]` - Extension Allowlist

| Field | Type | Description |
|-------|------|-------------|
| `allow` | Array | List of extension identifiers to allow |

Extensions must be explicitly allowed before use.

### `[permissions]` - Capability Grants

Per-extension permission grants. Keys are extension identifiers, values are arrays of capability strings.

**Common Capabilities:**
- `filesystem.read` - Read files
- `filesystem.write` - Write files
- `keychain.access` - Access system keychain
- `network.http` - Make HTTP requests
- `clipboard.read` - Read clipboard
- `clipboard.write` - Write clipboard
- `notifications.send` - Send notifications

### `[build]` - Build Configuration

| Field | Default | Type | Description |
|-------|---------|------|-------------|
| `target` | All | Array | Target platforms |
| `optimize` | false | bool | Enable release optimizations |
| `assets` | [] | Array | Asset glob patterns |

### `[dev]` - Development Configuration

| Field | Default | Type | Description |
|-------|---------|------|-------------|
| `hot_reload` | true | bool | Enable hot reload |
| `inspector` | false | bool | Enable inspector overlay |

## Example: Minimal

```toml
[app]
id = "com.example.minimal"
name = "Minimal App"
version = "0.1.0"

[core]
requires = ">=0.1.0"
```

## Example: Full

```toml
[app]
id = "com.example.dashboard"
name = "Dashboard Pro"
version = "1.0.0"
description = "A professional dashboard application"
authors = ["Team <team@example.com>"]
license = "MIT"

[core]
requires = ">=0.2.0,<0.3.0"

[window]
title = "Dashboard Pro"
width = 1440
height = 900
min_width = 1024
min_height = 768
resizable = true
decorations = true

[extensions]
allow = [
    "ui.charts",
    "ui.tables",
    "native.filesystem",
    "theme.dashboard.dark",
]

[permissions]
"native.filesystem" = ["filesystem.read"]

[build]
target = ["windows", "macos"]
optimize = true
assets = ["assets/**/*"]

[dev]
hot_reload = true
inspector = true
```
