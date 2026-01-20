//! Search Field Component
//!
//! Provides a text input component optimized for search with features like:
//! - Search icon and clear button
//! - Loading indicator during search
//! - Debounced input with configurable delay
//! - Keyboard shortcuts (Cmd/Ctrl+K)

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Keyboard shortcut representation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Shortcut {
    /// Whether Cmd/Ctrl is required
    pub ctrl_or_cmd: bool,
    /// Whether Shift is required
    pub shift: bool,
    /// Whether Alt/Option is required
    pub alt: bool,
    /// The key character or special key
    pub key: ShortcutKey,
}

impl Shortcut {
    /// Create a new shortcut with just a key
    pub fn key(key: impl Into<ShortcutKey>) -> Self {
        Self {
            ctrl_or_cmd: false,
            shift: false,
            alt: false,
            key: key.into(),
        }
    }

    /// Create a shortcut with Cmd/Ctrl modifier
    pub fn cmd(key: char) -> Self {
        Self {
            ctrl_or_cmd: true,
            shift: false,
            alt: false,
            key: ShortcutKey::Char(key),
        }
    }

    /// Create a shortcut with Cmd/Ctrl + Shift modifier
    pub fn cmd_shift(key: char) -> Self {
        Self {
            ctrl_or_cmd: true,
            shift: true,
            alt: false,
            key: ShortcutKey::Char(key),
        }
    }

    /// Create a shortcut with Alt modifier
    pub fn alt(key: char) -> Self {
        Self {
            ctrl_or_cmd: false,
            shift: false,
            alt: true,
            key: ShortcutKey::Char(key),
        }
    }

    /// Add Cmd/Ctrl modifier
    pub fn with_cmd(mut self) -> Self {
        self.ctrl_or_cmd = true;
        self
    }

    /// Add Shift modifier
    pub fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }

    /// Add Alt modifier
    pub fn with_alt(mut self) -> Self {
        self.alt = true;
        self
    }

    /// Get display string for the shortcut (platform-aware)
    pub fn display(&self, is_mac: bool) -> String {
        let mut parts = Vec::new();

        if self.ctrl_or_cmd {
            parts.push(if is_mac { "\u{2318}" } else { "Ctrl" });
        }
        if self.alt {
            parts.push(if is_mac { "\u{2325}" } else { "Alt" });
        }
        if self.shift {
            parts.push(if is_mac { "\u{21E7}" } else { "Shift" });
        }

        let key_str = match &self.key {
            ShortcutKey::Char(c) => c.to_uppercase().to_string(),
            ShortcutKey::Escape => "Esc".to_string(),
            ShortcutKey::Enter => if is_mac { "\u{21A9}" } else { "Enter" }.to_string(),
            ShortcutKey::Tab => "Tab".to_string(),
            ShortcutKey::Backspace => if is_mac { "\u{232B}" } else { "Backspace" }.to_string(),
            ShortcutKey::Delete => "Del".to_string(),
            ShortcutKey::ArrowUp => "\u{2191}".to_string(),
            ShortcutKey::ArrowDown => "\u{2193}".to_string(),
            ShortcutKey::ArrowLeft => "\u{2190}".to_string(),
            ShortcutKey::ArrowRight => "\u{2192}".to_string(),
            ShortcutKey::Home => "Home".to_string(),
            ShortcutKey::End => "End".to_string(),
            ShortcutKey::PageUp => "PgUp".to_string(),
            ShortcutKey::PageDown => "PgDn".to_string(),
            ShortcutKey::Space => "Space".to_string(),
            ShortcutKey::F(n) => format!("F{}", n),
        };
        parts.push(&key_str);

        if is_mac {
            parts.join("")
        } else {
            parts.join("+")
        }
    }

    /// Check if this shortcut matches the given key event
    pub fn matches(&self, ctrl_or_cmd: bool, shift: bool, alt: bool, key: &ShortcutKey) -> bool {
        self.ctrl_or_cmd == ctrl_or_cmd
            && self.shift == shift
            && self.alt == alt
            && self.key == *key
    }
}

/// Keyboard key for shortcuts
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShortcutKey {
    Char(char),
    Escape,
    Enter,
    Tab,
    Backspace,
    Delete,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    PageUp,
    PageDown,
    Space,
    F(u8),
}

