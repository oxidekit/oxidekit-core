//! Forbidden APIs and crates for verified builds.

/// Forbidden APIs that bypass OxideKit's security controls.
pub const FORBIDDEN_APIS: &[&str] = &[
    // Direct socket APIs bypass network allowlist
    "std::net::TcpStream::connect",
    "std::net::UdpSocket::bind",
    "std::net::UdpSocket::connect",
    "tokio::net::TcpStream::connect",
    "tokio::net::UdpSocket::bind",
    "async_std::net::TcpStream::connect",
    // Direct process execution
    "std::process::Command::new",
    "std::process::Command::spawn",
    "std::process::Command::output",
    // Raw file system access outside sandbox
    "std::fs::read",
    "std::fs::write",
    "std::fs::remove_file",
    "std::fs::remove_dir_all",
    "tokio::fs::read",
    "tokio::fs::write",
    // Environment variable access (potential secrets)
    "std::env::var",
    "std::env::vars",
    // Dynamic library loading
    "libloading::Library::new",
    "dlopen::raw::Library::open",
];

/// Forbidden crates that provide capabilities outside OxideKit's control.
pub const FORBIDDEN_CRATES: &[&str] = &[
    // Direct socket libraries
    "socket2",
    "mio", // Low-level I/O (use tokio through OxideKit instead)
    // Direct camera/media access
    "nokhwa",    // Camera
    "cpal",      // Audio (use OxideKit audio plugin)
    "rodio",     // Audio
    "eye",       // Camera
    "v4l",       // Video4Linux
    "escapi",    // Windows camera
    // Screenshot/screen capture
    "screenshots",
    "scrap",
    "captrs",
    "display-info",
    // Raw system access
    "nix",       // Unix system calls
    "windows",   // Raw Windows API (use OxideKit abstractions)
    "winapi",    // Legacy Windows API
    // Keylogger-like capabilities
    "rdev",      // Input device monitoring
    "inputbot",  // Input simulation
    "enigo",     // Input simulation
    // Process injection
    "dll-syringe",
    "proc-maps",
    // Privilege escalation
    "sudo",
    "runas",
];

/// APIs that are allowed but flagged for review.
pub const FLAGGED_APIS: &[&str] = &[
    // Memory operations
    "std::mem::transmute",
    "std::ptr::read",
    "std::ptr::write",
    // Unsafe blocks
    "unsafe",
    // FFI
    "extern \"C\"",
    // Raw pointers
    "*const",
    "*mut",
];

/// Crates that are allowed but flagged for review.
pub const FLAGGED_CRATES: &[&str] = &[
    // FFI crates
    "libc",
    "cc",
    // Serialization (potential for RCE via deserialization)
    "bincode",
    "rmp-serde",
    // Cryptography (ensure proper usage)
    "openssl",
    // Regex (potential for ReDoS)
    "regex",
];

/// A detected forbidden item.
#[derive(Debug, Clone)]
pub struct ForbiddenItem {
    /// The forbidden item (API or crate name).
    pub item: String,
    /// Type of item.
    pub item_type: ForbiddenItemType,
    /// Location where it was found.
    pub location: Option<String>,
    /// Reason it's forbidden.
    pub reason: String,
    /// Suggested alternative.
    pub alternative: Option<String>,
}

/// Type of forbidden item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForbiddenItemType {
    /// Forbidden API call.
    Api,
    /// Forbidden crate dependency.
    Crate,
    /// Flagged (allowed but needs review).
    Flagged,
}

impl ForbiddenItem {
    /// Create a new forbidden API item.
    pub fn api(
        api: impl Into<String>,
        location: Option<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            item: api.into(),
            item_type: ForbiddenItemType::Api,
            location,
            reason: reason.into(),
            alternative: None,
        }
    }

    /// Create a new forbidden crate item.
    pub fn crate_dep(
        crate_name: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            item: crate_name.into(),
            item_type: ForbiddenItemType::Crate,
            location: None,
            reason: reason.into(),
            alternative: None,
        }
    }

    /// Create a flagged item.
    pub fn flagged(
        item: impl Into<String>,
        item_type: ForbiddenItemType,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            item: item.into(),
            item_type,
            location: None,
            reason: reason.into(),
            alternative: None,
        }
    }

    /// Add an alternative suggestion.
    pub fn with_alternative(mut self, alternative: impl Into<String>) -> Self {
        self.alternative = Some(alternative.into());
        self
    }

    /// Add a location.
    pub fn at_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }
}

