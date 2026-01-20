//! Keyboard Handling
//!
//! Provides platform-specific keyboard behaviors including:
//! - Platform-specific shortcuts (Cmd on Mac, Ctrl on Windows/Linux)
//! - ShortcutManager with platform-aware key combinations
//! - Virtual keyboard handling (mobile)
//! - Keyboard avoidance (mobile)

use crate::detect::Platform;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Modifier keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Modifiers {
    /// Ctrl key (Windows/Linux) or Control key (Mac)
    pub ctrl: bool,
    /// Shift key
    pub shift: bool,
    /// Alt key (Windows/Linux) or Option key (Mac)
    pub alt: bool,
    /// Super/Meta key - Command on Mac, Windows key on Windows
    pub meta: bool,
}

impl Modifiers {
    /// No modifiers pressed.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: false,
            meta: false,
        }
    }

    /// Only Ctrl modifier.
    #[must_use]
    pub const fn ctrl() -> Self {
        Self {
            ctrl: true,
            shift: false,
            alt: false,
            meta: false,
        }
    }

    /// Only Shift modifier.
    #[must_use]
    pub const fn shift() -> Self {
        Self {
            ctrl: false,
            shift: true,
            alt: false,
            meta: false,
        }
    }

    /// Only Alt modifier.
    #[must_use]
    pub const fn alt() -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: true,
            meta: false,
        }
    }

    /// Only Meta/Command modifier.
    #[must_use]
    pub const fn meta() -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: false,
            meta: true,
        }
    }

    /// Ctrl + Shift modifiers.
    #[must_use]
    pub const fn ctrl_shift() -> Self {
        Self {
            ctrl: true,
            shift: true,
            alt: false,
            meta: false,
        }
    }

    /// Meta + Shift modifiers.
    #[must_use]
    pub const fn meta_shift() -> Self {
        Self {
            ctrl: false,
            shift: true,
            alt: false,
            meta: true,
        }
    }

    /// Get the "primary" modifier for a platform (Cmd on Mac, Ctrl elsewhere).
    #[must_use]
    pub fn primary(platform: Platform) -> Self {
        if platform.is_apple() {
            Self::meta()
        } else {
            Self::ctrl()
        }
    }

    /// Get the "primary + shift" modifier for a platform.
    #[must_use]
    pub fn primary_shift(platform: Platform) -> Self {
        if platform.is_apple() {
            Self::meta_shift()
        } else {
            Self::ctrl_shift()
        }
    }

    /// Check if any modifier is pressed.
    #[must_use]
    pub fn any(&self) -> bool {
        self.ctrl || self.shift || self.alt || self.meta
    }
}

impl Default for Modifiers {
    fn default() -> Self {
        Self::none()
    }
}

/// Common key codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyCode {
    // Letters
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    // Numbers
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,

    // Function keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    // Navigation
    Escape,
    Tab,
    Backspace,
    Enter,
    Space,
    Insert,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,

    // Punctuation
    Minus,
    Equal,
    BracketLeft,
    BracketRight,
    Backslash,
    Semicolon,
    Quote,
    Comma,
    Period,
    Slash,
    Backquote,

    // Other
    PrintScreen,
    ScrollLock,
    Pause,
    NumLock,
    CapsLock,

    // Unknown key
    Unknown,
}