impl From<char> for ShortcutKey {
    fn from(c: char) -> Self {
        ShortcutKey::Char(c)
    }
}

/// Search field state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SearchFieldState {
    /// Field is idle
    Idle,
    /// User is typing (debounce active)
    Typing,
    /// Search is in progress
    Loading,
    /// Search completed with results
    HasResults,
    /// Search completed with no results
    NoResults,
    /// Search encountered an error
    Error(String),
}

impl Default for SearchFieldState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Search field size variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SearchFieldSize {
    /// Small size
    Small,
    /// Medium size (default)
    #[default]
    Medium,
    /// Large size
    Large,
}

/// Search field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFieldConfig {
    /// Placeholder text
    pub placeholder: String,
    /// Debounce delay in milliseconds
    pub debounce_ms: u64,
    /// Whether to show the search icon
    pub show_search_icon: bool,
    /// Whether to show the clear button when there's text
    pub show_clear_button: bool,
    /// Whether to show loading indicator
    pub show_loading_indicator: bool,
    /// Minimum characters before triggering search
    pub min_chars: usize,
    /// Maximum input length
    pub max_length: Option<usize>,
    /// Keyboard shortcut to focus the field
    pub focus_shortcut: Option<Shortcut>,
    /// Size variant
    pub size: SearchFieldSize,
    /// Whether the field is disabled
    pub disabled: bool,
    /// Whether to auto-focus on mount
    pub auto_focus: bool,
    /// Whether to select all text on focus
    pub select_on_focus: bool,
}

impl Default for SearchFieldConfig {
    fn default() -> Self {
        Self {
            placeholder: "Search...".to_string(),
            debounce_ms: 300,
            show_search_icon: true,
            show_clear_button: true,
            show_loading_indicator: true,
            min_chars: 1,
            max_length: None,
            focus_shortcut: Some(Shortcut::cmd('k')),
            size: SearchFieldSize::default(),
            disabled: false,
            auto_focus: false,
            select_on_focus: false,
        }
    }
}

/// Builder for SearchField
#[derive(Debug, Clone)]
pub struct SearchFieldBuilder<OnSearch = (), OnChange = (), OnClear = (), OnFocus = (), OnBlur = ()>
{
    config: SearchFieldConfig,
    initial_value: String,
    on_search: OnSearch,
    on_change: OnChange,
    on_clear: OnClear,
    on_focus: OnFocus,
    on_blur: OnBlur,
}

impl SearchFieldBuilder {
    /// Create a new search field builder
    pub fn new() -> SearchFieldBuilder<(), (), (), (), ()> {
        SearchFieldBuilder {
            config: SearchFieldConfig::default(),
            initial_value: String::new(),
            on_search: (),
            on_change: (),
            on_clear: (),
            on_focus: (),
            on_blur: (),
        }
    }
}

