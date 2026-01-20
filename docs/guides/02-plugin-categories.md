# Plugin Categories: Understanding the OxideKit Extension Ecosystem

OxideKit organizes plugins into distinct categories based on their capabilities and trust requirements. This guide explains each category and helps you choose the right approach for your needs.

## The Six Plugin Categories

```
+------------------+-------------------+------------------+
|       UI         |     SERVICE       |     NATIVE       |
|  (No Permissions)|  (App-Level)      |  (OS Access)     |
+------------------+-------------------+------------------+
|     TOOLING      |      THEME        |     DESIGN       |
|  (Build-Time)    |  (Token Packs)    |  (Layout Kits)   |
+------------------+-------------------+------------------+
```

---

## 1. UI Plugins

**Purpose**: Pure UI components with no special permissions.

### Characteristics

- No filesystem access
- No network access
- No native OS APIs
- Zero permission requirements
- Safest category

### Examples

| Plugin | Description |
|--------|-------------|
| `ui.tables` | Data tables with sorting, filtering, pagination |
| `ui.charts` | Bar charts, line charts, pie charts |
| `ui.forms` | Form builders and validation |
| `ui.calendar` | Date pickers and calendars |
| `ui.editor` | Rich text editing |

### Usage

```bash
oxide add ui.tables
```

```oui
// Using table components
DataTable {
    columns: [
        Column { field: "name", label: "Name" }
        Column { field: "email", label: "Email" }
    ]
    data: $users
    on_row_click: { select_user($event.row) }
}
```

### When to Use UI Plugins

- You need ready-made components
- You want consistent styling with your theme
- You don't need any OS or network capabilities

### Trust Model

```
+---------------+
|   UI Plugin   |
+---------------+
       |
       v
[Sandbox: Full]
       |
       v
No permissions needed
```

UI plugins run in a full sandbox with no capabilities. They can only render UI and respond to events.

---

## 2. Service Plugins

**Purpose**: App-level building blocks that provide business logic.

### Characteristics

- May require specific permissions
- Provide application services
- Can manage state
- May include both logic and UI

### Examples

| Plugin | Description | Permissions |
|--------|-------------|-------------|
| `service.auth` | Authentication flows | network, secure-storage |
| `service.db` | Local database | filesystem |
| `service.query` | Data fetching/caching | network |
| `service.sync` | Cloud synchronization | network, filesystem |
| `service.analytics` | Usage analytics | network |

### Usage

```bash
oxide add service.auth
```

```oui
// Using auth service
AuthProvider {
    config: {
        provider: "oauth2"
        client_id: $env.AUTH_CLIENT_ID
    }

    // Protected content
    if $auth.is_authenticated {
        Dashboard { user: $auth.user }
    } else {
        LoginForm {
            on_success: { $auth.login($credentials) }
        }
    }
}
```

### Declaring Permissions

Service plugins declare required permissions in their manifest:

```toml
# extensions/service.auth/manifest.toml
[package]
id = "service.auth"
category = "service"

[permissions]
required = ["network", "secure-storage"]
optional = ["biometrics"]

[permissions.network]
reason = "Connect to authentication provider"
domains = ["auth.example.com"]

[permissions.secure-storage]
reason = "Store authentication tokens securely"
```

### When to Use Service Plugins

- You need reusable business logic
- The service might be used across multiple projects
- The functionality is common (auth, db, caching)

---

## 3. Native Plugins

**Purpose**: Access operating system capabilities.

### Characteristics

- Direct OS API access
- Require explicit user consent
- Capability-based permissions
- Highest trust requirement

### Examples

| Plugin | Description | Capabilities |
|--------|-------------|--------------|
| `native.fs` | Filesystem access | read, write, watch |
| `native.keychain` | OS keychain/keyring | read, write, delete |
| `native.notifications` | System notifications | send, schedule |
| `native.clipboard` | Clipboard access | read, write |
| `native.shell` | Shell commands | execute |

### Usage

```bash
oxide add native.fs
```

```rust
// In Rust code
use native_fs::{FileSystem, Permission};

async fn save_document(content: &str) -> Result<()> {
    let fs = FileSystem::request(Permission::Write)?;
    fs.write("~/Documents/draft.txt", content).await?;
    Ok(())
}
```

### Capability Model

Native plugins use capability-based security:

```
User grants permission
        |
        v
+------------------+
|   Capability     |  <- Unforgeable token
+------------------+
        |
        v
+------------------+
|  Limited Scope   |  <- Only what was granted
+------------------+
        |
        v
+------------------+
|   Native API     |
+------------------+
```

### Permission Prompts

When a native capability is first used:

```
+------------------------------------------------+
|  "My App" wants to access your Documents       |
|                                                |
|  This allows the app to read and write files   |
|  in your Documents folder.                     |
|                                                |
|         [Deny]            [Allow Once]         |
|                   [Always Allow]               |
+------------------------------------------------+
```

### When to Use Native Plugins

- You need OS-level functionality
- No pure-UI alternative exists
- You're willing to handle permission flows

---

## 4. Tooling Plugins

**Purpose**: Development and build-time tools.

### Characteristics

- Run during development only
- Never ship in production builds
- Full system access (trusted dev environment)
- Extend the `oxide` CLI

### Examples

| Plugin | Description |
|--------|-------------|
| `tool.codegen` | Generate code from schemas |
| `tool.mock` | Mock data generation |
| `tool.lint` | Custom linting rules |
| `tool.i18n` | Extract/manage translations |
| `tool.figma` | Import from Figma designs |

### Usage

```bash
oxide add tool.codegen --dev
```

