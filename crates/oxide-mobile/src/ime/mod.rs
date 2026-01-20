//! Mobile Input Method Editor (IME) integration.
//!
//! Provides abstractions for working with on-screen keyboards and
//! input method editors on mobile platforms.

use serde::{Deserialize, Serialize};

/// Keyboard type for text input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum KeyboardType {
    /// Default keyboard.
    #[default]
    Default,
    /// ASCII-capable keyboard.
    AsciiCapable,
    /// Numbers and punctuation.
    NumbersAndPunctuation,
    /// URL keyboard.
    Url,
    /// Number pad.
    NumberPad,
    /// Phone pad.
    PhonePad,
    /// Name phone pad.
    NamePhonePad,
    /// Email address keyboard.
    EmailAddress,
    /// Decimal pad.
    DecimalPad,
    /// Twitter keyboard.
    Twitter,
    /// Web search keyboard.
    WebSearch,
    /// ASCII-capable number pad.
    AsciiCapableNumberPad,
}

impl KeyboardType {
    /// Get the iOS keyboard type constant.
    pub fn ios_type(&self) -> i32 {
        match self {
            KeyboardType::Default => 0,
            KeyboardType::AsciiCapable => 1,
            KeyboardType::NumbersAndPunctuation => 2,
            KeyboardType::Url => 3,
            KeyboardType::NumberPad => 4,
            KeyboardType::PhonePad => 5,
            KeyboardType::NamePhonePad => 6,
            KeyboardType::EmailAddress => 7,
            KeyboardType::DecimalPad => 8,
            KeyboardType::Twitter => 9,
            KeyboardType::WebSearch => 10,
            KeyboardType::AsciiCapableNumberPad => 11,
        }
    }

    /// Get the Android input type constant.
    pub fn android_input_type(&self) -> i32 {
        match self {
            KeyboardType::Default => 0x00000001, // TYPE_CLASS_TEXT
            KeyboardType::AsciiCapable => 0x00000001,
            KeyboardType::NumbersAndPunctuation => 0x00000002, // TYPE_CLASS_NUMBER
            KeyboardType::Url => 0x00000011, // TYPE_CLASS_TEXT | TYPE_TEXT_VARIATION_URI
            KeyboardType::NumberPad => 0x00000002, // TYPE_CLASS_NUMBER
            KeyboardType::PhonePad => 0x00000003, // TYPE_CLASS_PHONE
            KeyboardType::NamePhonePad => 0x00000001 | 0x00000060, // TYPE_TEXT_VARIATION_PERSON_NAME
            KeyboardType::EmailAddress => 0x00000021, // TYPE_TEXT_VARIATION_EMAIL_ADDRESS
            KeyboardType::DecimalPad => 0x00002002, // TYPE_NUMBER_FLAG_DECIMAL
            KeyboardType::Twitter => 0x00000001,
            KeyboardType::WebSearch => 0x00000001,
            KeyboardType::AsciiCapableNumberPad => 0x00000002,
        }
    }
}

/// Return key type for keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ReturnKeyType {
    /// Default return key.
    #[default]
    Default,
    /// Go button.
    Go,
    /// Google search button.
    Google,
    /// Join button.
    Join,
    /// Next button.
    Next,
    /// Route button.
    Route,
    /// Search button.
    Search,
    /// Send button.
    Send,
    /// Yahoo search button.
    Yahoo,
    /// Done button.
    Done,
    /// Emergency call button.
    EmergencyCall,
    /// Continue button.
    Continue,
}

impl ReturnKeyType {
    /// Get the iOS return key type constant.
    pub fn ios_type(&self) -> i32 {
        match self {
            ReturnKeyType::Default => 0,
            ReturnKeyType::Go => 1,
            ReturnKeyType::Google => 2,
            ReturnKeyType::Join => 3,
            ReturnKeyType::Next => 4,
            ReturnKeyType::Route => 5,
            ReturnKeyType::Search => 6,
            ReturnKeyType::Send => 7,
            ReturnKeyType::Yahoo => 8,
            ReturnKeyType::Done => 9,
            ReturnKeyType::EmergencyCall => 10,
            ReturnKeyType::Continue => 11,
        }
    }

