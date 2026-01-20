//! Focus Management System
//!
//! This module provides comprehensive focus management including:
//! - Focus tracking and history
//! - Focus trapping for modal dialogs
//! - Focus traversal (next, previous, first, last)
//! - Skip links for keyboard navigation
//! - Focus ring styling

use crate::tree::NodeId;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Focus ring visual style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusRingStyle {
    /// Ring color (CSS color string)
    pub color: String,
    /// Ring width in pixels
    pub width: f32,
    /// Offset from element in pixels
    pub offset: f32,
    /// Border radius (None = inherit from element)
    pub radius: Option<f32>,
    /// Style type
    pub style: FocusRingType,
    /// Opacity (0.0 - 1.0)
    pub opacity: f32,
}

impl Default for FocusRingStyle {
    fn default() -> Self {
        Self {
            color: "rgba(59, 130, 246, 0.5)".into(), // Blue
            width: 2.0,
            offset: 2.0,
            radius: None,
            style: FocusRingType::Outline,
            opacity: 1.0,
        }
    }
}

impl FocusRingStyle {
    /// Create a blue focus ring (default)
    pub fn blue() -> Self {
        Self::default()
    }

    /// Create a high contrast focus ring
    pub fn high_contrast() -> Self {
        Self {
            color: "#000000".into(),
            width: 3.0,
            offset: 2.0,
            radius: None,
            style: FocusRingType::Solid,
            opacity: 1.0,
        }
    }

    /// Create a subtle focus ring
    pub fn subtle() -> Self {
        Self {
            color: "rgba(0, 0, 0, 0.3)".into(),
            width: 1.0,
            offset: 1.0,
            radius: None,
            style: FocusRingType::Outline,
            opacity: 0.8,
        }
    }

    /// Set the color
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = color.into();
        self
    }

    /// Set the width
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Set the offset
    pub fn with_offset(mut self, offset: f32) -> Self {
        self.offset = offset;
        self
    }
}

/// Focus ring rendering style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FocusRingType {
    /// Standard outline around element
    #[default]
    Outline,
    /// Inset shadow inside element
    Inset,
    /// Outer glow effect
    Glow,
    /// Solid border
    Solid,
    /// Dotted outline
    Dotted,
    /// Dashed outline
    Dashed,
}

/// Focus trap for modal dialogs and overlays
///
/// Prevents focus from escaping a specific region of the UI
#[derive(Debug, Clone)]
pub struct FocusTrap {
    /// Container element ID
    pub container_id: NodeId,

    /// Focusable elements within the trap in order
    focusable_ids: Vec<NodeId>,

    /// Current focus index
    current_index: usize,

    /// Whether the trap is active
    active: bool,

    /// ID to restore focus to when trap is released
    restore_focus_to: Option<NodeId>,

    /// Whether to auto-focus first element on activation
    auto_focus_first: bool,

    /// Whether to auto-focus last element (for Shift+Tab entry)
    auto_focus_last: bool,

    /// Whether to loop within the trap
    wrap: bool,
}

impl FocusTrap {
    /// Create a new focus trap for a container
    pub fn new(container_id: NodeId) -> Self {
        Self {
            container_id,
            focusable_ids: Vec::new(),
            current_index: 0,
            active: false,
            restore_focus_to: None,
            auto_focus_first: true,
            auto_focus_last: false,
            wrap: true,
        }
    }

    /// Add a focusable element to the trap
    pub fn add_focusable(&mut self, id: NodeId) {
        if !self.focusable_ids.contains(&id) {
            self.focusable_ids.push(id);
        }
    }

    /// Set focusable elements from a list
    pub fn set_focusable(&mut self, ids: Vec<NodeId>) {
        self.focusable_ids = ids;
        self.current_index = 0;
    }

    /// Remove a focusable element
    pub fn remove_focusable(&mut self, id: &NodeId) {
        if let Some(pos) = self.focusable_ids.iter().position(|i| i == id) {
            self.focusable_ids.remove(pos);
            if self.current_index >= self.focusable_ids.len() && !self.focusable_ids.is_empty() {
                self.current_index = self.focusable_ids.len() - 1;
            }
        }
    }

