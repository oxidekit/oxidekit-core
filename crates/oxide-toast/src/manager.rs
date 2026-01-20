//! Toast manager for queue management and display coordination.
//!
//! Provides a global toast queue with configurable stacking,
//! maximum visible toasts, and automatic lifecycle management.

use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::animation::{ActiveAnimation, ToastAnimation};
use crate::options::{ToastDefaults, ToastPosition, ToastStackOrder};
use crate::toast::{Toast, ToastState, ToastType};

/// Event emitted by the toast manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToastEvent {
    /// A toast was added to the queue
    Added { toast_id: Uuid },
    /// A toast started its entrance animation
    Entering { toast_id: Uuid },
    /// A toast became fully visible
    Shown { toast_id: Uuid },
    /// A toast's timer was paused
    Paused { toast_id: Uuid },
    /// A toast's timer was resumed
    Resumed { toast_id: Uuid },
    /// A toast started its exit animation
    Exiting { toast_id: Uuid },
    /// A toast was dismissed and removed
    Dismissed { toast_id: Uuid },
    /// An action button was clicked
    ActionClicked { toast_id: Uuid, action_id: String },
    /// A toast was swiped to dismiss
    SwipeDismissed { toast_id: Uuid },
}

/// Configuration for the toast manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToastManagerConfig {
    /// Maximum number of toasts visible at once
    pub max_visible: usize,
    /// Stack order for new toasts
    pub stack_order: ToastStackOrder,
    /// Gap between stacked toasts in pixels
    pub stack_gap: f32,
    /// Default settings for toasts
    pub defaults: ToastDefaults,
    /// Whether to automatically remove dismissed toasts
    pub auto_remove: bool,
    /// Animation duration for stack reordering
    pub stack_animation_ms: u64,
}

impl Default for ToastManagerConfig {
    fn default() -> Self {
        Self {
            max_visible: 5,
            stack_order: ToastStackOrder::NewestOnTop,
            stack_gap: 12.0,
            defaults: ToastDefaults::default(),
            auto_remove: true,
            stack_animation_ms: 150,
        }
    }
}

/// Manages a queue of toasts with lifecycle coordination.
#[derive(Debug)]
pub struct ToastManager {
    /// Configuration
    config: ToastManagerConfig,
    /// All toasts (including queued and dismissed)
    toasts: HashMap<Uuid, Toast>,
    /// Queue of toast IDs waiting to be shown
    queue: VecDeque<Uuid>,
    /// Currently visible toast IDs (in display order)
    visible: Vec<Uuid>,
    /// Active animations
    animations: HashMap<Uuid, ActiveAnimation>,
    /// Pending events to be processed
    events: Vec<ToastEvent>,
}

impl ToastManager {
    /// Creates a new toast manager with default configuration.
    pub fn new() -> Self {
        Self::with_config(ToastManagerConfig::default())
    }

    /// Creates a new toast manager with custom configuration.
    pub fn with_config(config: ToastManagerConfig) -> Self {
        Self {
            config,
            toasts: HashMap::new(),
            queue: VecDeque::new(),
            visible: Vec::new(),
            animations: HashMap::new(),
            events: Vec::new(),
        }
    }

    /// Returns the configuration.
    pub fn config(&self) -> &ToastManagerConfig {
        &self.config
    }

    /// Returns a mutable reference to the configuration.
    pub fn config_mut(&mut self) -> &mut ToastManagerConfig {
        &mut self.config
    }

    /// Adds a toast to the queue.
    pub fn add(&mut self, mut toast: Toast) -> Uuid {
        let id = toast.id;

        // Apply default settings
        if toast.duration == crate::options::ToastDuration::default() {
            toast.duration = self.config.defaults.duration;
        }
        if toast.position == ToastPosition::default() {
            toast.position = self.config.defaults.position;
        }
        if toast.entrance_animation == ToastAnimation::default() {
            toast.entrance_animation = self.config.defaults.entrance_animation.clone();
        }
        if toast.exit_animation == ToastAnimation::default() {
            toast.exit_animation = self.config.defaults.exit_animation.clone();
        }

        self.toasts.insert(id, toast);
        self.queue.push_back(id);
        self.events.push(ToastEvent::Added { toast_id: id });

        id
    }

    /// Creates and adds an info toast.
    pub fn info(&mut self, message: impl Into<String>) -> Uuid {
        self.add(Toast::info(message))
    }

    /// Creates and adds a success toast.
    pub fn success(&mut self, message: impl Into<String>) -> Uuid {
        self.add(Toast::success(message))
    }

    /// Creates and adds a warning toast.
    pub fn warning(&mut self, message: impl Into<String>) -> Uuid {
        self.add(Toast::warning(message))
    }