impl std::fmt::Display for KeyCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            KeyCode::A => "A",
            KeyCode::B => "B",
            KeyCode::C => "C",
            KeyCode::D => "D",
            KeyCode::E => "E",
            KeyCode::F => "F",
            KeyCode::G => "G",
            KeyCode::H => "H",
            KeyCode::I => "I",
            KeyCode::J => "J",
            KeyCode::K => "K",
            KeyCode::L => "L",
            KeyCode::M => "M",
            KeyCode::N => "N",
            KeyCode::O => "O",
            KeyCode::P => "P",
            KeyCode::Q => "Q",
            KeyCode::R => "R",
            KeyCode::S => "S",
            KeyCode::T => "T",
            KeyCode::U => "U",
            KeyCode::V => "V",
            KeyCode::W => "W",
            KeyCode::X => "X",
            KeyCode::Y => "Y",
            KeyCode::Z => "Z",
            KeyCode::Key0 => "0",
            KeyCode::Key1 => "1",
            KeyCode::Key2 => "2",
            KeyCode::Key3 => "3",
            KeyCode::Key4 => "4",
            KeyCode::Key5 => "5",
            KeyCode::Key6 => "6",
            KeyCode::Key7 => "7",
            KeyCode::Key8 => "8",
            KeyCode::Key9 => "9",
            KeyCode::F1 => "F1",
            KeyCode::F2 => "F2",
            KeyCode::F3 => "F3",
            KeyCode::F4 => "F4",
            KeyCode::F5 => "F5",
            KeyCode::F6 => "F6",
            KeyCode::F7 => "F7",
            KeyCode::F8 => "F8",
            KeyCode::F9 => "F9",
            KeyCode::F10 => "F10",
            KeyCode::F11 => "F11",
            KeyCode::F12 => "F12",
            KeyCode::Escape => "Esc",
            KeyCode::Tab => "Tab",
            KeyCode::Backspace => "Backspace",
            KeyCode::Enter => "Enter",
            KeyCode::Space => "Space",
            KeyCode::Insert => "Insert",
            KeyCode::Delete => "Delete",
            KeyCode::Home => "Home",
            KeyCode::End => "End",
            KeyCode::PageUp => "PageUp",
            KeyCode::PageDown => "PageDown",
            KeyCode::ArrowUp => "Up",
            KeyCode::ArrowDown => "Down",
            KeyCode::ArrowLeft => "Left",
            KeyCode::ArrowRight => "Right",
            KeyCode::Minus => "-",
            KeyCode::Equal => "=",
            KeyCode::BracketLeft => "[",
            KeyCode::BracketRight => "]",
            KeyCode::Backslash => "\\",
            KeyCode::Semicolon => ";",
            KeyCode::Quote => "'",
            KeyCode::Comma => ",",
            KeyCode::Period => ".",
            KeyCode::Slash => "/",
            KeyCode::Backquote => "`",
            _ => "?",
        };
        write!(f, "{}", name)
    }
}

/// A keyboard shortcut combination.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyboardShortcut {
    /// The key code
    pub key: KeyCode,
    /// The modifier keys
    pub modifiers: Modifiers,
}

impl KeyboardShortcut {
    /// Create a new keyboard shortcut.
    #[must_use]
    pub fn new(key: KeyCode, modifiers: Modifiers) -> Self {
        Self { key, modifiers }
    }

    /// Create a shortcut with just a key (no modifiers).
    #[must_use]
    pub fn key(key: KeyCode) -> Self {
        Self {
            key,
            modifiers: Modifiers::none(),
        }
    }

    /// Get the display string for this shortcut on a platform.
    #[must_use]
    pub fn display(&self, platform: Platform) -> String {
        let mut parts = Vec::new();

        if platform.is_apple() {
            // Mac uses symbols
            if self.modifiers.ctrl {
                parts.push("^");
            }
            if self.modifiers.alt {
                parts.push("\u{2325}"); // Option symbol
            }
            if self.modifiers.shift {
                parts.push("\u{21E7}"); // Shift symbol
            }
            if self.modifiers.meta {
                parts.push("\u{2318}"); // Command symbol
            }
        } else {
            // Windows/Linux use text
            if self.modifiers.ctrl {
                parts.push("Ctrl+");
            }
            if self.modifiers.alt {
                parts.push("Alt+");
            }
            if self.modifiers.shift {
                parts.push("Shift+");
            }
            if self.modifiers.meta {
                parts.push("Win+");
            }
        }

        let key_str = self.key.to_string();
        parts.push(&key_str);
        parts.concat()
    }
}

