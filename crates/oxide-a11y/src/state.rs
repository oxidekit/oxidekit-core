//! Accessibility States and Properties
//!
//! This module defines all ARIA states and properties for accessible elements:
//! - State flags (checked, selected, expanded, etc.)
//! - Value properties (min, max, now, text)
//! - Relationship properties (controls, owns, describedby)
//! - Live region settings

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    /// Bitflags for common accessibility states
    ///
    /// These represent boolean states that can be efficiently combined
    /// and tested using bitwise operations.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
    pub struct StateFlags: u32 {
        /// Element is disabled and not interactive
        const DISABLED = 1 << 0;
        /// Element is read-only (editable but not by user)
        const READONLY = 1 << 1;
        /// Element is required to have a value
        const REQUIRED = 1 << 2;
        /// Element is selected
        const SELECTED = 1 << 3;
        /// Element is expanded (shows additional content)
        const EXPANDED = 1 << 4;
        /// Element is pressed (toggle button state)
        const PRESSED = 1 << 5;
        /// Element has an invalid value
        const INVALID = 1 << 6;
        /// Element has a popup associated
        const HAS_POPUP = 1 << 7;
        /// Element is busy/loading
        const BUSY = 1 << 8;
        /// Element is hidden from accessibility tree
        const HIDDEN = 1 << 9;
        /// Element is the current item in a set
        const CURRENT = 1 << 10;
        /// Element is grabbed (drag and drop)
        const GRABBED = 1 << 11;
        /// Element allows dropping
        const DROP_EFFECT = 1 << 12;
        /// Element is a modal dialog
        const MODAL = 1 << 13;
        /// Element content is atomic (announce as whole)
        const ATOMIC = 1 << 14;
        /// Element supports multiline input
        const MULTILINE = 1 << 15;
        /// Element supports multiple selection
        const MULTISELECTABLE = 1 << 16;
        /// Element is focusable
        const FOCUSABLE = 1 << 17;
        /// Element is currently focused
        const FOCUSED = 1 << 18;
        /// Element is editable
        const EDITABLE = 1 << 19;
        /// Element is autocomplete enabled
        const AUTOCOMPLETE = 1 << 20;
    }
}

impl StateFlags {
    /// Check if the element is interactive (not disabled)
    #[inline]
    pub fn is_interactive(&self) -> bool {
        !self.contains(StateFlags::DISABLED)
    }

    /// Check if the element can receive focus
    #[inline]
    pub fn can_focus(&self) -> bool {
        self.contains(StateFlags::FOCUSABLE) && !self.contains(StateFlags::DISABLED)
    }

    /// Check if the element is in an error state
    #[inline]
    pub fn has_error(&self) -> bool {
        self.contains(StateFlags::INVALID)
    }
}

/// Tri-state value for checkboxes and similar controls
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckedState {
    /// Not checked
    #[default]
    False,
    /// Checked
    True,
    /// Partially checked (indeterminate)
    Mixed,
}

impl CheckedState {
    /// Toggle between true and false (mixed becomes false)
    pub fn toggle(&self) -> Self {
        match self {
            CheckedState::False => CheckedState::True,
            CheckedState::True | CheckedState::Mixed => CheckedState::False,
        }
    }

    /// Check if the state is checked or mixed
    pub fn is_checked_or_mixed(&self) -> bool {
        !matches!(self, CheckedState::False)
    }
}

impl From<bool> for CheckedState {
    fn from(value: bool) -> Self {
        if value {
            CheckedState::True
        } else {
            CheckedState::False
        }
    }
}

impl From<CheckedState> for Option<bool> {
    fn from(state: CheckedState) -> Self {
        match state {
            CheckedState::True => Some(true),
            CheckedState::False => Some(false),
            CheckedState::Mixed => None,
        }
    }
}

/// Live region politeness level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LivePoliteness {
    /// Updates are not announced
    #[default]
    Off,
    /// Updates announced when user is idle
    Polite,
    /// Updates announced immediately, interrupting
    Assertive,
}

/// What changes are relevant for live regions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LiveRelevant {
    /// Additions of nodes
    Additions,
    /// Removals of nodes
    Removals,
    /// Text changes
    Text,
    /// All changes (default)
    All,
}

impl Default for LiveRelevant {
    fn default() -> Self {
        LiveRelevant::All
    }
}