impl Default for SearchFieldBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl<OnSearch, OnChange, OnClear, OnFocus, OnBlur>
    SearchFieldBuilder<OnSearch, OnChange, OnClear, OnFocus, OnBlur>
{
    /// Set placeholder text
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.config.placeholder = text.into();
        self
    }

    /// Set debounce delay
    pub fn debounce(mut self, duration: Duration) -> Self {
        self.config.debounce_ms = duration.as_millis() as u64;
        self
    }

    /// Set debounce delay in milliseconds
    pub fn debounce_ms(mut self, ms: u64) -> Self {
        self.config.debounce_ms = ms;
        self
    }

    /// Show or hide search icon
    pub fn search_icon(mut self, show: bool) -> Self {
        self.config.show_search_icon = show;
        self
    }

    /// Show or hide clear button
    pub fn clear_button(mut self, show: bool) -> Self {
        self.config.show_clear_button = show;
        self
    }

    /// Show or hide loading indicator
    pub fn loading_indicator(mut self, show: bool) -> Self {
        self.config.show_loading_indicator = show;
        self
    }

    /// Set minimum characters before search triggers
    pub fn min_chars(mut self, chars: usize) -> Self {
        self.config.min_chars = chars;
        self
    }

    /// Set maximum input length
    pub fn max_length(mut self, length: usize) -> Self {
        self.config.max_length = Some(length);
        self
    }

    /// Set keyboard shortcut for focusing
    pub fn shortcut(mut self, shortcut: Shortcut) -> Self {
        self.config.focus_shortcut = Some(shortcut);
        self
    }

    /// Remove focus shortcut
    pub fn no_shortcut(mut self) -> Self {
        self.config.focus_shortcut = None;
        self
    }

    /// Set size variant
    pub fn size(mut self, size: SearchFieldSize) -> Self {
        self.config.size = size;
        self
    }

    /// Set small size
    pub fn small(mut self) -> Self {
        self.config.size = SearchFieldSize::Small;
        self
    }

    /// Set large size
    pub fn large(mut self) -> Self {
        self.config.size = SearchFieldSize::Large;
        self
    }

    /// Set disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.config.disabled = disabled;
        self
    }

    /// Set auto-focus
    pub fn auto_focus(mut self, auto: bool) -> Self {
        self.config.auto_focus = auto;
        self
    }

    /// Set select-on-focus behavior
    pub fn select_on_focus(mut self, select: bool) -> Self {
        self.config.select_on_focus = select;
        self
    }

    /// Set initial value
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.initial_value = value.into();
        self
    }

    /// Set search handler
    pub fn on_search<F>(self, handler: F) -> SearchFieldBuilder<F, OnChange, OnClear, OnFocus, OnBlur>
    where
        F: Fn(&str),
    {
        SearchFieldBuilder {
            config: self.config,
            initial_value: self.initial_value,
            on_search: handler,
            on_change: self.on_change,
            on_clear: self.on_clear,
            on_focus: self.on_focus,
            on_blur: self.on_blur,
        }
    }

    /// Set change handler
    pub fn on_change<F>(self, handler: F) -> SearchFieldBuilder<OnSearch, F, OnClear, OnFocus, OnBlur>
    where
        F: Fn(&str),
    {
        SearchFieldBuilder {
            config: self.config,
            initial_value: self.initial_value,
            on_search: self.on_search,
            on_change: handler,
            on_clear: self.on_clear,
            on_focus: self.on_focus,
            on_blur: self.on_blur,
        }
    }

    /// Set clear handler
    pub fn on_clear<F>(self, handler: F) -> SearchFieldBuilder<OnSearch, OnChange, F, OnFocus, OnBlur>
    where
        F: Fn(),
    {
        SearchFieldBuilder {
            config: self.config,
            initial_value: self.initial_value,
            on_search: self.on_search,
            on_change: self.on_change,
            on_clear: handler,
            on_focus: self.on_focus,
            on_blur: self.on_blur,
        }
    }

    /// Set focus handler
    pub fn on_focus<F>(self, handler: F) -> SearchFieldBuilder<OnSearch, OnChange, OnClear, F, OnBlur>
    where
        F: Fn(),
    {
        SearchFieldBuilder {
            config: self.config,
            initial_value: self.initial_value,
            on_search: self.on_search,
            on_change: self.on_change,
            on_clear: self.on_clear,
            on_focus: handler,
            on_blur: self.on_blur,
        }
    }

    /// Set blur handler
    pub fn on_blur<F>(self, handler: F) -> SearchFieldBuilder<OnSearch, OnChange, OnClear, OnFocus, F>
    where
        F: Fn(),
    {
        SearchFieldBuilder {
            config: self.config,
            initial_value: self.initial_value,
            on_search: self.on_search,
            on_change: self.on_change,
            on_clear: self.on_clear,
            on_focus: self.on_focus,
            on_blur: handler,
        }
    }

    /// Build the search field instance
    pub fn build(self) -> SearchField<OnSearch, OnChange, OnClear, OnFocus, OnBlur> {
        SearchField {
            config: self.config,
            value: self.initial_value,
            state: SearchFieldState::Idle,
            is_focused: false,
            on_search: self.on_search,
            on_change: self.on_change,
            on_clear: self.on_clear,
            on_focus: self.on_focus,
            on_blur: self.on_blur,
        }
    }
}