/// Common shortcuts that adapt to the platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommonShortcut {
    /// Save (Cmd+S / Ctrl+S)
    Save,
    /// Save As (Cmd+Shift+S / Ctrl+Shift+S)
    SaveAs,
    /// Open (Cmd+O / Ctrl+O)
    Open,
    /// New (Cmd+N / Ctrl+N)
    New,
    /// Close (Cmd+W / Ctrl+W)
    Close,
    /// Quit (Cmd+Q / Alt+F4)
    Quit,
    /// Undo (Cmd+Z / Ctrl+Z)
    Undo,
    /// Redo (Cmd+Shift+Z / Ctrl+Y)
    Redo,
    /// Cut (Cmd+X / Ctrl+X)
    Cut,
    /// Copy (Cmd+C / Ctrl+C)
    Copy,
    /// Paste (Cmd+V / Ctrl+V)
    Paste,
    /// Select All (Cmd+A / Ctrl+A)
    SelectAll,
    /// Find (Cmd+F / Ctrl+F)
    Find,
    /// Find Next (Cmd+G / F3)
    FindNext,
    /// Find Previous (Cmd+Shift+G / Shift+F3)
    FindPrevious,
    /// Replace (Cmd+Option+F / Ctrl+H)
    Replace,
    /// Print (Cmd+P / Ctrl+P)
    Print,
    /// Bold (Cmd+B / Ctrl+B)
    Bold,
    /// Italic (Cmd+I / Ctrl+I)
    Italic,
    /// Underline (Cmd+U / Ctrl+U)
    Underline,
    /// Preferences/Settings (Cmd+, / Ctrl+,)
    Preferences,
    /// Zoom In (Cmd++ / Ctrl++)
    ZoomIn,
    /// Zoom Out (Cmd+- / Ctrl+-)
    ZoomOut,
    /// Reset Zoom (Cmd+0 / Ctrl+0)
    ZoomReset,
    /// Full Screen (Cmd+Ctrl+F / F11)
    FullScreen,
    /// Minimize (Cmd+M / Win+Down)
    Minimize,
    /// Refresh (Cmd+R / F5)
    Refresh,
}