/// Invalid state with reason
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InvalidState {
    /// Value is valid
    #[default]
    False,
    /// Value is invalid (generic)
    True,
    /// Grammar error
    Grammar,
    /// Spelling error
    Spelling,
}

impl InvalidState {
    /// Check if the state represents any kind of invalidity
    pub fn is_invalid(&self) -> bool {
        !matches!(self, InvalidState::False)
    }
}

impl From<bool> for InvalidState {
    fn from(value: bool) -> Self {
        if value {
            InvalidState::True
        } else {
            InvalidState::False
        }
    }
}

/// Type of popup associated with element
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HasPopup {
    /// No popup
    #[default]
    False,
    /// Generic popup (true)
    True,
    /// Menu popup
    Menu,
    /// Listbox popup
    Listbox,
    /// Tree popup
    Tree,
    /// Grid popup
    Grid,
    /// Dialog popup
    Dialog,
}

impl HasPopup {
    /// Check if any popup type is associated
    pub fn has_popup(&self) -> bool {
        !matches!(self, HasPopup::False)
    }
}

impl From<bool> for HasPopup {
    fn from(value: bool) -> Self {
        if value {
            HasPopup::True
        } else {
            HasPopup::False
        }
    }
}

/// Current item indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Current {
    /// Not the current item
    #[default]
    False,
    /// Current item (generic)
    True,
    /// Current page
    Page,
    /// Current step
    Step,
    /// Current location
    Location,
    /// Current date
    Date,
    /// Current time
    Time,
}

impl Current {
    /// Check if this is the current item
    pub fn is_current(&self) -> bool {
        !matches!(self, Current::False)
    }
}

impl From<bool> for Current {
    fn from(value: bool) -> Self {
        if value {
            Current::True
        } else {
            Current::False
        }
    }
}

/// Autocomplete type for input fields
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Autocomplete {
    /// No autocomplete
    #[default]
    None,
    /// Inline completion
    Inline,
    /// List of suggestions
    List,
    /// Both inline and list
    Both,
}

/// Sort direction for tables/grids
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    /// No sort
    #[default]
    None,
    /// Ascending order
    Ascending,
    /// Descending order
    Descending,
    /// Other sort order
    Other,
}

/// Orientation of an element
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    /// Horizontal orientation (default for most)
    #[default]
    Horizontal,
    /// Vertical orientation
    Vertical,
    /// Undefined/both
    Undefined,
}

/// Value range for sliders, spinbuttons, progressbars
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValueRange {
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Current value
    pub now: f64,
    /// Human-readable value text
    pub text: Option<String>,
}

impl Default for ValueRange {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 100.0,
            now: 0.0,
            text: None,
        }
    }
}

impl ValueRange {
    /// Create a new value range
    pub fn new(min: f64, max: f64, now: f64) -> Self {
        Self {
            min,
            max,
            now: now.clamp(min, max),
            text: None,
        }
    }

    /// Create a percentage-based range (0-100)
    pub fn percentage(value: f64) -> Self {
        Self::new(0.0, 100.0, value)
    }

    /// Set the value with clamping
    pub fn set_value(&mut self, value: f64) {
        self.now = value.clamp(self.min, self.max);
    }

    /// Get the value as a percentage of the range
    pub fn percentage_value(&self) -> f64 {
        if (self.max - self.min).abs() < f64::EPSILON {
            0.0
        } else {
            ((self.now - self.min) / (self.max - self.min)) * 100.0
        }
    }

    /// Set a human-readable text description
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Check if the value is at minimum
    pub fn is_at_min(&self) -> bool {
        (self.now - self.min).abs() < f64::EPSILON
    }

    /// Check if the value is at maximum
    pub fn is_at_max(&self) -> bool {
        (self.now - self.max).abs() < f64::EPSILON
    }

    /// Increment by a step amount
    pub fn increment(&mut self, step: f64) {
        self.set_value(self.now + step);
    }

    /// Decrement by a step amount
    pub fn decrement(&mut self, step: f64) {
        self.set_value(self.now - step);
    }
}

/// Position in a set (for listbox items, tabs, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct SetPosition {
    /// Position in the set (1-based)
    pub pos: u32,
    /// Total size of the set
    pub size: u32,
}

impl SetPosition {
    /// Create a new set position
    pub fn new(pos: u32, size: u32) -> Self {
        Self { pos, size }
    }

