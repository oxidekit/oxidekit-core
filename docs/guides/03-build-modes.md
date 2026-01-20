# Build Modes: Dev vs Release vs Diagnostics

OxideKit has three distinct build modes, each serving a specific purpose. Understanding these modes ensures you ship the right build for each situation.

## The Three Modes

```
+----------------+   +----------------+   +------------------------+
|      DEV       |   |    RELEASE     |   | RELEASE + DIAGNOSTICS  |
+----------------+   +----------------+   +------------------------+
|                |   |                |   |                        |
|  Hot Reload    |   |  Optimized     |   |  Optimized             |
|  Dev Tools     |   |  No Dev Tools  |   |  No Dev Tools          |
|  Full Logging  |   |  Minimal Logs  |   |  Structured Logging    |
|  Diagnostics   |   |  No Diagnostics|   |  Diagnostics           |
|                |   |                |   |                        |
+----------------+   +----------------+   +------------------------+
     Debug              Production          Production + Support
```

---

## 1. Development Mode (Dev)

**Command**: `oxide dev` or `oxide build`

### Purpose

Fast iteration during development with full debugging capabilities.

### What's Included

| Feature | Status | Notes |
|---------|--------|-------|
| Hot Reload | Enabled | State-preserving UI updates |
| Dev Tools | Full | Inspector, profiler, overlays |
| Logging | Verbose | All levels (trace, debug, info, warn, error) |
| Diagnostics | Full | Complete error context |
| Optimizations | None | Fast compilation |
| Assertions | Enabled | Runtime checks |

### Development Features

#### Hot Reload

```
File changed: src/pages/dashboard.oui
             |
             v
    +------------------+
    |  Recompile View  |  (50-100ms)
    +------------------+
             |
             v
    +------------------+
    |  Preserve State  |  (Form data, scroll position)
    +------------------+
             |
             v
    +------------------+
    |  Update UI       |  (Instant)
    +------------------+
```

#### Dev Tools Panel

Open with `Ctrl+Shift+D` (or `Cmd+Shift+D` on macOS):

```
+--------------------------------------------------+
|  OxideKit Dev Tools                         [X]  |
+--------------------------------------------------+
|  [Inspector] [Profiler] [State] [Network] [Logs] |
+--------------------------------------------------+
|                                                  |
|  Component Tree:                                 |
|  + App                                           |
|    + Layout                                      |
|      + Sidebar                                   |
|      > MainContent                               |
|        > Dashboard                               |
|          - MetricCard (x4)                       |
|          - ChartPanel                            |
|                                                  |
|  Selected: MetricCard                            |
|  Props: { title: "Users", value: 1234 }          |
|  State: { loading: false }                       |
|  Tokens: color.primary, spacing.md              |
|                                                  |
+--------------------------------------------------+
```

#### Layout Bounds Overlay

Toggle with `Ctrl+Shift+L`:

```
+--------------------------------------------------+
|  +-----------------------------------------+     |
|  | Sidebar       |  +--------------------+ |     |
|  | (flex: 0 0    |  | Header             | |     |
|  |  200px)       |  | (height: 64px)     | |     |
|  |               |  +--------------------+ |     |
|  |               |  +--------------------+ |     |
|  |               |  | Content            | |     |
|  |               |  | (flex: 1)          | |     |
|  |               |  |                    | |     |
|  |               |  |                    | |     |
|  +-----------------------------------------+     |
+--------------------------------------------------+
```

### When to Use Dev Mode

- Active development
- Debugging issues
- Performance profiling
- UI iteration

---

## 2. Release Mode

**Command**: `oxide build --release`

### Purpose

Optimized production build with minimal footprint.

### What's Included

| Feature | Status | Notes |
|---------|--------|-------|
| Hot Reload | Removed | No dev server code |
| Dev Tools | Removed | Zero dev UI |
| Logging | Minimal | Error and critical only |
| Diagnostics | Removed | No crash reporting |
| Optimizations | Full | LTO, dead code elimination |
| Assertions | Removed | No runtime checks |

### Binary Size Impact

```
Dev Build:     ~45 MB
Release Build: ~12 MB  (73% smaller)
```

### What Gets Stripped

```rust
// This code is completely removed in release:
#[cfg(feature = "dev-editor")]
pub mod editor;

#[cfg(feature = "dev-editor")]
pub mod overlay;

// Assertions are removed:
debug_assert!(value > 0);  // Gone in release
```

### When to Use Release Mode

- Shipping to users
- Performance benchmarks
- Final testing before deployment

---

## 3. Release + Diagnostics Mode

**Command**: `oxide build --release --features diagnostics`

### Purpose

Production build with optional crash reporting and support tools.

### What's Included

| Feature | Status | Notes |
|---------|--------|-------|
| Hot Reload | Removed | No dev server code |
| Dev Tools | Removed | Zero dev UI |
| Logging | Structured | Error events with context |
| Diagnostics | Bundle Export | User-controlled export |
| Optimizations | Full | Same as release |
| Crash Handler | Optional | Capture crash info |

### The Diagnostics Bundle

When users encounter issues, they can export a diagnostics bundle:

```
Help > Export Diagnostics...
          |
          v
+-------------------------------------------+
|  Export Diagnostics Bundle                |
+-------------------------------------------+
|                                           |
|  This creates a file to help us diagnose  |
|  your issue. It includes:                 |
|                                           |
|  [x] Error messages and codes             |
|  [x] Component state at time of error     |
|  [x] Performance metrics                  |
|  [ ] Full logs (may contain sensitive     |
|      data - review before sending)        |
|                                           |
|  Personal information is automatically    |
|  removed. You can review the file before  |
|  sharing.                                 |
|                                           |
|        [Cancel]        [Export...]        |
+-------------------------------------------+
```

