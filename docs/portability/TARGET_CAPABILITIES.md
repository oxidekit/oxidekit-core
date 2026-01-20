# Target Capabilities Reference

This document lists all supported targets and their capabilities.

## Desktop Targets

### macOS (aarch64-apple-darwin, x86_64-apple-darwin)

| Capability | Available | Notes |
|-----------|-----------|-------|
| filesystem | Yes | Full access |
| native_windows | Yes | NSWindow |
| gpu | Yes | Metal + WebGPU |
| threads | Yes | Native threads |
| network | Yes | URLSession |
| clipboard | Yes | NSPasteboard |
| notifications | Yes | UserNotifications |
| native_menus | Yes | NSMenu |
| system_tray | Yes | NSStatusItem |
| file_dialogs | Yes | NSOpenPanel |
| input | Yes | Mouse + keyboard |
| touch | No | Some touchscreens |
| camera | Yes | AVFoundation |
| microphone | Yes | AVFoundation |
| geolocation | No | Not typical |
| biometrics | Yes | Touch ID on supported Macs |
| push_notifications | No | Not typical for desktop |
| persistent_storage | Yes | File system |
| secure_storage | Yes | Keychain |

### Windows (x86_64-pc-windows-msvc)

| Capability | Available | Notes |
|-----------|-----------|-------|
| filesystem | Yes | Full access |
| native_windows | Yes | HWND |
| gpu | Yes | DirectX + WebGPU |
| threads | Yes | Native threads |
| network | Yes | WinHTTP |
| clipboard | Yes | Windows clipboard |
| notifications | Yes | Toast notifications |
| native_menus | Yes | Win32 menus |
| system_tray | Yes | Shell_NotifyIcon |
| file_dialogs | Yes | CommonDialogs |
| input | Yes | Mouse + keyboard |
| touch | No | Some touchscreens |
| camera | Yes | Windows Media Foundation |
| microphone | Yes | Windows Audio |
| geolocation | No | Not typical |
| biometrics | Yes | Windows Hello |
| push_notifications | No | Not typical for desktop |
| persistent_storage | Yes | File system |
| secure_storage | Yes | Credential Manager |

### Linux (x86_64-unknown-linux-gnu)

| Capability | Available | Notes |
|-----------|-----------|-------|
| filesystem | Yes | Full access |
| native_windows | Yes | X11/Wayland |
| gpu | Yes | Vulkan + WebGPU |
| threads | Yes | pthreads |
| network | Yes | libcurl/native |
| clipboard | Yes | X11/Wayland clipboard |
| notifications | Yes | libnotify |
| native_menus | Yes | GTK/Qt menus |
| system_tray | Yes | AppIndicator |
| file_dialogs | Yes | GTK/Qt dialogs |
| input | Yes | Mouse + keyboard |
| touch | No | Some touchscreens |
| camera | Yes | V4L2 |
| microphone | Yes | ALSA/PulseAudio |
| geolocation | No | Not typical |
| biometrics | No | No unified API |
| push_notifications | No | Not typical for desktop |
| persistent_storage | Yes | File system |
| secure_storage | Yes | libsecret |

## Web Target

### WASM (wasm32-unknown-unknown)

| Capability | Available | Notes |
|-----------|-----------|-------|
| filesystem | No | File System Access API limited |
| native_windows | No | Browser tabs only |
| gpu | Yes | WebGPU |
| threads | Yes | Web Workers |
| network | Yes | Fetch API |
| clipboard | Yes | Clipboard API |
| notifications | Yes | Notification API |
| native_menus | No | Not available |
| system_tray | No | Not available |
| file_dialogs | Yes | File picker |
| input | Yes | Mouse + keyboard |
| touch | Yes | Touch events |
| camera | Yes | MediaDevices |
| microphone | Yes | MediaDevices |
| geolocation | Yes | Geolocation API |
| biometrics | Yes | WebAuthn |
| push_notifications | Yes | Service Workers |
| persistent_storage | Yes | IndexedDB |
| secure_storage | No | No true secure storage |