    /// Check if this is the first item
    pub fn is_first(&self) -> bool {
        self.pos == 1
    }

    /// Check if this is the last item
    pub fn is_last(&self) -> bool {
        self.pos == self.size
    }

    /// Get position as a string (e.g., "3 of 10")
    pub fn description(&self) -> String {
        format!("{} of {}", self.pos, self.size)
    }
}

/// Table/grid position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct GridPosition {
    /// Row index (1-based)
    pub row: u32,
    /// Column index (1-based)
    pub col: u32,
    /// Row span
    pub row_span: u32,
    /// Column span
    pub col_span: u32,
}

impl GridPosition {
    /// Create a simple single-cell position
    pub fn cell(row: u32, col: u32) -> Self {
        Self {
            row,
            col,
            row_span: 1,
            col_span: 1,
        }
    }

    /// Create a position with spans
    pub fn with_span(row: u32, col: u32, row_span: u32, col_span: u32) -> Self {
        Self {
            row,
            col,
            row_span,
            col_span,
        }
    }
}

/// Heading level (1-6)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HeadingLevel(u8);

impl HeadingLevel {
    /// Create a heading level (clamped to 1-6)
    pub fn new(level: u8) -> Self {
        Self(level.clamp(1, 6))
    }

    /// Get the level value
    pub fn level(&self) -> u8 {
        self.0
    }

    /// Check if this is the top-level heading
    pub fn is_top_level(&self) -> bool {
        self.0 == 1
    }
}

impl Default for HeadingLevel {
    fn default() -> Self {
        Self(2) // Most common default
    }
}

impl From<u8> for HeadingLevel {
    fn from(level: u8) -> Self {
        Self::new(level)
    }
}

/// Live region configuration
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LiveRegion {
    /// Politeness level
    pub politeness: LivePoliteness,
    /// Whether to announce the entire region on updates
    pub atomic: bool,
    /// What changes are relevant
    pub relevant: Vec<LiveRelevant>,
    /// Whether the region is currently busy
    pub busy: bool,
}

impl LiveRegion {
    /// Create a polite live region
    pub fn polite() -> Self {
        Self {
            politeness: LivePoliteness::Polite,
            atomic: false,
            relevant: vec![LiveRelevant::All],
            busy: false,
        }
    }

    /// Create an assertive live region
    pub fn assertive() -> Self {
        Self {
            politeness: LivePoliteness::Assertive,
            atomic: false,
            relevant: vec![LiveRelevant::All],
            busy: false,
        }
    }

    /// Set atomic behavior
    pub fn with_atomic(mut self, atomic: bool) -> Self {
        self.atomic = atomic;
        self
    }

    /// Set relevant changes
    pub fn with_relevant(mut self, relevant: Vec<LiveRelevant>) -> Self {
        self.relevant = relevant;
        self
    }

    /// Mark as busy (loading)
    pub fn set_busy(&mut self, busy: bool) {
        self.busy = busy;
    }
}

/// Complete accessibility state for a node
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccessibilityState {
    /// Boolean state flags
    #[serde(default)]
    pub flags: StateFlags,

    /// Checked state (for checkboxes, switches)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked: Option<CheckedState>,

    /// Value range (for sliders, progressbars)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<ValueRange>,

    /// Heading level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<HeadingLevel>,

    /// Position in set
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_position: Option<SetPosition>,

    /// Grid position
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid_position: Option<GridPosition>,

    /// Invalid state
    #[serde(default)]
    pub invalid: InvalidState,

    /// Has popup type
    #[serde(default)]
    pub has_popup: HasPopup,

    /// Current item indicator
    #[serde(default)]
    pub current: Current,

    /// Autocomplete type
    #[serde(default)]
    pub autocomplete: Autocomplete,

    /// Sort direction
    #[serde(default)]
    pub sort: SortDirection,

    /// Orientation
    #[serde(default)]
    pub orientation: Orientation,

    /// Live region settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub live_region: Option<LiveRegion>,
}

impl AccessibilityState {
    /// Create a new empty state
    pub fn new() -> Self {
        Self::default()
    }

    /// Create state for a button
    pub fn button() -> Self {
        Self {
            flags: StateFlags::FOCUSABLE,
            ..Default::default()
        }
    }

    /// Create state for a checkbox
    pub fn checkbox(checked: bool) -> Self {
        Self {
            flags: StateFlags::FOCUSABLE,
            checked: Some(checked.into()),
            ..Default::default()
        }
    }