    /// Activate the trap
    pub fn activate(&mut self, current_focus: Option<NodeId>) {
        self.restore_focus_to = current_focus;
        self.active = true;

        if self.auto_focus_first && !self.focusable_ids.is_empty() {
            self.current_index = 0;
        } else if self.auto_focus_last && !self.focusable_ids.is_empty() {
            self.current_index = self.focusable_ids.len() - 1;
        }
    }

    /// Deactivate the trap and return the ID to restore focus to
    pub fn deactivate(&mut self) -> Option<NodeId> {
        self.active = false;
        self.restore_focus_to.take()
    }

    /// Check if the trap is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get the currently focused element ID
    pub fn current_focus(&self) -> Option<NodeId> {
        if self.active && !self.focusable_ids.is_empty() {
            self.focusable_ids.get(self.current_index).copied()
        } else {
            None
        }
    }

    /// Move focus to the next element
    pub fn focus_next(&mut self) -> Option<NodeId> {
        if !self.active || self.focusable_ids.is_empty() {
            return None;
        }

        if self.wrap {
            self.current_index = (self.current_index + 1) % self.focusable_ids.len();
        } else if self.current_index < self.focusable_ids.len() - 1 {
            self.current_index += 1;
        }

        self.focusable_ids.get(self.current_index).copied()
    }

    /// Move focus to the previous element
    pub fn focus_previous(&mut self) -> Option<NodeId> {
        if !self.active || self.focusable_ids.is_empty() {
            return None;
        }

        if self.wrap {
            self.current_index = if self.current_index == 0 {
                self.focusable_ids.len() - 1
            } else {
                self.current_index - 1
            };
        } else if self.current_index > 0 {
            self.current_index -= 1;
        }

        self.focusable_ids.get(self.current_index).copied()
    }

    /// Move focus to a specific element
    pub fn focus_element(&mut self, id: NodeId) -> bool {
        if !self.active {
            return false;
        }

        if let Some(idx) = self.focusable_ids.iter().position(|i| *i == id) {
            self.current_index = idx;
            true
        } else {
            false
        }
    }

    /// Move focus to the first element
    pub fn focus_first(&mut self) -> Option<NodeId> {
        if !self.active || self.focusable_ids.is_empty() {
            return None;
        }
        self.current_index = 0;
        self.focusable_ids.first().copied()
    }

    /// Move focus to the last element
    pub fn focus_last(&mut self) -> Option<NodeId> {
        if !self.active || self.focusable_ids.is_empty() {
            return None;
        }
        self.current_index = self.focusable_ids.len() - 1;
        self.focusable_ids.last().copied()
    }

    /// Check if an element is within the trap
    pub fn contains(&self, id: &NodeId) -> bool {
        self.focusable_ids.contains(id)
    }

    /// Get the number of focusable elements
    pub fn len(&self) -> usize {
        self.focusable_ids.len()
    }

    /// Check if the trap has no focusable elements
    pub fn is_empty(&self) -> bool {
        self.focusable_ids.is_empty()
    }

    /// Enable or disable wrapping
    pub fn set_wrap(&mut self, wrap: bool) {
        self.wrap = wrap;
    }

    /// Enable auto-focus on first element when activated
    pub fn set_auto_focus_first(&mut self, auto: bool) {
        self.auto_focus_first = auto;
    }
}

/// Skip link for keyboard navigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkipLink {
    /// Unique identifier
    pub id: String,
    /// Display label
    pub label: String,
    /// Target element ID to skip to
    pub target_id: NodeId,
    /// Whether currently visible
    pub visible: bool,
    /// Keyboard shortcut (optional)
    pub shortcut: Option<String>,
}