    /// Get the Android IME action constant.
    pub fn android_ime_action(&self) -> i32 {
        match self {
            ReturnKeyType::Default => 0, // IME_ACTION_UNSPECIFIED
            ReturnKeyType::Go => 2, // IME_ACTION_GO
            ReturnKeyType::Google | ReturnKeyType::Yahoo | ReturnKeyType::Search => 3, // IME_ACTION_SEARCH
            ReturnKeyType::Send => 4, // IME_ACTION_SEND
            ReturnKeyType::Next => 5, // IME_ACTION_NEXT
            ReturnKeyType::Done => 6, // IME_ACTION_DONE
            ReturnKeyType::Join | ReturnKeyType::Route | ReturnKeyType::Continue => 2,
            ReturnKeyType::EmergencyCall => 0,
        }
    }
}

/// Text autocapitalization type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum AutocapitalizationType {
    /// No autocapitalization.
    None,
    /// Capitalize words.
    #[default]
    Words,
    /// Capitalize sentences.
    Sentences,
    /// Capitalize all characters.
    AllCharacters,
}

impl AutocapitalizationType {
    /// Get the iOS autocapitalization type constant.
    pub fn ios_type(&self) -> i32 {
        match self {
            AutocapitalizationType::None => 0,
            AutocapitalizationType::Words => 2,
            AutocapitalizationType::Sentences => 1,
            AutocapitalizationType::AllCharacters => 3,
        }
    }

    /// Get the Android input type flags.
    pub fn android_flags(&self) -> i32 {
        match self {
            AutocapitalizationType::None => 0,
            AutocapitalizationType::Words => 0x00001000, // TYPE_TEXT_FLAG_CAP_WORDS
            AutocapitalizationType::Sentences => 0x00004000, // TYPE_TEXT_FLAG_CAP_SENTENCES
            AutocapitalizationType::AllCharacters => 0x00002000, // TYPE_TEXT_FLAG_CAP_CHARACTERS
        }
    }
}

/// Keyboard appearance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum KeyboardAppearance {
    /// Default appearance (follows system).
    #[default]
    Default,
    /// Light appearance.
    Light,
    /// Dark appearance.
    Dark,
}

/// Configuration for text input field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextInputConfig {
    /// Keyboard type.
    pub keyboard_type: KeyboardType,
    /// Return key type.
    pub return_key_type: ReturnKeyType,
    /// Autocapitalization type.
    pub autocapitalization: AutocapitalizationType,
    /// Keyboard appearance.
    pub appearance: KeyboardAppearance,
    /// Enable autocorrection.
    pub autocorrect: bool,
    /// Enable spell checking.
    pub spell_check: bool,
    /// Enable secure text entry (password mode).
    pub secure_entry: bool,
    /// Placeholder text.
    pub placeholder: Option<String>,
    /// Maximum length (None = unlimited).
    pub max_length: Option<usize>,
}

impl Default for TextInputConfig {
    fn default() -> Self {
        Self {
            keyboard_type: KeyboardType::Default,
            return_key_type: ReturnKeyType::Default,
            autocapitalization: AutocapitalizationType::Sentences,
            appearance: KeyboardAppearance::Default,
            autocorrect: true,
            spell_check: true,
            secure_entry: false,
            placeholder: None,
            max_length: None,
        }
    }
}

impl TextInputConfig {
    /// Create a configuration for email input.
    pub fn email() -> Self {
        Self {
            keyboard_type: KeyboardType::EmailAddress,
            autocapitalization: AutocapitalizationType::None,
            autocorrect: false,
            ..Default::default()
        }
    }

    /// Create a configuration for password input.
    pub fn password() -> Self {
        Self {
            keyboard_type: KeyboardType::Default,
            autocapitalization: AutocapitalizationType::None,
            autocorrect: false,
            spell_check: false,
            secure_entry: true,
            ..Default::default()
        }
    }

    /// Create a configuration for URL input.
    pub fn url() -> Self {
        Self {
            keyboard_type: KeyboardType::Url,
            autocapitalization: AutocapitalizationType::None,
            autocorrect: false,
            return_key_type: ReturnKeyType::Go,
            ..Default::default()
        }
    }

    /// Create a configuration for number input.
    pub fn number() -> Self {
        Self {
            keyboard_type: KeyboardType::NumberPad,
            autocapitalization: AutocapitalizationType::None,
            autocorrect: false,
            spell_check: false,
            ..Default::default()
        }
    }

    /// Create a configuration for decimal input.
    pub fn decimal() -> Self {
        Self {
            keyboard_type: KeyboardType::DecimalPad,
            autocapitalization: AutocapitalizationType::None,
            autocorrect: false,
            spell_check: false,
            ..Default::default()
        }
    }