## Mobile Targets

### iOS (aarch64-apple-ios)

| Capability | Available | Notes |
|-----------|-----------|-------|
| filesystem | Yes | App sandbox |
| native_windows | No | UIKit views |
| gpu | Yes | Metal |
| threads | Yes | Grand Central Dispatch |
| network | Yes | URLSession |
| clipboard | Yes | UIPasteboard |
| notifications | Yes | UserNotifications |
| native_menus | No | Different paradigm |
| system_tray | No | Not available |
| file_dialogs | Yes | UIDocumentPickerViewController |
| input | Yes | Touch + keyboard |
| touch | Yes | Primary input |
| camera | Yes | AVFoundation |
| microphone | Yes | AVAudioSession |
| geolocation | Yes | Core Location |
| biometrics | Yes | Face ID, Touch ID |
| push_notifications | Yes | APNs |
| persistent_storage | Yes | App sandbox |
| secure_storage | Yes | Keychain |

### Android (aarch64-linux-android)

| Capability | Available | Notes |
|-----------|-----------|-------|
| filesystem | Yes | App-specific storage |
| native_windows | No | Android views |
| gpu | Yes | Vulkan/OpenGL ES |
| threads | Yes | Java threads |
| network | Yes | OkHttp/HttpURLConnection |
| clipboard | Yes | ClipboardManager |
| notifications | Yes | NotificationManager |
| native_menus | No | Different paradigm |
| system_tray | No | Not available |
| file_dialogs | Yes | Storage Access Framework |
| input | Yes | Touch + keyboard |
| touch | Yes | Primary input |
| camera | Yes | Camera2 API |
| microphone | Yes | AudioRecord |
| geolocation | Yes | FusedLocationProvider |
| biometrics | Yes | BiometricPrompt |
| push_notifications | Yes | Firebase Cloud Messaging |
| persistent_storage | Yes | App-specific storage |
| secure_storage | Yes | Android Keystore |

## Capability Groups

### Portable Capabilities

These capabilities are available on ALL targets:

- `gpu` (WebGPU abstraction)
- `threads` (abstracted workers)
- `network` (HTTP)
- `clipboard` (with permission)
- `notifications` (with permission)
- `input` (unified mouse/touch)
- `persistent_storage` (abstracted)

### Desktop-Only Capabilities

These capabilities require a desktop OS:

- `native_windows`
- `native_menus`
- `system_tray`

### Native-Only Capabilities

These capabilities work on desktop + mobile, but not web:

- `filesystem` (full access)
- `secure_storage`

### Mobile-Specific Capabilities

These capabilities are primarily for mobile:

- `touch` (as primary input)
- `geolocation`
- `push_notifications`
- `biometrics`

## Using Capabilities in Code

```rust
use oxide_portable::{Target, PortabilityChecker};

fn check_capabilities() {
    let target = Target::current();

    // Check individual capabilities
    if target.has_capability("filesystem") {
        // Can use filesystem APIs
    }

    if target.has_capability("secure_storage") {
        // Can use keychain/keystore
    }

    // Check target family
    if target.is_desktop() {
        // Desktop-specific code
    } else if target.is_web() {
        // Web-specific code
    } else if target.is_mobile() {
        // Mobile-specific code
    }
}
```

## Programmatic Access

```rust
use oxide_portable::target::targets;

fn list_targets() {
    for target in targets::all() {
        println!("Target: {}", target.triple());
        println!("  Platform: {}", target.platform());
        println!("  Family: {}", target.family());
        println!("  Capabilities:");

        let caps = target.capabilities();
        if caps.filesystem { println!("    - filesystem"); }
        if caps.native_windows { println!("    - native_windows"); }
        if caps.gpu { println!("    - gpu"); }
        // etc.
    }
}
```