    /// Creates and adds an error toast.
    pub fn error(&mut self, message: impl Into<String>) -> Uuid {
        self.add(Toast::error(message))
    }

    /// Dismisses a toast by ID.
    pub fn dismiss(&mut self, toast_id: Uuid, current_time: u64) {
        if let Some(toast) = self.toasts.get_mut(&toast_id) {
            match toast.state {
                ToastState::Queued => {
                    // Remove from queue without animation
                    self.queue.retain(|id| *id != toast_id);
                    toast.dismiss();
                    self.events.push(ToastEvent::Dismissed { toast_id });
                }
                ToastState::Entering | ToastState::Visible | ToastState::Paused => {
                    // Start exit animation
                    toast.exit();
                    let animation =
                        ActiveAnimation::exit(toast.exit_animation.clone(), current_time);
                    self.animations.insert(toast_id, animation);
                    self.events.push(ToastEvent::Exiting { toast_id });
                }
                ToastState::Exiting | ToastState::Dismissed => {
                    // Already dismissing or dismissed
                }
            }
        }
    }

    /// Dismisses all toasts.
    pub fn dismiss_all(&mut self, current_time: u64) {
        let ids: Vec<Uuid> = self.toasts.keys().copied().collect();
        for id in ids {
            self.dismiss(id, current_time);
        }
    }

    /// Dismisses all toasts of a specific type.
    pub fn dismiss_by_type(&mut self, toast_type: ToastType, current_time: u64) {
        let ids: Vec<Uuid> = self
            .toasts
            .iter()
            .filter(|(_, t)| t.toast_type == toast_type)
            .map(|(id, _)| *id)
            .collect();
        for id in ids {
            self.dismiss(id, current_time);
        }
    }

    /// Pauses a toast's timer (e.g., on hover).
    pub fn pause(&mut self, toast_id: Uuid) {
        if let Some(toast) = self.toasts.get_mut(&toast_id) {
            if toast.pause_on_hover && toast.state == ToastState::Visible {
                toast.pause();
                self.events.push(ToastEvent::Paused { toast_id });
            }
        }
    }

    /// Resumes a toast's timer.
    pub fn resume(&mut self, toast_id: Uuid) {
        if let Some(toast) = self.toasts.get_mut(&toast_id) {
            if toast.state == ToastState::Paused {
                toast.resume();
                self.events.push(ToastEvent::Resumed { toast_id });
            }
        }
    }

    /// Handles an action button click.
    pub fn handle_action(&mut self, toast_id: Uuid, current_time: u64) -> Option<String> {
        if let Some(toast) = self.toasts.get(&toast_id) {
            if let Some(action) = &toast.action {
                let action_id = action.action_id.clone();
                let dismiss = action.dismiss_on_click;

                self.events.push(ToastEvent::ActionClicked {
                    toast_id,
                    action_id: action_id.clone(),
                });

                if dismiss {
                    self.dismiss(toast_id, current_time);
                }

                return Some(action_id);
            }
        }
        None
    }

    /// Handles a swipe-to-dismiss gesture.
    pub fn handle_swipe_dismiss(&mut self, toast_id: Uuid, current_time: u64) {
        if let Some(toast) = self.toasts.get(&toast_id) {
            if toast.swipe_to_dismiss {
                self.events
                    .push(ToastEvent::SwipeDismissed { toast_id });
                self.dismiss(toast_id, current_time);
            }
        }
    }

    /// Updates the manager state. Call this on each frame.
    /// Returns true if any state changed.
    pub fn update(&mut self, current_time: u64, delta_ms: u64) -> bool {
        let mut changed = false;

        // Show queued toasts if we have room
        while self.visible.len() < self.config.max_visible && !self.queue.is_empty() {
            if let Some(toast_id) = self.queue.pop_front() {
                if let Some(toast) = self.toasts.get_mut(&toast_id) {
                    toast.enter();
                    let animation =
                        ActiveAnimation::entrance(toast.entrance_animation.clone(), current_time);
                    self.animations.insert(toast_id, animation);

                    match self.config.stack_order {
                        ToastStackOrder::NewestOnTop => self.visible.insert(0, toast_id),
                        ToastStackOrder::NewestOnBottom => self.visible.push(toast_id),
                    }

                    self.events.push(ToastEvent::Entering { toast_id });
                    changed = true;
                }
            }
        }

        // Update animations
        let animation_ids: Vec<Uuid> = self.animations.keys().copied().collect();
        for toast_id in animation_ids {
            if let Some(animation) = self.animations.get_mut(&toast_id) {
                if animation.update(current_time) {
                    changed = true;

                    if animation.completed {
                        if let Some(toast) = self.toasts.get_mut(&toast_id) {
                            if animation.is_entrance {
                                toast.show();
                                self.events.push(ToastEvent::Shown { toast_id });
                            } else {
                                toast.dismiss();
                                self.visible.retain(|id| *id != toast_id);
                                self.events.push(ToastEvent::Dismissed { toast_id });
                            }
                        }
                        self.animations.remove(&toast_id);
                    }
                }
            }
        }

        // Update visible toast timers
        let visible_ids: Vec<Uuid> = self.visible.clone();
        for toast_id in visible_ids {
            if let Some(toast) = self.toasts.get_mut(&toast_id) {
                if toast.state == ToastState::Visible && toast.should_auto_dismiss() {
                    if let Some(total_ms) = toast.duration.as_millis() {
                        let progress_delta = delta_ms as f32 / total_ms as f32;
                        toast.progress = (toast.progress + progress_delta).min(1.0);
                        changed = true;

                        if toast.progress >= 1.0 {
                            self.dismiss(toast_id, current_time);
                        }
                    }
                }
            }
        }

        // Remove dismissed toasts if auto_remove is enabled
        if self.config.auto_remove {
            self.toasts.retain(|_, toast| toast.state != ToastState::Dismissed);
        }

        changed
    }