    /// Create a configuration for phone number input.
    pub fn phone() -> Self {
        Self {
            keyboard_type: KeyboardType::PhonePad,
            autocapitalization: AutocapitalizationType::None,
            autocorrect: false,
            spell_check: false,
            ..Default::default()
        }
    }

    /// Create a configuration for search input.
    pub fn search() -> Self {
        Self {
            keyboard_type: KeyboardType::WebSearch,
            return_key_type: ReturnKeyType::Search,
            ..Default::default()
        }
    }
}

/// IME event types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImeEvent {
    /// Keyboard will show.
    KeyboardWillShow {
        /// Height of the keyboard in points.
        height: f32,
        /// Animation duration in seconds.
        animation_duration: f32,
    },
    /// Keyboard did show.
    KeyboardDidShow {
        /// Height of the keyboard in points.
        height: f32,
    },
    /// Keyboard will hide.
    KeyboardWillHide {
        /// Animation duration in seconds.
        animation_duration: f32,
    },
    /// Keyboard did hide.
    KeyboardDidHide,
    /// Keyboard frame changed.
    KeyboardFrameChanged {
        /// Height of the keyboard in points.
        height: f32,
    },
    /// Text was committed from IME.
    TextCommitted {
        /// The committed text.
        text: String,
    },
    /// Composition text changed (for CJK input).
    CompositionChanged {
        /// The composition text.
        text: String,
        /// Cursor position in composition.
        cursor: usize,
    },
    /// Composition was cancelled.
    CompositionCancelled,
}

/// IME handler trait.
pub trait ImeHandler {
    /// Handle an IME event.
    fn on_ime_event(&mut self, event: ImeEvent);

    /// Called when keyboard visibility changes.
    fn on_keyboard_visibility_changed(&mut self, visible: bool, height: f32);
}

/// Keyboard state tracking.
#[derive(Debug, Clone, Default)]
pub struct KeyboardState {
    /// Whether the keyboard is visible.
    pub visible: bool,
    /// Current keyboard height in points.
    pub height: f32,
    /// Whether the keyboard is animating.
    pub animating: bool,
}

impl KeyboardState {
    /// Create a new keyboard state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Update state from an IME event.
    pub fn handle_event(&mut self, event: &ImeEvent) {
        match event {
            ImeEvent::KeyboardWillShow { height, .. } => {
                self.animating = true;
                self.height = *height;
            }
            ImeEvent::KeyboardDidShow { height } => {
                self.visible = true;
                self.animating = false;
                self.height = *height;
            }
            ImeEvent::KeyboardWillHide { .. } => {
                self.animating = true;
            }
            ImeEvent::KeyboardDidHide => {
                self.visible = false;
                self.animating = false;
                self.height = 0.0;
            }
            ImeEvent::KeyboardFrameChanged { height } => {
                self.height = *height;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_type_constants() {
        assert_eq!(KeyboardType::Default.ios_type(), 0);
        assert_eq!(KeyboardType::NumberPad.ios_type(), 4);
        assert_eq!(KeyboardType::EmailAddress.ios_type(), 7);
    }

    #[test]
    fn test_return_key_type_constants() {
        assert_eq!(ReturnKeyType::Default.ios_type(), 0);
        assert_eq!(ReturnKeyType::Search.ios_type(), 6);
        assert_eq!(ReturnKeyType::Done.ios_type(), 9);
    }

    #[test]
    fn test_text_input_presets() {
        let email = TextInputConfig::email();
        assert!(matches!(email.keyboard_type, KeyboardType::EmailAddress));
        assert!(!email.autocorrect);

        let password = TextInputConfig::password();
        assert!(password.secure_entry);
        assert!(!password.autocorrect);
        assert!(!password.spell_check);

        let url = TextInputConfig::url();
        assert!(matches!(url.keyboard_type, KeyboardType::Url));
        assert!(matches!(url.return_key_type, ReturnKeyType::Go));
    }

    #[test]
    fn test_keyboard_state() {
        let mut state = KeyboardState::new();
        assert!(!state.visible);
        assert_eq!(state.height, 0.0);

        state.handle_event(&ImeEvent::KeyboardWillShow {
            height: 300.0,
            animation_duration: 0.25,
        });
        assert!(state.animating);

        state.handle_event(&ImeEvent::KeyboardDidShow { height: 300.0 });
        assert!(state.visible);
        assert!(!state.animating);
        assert_eq!(state.height, 300.0);

        state.handle_event(&ImeEvent::KeyboardDidHide);
        assert!(!state.visible);
        assert_eq!(state.height, 0.0);
    }
}