impl CommonShortcut {
    /// Get the keyboard shortcut for a platform.
    #[must_use]
    pub fn for_platform(&self, platform: Platform) -> KeyboardShortcut {
        let is_mac = platform.is_apple();

        match self {
            CommonShortcut::Save => KeyboardShortcut::new(
                KeyCode::S,
                Modifiers::primary(platform),
            ),
            CommonShortcut::SaveAs => KeyboardShortcut::new(
                KeyCode::S,
                Modifiers::primary_shift(platform),
            ),
            CommonShortcut::Open => KeyboardShortcut::new(
                KeyCode::O,
                Modifiers::primary(platform),
            ),
            CommonShortcut::New => KeyboardShortcut::new(
                KeyCode::N,
                Modifiers::primary(platform),
            ),
            CommonShortcut::Close => KeyboardShortcut::new(
                KeyCode::W,
                Modifiers::primary(platform),
            ),
            CommonShortcut::Quit => {
                if is_mac {
                    KeyboardShortcut::new(KeyCode::Q, Modifiers::meta())
                } else {
                    KeyboardShortcut::new(KeyCode::F4, Modifiers::alt())
                }
            }
            CommonShortcut::Undo => KeyboardShortcut::new(
                KeyCode::Z,
                Modifiers::primary(platform),
            ),
            CommonShortcut::Redo => {
                if is_mac {
                    KeyboardShortcut::new(KeyCode::Z, Modifiers::meta_shift())
                } else {
                    KeyboardShortcut::new(KeyCode::Y, Modifiers::ctrl())
                }
            }
            CommonShortcut::Cut => KeyboardShortcut::new(
                KeyCode::X,
                Modifiers::primary(platform),
            ),
            CommonShortcut::Copy => KeyboardShortcut::new(
                KeyCode::C,
                Modifiers::primary(platform),
            ),
            CommonShortcut::Paste => KeyboardShortcut::new(
                KeyCode::V,
                Modifiers::primary(platform),
            ),
            CommonShortcut::SelectAll => KeyboardShortcut::new(
                KeyCode::A,
                Modifiers::primary(platform),
            ),
            CommonShortcut::Find => KeyboardShortcut::new(
                KeyCode::F,
                Modifiers::primary(platform),
            ),
            CommonShortcut::FindNext => {
                if is_mac {
                    KeyboardShortcut::new(KeyCode::G, Modifiers::meta())
                } else {
                    KeyboardShortcut::key(KeyCode::F3)
                }
            }
            CommonShortcut::FindPrevious => {
                if is_mac {
                    KeyboardShortcut::new(KeyCode::G, Modifiers::meta_shift())
                } else {
                    KeyboardShortcut::new(KeyCode::F3, Modifiers::shift())
                }
            }
            CommonShortcut::Replace => {
                if is_mac {
                    KeyboardShortcut::new(
                        KeyCode::F,
                        Modifiers {
                            ctrl: false,
                            shift: false,
                            alt: true,
                            meta: true,
                        },
                    )
                } else {
                    KeyboardShortcut::new(KeyCode::H, Modifiers::ctrl())
                }
            }
            CommonShortcut::Print => KeyboardShortcut::new(
                KeyCode::P,
                Modifiers::primary(platform),
            ),
            CommonShortcut::Bold => KeyboardShortcut::new(
                KeyCode::B,
                Modifiers::primary(platform),
            ),
            CommonShortcut::Italic => KeyboardShortcut::new(
                KeyCode::I,
                Modifiers::primary(platform),
            ),
            CommonShortcut::Underline => KeyboardShortcut::new(
                KeyCode::U,
                Modifiers::primary(platform),
            ),
            CommonShortcut::Preferences => KeyboardShortcut::new(
                KeyCode::Comma,
                Modifiers::primary(platform),
            ),
            CommonShortcut::ZoomIn => KeyboardShortcut::new(
                KeyCode::Equal,
                Modifiers::primary(platform),
            ),
            CommonShortcut::ZoomOut => KeyboardShortcut::new(
                KeyCode::Minus,
                Modifiers::primary(platform),
            ),
            CommonShortcut::ZoomReset => KeyboardShortcut::new(
                KeyCode::Key0,
                Modifiers::primary(platform),
            ),
            CommonShortcut::FullScreen => {
                if is_mac {
                    KeyboardShortcut::new(
                        KeyCode::F,
                        Modifiers {
                            ctrl: true,
                            shift: false,
                            alt: false,
                            meta: true,
                        },
                    )
                } else {
                    KeyboardShortcut::key(KeyCode::F11)
                }
            }
            CommonShortcut::Minimize => {
                if is_mac {
                    KeyboardShortcut::new(KeyCode::M, Modifiers::meta())
                } else {
                    KeyboardShortcut::new(KeyCode::ArrowDown, Modifiers::meta())
                }
            }
            CommonShortcut::Refresh => {
                if is_mac {
                    KeyboardShortcut::new(KeyCode::R, Modifiers::meta())
                } else {
                    KeyboardShortcut::key(KeyCode::F5)
                }
            }
        }
    }

    /// Get the keyboard shortcut for the current platform.
    #[must_use]
    pub fn current(&self) -> KeyboardShortcut {
        self.for_platform(Platform::current())
    }
}

/// Shortcut manager that handles platform-aware keyboard shortcuts.
#[derive(Debug, Clone)]
pub struct ShortcutManager {
    /// Registered shortcuts
    shortcuts: HashMap<String, KeyboardShortcut>,
    /// Platform for shortcut resolution
    platform: Platform,
}

