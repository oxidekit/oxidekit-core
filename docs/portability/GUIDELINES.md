# OxideKit Portability Guidelines

This document provides comprehensive guidelines for writing portable OxideKit code that can run across desktop, web, and mobile platforms.

## Overview

OxideKit is designed to support multiple target platforms:

- **Desktop**: macOS, Windows, Linux
- **Web**: WASM/Browser (wasm32-unknown-unknown)
- **Mobile**: iOS, Android

To ensure your code can eventually run on all platforms, follow these guidelines.

## Portability Levels

### Portable

APIs marked as `#[portable]` work on all platforms without modification.

```rust
use oxide_portable_macros::portable;

#[portable]
fn calculate_layout(width: f32, height: f32) -> Layout {
    // This logic works everywhere
    Layout::new(width, height)
}
```

### Desktop Only

APIs marked as `#[desktop_only]` require desktop OS features:

```rust
use oxide_portable_macros::desktop_only;

#[desktop_only(reason = "Requires native file system access")]
fn read_config_file(path: &str) -> Result<Config, Error> {
    let content = std::fs::read_to_string(path)?;
    toml::from_str(&content)
}
```

### Web Only

APIs marked as `#[web_only]` require browser/WASM environment:

```rust
use oxide_portable_macros::web_only;

#[web_only]
fn get_local_storage(key: &str) -> Option<String> {
    // Uses browser's localStorage API
}
```

### Mobile Only

APIs marked as `#[mobile_only]` require iOS or Android:

```rust
use oxide_portable_macros::mobile_only;

#[mobile_only]
fn trigger_haptic_feedback(intensity: HapticIntensity) {
    // Uses device haptic engine
}
```

## API Categories

Different API categories have different default portability:

| Category | Default Level | Notes |
|----------|---------------|-------|
| Core | Portable | Runtime, types, traits |
| UI | Portable | Components, styling |
| Layout | Portable | Flexbox, grid |
| Render | Portable | WebGPU abstracted |
| Input | Portable | Touch + mouse unified |
| Network | Portable | HTTP, WebSocket |
| FileSystem | Native Only | No web support |
| Window | Desktop Only | Native windows |
| System | Native Only | OS integration |
| Clipboard | Portable | Browser + native |
| Notifications | Portable | All platforms |
| Storage | Portable | Abstracted persistence |

## Writing Portable Code

### 1. Use Abstraction Layers

Instead of directly using platform APIs, use OxideKit's abstractions:

```rust
// Bad: Direct std::fs usage
use std::fs;
let content = fs::read_to_string("config.toml")?;

// Good: Use portable storage API
use oxide_storage::Storage;
let content = Storage::read("config.toml").await?;
```

### 2. Avoid `std::process`

Process spawning is not available on web or mobile:

```rust
// Bad: Direct process spawn
use std::process::Command;
Command::new("ls").output()?;

// Good: Use alternative approaches or feature gate
#[cfg(not(target_arch = "wasm32"))]
fn run_external_command() {
    // Desktop-only implementation
}
```

### 3. Handle Async Correctly

Web requires async for many operations that are sync on desktop:

```rust
// Portable async approach
async fn fetch_data(url: &str) -> Result<String, Error> {
    oxide_network::fetch(url).await
}
```

### 4. Use Conditional Compilation

For platform-specific code, use `cfg` attributes:

```rust
#[cfg(target_os = "macos")]
fn macos_specific() {
    // macOS implementation
}

#[cfg(target_os = "windows")]
fn windows_specific() {
    // Windows implementation
}

#[cfg(target_arch = "wasm32")]
fn web_specific() {
    // Web implementation
}

#[cfg(any(target_os = "ios", target_os = "android"))]
fn mobile_specific() {
    // Mobile implementation
}
```

### 5. Provide Fallbacks

When a feature isn't available, provide graceful degradation:

```rust
fn get_device_id() -> String {
    #[cfg(target_os = "macos")]
    {
        // Get hardware UUID
    }

    #[cfg(target_arch = "wasm32")]
    {
        // Generate or retrieve from localStorage
    }

    #[cfg(not(any(target_os = "macos", target_arch = "wasm32")))]
    {
        // Generate random ID
        uuid::Uuid::new_v4().to_string()
    }
}
```

## Plugin Portability

### Declaring Portability

In your plugin manifest (`oxide-plugin.toml`):

```toml
[portability]
level = "portable"  # or "desktop-only", "native-only", etc.

# Required capabilities
required_capabilities = ["network"]

# Optional capabilities that enhance functionality
optional_capabilities = ["filesystem"]

# Platform-specific notes
[portability.platform_notes]
web = "CSV export disabled on web"
ios = "Requires iOS 14+"
```

### Per-API Portability

Override portability for specific APIs:

```toml
[portability]
level = "portable"

[portability.apis.export_csv]
level = "desktop-only"
requires = ["filesystem"]
```

## Running Portability Checks

### Check Current Project

```bash
# Check against default targets (current + web + mobile)
oxide check portability

# Check against specific target
oxide check portability --target wasm32-unknown-unknown

# Check all supported targets
oxide check portability --all-targets

# Strict mode (warnings become errors)
oxide check portability --strict
```

### Check Plugin Portability

```bash
oxide check plugin --manifest oxide-plugin.toml
```

### List Available Targets

```bash
oxide check targets
```

## Common Portability Issues

### Issue: Using `std::fs` directly

**Problem**: File system APIs don't work on web.

**Solution**: Use `oxide_storage` or guard with `#[cfg(not(target_arch = "wasm32"))]`.

### Issue: Using `std::thread::spawn`

**Problem**: Native threads work differently on web (Web Workers).

**Solution**: Use `oxide_runtime::spawn` which abstracts threading.

### Issue: Using `std::time::Instant`

**Problem**: Limited precision on web.

**Solution**: Use `oxide_time::Instant` for portable timing.

### Issue: Using `std::env`

**Problem**: Environment variables don't exist on web/mobile.

**Solution**: Use configuration files or `oxide_config`.

### Issue: Native FFI

**Problem**: C/C++ FFI doesn't work on web.

**Solution**: Provide WASM-compatible alternatives or feature-gate.

## Testing Portability

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_portable_function() {
        // This test runs on all targets
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_desktop_only_function() {
        // This test only runs on desktop
    }
}
```

### CI/CD

Run tests on multiple targets in CI:

```yaml
strategy:
  matrix:
    target:
      - x86_64-apple-darwin
      - x86_64-pc-windows-msvc
      - x86_64-unknown-linux-gnu
      - wasm32-unknown-unknown
      - aarch64-apple-ios
      - aarch64-linux-android

steps:
  - name: Run portability check
    run: oxide check portability --target ${{ matrix.target }} --strict
```

## Resources

- [Target Capabilities Reference](./TARGET_CAPABILITIES.md)
- [API Portability Matrix](./API_MATRIX.md)
- [Migration Guide](./MIGRATION.md)