    /// Returns a toast by ID.
    pub fn get(&self, toast_id: Uuid) -> Option<&Toast> {
        self.toasts.get(&toast_id)
    }

    /// Returns a mutable toast by ID.
    pub fn get_mut(&mut self, toast_id: Uuid) -> Option<&mut Toast> {
        self.toasts.get_mut(&toast_id)
    }

    /// Returns all visible toasts in display order.
    pub fn visible_toasts(&self) -> Vec<&Toast> {
        self.visible
            .iter()
            .filter_map(|id| self.toasts.get(id))
            .collect()
    }

    /// Returns the number of visible toasts.
    pub fn visible_count(&self) -> usize {
        self.visible.len()
    }

    /// Returns the number of queued toasts.
    pub fn queued_count(&self) -> usize {
        self.queue.len()
    }

    /// Returns the total number of toasts (visible + queued).
    pub fn total_count(&self) -> usize {
        self.toasts.len()
    }

    /// Returns the animation state for a toast.
    pub fn animation_state(&self, toast_id: Uuid) -> Option<&ActiveAnimation> {
        self.animations.get(&toast_id)
    }

    /// Drains all pending events.
    pub fn drain_events(&mut self) -> Vec<ToastEvent> {
        std::mem::take(&mut self.events)
    }

    /// Returns pending events without removing them.
    pub fn peek_events(&self) -> &[ToastEvent] {
        &self.events
    }

    /// Calculates the vertical offset for a toast at the given index.
    pub fn calculate_offset(&self, index: usize, toast_height: f32) -> f32 {
        let gap = self.config.stack_gap;
        index as f32 * (toast_height + gap)
    }

    /// Clears all toasts immediately without animations.
    pub fn clear(&mut self) {
        self.toasts.clear();
        self.queue.clear();
        self.visible.clear();
        self.animations.clear();
        self.events.clear();
    }
}