impl ShortcutManager {
    /// Create a new shortcut manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            shortcuts: HashMap::new(),
            platform: Platform::current(),
        }
    }

    /// Create a new shortcut manager for a specific platform.
    #[must_use]
    pub fn for_platform(platform: Platform) -> Self {
        Self {
            shortcuts: HashMap::new(),
            platform,
        }
    }

    /// Register a common shortcut with an action name.
    pub fn register_common(&mut self, name: impl Into<String>, shortcut: CommonShortcut) {
        let kb_shortcut = shortcut.for_platform(self.platform);
        self.shortcuts.insert(name.into(), kb_shortcut);
    }

    /// Register a custom shortcut.
    pub fn register(&mut self, name: impl Into<String>, shortcut: KeyboardShortcut) {
        self.shortcuts.insert(name.into(), shortcut);
    }

    /// Unregister a shortcut.
    pub fn unregister(&mut self, name: &str) -> Option<KeyboardShortcut> {
        self.shortcuts.remove(name)
    }

    /// Check if a key event matches any registered shortcut.
    pub fn matches(&self, key: KeyCode, modifiers: Modifiers) -> Option<&str> {
        for (name, shortcut) in &self.shortcuts {
            if shortcut.key == key && shortcut.modifiers == modifiers {
                return Some(name.as_str());
            }
        }
        None
    }

    /// Get a shortcut by name.
    pub fn get(&self, name: &str) -> Option<&KeyboardShortcut> {
        self.shortcuts.get(name)
    }

    /// Get all registered shortcuts.
    pub fn all(&self) -> &HashMap<String, KeyboardShortcut> {
        &self.shortcuts
    }

    /// Get the display string for a shortcut by name.
    pub fn display(&self, name: &str) -> Option<String> {
        self.shortcuts
            .get(name)
            .map(|s| s.display(self.platform))
    }

    /// Create a manager with common shortcuts pre-registered.
    #[must_use]
    pub fn with_common_shortcuts() -> Self {
        let mut manager = Self::new();
        manager.register_common("save", CommonShortcut::Save);
        manager.register_common("save_as", CommonShortcut::SaveAs);
        manager.register_common("open", CommonShortcut::Open);
        manager.register_common("new", CommonShortcut::New);
        manager.register_common("close", CommonShortcut::Close);
        manager.register_common("quit", CommonShortcut::Quit);
        manager.register_common("undo", CommonShortcut::Undo);
        manager.register_common("redo", CommonShortcut::Redo);
        manager.register_common("cut", CommonShortcut::Cut);
        manager.register_common("copy", CommonShortcut::Copy);
        manager.register_common("paste", CommonShortcut::Paste);
        manager.register_common("select_all", CommonShortcut::SelectAll);
        manager.register_common("find", CommonShortcut::Find);
        manager
    }
}

impl Default for ShortcutManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Virtual keyboard state (mobile).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VirtualKeyboardState {
    /// Whether the virtual keyboard is visible
    pub is_visible: bool,
    /// Height of the keyboard (in pixels)
    pub height: f32,
    /// Type of keyboard being shown
    pub keyboard_type: VirtualKeyboardType,
    /// Animation duration for showing/hiding (ms)
    pub animation_duration_ms: u32,
}

impl VirtualKeyboardState {
    /// Create a new virtual keyboard state (hidden).
    #[must_use]
    pub fn new() -> Self {
        Self {
            is_visible: false,
            height: 0.0,
            keyboard_type: VirtualKeyboardType::Default,
            animation_duration_ms: 250,
        }
    }

    /// Show the keyboard.
    pub fn show(&mut self, height: f32, keyboard_type: VirtualKeyboardType) {
        self.is_visible = true;
        self.height = height;
        self.keyboard_type = keyboard_type;
    }

    /// Hide the keyboard.
    pub fn hide(&mut self) {
        self.is_visible = false;
        self.height = 0.0;
    }
}

impl Default for VirtualKeyboardState {
    fn default() -> Self {
        Self::new()
    }
}

/// Virtual keyboard types for mobile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VirtualKeyboardType {
    /// Default/text keyboard
    Default,
    /// Numeric keypad
    Numeric,
    /// Decimal pad (numbers with decimal point)
    Decimal,
    /// Phone pad
    Phone,
    /// Email address keyboard
    Email,
    /// URL keyboard
    Url,
    /// Password keyboard
    Password,
    /// Search keyboard
    Search,
}

/// Keyboard avoidance mode for mobile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyboardAvoidanceMode {
    /// Resize the view to fit above keyboard
    Resize,
    /// Scroll to keep focused element visible
    Scroll,
    /// Pad the bottom of the view
    Padding,
    /// No automatic avoidance (manual handling)
    None,
}

impl Default for KeyboardAvoidanceMode {
    fn default() -> Self {
        Self::Resize
    }
}

/// Keyboard avoidance configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyboardAvoidanceConfig {
    /// Avoidance mode
    pub mode: KeyboardAvoidanceMode,
    /// Extra padding above keyboard (pixels)
    pub extra_padding: f32,
    /// Animate the avoidance
    pub animate: bool,
    /// Animation duration (ms)
    pub animation_duration_ms: u32,
}