### Bundle Contents

```json
{
  "format_version": "1.0",
  "timestamp": "2024-01-15T10:30:00Z",
  "app": {
    "name": "MyApp",
    "version": "2.1.0",
    "build_id": "abc123",
    "oxidekit_version": "0.5.0",
    "build_profile": "release_diagnostics"
  },
  "system": {
    "os": "macos",
    "arch": "aarch64",
    "memory_mb": 16384
  },
  "events": [
    {
      "code": "UI-404",
      "severity": "error",
      "message": "Component not found: CustomWidget",
      "context": {
        "file": "pages/dashboard.oui",
        "line": 45
      }
    }
  ],
  "logs": [
    // Redacted log entries
  ]
}
```

### Privacy Safeguards

All diagnostic data goes through redaction:

```
Before:  "User john@example.com logged in"
After:   "User [EMAIL] logged in"

Before:  "API key: sk-1234567890abcdef"
After:   "API key: [REDACTED]"

Before:  "File: /Users/john/Documents/secret.txt"
After:   "File: ~/Documents/secret.txt"
```

### Error Codes

Structured error codes make debugging easier:

```
Error Code Format: DOMAIN-NUMBER

Domains:
  UI   - User interface errors
  RT   - Runtime errors
  FS   - Filesystem errors
  NET  - Network errors
  EXT  - Extension errors
  CFG  - Configuration errors

Examples:
  UI-100  - Component not found
  UI-200  - Invalid prop type
  RT-100  - Out of memory
  EXT-300 - Permission denied
```

### When to Use Release + Diagnostics

- Production apps that need support tooling
- Beta/preview releases
- Enterprise deployments
- Any app where you need to debug user issues

---

## Comparison Table

| Feature | Dev | Release | Release + Diag |
|---------|-----|---------|----------------|
| Binary Size | Large | Small | Small + ~500KB |
| Startup Time | Slow | Fast | Fast |
| Hot Reload | Yes | No | No |
| Dev Tools | Full | None | None |
| Inspector | Yes | No | No |
| Profiler | Yes | No | No |
| Layout Overlay | Yes | No | No |
| Logging | Verbose | Minimal | Structured |
| Error Codes | Yes | Minimal | Yes |
| Bundle Export | Yes | No | Yes |
| Crash Handler | Yes | No | Optional |
| Auto-Report | No | No | Opt-in |

---

## Build Configuration

### oxide.toml

```toml
[build]
# Default target
target = "desktop"

[build.dev]
# Dev-specific settings
hot_reload = true
dev_tools = true
log_level = "trace"

[build.release]
# Release-specific settings
strip = true
lto = true
opt_level = 3

[build.release_diagnostics]
# Diagnostics settings
strip = true
lto = true
opt_level = 3
error_codes = true
bundle_export = true
crash_handler = true
auto_report = false  # Requires opt-in
```

### Feature Flags in Code

```rust
// Dev-only code
#[cfg(feature = "dev-editor")]
fn enable_inspector() {
    // Only compiled in dev builds
}

// Diagnostics code (dev and release+diag)
#[cfg(any(debug_assertions, feature = "diagnostics-export"))]
fn capture_error(error: &Error) {
    // Compiled in dev and release+diagnostics
}

// Release-only optimization
#[cfg(not(debug_assertions))]
fn use_optimized_path() {
    // Only in release builds
}
```

---

## Build Commands Reference

```bash
# Development (default)
oxide dev                          # Start dev server
oxide build                        # Dev build

# Release
oxide build --release              # Production build

# Release + Diagnostics
oxide build --release --features diagnostics

# Platform-specific
oxide build --release --target macos
oxide build --release --target windows
oxide build --release --target linux

# With specific features
oxide build --release --features "diagnostics,auto-report"
```

---

## Best Practices

### 1. Never Ship Dev Builds

```
BAD:  Ship `oxide build` output to users
GOOD: Ship `oxide build --release` output
```

### 2. Use Diagnostics for Support

```
// Enable diagnostics for supportable apps
oxide build --release --features diagnostics
```

### 3. Test in Release Mode

```bash
# Before shipping, test release builds
oxide build --release
./target/release/my-app

# Test startup time, binary size, behavior
```

### 4. Check Binary Size

```bash
# Monitor binary size growth
oxide build --release
ls -lh target/release/my-app

# If too large, check dependencies
cargo bloat --release
```

---

## Troubleshooting

### Dev Tools Not Showing

```bash
# Ensure dev-editor feature is enabled
oxide dev  # Should work

# Check Cargo.toml has the feature
[features]
dev-editor = ["oxide-devtools/dev-editor"]
```

### Release Build Too Large

```bash
# Check for debug symbols
strip target/release/my-app

# Enable LTO in Cargo.toml
[profile.release]
lto = true
strip = true
```

### Diagnostics Bundle Empty

```bash
# Ensure feature is enabled
oxide build --release --features diagnostics

# Check that errors are being captured
# The bundle only contains captured events
```

---

## Next Steps

- [AI Assistant Philosophy](./04-ai-philosophy.md) - How AI helps without taking over
- [Quick Start](./05-quick-start.md) - Get building in 5 minutes
- [CLI Reference](../reference/cli.md) - Full command documentation
