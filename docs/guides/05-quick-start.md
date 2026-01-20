# Quick Start: Build Your First OxideKit App

Get from zero to a running application in under 5 minutes.

## Prerequisites

Before you begin, ensure you have:

- **Rust** (1.75+): `rustup update stable`
- **Git**: For version control
- **A code editor**: VS Code, Zed, or similar

## Step 1: Install the CLI

Install the OxideKit CLI tool:

```bash
cargo install oxide-cli
```

Verify the installation:

```bash
oxide --version
# oxide 0.1.0
```

## Step 2: Create Your Project

### Option A: Minimal Project

Create a basic project to learn the fundamentals:

```bash
oxide new my-app
cd my-app
```

### Option B: Starter Template (Recommended)

Use a starter template for a production-ready foundation:

```bash
oxide new my-dashboard --starter admin-panel
cd my-dashboard
```

Available starters:

| Starter | Best For | Includes |
|---------|----------|----------|
| `admin-panel` | Internal tools, dashboards | Tables, charts, sidebar nav |
| `docs-site` | Documentation | Search, navigation, markdown |
| `desktop-wallet` | Financial apps | Secure UI patterns |
| `website-single` | Landing pages | Marketing layouts |

List all starters:

```bash
oxide starters list
```

## Step 3: Explore the Project Structure

```
my-app/
├── oxide.toml           # Project configuration
├── src/
│   ├── app.oui          # Root component
│   ├── pages/
│   │   └── home.oui     # Home page
│   └── components/      # Your custom components
├── theme.toml           # Theme customization (optional)
└── extensions.lock      # Locked dependencies
```

### Key Files

**oxide.toml** - Project configuration:

```toml
[package]
name = "my-app"
version = "0.1.0"
description = "My first OxideKit app"

[build]
target = "desktop"

[extensions]
ui.core = "^1.0"
```

**src/app.oui** - Root component:

```oui
App {
    // Global theme provider
    ThemeProvider {
        theme: "dark"

        // Your app layout
        Layout {
            Header {
                title: "My App"
            }

            Router {
                Route { path: "/", component: HomePage }
            }
        }
    }
}
```

## Step 4: Run in Development Mode

Start the development server:

```bash
oxide dev
```

Your app opens automatically. You should see:

```
  OxideKit Dev Server
  ===================

  Local:   http://localhost:3000
  Network: http://192.168.1.100:3000

  Hot reload enabled
  Press Ctrl+C to stop
```

### Development Features

- **Hot Reload**: Changes apply instantly without losing state
- **Dev Tools**: Press `Ctrl+Shift+D` to open the inspector
- **Layout Overlay**: Press `Ctrl+Shift+L` to see flex boundaries
- **Error Messages**: Clear, actionable error messages

## Step 5: Make Your First Change

Open `src/pages/home.oui` and modify it:

```oui
Page {
    VStack {
        spacing: $spacing.lg

        Text {
            role: "heading.h1"
            content: "Welcome to OxideKit!"
        }

        Text {
            role: "body.default"
            content: "This is your first page."
        }

        Button {
            variant: "primary"
            label: "Click Me"
            on_click: {
                print("Button clicked!")
            }
        }
    }
}
```

Save the file. The change appears instantly in your running app.

## Step 6: Add a Component

Create a new component at `src/components/Counter.oui`:

```oui
Component Counter {
    // Component state
    state count = 0

    // Component UI
    HStack {
        spacing: $spacing.md
        align: "center"

        Button {
            variant: "outline"
            label: "-"
            on_click: { $count = $count - 1 }
        }

        Text {
            role: "heading.h2"
            content: $count.to_string()
        }

        Button {
            variant: "primary"
            label: "+"
            on_click: { $count = $count + 1 }
        }
    }
}
```

Use it in your home page:

```oui
Page {
    VStack {
        spacing: $spacing.lg

        Text {
            role: "heading.h1"
            content: "Counter Example"
        }

        Counter {}
    }
}
```

## Step 7: Customize the Theme

Create or edit `theme.toml`:

```toml
name = "My Theme"
extends = "oxidekit-dark"

[tokens.color]
primary = { value = "#8B5CF6", light = "#A78BFA", dark = "#7C3AED" }
secondary = { value = "#EC4899" }

[tokens.radius]
button = 12
card = 16
```

Your entire app updates to use the new colors.

## Step 8: Add a Plugin

Add the tables plugin:

```bash
oxide add ui.tables
```

Use it in your code:

```oui
DataTable {
    columns: [
        Column { field: "name", label: "Name", sortable: true }
        Column { field: "email", label: "Email" }
        Column { field: "role", label: "Role" }
    ]
    data: $users
    on_row_click: { select_user($event.row) }
}
```

## Step 9: Build for Production

When you're ready to ship:

```bash
oxide build --release
```

Find your binary at `target/release/my-app`.

### Binary Size

| Mode | Size |
|------|------|
| Dev | ~45 MB |
| Release | ~12 MB |

## Common Commands Reference

```bash
# Development
oxide dev                    # Start dev server with hot reload
oxide build                  # Debug build

# Production
oxide build --release        # Optimized production build
oxide run --release          # Run the release build

# Plugins
oxide add ui.tables          # Add a plugin
oxide add tool.codegen --dev # Add a dev-only tool

# Information
oxide starters list          # List starter templates
oxide doctor                 # Diagnose project issues
oxide learn                  # Interactive tutorials

# Export
oxide export theme dark      # Export theme tokens
oxide export ai-schema       # Export AI catalog
```

## Troubleshooting

### "oxide: command not found"

Ensure `~/.cargo/bin` is in your PATH:

```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/.cargo/bin:$PATH"
```

### Hot Reload Not Working

Check that the file watcher is running:

```bash
oxide doctor
```

### Build Errors

Run diagnostics:

```bash
oxide doctor
oxide lint
```

## Next Steps

Now that you have a running app:

1. **Read the Guides**
   - [Core Concepts](./01-core-concepts.md) - The OxideKit mental model
   - [Plugin Categories](./02-plugin-categories.md) - Extend your app
   - [Build Modes](./03-build-modes.md) - Dev, release, diagnostics

2. **Explore Starters**
   ```bash
   oxide starters list
   oxide starters info admin-panel
   ```

3. **Learn Interactively**
   ```bash
   oxide learn
   ```

4. **Browse Components**
   ```bash
   oxide export ai-schema
   # Opens oxide.ai.json with full component specs
   ```

## Getting Help

- **Documentation**: [oxidekit.com/docs](https://oxidekit.com/docs)
- **Tutorials**: `oxide learn`
- **Diagnostics**: `oxide doctor`
- **Community**: [discord.gg/oxidekit](https://discord.gg/oxidekit)

---

Happy building with OxideKit!