    /// Create state for a switch
    pub fn switch(on: bool) -> Self {
        Self {
            flags: StateFlags::FOCUSABLE,
            checked: Some(on.into()),
            ..Default::default()
        }
    }

    /// Create state for a text input
    pub fn textbox() -> Self {
        Self {
            flags: StateFlags::FOCUSABLE | StateFlags::EDITABLE,
            ..Default::default()
        }
    }

    /// Create state for a slider
    pub fn slider(min: f64, max: f64, value: f64) -> Self {
        Self {
            flags: StateFlags::FOCUSABLE,
            value: Some(ValueRange::new(min, max, value)),
            orientation: Orientation::Horizontal,
            ..Default::default()
        }
    }

    /// Create state for a progress bar
    pub fn progressbar(value: f64) -> Self {
        Self {
            value: Some(ValueRange::percentage(value)),
            ..Default::default()
        }
    }

    /// Create state for a tab
    pub fn tab(selected: bool, pos: u32, size: u32) -> Self {
        let mut flags = StateFlags::FOCUSABLE;
        if selected {
            flags |= StateFlags::SELECTED;
        }
        Self {
            flags,
            set_position: Some(SetPosition::new(pos, size)),
            ..Default::default()
        }
    }

    /// Create state for a heading
    pub fn heading(level: u8) -> Self {
        Self {
            level: Some(HeadingLevel::new(level)),
            ..Default::default()
        }
    }

    /// Create state for a dialog
    pub fn dialog(modal: bool) -> Self {
        let mut flags = StateFlags::empty();
        if modal {
            flags |= StateFlags::MODAL;
        }
        Self {
            flags,
            ..Default::default()
        }
    }

    /// Set disabled state
    pub fn set_disabled(&mut self, disabled: bool) {
        if disabled {
            self.flags |= StateFlags::DISABLED;
        } else {
            self.flags -= StateFlags::DISABLED;
        }
    }

    /// Set expanded state
    pub fn set_expanded(&mut self, expanded: bool) {
        if expanded {
            self.flags |= StateFlags::EXPANDED;
        } else {
            self.flags -= StateFlags::EXPANDED;
        }
    }

    /// Set selected state
    pub fn set_selected(&mut self, selected: bool) {
        if selected {
            self.flags |= StateFlags::SELECTED;
        } else {
            self.flags -= StateFlags::SELECTED;
        }
    }

    /// Set focused state
    pub fn set_focused(&mut self, focused: bool) {
        if focused {
            self.flags |= StateFlags::FOCUSED;
        } else {
            self.flags -= StateFlags::FOCUSED;
        }
    }

    /// Check if disabled
    pub fn is_disabled(&self) -> bool {
        self.flags.contains(StateFlags::DISABLED)
    }

    /// Check if expanded
    pub fn is_expanded(&self) -> bool {
        self.flags.contains(StateFlags::EXPANDED)
    }

    /// Check if selected
    pub fn is_selected(&self) -> bool {
        self.flags.contains(StateFlags::SELECTED)
    }

    /// Check if focusable
    pub fn is_focusable(&self) -> bool {
        self.flags.can_focus()
    }

