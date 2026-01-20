# Migration Guide: Electron/Tauri to OxideKit

This guide covers migrating existing applications from Electron or Tauri to OxideKit.

## Overview

OxideKit provides compatibility layers to ease migration, but the recommended path is to migrate to native OxideKit components for best performance and security.

### Migration Approaches

1. **Full Rewrite** - Best for new projects or simple apps
2. **Incremental Migration** - Migrate piece by piece
3. **Hybrid Approach** - Use compat layers temporarily while migrating
4. **Compatibility Layer** - Use compat.webview for complex UI (NOT RECOMMENDED for production)

## Quick Start

```bash
# Analyze your project
oxide migrate analyze /path/to/electron-app

# Generate migration plan
oxide migrate plan /path/to/electron-app --output migration-plan.json

# Run migration (dry run first)
oxide migrate run /path/to/electron-app --dry-run

# Execute migration
oxide migrate run /path/to/electron-app
```

## Electron Migration

### IPC Communication

**Electron:**
```javascript
// Main process
const { ipcMain } = require('electron');
ipcMain.handle('get-data', async (event, id) => {
  return await fetchData(id);
});

// Renderer process
const data = await ipcRenderer.invoke('get-data', 123);
```

**OxideKit:**
```rust
// In your Rust backend
#[oxide::command]
async fn get_data(id: u32) -> Result<Data, Error> {
    fetch_data(id).await
}

// In your UI
let data = oxide.invoke("get_data", { id: 123 }).await;
```

### Window Management

**Electron:**
```javascript
const { BrowserWindow } = require('electron');
const win = new BrowserWindow({ width: 800, height: 600 });
win.loadFile('index.html');
```

**OxideKit:**
```rust
use oxide_runtime::Window;

let window = Window::builder()
    .title("My App")
    .size(800, 600)
    .build()?;
```

### File System Access

**Electron:**
```javascript
const fs = require('fs');
const data = fs.readFileSync('/path/to/file');
```

**OxideKit:**
```rust
use std::fs;
let data = fs::read("/path/to/file")?;
```

### Menus

**Electron:**
```javascript
const { Menu, MenuItem } = require('electron');
const menu = new Menu();
menu.append(new MenuItem({ label: 'File' }));
```

**OxideKit:**
```rust
use oxide_ui::Menu;

let menu = Menu::new()
    .item("File", vec![
        MenuItem::action("New", || { /* handler */ }),
        MenuItem::action("Open", || { /* handler */ }),
    ]);
```

## Tauri Migration

Tauri migration is simpler since Tauri already uses Rust.

### Commands

**Tauri:**
```rust
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
```

**OxideKit:**
```rust
#[oxide::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
```

### Events

**Tauri:**
```rust
window.emit("event-name", payload)?;
```

**OxideKit:**
```rust
window.emit("event-name", payload)?;
```

### Configuration

**Tauri (tauri.conf.json):**
```json
{
  "package": {
    "productName": "My App",
    "version": "1.0.0"
  }
}
```

**OxideKit (oxide.toml):**
```toml
[app]
name = "My App"
version = "1.0.0"

[build]
target = ["desktop"]
```

## UI Migration

### React/Vue/Svelte

If you have a React, Vue, or Svelte frontend:

**Option 1: Native Migration (Recommended)**
- Rewrite UI using OxideKit's native component system
- Better performance, smaller bundle size
- Full native integration

**Option 2: Compatibility Layer (Temporary)**
```toml
# oxide.toml
[policy]
allow_webview = true
```

```rust
use oxide_compat::webview::WebWidget;

let widget = WebWidget::builder()
    .bundled("assets/react-app")
    .build()?;
```

### Native Components

OxideKit provides native equivalents for common UI patterns:

| Web Pattern | OxideKit Equivalent |
|------------|---------------------|
| `<button>` | `<Button>` |
| `<input>` | `<Input>` |
| `<select>` | `<Select>` |
| `<table>` | `<Table>` |
| React state | `oxide_state` |
| CSS Flexbox | OxideKit Layout |

## Using Compatibility Layers

### WebView (compat.webview)

For complex web UIs that can't be immediately migrated:

```bash
oxide compat-layer add webview
```

```toml
# oxide.toml
[policy]
allow_webview = true
webview_csp = "default-src 'self'; script-src 'self'"
```

**WARNING**: WebView increases attack surface and is not recommended for production.

### JavaScript Runtime (compat.js)

For running JavaScript utilities (not for UI):

```bash
oxide compat-layer add js-runtime
```

```toml
# oxide.toml
[policy]
allow_js_runtime = true
js_memory_limit_mb = 64
js_timeout_ms = 5000
```

Use cases:
- JSON schema validation
- Markdown parsing
- Data transformation

**WARNING**: Consider porting to Rust for better performance.

### NPM Bundling

For build-time NPM package bundling:

```bash
oxide compat-layer npm add lodash@4.17.21
oxide compat-layer npm build
```

This uses Node.js at build time only, not at runtime.

## Step-by-Step Migration

### Phase 1: Analysis

1. Run `oxide migrate analyze` on your project
2. Review the migration plan
3. Identify critical paths and dependencies

### Phase 2: Setup

1. Create new OxideKit project: `oxide new my-app --template migration`
2. Copy configuration
3. Set up build pipeline

### Phase 3: Backend Migration

1. Port IPC handlers to OxideKit commands
2. Migrate file system operations
3. Update native API calls

### Phase 4: UI Migration

1. **Option A**: Rewrite in OxideKit native components
2. **Option B**: Use compat.webview temporarily

### Phase 5: Testing

1. Run `oxide doctor` to check configuration
2. Test all functionality
3. Performance testing

### Phase 6: Cleanup

1. Remove compat layers where possible
2. Optimize native implementations
3. Final security review

## Common Patterns

### State Management

**Redux/Vuex pattern:**
```rust
use oxide_state::Store;

let store = Store::new(initial_state);

// Update state
store.dispatch(Action::UpdateCounter(42));

// Subscribe to changes
store.subscribe(|state| {
    // React to state changes
});
```

### Routing

**React Router pattern:**
```rust
use oxide_router::Router;

let router = Router::new()
    .route("/", HomePage)
    .route("/about", AboutPage)
    .route("/user/:id", UserPage);
```

### Async Operations

**Promise pattern:**
```rust
use oxide_runtime::spawn;

spawn(async {
    let result = fetch_data().await?;
    update_ui(result);
    Ok(())
});
```

## Troubleshooting

### "Node APIs not available"

OxideKit doesn't include Node.js. Replace with:
- `fs` -> `std::fs`
- `path` -> `std::path`
- `http` -> `oxide_network`

### "Cannot find React/Vue/Svelte"

For web frameworks, either:
1. Migrate to native OxideKit components
2. Use compat.webview temporarily

### "IPC not working"

Replace Electron/Tauri IPC with OxideKit commands:
```rust
#[oxide::command]
fn my_command(args: Args) -> Result<Output, Error> {
    // Implementation
}
```

## Resources

- [OxideKit Documentation](https://oxidekit.com/docs)
- [Component Reference](https://oxidekit.com/components)
- [API Reference](https://oxidekit.com/api)
- [Examples](https://github.com/oxidekit/examples)

## Getting Help

- GitHub Issues: https://github.com/oxidekit/oxidekit-core/issues
- Discord: https://discord.gg/oxidekit
- Stack Overflow: Tag `oxidekit`