impl SkipLink {
    /// Create a new skip link
    pub fn new(id: impl Into<String>, label: impl Into<String>, target_id: NodeId) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            target_id,
            visible: false,
            shortcut: None,
        }
    }

    /// Create a skip to main content link
    pub fn to_main(main_id: NodeId) -> Self {
        Self::new("skip-main", "Skip to main content", main_id)
    }

    /// Create a skip to navigation link
    pub fn to_navigation(nav_id: NodeId) -> Self {
        Self::new("skip-nav", "Skip to navigation", nav_id)
    }

    /// Create a skip to search link
    pub fn to_search(search_id: NodeId) -> Self {
        Self::new("skip-search", "Skip to search", search_id)
    }

    /// Set a keyboard shortcut
    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Show the skip link
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the skip link
    pub fn hide(&mut self) {
        self.visible = false;
    }
}

/// Focus event type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusEvent {
    /// Element gained focus
    FocusIn,
    /// Element lost focus
    FocusOut,
    /// Focus moved to element (via keyboard)
    Focus,
    /// Focus left element (via keyboard)
    Blur,
}

/// Focus change reason
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusChangeReason {
    /// User pressed Tab or Shift+Tab
    Tab,
    /// User clicked on element
    Click,
    /// Programmatic focus call
    Programmatic,
    /// Focus trap navigation
    Trap,
    /// Arrow key navigation
    Arrow,
    /// Focus restoration after trap
    Restore,
    /// Skip link activation
    SkipLink,
    /// Initial page load
    Initial,
}

/// Manages application focus state
#[derive(Debug)]
pub struct FocusManager {
    /// Currently focused element ID
    current_focus: Option<NodeId>,

    /// Focus history for restoration
    focus_history: VecDeque<NodeId>,

    /// Maximum history size
    max_history: usize,

    /// Stack of focus traps
    traps: Vec<FocusTrap>,

    /// Skip links
    skip_links: Vec<SkipLink>,

    /// Focus ring style
    focus_ring_style: FocusRingStyle,

    /// Whether focus ring should be visible
    focus_visible: bool,

    /// Last interaction was keyboard
    keyboard_mode: bool,