impl Default for KeyboardAvoidanceConfig {
    fn default() -> Self {
        Self {
            mode: KeyboardAvoidanceMode::Resize,
            extra_padding: 16.0,
            animate: true,
            animation_duration_ms: 250,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modifiers_primary() {
        let mac_primary = Modifiers::primary(Platform::MacOS);
        assert!(mac_primary.meta);
        assert!(!mac_primary.ctrl);

        let win_primary = Modifiers::primary(Platform::Windows);
        assert!(win_primary.ctrl);
        assert!(!win_primary.meta);
    }

    #[test]
    fn test_keyboard_shortcut_display_mac() {
        let shortcut = KeyboardShortcut::new(KeyCode::S, Modifiers::meta());
        let display = shortcut.display(Platform::MacOS);
        assert!(display.contains('\u{2318}')); // Command symbol
        assert!(display.contains('S'));
    }

    #[test]
    fn test_keyboard_shortcut_display_windows() {
        let shortcut = KeyboardShortcut::new(KeyCode::S, Modifiers::ctrl());
        let display = shortcut.display(Platform::Windows);
        assert!(display.contains("Ctrl"));
        assert!(display.contains('S'));
    }

    #[test]
    fn test_common_shortcut_save() {
        let mac_save = CommonShortcut::Save.for_platform(Platform::MacOS);
        assert_eq!(mac_save.key, KeyCode::S);
        assert!(mac_save.modifiers.meta);

        let win_save = CommonShortcut::Save.for_platform(Platform::Windows);
        assert_eq!(win_save.key, KeyCode::S);
        assert!(win_save.modifiers.ctrl);
    }

    #[test]
    fn test_common_shortcut_redo_differs() {
        let mac_redo = CommonShortcut::Redo.for_platform(Platform::MacOS);
        assert_eq!(mac_redo.key, KeyCode::Z);
        assert!(mac_redo.modifiers.meta);
        assert!(mac_redo.modifiers.shift);

        let win_redo = CommonShortcut::Redo.for_platform(Platform::Windows);
        assert_eq!(win_redo.key, KeyCode::Y);
        assert!(win_redo.modifiers.ctrl);
    }

    #[test]
    fn test_shortcut_manager_register() {
        let mut manager = ShortcutManager::for_platform(Platform::MacOS);
        manager.register_common("save", CommonShortcut::Save);

        let shortcut = manager.get("save").unwrap();
        assert_eq!(shortcut.key, KeyCode::S);
        assert!(shortcut.modifiers.meta);
    }

    #[test]
    fn test_shortcut_manager_matches() {
        let mut manager = ShortcutManager::for_platform(Platform::Windows);
        manager.register_common("copy", CommonShortcut::Copy);
        manager.register_common("paste", CommonShortcut::Paste);

        let matched = manager.matches(KeyCode::C, Modifiers::ctrl());
        assert_eq!(matched, Some("copy"));

        let matched = manager.matches(KeyCode::V, Modifiers::ctrl());
        assert_eq!(matched, Some("paste"));

        let matched = manager.matches(KeyCode::C, Modifiers::none());
        assert_eq!(matched, None);
    }

    #[test]
    fn test_shortcut_manager_with_common() {
        let manager = ShortcutManager::with_common_shortcuts();
        assert!(manager.get("save").is_some());
        assert!(manager.get("copy").is_some());
        assert!(manager.get("undo").is_some());
    }

    #[test]
    fn test_virtual_keyboard_state() {
        let mut state = VirtualKeyboardState::new();
        assert!(!state.is_visible);

        state.show(300.0, VirtualKeyboardType::Default);
        assert!(state.is_visible);
        assert!((state.height - 300.0).abs() < f32::EPSILON);

        state.hide();
        assert!(!state.is_visible);
    }

    #[test]
    fn test_quit_shortcut_differs() {
        let mac_quit = CommonShortcut::Quit.for_platform(Platform::MacOS);
        assert_eq!(mac_quit.key, KeyCode::Q);
        assert!(mac_quit.modifiers.meta);

        let win_quit = CommonShortcut::Quit.for_platform(Platform::Windows);
        assert_eq!(win_quit.key, KeyCode::F4);
        assert!(win_quit.modifiers.alt);
    }
}