impl Default for ToastManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = ToastManager::new();
        assert_eq!(manager.visible_count(), 0);
        assert_eq!(manager.queued_count(), 0);
        assert_eq!(manager.total_count(), 0);
    }

    #[test]
    fn test_manager_add_toast() {
        let mut manager = ToastManager::new();
        let id = manager.info("Test message");

        assert_eq!(manager.total_count(), 1);
        assert_eq!(manager.queued_count(), 1);
        assert_eq!(manager.visible_count(), 0);

        let toast = manager.get(id).unwrap();
        assert_eq!(toast.message, "Test message");
        assert_eq!(toast.toast_type, ToastType::Info);
    }

    #[test]
    fn test_manager_show_toast() {
        let mut manager = ToastManager::new();
        let id = manager.success("Success!");

        // Update to show the toast
        manager.update(0, 0);

        assert_eq!(manager.visible_count(), 1);
        assert_eq!(manager.queued_count(), 0);

        let toast = manager.get(id).unwrap();
        assert_eq!(toast.state, ToastState::Entering);
    }

    #[test]
    fn test_manager_max_visible() {
        let mut config = ToastManagerConfig::default();
        config.max_visible = 3;
        let mut manager = ToastManager::with_config(config);

        for i in 0..5 {
            manager.info(format!("Toast {}", i));
        }

        manager.update(0, 0);

        assert_eq!(manager.visible_count(), 3);
        assert_eq!(manager.queued_count(), 2);
    }

    #[test]
    fn test_manager_dismiss() {
        let mut manager = ToastManager::new();
        let id = manager.info("Test");

        manager.update(0, 0);
        manager.update(300, 0); // Complete entrance animation

        manager.dismiss(id, 300);

        let toast = manager.get(id).unwrap();
        assert_eq!(toast.state, ToastState::Exiting);
    }

    #[test]
    fn test_manager_dismiss_queued() {
        let mut manager = ToastManager::new();
        manager.config_mut().max_visible = 0; // Force queuing

        let id = manager.info("Test");
        assert_eq!(manager.queued_count(), 1);

        manager.dismiss(id, 0);
        assert_eq!(manager.queued_count(), 0);
    }

    #[test]
    fn test_manager_pause_resume() {
        let mut manager = ToastManager::new();
        let id = manager.info("Test");

        // Get toast to visible state
        manager.update(0, 0);
        manager.update(300, 0);

        manager.pause(id);
        assert_eq!(manager.get(id).unwrap().state, ToastState::Paused);

        manager.resume(id);
        assert_eq!(manager.get(id).unwrap().state, ToastState::Visible);
    }

    #[test]
    fn test_manager_action_handling() {
        let mut manager = ToastManager::new();
        let toast = Toast::info("Test").action("Undo", "undo_action");
        let id = manager.add(toast);

        manager.update(0, 0);
        manager.update(300, 0);

        let action_id = manager.handle_action(id, 300);
        assert_eq!(action_id, Some("undo_action".to_string()));
    }

    #[test]
    fn test_manager_events() {
        let mut manager = ToastManager::new();
        let id = manager.info("Test");

        let events = manager.drain_events();
        assert_eq!(events.len(), 1);
        match &events[0] {
            ToastEvent::Added { toast_id } => assert_eq!(*toast_id, id),
            _ => panic!("Expected Added event"),
        }

        // Events should be drained
        assert!(manager.drain_events().is_empty());
    }

    #[test]
    fn test_manager_stack_order_newest_on_top() {
        let mut config = ToastManagerConfig::default();
        config.stack_order = ToastStackOrder::NewestOnTop;
        let mut manager = ToastManager::with_config(config);

        let id1 = manager.info("First");
        manager.update(0, 0);

        let id2 = manager.info("Second");
        manager.update(100, 0);

        let visible = manager.visible_toasts();
        assert_eq!(visible.len(), 2);
        assert_eq!(visible[0].id, id2); // Newest on top (first in list)
        assert_eq!(visible[1].id, id1);
    }

    #[test]
    fn test_manager_stack_order_newest_on_bottom() {
        let mut config = ToastManagerConfig::default();
        config.stack_order = ToastStackOrder::NewestOnBottom;
        let mut manager = ToastManager::with_config(config);

        let id1 = manager.info("First");
        manager.update(0, 0);

        let id2 = manager.info("Second");
        manager.update(100, 0);

        let visible = manager.visible_toasts();
        assert_eq!(visible.len(), 2);
        assert_eq!(visible[0].id, id1);
        assert_eq!(visible[1].id, id2); // Newest on bottom (last in list)
    }

    #[test]
    fn test_manager_calculate_offset() {
        let manager = ToastManager::new();
        let toast_height = 60.0;

        assert!((manager.calculate_offset(0, toast_height) - 0.0).abs() < 0.001);
        assert!((manager.calculate_offset(1, toast_height) - 72.0).abs() < 0.001); // 60 + 12 gap
        assert!((manager.calculate_offset(2, toast_height) - 144.0).abs() < 0.001);
    }

    #[test]
    fn test_manager_clear() {
        let mut manager = ToastManager::new();
        manager.info("Test 1");
        manager.info("Test 2");
        manager.update(0, 0);

        manager.clear();

        assert_eq!(manager.total_count(), 0);
        assert_eq!(manager.visible_count(), 0);
        assert_eq!(manager.queued_count(), 0);
    }

    #[test]
    fn test_manager_dismiss_by_type() {
        let mut manager = ToastManager::new();
        let _info_id = manager.info("Info");
        let _error_id = manager.error("Error");
        let _success_id = manager.success("Success");

        manager.update(0, 0);
        manager.update(300, 0);

        manager.dismiss_by_type(ToastType::Info, 300);

        // Info toast should be exiting
        let visible = manager.visible_toasts();
        let info_toast = visible.iter().find(|t| t.toast_type == ToastType::Info);
        if let Some(toast) = info_toast {
            assert_eq!(toast.state, ToastState::Exiting);
        }
    }

    #[test]
    fn test_manager_auto_dismiss() {
        use crate::options::ToastDuration;

        let mut manager = ToastManager::new();
        let toast = Toast::info("Test").duration(ToastDuration::Custom(100));
        let id = manager.add(toast);

        // Show the toast
        manager.update(0, 0);
        manager.update(300, 0); // Complete entrance

        // Progress timer
        manager.update(350, 50);
        assert!(manager.get(id).unwrap().progress > 0.0);

        // Complete timer
        manager.update(500, 100);
        assert_eq!(manager.get(id).unwrap().state, ToastState::Exiting);
    }
}