    /// Whether focus management is enabled
    enabled: bool,
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusManager {
    /// Create a new focus manager
    pub fn new() -> Self {
        Self {
            current_focus: None,
            focus_history: VecDeque::new(),
            max_history: 50,
            traps: Vec::new(),
            skip_links: Vec::new(),
            focus_ring_style: FocusRingStyle::default(),
            focus_visible: false,
            keyboard_mode: false,
            enabled: true,
        }
    }

    /// Enable or disable focus management
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if focus management is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set focus to an element
    pub fn focus(&mut self, id: NodeId, reason: FocusChangeReason) -> Option<NodeId> {
        if !self.enabled {
            return None;
        }

        // Store previous focus in history
        if let Some(prev) = self.current_focus {
            self.push_history(prev);
        }

        self.current_focus = Some(id);

        // Update keyboard mode based on reason
        self.keyboard_mode = matches!(
            reason,
            FocusChangeReason::Tab | FocusChangeReason::Arrow | FocusChangeReason::SkipLink
        );
        self.focus_visible = self.keyboard_mode;

        // Update active trap if any
        for trap in self.traps.iter_mut().rev() {
            if trap.is_active() && trap.contains(&id) {
                trap.focus_element(id);
                break;
            }
        }

        Some(id)
    }

    /// Clear focus
    pub fn blur(&mut self) {
        self.current_focus = None;
    }

    /// Get the currently focused element
    pub fn current(&self) -> Option<NodeId> {
        // Check active traps first
        for trap in self.traps.iter().rev() {
            if trap.is_active() {
                return trap.current_focus();
            }
        }
        self.current_focus
    }

    /// Push a focus trap onto the stack
    pub fn push_trap(&mut self, mut trap: FocusTrap) {
        trap.activate(self.current_focus);
        if let Some(first_focus) = trap.current_focus() {
            self.current_focus = Some(first_focus);
        }
        self.traps.push(trap);
    }

    /// Pop the top focus trap and restore focus
    pub fn pop_trap(&mut self) -> Option<FocusTrap> {
        if let Some(mut trap) = self.traps.pop() {
            if let Some(restore_id) = trap.deactivate() {
                self.focus(restore_id, FocusChangeReason::Restore);
            }
            Some(trap)
        } else {
            None
        }
    }

    /// Get the active focus trap
    pub fn active_trap(&self) -> Option<&FocusTrap> {
        self.traps.iter().rev().find(|t| t.is_active())
    }

    /// Get the active focus trap mutably
    pub fn active_trap_mut(&mut self) -> Option<&mut FocusTrap> {
        self.traps.iter_mut().rev().find(|t| t.is_active())
    }

    /// Check if any focus trap is active
    pub fn has_active_trap(&self) -> bool {
        self.traps.iter().any(|t| t.is_active())
    }

    /// Handle Tab key press
    pub fn handle_tab(&mut self, shift: bool) -> Option<NodeId> {
        self.keyboard_mode = true;
        self.focus_visible = true;

        // Show skip links on first Tab
        if self.current_focus.is_none() {
            self.show_skip_links();
        }

        // Check for active trap
        if let Some(trap) = self.active_trap_mut() {
            let next = if shift {
                trap.focus_previous()
            } else {
                trap.focus_next()
            };

            if let Some(id) = next {
                self.current_focus = Some(id);
            }

            return next;
        }

        None
    }

    /// Restore focus from history
    pub fn restore(&mut self) -> Option<NodeId> {
        if let Some(prev_id) = self.pop_history() {
            self.focus(prev_id, FocusChangeReason::Restore);
            Some(prev_id)
        } else {
            None
        }
    }

    /// Add a skip link
    pub fn add_skip_link(&mut self, link: SkipLink) {
        self.skip_links.push(link);
    }

    /// Remove a skip link by ID
    pub fn remove_skip_link(&mut self, id: &str) {
        self.skip_links.retain(|l| l.id != id);
    }

    /// Get all skip links
    pub fn skip_links(&self) -> &[SkipLink] {
        &self.skip_links
    }

    /// Show all skip links
    pub fn show_skip_links(&mut self) {
        for link in &mut self.skip_links {
            link.show();
        }
    }

    /// Hide all skip links
    pub fn hide_skip_links(&mut self) {
        for link in &mut self.skip_links {
            link.hide();
        }
    }

    /// Activate a skip link by ID
    pub fn activate_skip_link(&mut self, id: &str) -> Option<NodeId> {
        self.hide_skip_links();

        if let Some(link) = self.skip_links.iter().find(|l| l.id == id) {
            let target = link.target_id;
            self.focus(target, FocusChangeReason::SkipLink);
            Some(target)
        } else {
            None
        }
    }

    /// Set whether focus ring should be visible
    pub fn set_focus_visible(&mut self, visible: bool) {
        self.focus_visible = visible;
    }

    /// Check if focus ring should be visible
    pub fn is_focus_visible(&self) -> bool {
        self.focus_visible
    }

    /// Set keyboard mode (shows focus ring)
    pub fn set_keyboard_mode(&mut self, keyboard: bool) {
        self.keyboard_mode = keyboard;
        self.focus_visible = keyboard;
    }

    /// Check if in keyboard mode
    pub fn is_keyboard_mode(&self) -> bool {
        self.keyboard_mode
    }

    /// Get the focus ring style
    pub fn focus_ring_style(&self) -> &FocusRingStyle {
        &self.focus_ring_style
    }

    /// Set the focus ring style
    pub fn set_focus_ring_style(&mut self, style: FocusRingStyle) {
        self.focus_ring_style = style;
    }

    /// Push to focus history
    fn push_history(&mut self, id: NodeId) {
        if self.focus_history.len() >= self.max_history {
            self.focus_history.pop_front();
        }
        self.focus_history.push_back(id);
    }

    /// Pop from focus history
    fn pop_history(&mut self) -> Option<NodeId> {
        self.focus_history.pop_back()
    }

    /// Clear focus history
    pub fn clear_history(&mut self) {
        self.focus_history.clear();
    }

    /// Get focus history
    pub fn history(&self) -> impl Iterator<Item = &NodeId> {
        self.focus_history.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node_id() -> NodeId {
        NodeId::new()
    }

    #[test]
    fn test_focus_ring_style_default() {
        let style = FocusRingStyle::default();
        assert_eq!(style.width, 2.0);
        assert_eq!(style.style, FocusRingType::Outline);
    }

    #[test]
    fn test_focus_ring_style_high_contrast() {
        let style = FocusRingStyle::high_contrast();
        assert_eq!(style.color, "#000000");
        assert_eq!(style.width, 3.0);
    }

    #[test]
    fn test_focus_trap_basic() {
        let container = node_id();
        let mut trap = FocusTrap::new(container);

        let btn1 = node_id();
        let btn2 = node_id();
        let btn3 = node_id();

        trap.add_focusable(btn1);
        trap.add_focusable(btn2);
        trap.add_focusable(btn3);

        trap.activate(None);

        assert!(trap.is_active());
        assert_eq!(trap.current_focus(), Some(btn1));

        assert_eq!(trap.focus_next(), Some(btn2));
        assert_eq!(trap.focus_next(), Some(btn3));
        assert_eq!(trap.focus_next(), Some(btn1)); // wraps
    }

    #[test]
    fn test_focus_trap_previous() {
        let container = node_id();
        let mut trap = FocusTrap::new(container);

        let btn1 = node_id();
        let btn2 = node_id();

        trap.add_focusable(btn1);
        trap.add_focusable(btn2);
        trap.activate(None);

        assert_eq!(trap.focus_previous(), Some(btn2)); // wraps back
        assert_eq!(trap.focus_previous(), Some(btn1));
    }

    #[test]
    fn test_focus_trap_restore() {
        let container = node_id();
        let mut trap = FocusTrap::new(container);
        trap.add_focusable(node_id());

        let trigger = node_id();
        trap.activate(Some(trigger));

        assert!(trap.is_active());
        let restore = trap.deactivate();
        assert_eq!(restore, Some(trigger));
        assert!(!trap.is_active());
    }

    #[test]
    fn test_focus_trap_no_wrap() {
        let container = node_id();
        let mut trap = FocusTrap::new(container);
        trap.set_wrap(false);

        let btn1 = node_id();
        let btn2 = node_id();

        trap.add_focusable(btn1);
        trap.add_focusable(btn2);
        trap.activate(None);

        // At first element, previous stays at first
        assert_eq!(trap.current_focus(), Some(btn1));
        trap.focus_previous();
        assert_eq!(trap.current_focus(), Some(btn1));

        // At last element, next stays at last
        trap.focus_last();
        trap.focus_next();
        assert_eq!(trap.current_focus(), Some(btn2));
    }

    #[test]
    fn test_skip_link_creation() {
        let target = node_id();
        let link = SkipLink::to_main(target);

        assert_eq!(link.id, "skip-main");
        assert_eq!(link.label, "Skip to main content");
        assert!(!link.visible);
    }

    #[test]
    fn test_skip_link_visibility() {
        let mut link = SkipLink::to_main(node_id());
        assert!(!link.visible);

        link.show();
        assert!(link.visible);

        link.hide();
        assert!(!link.visible);
    }

    #[test]
    fn test_focus_manager_basic() {
        let mut manager = FocusManager::new();

        let btn1 = node_id();
        let btn2 = node_id();

        manager.focus(btn1, FocusChangeReason::Programmatic);
        assert_eq!(manager.current(), Some(btn1));

        manager.focus(btn2, FocusChangeReason::Click);
        assert_eq!(manager.current(), Some(btn2));
    }

    #[test]
    fn test_focus_manager_restore() {
        let mut manager = FocusManager::new();

        let btn1 = node_id();
        let btn2 = node_id();

        manager.focus(btn1, FocusChangeReason::Programmatic);
        manager.focus(btn2, FocusChangeReason::Programmatic);

        assert_eq!(manager.restore(), Some(btn1));
        assert_eq!(manager.current(), Some(btn1));
    }

    #[test]
    fn test_focus_manager_with_trap() {
        let mut manager = FocusManager::new();

        let page_element = node_id();
        manager.focus(page_element, FocusChangeReason::Programmatic);

        let container = node_id();
        let mut trap = FocusTrap::new(container);
        let dialog_btn1 = node_id();
        let dialog_btn2 = node_id();
        trap.add_focusable(dialog_btn1);
        trap.add_focusable(dialog_btn2);

        manager.push_trap(trap);
        assert_eq!(manager.current(), Some(dialog_btn1));

        manager.handle_tab(false);
        assert_eq!(manager.current(), Some(dialog_btn2));

        manager.pop_trap();
        assert_eq!(manager.current(), Some(page_element));
    }

    #[test]
    fn test_focus_manager_keyboard_mode() {
        let mut manager = FocusManager::new();

        let btn = node_id();
        manager.focus(btn, FocusChangeReason::Click);
        assert!(!manager.is_keyboard_mode());
        assert!(!manager.is_focus_visible());

        manager.focus(btn, FocusChangeReason::Tab);
        assert!(manager.is_keyboard_mode());
        assert!(manager.is_focus_visible());
    }

    #[test]
    fn test_focus_manager_skip_links() {
        let mut manager = FocusManager::new();

        let main = node_id();
        let nav = node_id();

        manager.add_skip_link(SkipLink::to_main(main));
        manager.add_skip_link(SkipLink::to_navigation(nav));

        assert_eq!(manager.skip_links().len(), 2);

        manager.show_skip_links();
        assert!(manager.skip_links()[0].visible);

        let result = manager.activate_skip_link("skip-main");
        assert_eq!(result, Some(main));
        assert_eq!(manager.current(), Some(main));
        assert!(!manager.skip_links()[0].visible);
    }

    #[test]
    fn test_focus_manager_blur() {
        let mut manager = FocusManager::new();
        manager.focus(node_id(), FocusChangeReason::Programmatic);
        assert!(manager.current().is_some());

        manager.blur();
        assert!(manager.current().is_none());
    }

    #[test]
    fn test_focus_manager_disabled() {
        let mut manager = FocusManager::new();
        manager.set_enabled(false);

        let result = manager.focus(node_id(), FocusChangeReason::Programmatic);
        assert!(result.is_none());
        assert!(manager.current().is_none());
    }

    #[test]
    fn test_focus_trap_contains() {
        let container = node_id();
        let mut trap = FocusTrap::new(container);

        let inside = node_id();
        let outside = node_id();

        trap.add_focusable(inside);

        assert!(trap.contains(&inside));
        assert!(!trap.contains(&outside));
    }

    #[test]
    fn test_focus_trap_focus_first_last() {
        let container = node_id();
        let mut trap = FocusTrap::new(container);

        let btn1 = node_id();
        let btn2 = node_id();
        let btn3 = node_id();

        trap.add_focusable(btn1);
        trap.add_focusable(btn2);
        trap.add_focusable(btn3);
        trap.activate(None);

        assert_eq!(trap.focus_last(), Some(btn3));
        assert_eq!(trap.current_focus(), Some(btn3));

        assert_eq!(trap.focus_first(), Some(btn1));
        assert_eq!(trap.current_focus(), Some(btn1));
    }

    #[test]
    fn test_focus_trap_remove_focusable() {
        let container = node_id();
        let mut trap = FocusTrap::new(container);

        let btn1 = node_id();
        let btn2 = node_id();

        trap.add_focusable(btn1);
        trap.add_focusable(btn2);
        assert_eq!(trap.len(), 2);

        trap.remove_focusable(&btn1);
        assert_eq!(trap.len(), 1);
        assert!(!trap.contains(&btn1));
    }

    #[test]
    fn test_focus_history_limit() {
        let mut manager = FocusManager::new();
        manager.max_history = 3;

        for _ in 0..5 {
            manager.focus(node_id(), FocusChangeReason::Programmatic);
        }

        // History should be limited to 3
        assert_eq!(manager.history().count(), 3);
    }
}