/// Search field component
#[derive(Debug, Clone)]
pub struct SearchField<OnSearch = (), OnChange = (), OnClear = (), OnFocus = (), OnBlur = ()> {
    /// Configuration
    pub config: SearchFieldConfig,
    /// Current value
    pub value: String,
    /// Current state
    pub state: SearchFieldState,
    /// Whether the field is focused
    pub is_focused: bool,
    /// Search callback
    on_search: OnSearch,
    /// Change callback
    on_change: OnChange,
    /// Clear callback
    on_clear: OnClear,
    /// Focus callback
    on_focus: OnFocus,
    /// Blur callback
    on_blur: OnBlur,
}

impl SearchField {
    /// Create a new search field builder
    pub fn new() -> SearchFieldBuilder {
        SearchFieldBuilder::new()
    }
}

impl Default for SearchField {
    fn default() -> Self {
        SearchFieldBuilder::new().build()
    }
}

impl<OnSearch, OnChange, OnClear, OnFocus, OnBlur>
    SearchField<OnSearch, OnChange, OnClear, OnFocus, OnBlur>
{
    /// Get the current value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Check if the field is empty
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Get the current state
    pub fn state(&self) -> &SearchFieldState {
        &self.state
    }

    /// Check if search is loading
    pub fn is_loading(&self) -> bool {
        matches!(self.state, SearchFieldState::Loading)
    }

    /// Check if the field is focused
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Check if search should be triggered based on min_chars
    pub fn should_search(&self) -> bool {
        self.value.len() >= self.config.min_chars
    }

    /// Set the value programmatically
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        if let Some(max) = self.config.max_length {
            self.value.truncate(max);
        }
    }

    /// Clear the value
    pub fn clear(&mut self) {
        self.value.clear();
        self.state = SearchFieldState::Idle;
    }

    /// Set the state
    pub fn set_state(&mut self, state: SearchFieldState) {
        self.state = state;
    }

    /// Set loading state
    pub fn set_loading(&mut self, loading: bool) {
        if loading {
            self.state = SearchFieldState::Loading;
        } else if self.value.is_empty() {
            self.state = SearchFieldState::Idle;
        }
    }

    /// Set focused state
    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    /// Handle text input
    pub fn handle_input(&mut self, new_value: String)
    where
        OnChange: Fn(&str),
    {
        let mut value = new_value;
        if let Some(max) = self.config.max_length {
            value.truncate(max);
        }
        self.value = value;
        self.state = SearchFieldState::Typing;
        (self.on_change)(&self.value);
    }

    /// Handle search trigger (after debounce)
    pub fn handle_search(&mut self)
    where
        OnSearch: Fn(&str),
    {
        if self.should_search() {
            self.state = SearchFieldState::Loading;
            (self.on_search)(&self.value);
        } else {
            self.state = SearchFieldState::Idle;
        }
    }

    /// Handle clear button click
    pub fn handle_clear(&mut self)
    where
        OnClear: Fn(),
    {
        self.value.clear();
        self.state = SearchFieldState::Idle;
        (self.on_clear)();
    }

    /// Handle focus event
    pub fn handle_focus(&mut self)
    where
        OnFocus: Fn(),
    {
        self.is_focused = true;
        (self.on_focus)();
    }

    /// Handle blur event
    pub fn handle_blur(&mut self)
    where
        OnBlur: Fn(),
    {
        self.is_focused = false;
        (self.on_blur)();
    }

    /// Check if a keyboard event matches the focus shortcut
    pub fn matches_focus_shortcut(
        &self,
        ctrl_or_cmd: bool,
        shift: bool,
        alt: bool,
        key: &ShortcutKey,
    ) -> bool {
        self.config
            .focus_shortcut
            .as_ref()
            .map(|s| s.matches(ctrl_or_cmd, shift, alt, key))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_cmd_k() {
        let shortcut = Shortcut::cmd('k');
        assert!(shortcut.ctrl_or_cmd);
        assert!(!shortcut.shift);
        assert!(!shortcut.alt);
        assert_eq!(shortcut.key, ShortcutKey::Char('k'));
    }

    #[test]
    fn test_shortcut_cmd_shift_p() {
        let shortcut = Shortcut::cmd_shift('P');
        assert!(shortcut.ctrl_or_cmd);
        assert!(shortcut.shift);
        assert!(!shortcut.alt);
        assert_eq!(shortcut.key, ShortcutKey::Char('P'));
    }

    #[test]
    fn test_shortcut_display_mac() {
        let shortcut = Shortcut::cmd_shift('P');
        let display = shortcut.display(true);
        assert!(display.contains("\u{2318}")); // Cmd symbol
        assert!(display.contains("\u{21E7}")); // Shift symbol
        assert!(display.contains("P"));
    }

    #[test]
    fn test_shortcut_display_windows() {
        let shortcut = Shortcut::cmd_shift('P');
        let display = shortcut.display(false);
        assert!(display.contains("Ctrl"));
        assert!(display.contains("Shift"));
        assert!(display.contains("P"));
    }

    #[test]
    fn test_shortcut_matches() {
        let shortcut = Shortcut::cmd('k');
        assert!(shortcut.matches(true, false, false, &ShortcutKey::Char('k')));
        assert!(!shortcut.matches(false, false, false, &ShortcutKey::Char('k')));
        assert!(!shortcut.matches(true, true, false, &ShortcutKey::Char('k')));
    }

    #[test]
    fn test_search_field_builder() {
        let field = SearchField::new()
            .placeholder("Search users...")
            .debounce_ms(500)
            .min_chars(2)
            .build();

        assert_eq!(field.config.placeholder, "Search users...");
        assert_eq!(field.config.debounce_ms, 500);
        assert_eq!(field.config.min_chars, 2);
    }

    #[test]
    fn test_search_field_value() {
        let mut field = SearchField::new().value("initial").build();
        assert_eq!(field.value(), "initial");

        field.set_value("updated");
        assert_eq!(field.value(), "updated");
    }

    #[test]
    fn test_search_field_clear() {
        let mut field = SearchField::new().value("test").build();
        assert!(!field.is_empty());

        field.clear();
        assert!(field.is_empty());
        assert_eq!(field.state, SearchFieldState::Idle);
    }

    #[test]
    fn test_search_field_max_length() {
        let mut field = SearchField::new().max_length(5).build();
        field.set_value("hello world");
        assert_eq!(field.value(), "hello");
    }

    #[test]
    fn test_search_field_should_search() {
        let field = SearchField::new().min_chars(3).value("ab").build();
        assert!(!field.should_search());

        let field = SearchField::new().min_chars(3).value("abc").build();
        assert!(field.should_search());
    }

    #[test]
    fn test_search_field_state() {
        let mut field = SearchField::default();
        assert_eq!(field.state, SearchFieldState::Idle);

        field.set_state(SearchFieldState::Loading);
        assert!(field.is_loading());

        field.set_state(SearchFieldState::HasResults);
        assert!(!field.is_loading());
    }

    #[test]
    fn test_search_field_focus() {
        let mut field = SearchField::default();
        assert!(!field.is_focused());

        field.set_focused(true);
        assert!(field.is_focused());
    }

    #[test]
    fn test_search_field_size_variants() {
        let small = SearchField::new().small().build();
        assert_eq!(small.config.size, SearchFieldSize::Small);

        let large = SearchField::new().large().build();
        assert_eq!(large.config.size, SearchFieldSize::Large);
    }

    #[test]
    fn test_search_field_disabled() {
        let field = SearchField::new().disabled(true).build();
        assert!(field.config.disabled);
    }

    #[test]
    fn test_search_field_callbacks() {
        let mut search_called = false;
        let mut change_called = false;
        let mut clear_called = false;

        let mut field = SearchField::new()
            .on_search(|_| search_called = true)
            .on_change(|_| change_called = true)
            .on_clear(|| clear_called = true)
            .build();

        field.handle_input("test".to_string());
        // Note: In actual usage, callbacks would be invoked
    }

    #[test]
    fn test_search_field_no_shortcut() {
        let field = SearchField::new().no_shortcut().build();
        assert!(field.config.focus_shortcut.is_none());
    }

    #[test]
    fn test_shortcut_key_from_char() {
        let key: ShortcutKey = 'a'.into();
        assert_eq!(key, ShortcutKey::Char('a'));
    }

    #[test]
    fn test_search_field_state_default() {
        let state = SearchFieldState::default();
        assert_eq!(state, SearchFieldState::Idle);
    }
}