/// Get the reason a crate is forbidden.
pub fn crate_forbidden_reason(crate_name: &str) -> Option<&'static str> {
    match crate_name {
        "socket2" => Some("Direct socket access bypasses network allowlist"),
        "mio" => Some("Low-level I/O bypasses OxideKit abstractions"),
        "nokhwa" | "eye" | "v4l" | "escapi" => Some("Direct camera access bypasses camera permission"),
        "cpal" | "rodio" => Some("Direct audio access bypasses microphone permission"),
        "screenshots" | "scrap" | "captrs" => Some("Direct screenshot access bypasses screenshot permission"),
        "nix" => Some("Direct Unix system calls bypass security controls"),
        "windows" | "winapi" => Some("Direct Windows API calls bypass security controls"),
        "rdev" | "inputbot" | "enigo" => Some("Input monitoring/simulation is a security risk"),
        "dll-syringe" | "proc-maps" => Some("Process manipulation is a security risk"),
        "sudo" | "runas" => Some("Privilege escalation is not allowed"),
        _ => None,
    }
}

/// Get the OxideKit alternative for a forbidden crate.
pub fn crate_alternative(crate_name: &str) -> Option<&'static str> {
    match crate_name {
        "socket2" | "mio" => Some("Use oxide-runtime network APIs with allowlist"),
        "nokhwa" | "eye" | "v4l" | "escapi" => Some("Use native.camera OxideKit plugin"),
        "cpal" | "rodio" => Some("Use native.audio OxideKit plugin"),
        "screenshots" | "scrap" | "captrs" => Some("Use native.screenshot OxideKit plugin"),
        "nix" | "windows" | "winapi" => Some("Use OxideKit platform abstractions"),
        _ => None,
    }
}

/// Get the reason an API is forbidden.
pub fn api_forbidden_reason(api: &str) -> Option<&'static str> {
    if api.contains("TcpStream::connect") || api.contains("UdpSocket") {
        Some("Direct socket connections bypass network allowlist")
    } else if api.contains("Command") {
        Some("Process spawning is a security risk")
    } else if api.contains("fs::") {
        Some("Direct filesystem access bypasses permission checks")
    } else if api.contains("env::var") {
        Some("Environment variable access may leak secrets")
    } else if api.contains("Library::") || api.contains("dlopen") {
        Some("Dynamic library loading is a security risk")
    } else {
        None
    }
}

/// Get the OxideKit alternative for a forbidden API.
pub fn api_alternative(api: &str) -> Option<&'static str> {
    if api.contains("TcpStream::connect") || api.contains("UdpSocket") {
        Some("Use oxide_permissions::network APIs")
    } else if api.contains("fs::read") || api.contains("fs::write") {
        Some("Use native.filesystem OxideKit plugin")
    } else if api.contains("Command") {
        Some("Consider using a sandboxed plugin or native extension")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forbidden_crates() {
        assert!(FORBIDDEN_CRATES.contains(&"socket2"));
        assert!(FORBIDDEN_CRATES.contains(&"nokhwa"));
    }

    #[test]
    fn test_forbidden_apis() {
        assert!(FORBIDDEN_APIS.contains(&"std::net::TcpStream::connect"));
        assert!(FORBIDDEN_APIS.contains(&"std::process::Command::new"));
    }

    #[test]
    fn test_crate_reasons() {
        assert!(crate_forbidden_reason("socket2").is_some());
        assert!(crate_alternative("nokhwa").is_some());
    }

    #[test]
    fn test_forbidden_item() {
        let item = ForbiddenItem::crate_dep("socket2", "Bypasses network allowlist")
            .with_alternative("Use oxide network APIs");

        assert_eq!(item.item_type, ForbiddenItemType::Crate);
        assert!(item.alternative.is_some());
    }
}