```bash
# Run tooling
oxide codegen --schema schema.graphql
oxide mock --generate users:100
oxide lint --fix
```

### Development Only

Tooling plugins are excluded from production builds:

```toml
# oxide.toml
[extensions]
ui.tables = "^1.0"       # Ships in production
tool.codegen = "^2.0"    # Dev only (--dev)

[build.release]
exclude = ["tool.*"]     # Automatic exclusion pattern
```

### When to Use Tooling Plugins

- You need build-time code generation
- You want custom development workflows
- The functionality is not needed at runtime

---

## 5. Theme Plugins

**Purpose**: Design token packages and visual presets.

### Characteristics

- Define token values
- May extend base themes
- Include color schemes
- No code execution (pure data)

### Examples

| Plugin | Description |
|--------|-------------|
| `theme.catppuccin` | Catppuccin color scheme |
| `theme.nord` | Nord color palette |
| `theme.corporate` | Enterprise-friendly design |
| `theme.accessibility` | High-contrast, large text |

### Usage

```bash
oxide add theme.catppuccin
```

```toml
# oxide.toml
[theme]
base = "theme.catppuccin.mocha"
```

### Theme Structure

```toml
# theme.catppuccin/mocha.toml
name = "Catppuccin Mocha"
extends = "oxidekit-dark"

[tokens.color]
primary = { value = "#89B4FA" }
secondary = { value = "#F5C2E7" }
background = { value = "#1E1E2E" }
surface = { value = "#313244" }
text = { value = "#CDD6F4" }

[tokens.radius]
default = 8
```

### When to Use Theme Plugins

- You want a pre-made color scheme
- You need consistent branding
- You're building on top of established design systems

---

## 6. Design Plugins

**Purpose**: Complete layout kits and UI patterns.

### Characteristics

- Pre-built layouts and shells
- Include multiple components
- Provide structural patterns
- May include themes

### Examples

| Plugin | Description |
|--------|-------------|
| `design.admin-shell` | Dashboard layout with sidebar |
| `design.marketing` | Landing page patterns |
| `design.settings` | Settings panel patterns |
| `design.wizard` | Multi-step form patterns |

### Usage

```bash
oxide add design.admin-shell
```

```oui
// Using admin shell layout
AdminShell {
    sidebar: [
        NavItem { icon: "home", label: "Dashboard", href: "/" }
        NavItem { icon: "users", label: "Users", href: "/users" }
        NavItem { icon: "settings", label: "Settings", href: "/settings" }
    ]
    header: {
        title: "My Admin"
        user: $current_user
    }

    // Page content
    slot content {
        Router {
            Route { path: "/", component: DashboardPage }
            Route { path: "/users", component: UsersPage }
        }
    }
}
```

### When to Use Design Plugins

- You need pre-built application shells
- You want consistent structural patterns
- You're building common application types (admin, docs, etc.)

---

## Trust Levels

All plugins have an associated trust level:

```
+------------------+----------------------------------------+
|  Trust Level     |  Requirements                          |
+------------------+----------------------------------------+
|  Official        |  Maintained by OxideKit org            |
|  Verified        |  Identity verified + signed releases   |
|  Community       |  Sandbox by default + warnings         |
+------------------+----------------------------------------+
```

### Trust Indicators

```bash
oxide add ui.tables
```

```
ui.tables v1.2.0
  Publisher: oxidekit (Official)
  Downloads: 50,000+
  Verified: Yes (signature valid)

  Permissions: None required

  [Install] [View Source]
```

### Sandbox Behavior

| Trust Level | UI | Service | Native | Tooling |
|-------------|-----|---------|--------|---------|
| Official | Full access | Declared perms | Declared caps | Full dev access |
| Verified | Full access | Declared perms | Declared caps | Full dev access |
| Community | Sandboxed | Extra warnings | Denied by default | Extra warnings |

---

## Category Decision Matrix

Use this matrix to choose the right category:

| Need | Category | Trust Required |
|------|----------|----------------|
| Display data in a table | `ui` | Low |
| User authentication | `service` | Medium |
| Read/write files | `native` | High |
| Generate code at build time | `tooling` | Dev only |
| Apply a color scheme | `theme` | Low |
| Pre-built dashboard layout | `design` | Low |

---

## Installation Sources

Plugins can come from multiple sources:

```bash
# Official registry
oxide add ui.tables

# Specific version
oxide add ui.tables@1.2.0

# Git repository
oxide add git github.com/acme/custom-plugin@v1.0

# Local path (for development)
oxide add path ../my-plugin
```

### Lock File

All installations are recorded in `extensions.lock`:

```toml
[[extension]]
id = "ui.tables"
version = "1.2.0"
source = "registry"
checksum = "sha256:abc123..."
resolved = "2024-01-15"
```

---

## Best Practices

### 1. Minimize Native Usage

```
Prefer: ui.* and service.* plugins
Use native.* only when necessary
```

### 2. Audit Community Plugins

```bash
oxide audit service.unknown-auth
```

### 3. Pin Versions in Production

```toml
# oxide.toml
[extensions]
ui.tables = "=1.2.0"  # Exact version
```

### 4. Separate Dev and Production

```toml
[extensions]
ui.tables = "^1.0"

[extensions.dev]
tool.codegen = "^2.0"
tool.mock = "^1.0"
```

---

## Next Steps

- [Build Modes](./03-build-modes.md) - Understand Dev vs Release vs Diagnostics
- [AI Assistant Philosophy](./04-ai-philosophy.md) - How AI helps without taking over
- [Quick Start](./05-quick-start.md) - Get building in 5 minutes