    /// Check if focused
    pub fn is_focused(&self) -> bool {
        self.flags.contains(StateFlags::FOCUSED)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_flags() {
        let mut flags = StateFlags::empty();
        assert!(!flags.contains(StateFlags::DISABLED));

        flags |= StateFlags::DISABLED;
        assert!(flags.contains(StateFlags::DISABLED));
        assert!(!flags.is_interactive());

        flags -= StateFlags::DISABLED;
        assert!(flags.is_interactive());
    }

    #[test]
    fn test_state_flags_can_focus() {
        let flags = StateFlags::FOCUSABLE;
        assert!(flags.can_focus());

        let disabled_flags = StateFlags::FOCUSABLE | StateFlags::DISABLED;
        assert!(!disabled_flags.can_focus());
    }

    #[test]
    fn test_checked_state_toggle() {
        assert_eq!(CheckedState::False.toggle(), CheckedState::True);
        assert_eq!(CheckedState::True.toggle(), CheckedState::False);
        assert_eq!(CheckedState::Mixed.toggle(), CheckedState::False);
    }

    #[test]
    fn test_checked_state_from_bool() {
        assert_eq!(CheckedState::from(true), CheckedState::True);
        assert_eq!(CheckedState::from(false), CheckedState::False);
    }

    #[test]
    fn test_value_range() {
        let mut range = ValueRange::new(0.0, 100.0, 50.0);
        assert_eq!(range.percentage_value(), 50.0);

        range.set_value(150.0); // Should clamp
        assert_eq!(range.now, 100.0);
        assert!(range.is_at_max());

        range.set_value(-10.0); // Should clamp
        assert_eq!(range.now, 0.0);
        assert!(range.is_at_min());
    }

    #[test]
    fn test_value_range_increment() {
        let mut range = ValueRange::new(0.0, 10.0, 5.0);
        range.increment(2.0);
        assert_eq!(range.now, 7.0);

        range.increment(10.0); // Should clamp to max
        assert_eq!(range.now, 10.0);
    }

    #[test]
    fn test_set_position() {
        let pos = SetPosition::new(1, 5);
        assert!(pos.is_first());
        assert!(!pos.is_last());
        assert_eq!(pos.description(), "1 of 5");

        let last = SetPosition::new(5, 5);
        assert!(last.is_last());
    }

    #[test]
    fn test_heading_level() {
        let h1 = HeadingLevel::new(1);
        assert!(h1.is_top_level());
        assert_eq!(h1.level(), 1);

        let h_clamped = HeadingLevel::new(10);
        assert_eq!(h_clamped.level(), 6);
    }

    #[test]
    fn test_invalid_state() {
        assert!(!InvalidState::False.is_invalid());
        assert!(InvalidState::True.is_invalid());
        assert!(InvalidState::Grammar.is_invalid());
        assert!(InvalidState::Spelling.is_invalid());
    }

    #[test]
    fn test_has_popup() {
        assert!(!HasPopup::False.has_popup());
        assert!(HasPopup::True.has_popup());
        assert!(HasPopup::Menu.has_popup());
        assert!(HasPopup::Dialog.has_popup());
    }

    #[test]
    fn test_live_region() {
        let live = LiveRegion::polite().with_atomic(true);
        assert_eq!(live.politeness, LivePoliteness::Polite);
        assert!(live.atomic);

        let assertive = LiveRegion::assertive();
        assert_eq!(assertive.politeness, LivePoliteness::Assertive);
    }

    #[test]
    fn test_accessibility_state_button() {
        let state = AccessibilityState::button();
        assert!(state.flags.contains(StateFlags::FOCUSABLE));
        assert!(!state.is_disabled());
    }

    #[test]
    fn test_accessibility_state_checkbox() {
        let state = AccessibilityState::checkbox(true);
        assert_eq!(state.checked, Some(CheckedState::True));
        assert!(state.is_focusable());
    }

    #[test]
    fn test_accessibility_state_slider() {
        let state = AccessibilityState::slider(0.0, 100.0, 50.0);
        assert!(state.value.is_some());
        assert_eq!(state.value.as_ref().unwrap().now, 50.0);
    }

    #[test]
    fn test_accessibility_state_disabled() {
        let mut state = AccessibilityState::button();
        assert!(!state.is_disabled());

        state.set_disabled(true);
        assert!(state.is_disabled());
        assert!(!state.is_focusable());

        state.set_disabled(false);
        assert!(!state.is_disabled());
    }

    #[test]
    fn test_accessibility_state_expanded() {
        let mut state = AccessibilityState::new();
        assert!(!state.is_expanded());

        state.set_expanded(true);
        assert!(state.is_expanded());
    }

    #[test]
    fn test_state_serialization() {
        let state = AccessibilityState::slider(0.0, 100.0, 50.0);
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("\"now\":50.0"));

        let parsed: AccessibilityState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.value.unwrap().now, 50.0);
    }

    #[test]
    fn test_grid_position() {
        let cell = GridPosition::cell(2, 3);
        assert_eq!(cell.row, 2);
        assert_eq!(cell.col, 3);
        assert_eq!(cell.row_span, 1);
        assert_eq!(cell.col_span, 1);

        let spanned = GridPosition::with_span(1, 1, 2, 3);
        assert_eq!(spanned.row_span, 2);
        assert_eq!(spanned.col_span, 3);
    }

    #[test]
    fn test_orientation() {
        assert_eq!(Orientation::default(), Orientation::Horizontal);
    }
}
