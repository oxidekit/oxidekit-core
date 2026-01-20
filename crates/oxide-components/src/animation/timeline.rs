//! Animation Timeline
//!
//! Provides timeline support for sequencing and grouping animations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::interpolate::AnimatableValue;
use super::state::{Animation, AnimationStatus, PlayDirection};

/// Unique identifier for a timeline entry
pub type EntryId = u64;

/// Kind of timeline entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimelineEntryKind {
    /// A single animation
    Animation(Animation),
    /// A nested timeline (for grouping)
    Timeline(Box<Timeline>),
}

/// An entry in a timeline with timing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    /// Unique entry ID
    pub id: EntryId,
    /// The animation or nested timeline
    pub kind: TimelineEntryKind,
    /// Start time relative to timeline start (in seconds)
    pub start_time: f32,
    /// Label for this entry (optional, for seeking)
    #[serde(default)]
    pub label: Option<String>,
}

impl TimelineEntry {
    /// Get the duration of this entry
    pub fn duration(&self) -> f32 {
        match &self.kind {
            TimelineEntryKind::Animation(anim) => anim.total_duration(),
            TimelineEntryKind::Timeline(timeline) => timeline.duration(),
        }
    }

    /// Get the end time (start + duration)
    pub fn end_time(&self) -> f32 {
        self.start_time + self.duration()
    }

    /// Check if this entry is active at a given time
    pub fn is_active_at(&self, time: f32) -> bool {
        time >= self.start_time && time < self.end_time()
    }
}

/// Position mode for adding entries
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimelinePosition {
    /// Add at a specific time
    At(f32),
    /// Add after the previous entry ends
    AfterPrevious,
    /// Add at the same time as the previous entry
    WithPrevious,
    /// Add relative to a label
    AfterLabel(f32), // offset from label
}

/// Timeline status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TimelineStatus {
    #[default]
    Idle,
    Running,
    Paused,
    Completed,
}

/// A timeline for sequencing animations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    /// All entries in the timeline
    entries: Vec<TimelineEntry>,

    /// Labels for seeking (label -> time)
    #[serde(skip)]
    labels: HashMap<String, f32>,

    /// Next entry ID
    #[serde(skip)]
    next_id: EntryId,

    /// Current playback time (in seconds)
    #[serde(skip)]
    current_time: f32,

    /// Total duration (cached)
    #[serde(skip)]
    cached_duration: f32,

    /// Current status
    #[serde(skip)]
    status: TimelineStatus,

    /// Playback speed multiplier
    #[serde(default = "default_speed")]
    pub speed: f32,

    /// Number of times to repeat (0 = infinite, 1 = play once)
    #[serde(default = "default_repeat")]
    pub repeat: u32,

    /// Current repeat iteration
    #[serde(skip)]
    current_iteration: u32,

    /// Play direction
    #[serde(default)]
    pub direction: PlayDirection,

    /// Whether to yoyo (alternate direction each iteration)
    #[serde(default)]
    pub yoyo: bool,
}

fn default_speed() -> f32 {
    1.0
}

fn default_repeat() -> u32 {
    1
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new()
    }
}

impl Timeline {
    /// Create a new empty timeline
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            labels: HashMap::new(),
            next_id: 0,
            current_time: 0.0,
            cached_duration: 0.0,
            status: TimelineStatus::Idle,
            speed: 1.0,
            repeat: 1,
            current_iteration: 0,
            direction: PlayDirection::Forward,
            yoyo: false,
        }
    }

    /// Add an animation to the timeline
    pub fn add(&mut self, animation: Animation, position: TimelinePosition) -> EntryId {
        let start_time = self.resolve_position(position);
        let id = self.next_id;
        self.next_id += 1;

        let entry = TimelineEntry {
            id,
            kind: TimelineEntryKind::Animation(animation),
            start_time,
            label: None,
        };

        self.entries.push(entry);
        self.update_duration();
        id
    }

    /// Add an animation with a label
    pub fn add_labeled(
        &mut self,
        animation: Animation,
        position: TimelinePosition,
        label: impl Into<String>,
    ) -> EntryId {
        let start_time = self.resolve_position(position);
        let label_str = label.into();
        let id = self.next_id;
        self.next_id += 1;

        self.labels.insert(label_str.clone(), start_time);

        let entry = TimelineEntry {
            id,
            kind: TimelineEntryKind::Animation(animation),
            start_time,
            label: Some(label_str),
        };

        self.entries.push(entry);
        self.update_duration();
        id
    }

    /// Add a nested timeline
    pub fn add_timeline(&mut self, timeline: Timeline, position: TimelinePosition) -> EntryId {
        let start_time = self.resolve_position(position);
        let id = self.next_id;
        self.next_id += 1;

        let entry = TimelineEntry {
            id,
            kind: TimelineEntryKind::Timeline(Box::new(timeline)),
            start_time,
            label: None,
        };

        self.entries.push(entry);
        self.update_duration();
        id
    }

    /// Add a label at a position
    pub fn add_label(&mut self, label: impl Into<String>, position: TimelinePosition) {
        let time = self.resolve_position(position);
        self.labels.insert(label.into(), time);
    }

    /// Resolve a position to a time value
    fn resolve_position(&self, position: TimelinePosition) -> f32 {
        match position {
            TimelinePosition::At(time) => time,
            TimelinePosition::AfterPrevious => {
                self.entries
                    .last()
                    .map(|e| e.end_time())
                    .unwrap_or(0.0)
            }
            TimelinePosition::WithPrevious => {
                self.entries
                    .last()
                    .map(|e| e.start_time)
                    .unwrap_or(0.0)
            }
            TimelinePosition::AfterLabel(offset) => offset, // Label handling would need more context
        }
    }

    /// Update the cached duration
    fn update_duration(&mut self) {
        self.cached_duration = self
            .entries
            .iter()
            .map(|e| e.end_time())
            .fold(0.0_f32, f32::max);
    }

    /// Get the total duration
    pub fn duration(&self) -> f32 {
        self.cached_duration
    }

    /// Get the current time
    pub fn current_time(&self) -> f32 {
        self.current_time
    }

    /// Get the current progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if self.cached_duration > 0.0 {
            (self.current_time / self.cached_duration).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Play the timeline
    pub fn play(&mut self) {
        match self.status {
            TimelineStatus::Idle | TimelineStatus::Completed => {
                self.current_time = 0.0;
                self.current_iteration = 0;
                self.status = TimelineStatus::Running;
                // Reset all animations
                for entry in &mut self.entries {
                    if let TimelineEntryKind::Animation(anim) = &mut entry.kind {
                        anim.reset();
                    }
                }
            }
            TimelineStatus::Paused => {
                self.status = TimelineStatus::Running;
            }
            _ => {}
        }
    }

    /// Pause the timeline
    pub fn pause(&mut self) {
        if self.status == TimelineStatus::Running {
            self.status = TimelineStatus::Paused;
        }
    }

    /// Resume the timeline
    pub fn resume(&mut self) {
        if self.status == TimelineStatus::Paused {
            self.status = TimelineStatus::Running;
        }
    }

    /// Stop and reset the timeline
    pub fn stop(&mut self) {
        self.status = TimelineStatus::Idle;
        self.current_time = 0.0;
        self.current_iteration = 0;
    }

    /// Seek to a specific time
    pub fn seek(&mut self, time: f32) {
        self.current_time = time.clamp(0.0, self.cached_duration);
        self.update_animations_to_time();
    }

    /// Seek to a label
    pub fn seek_label(&mut self, label: &str) {
        if let Some(&time) = self.labels.get(label) {
            self.seek(time);
        }
    }

    /// Seek to a progress value (0.0 to 1.0)
    pub fn seek_progress(&mut self, progress: f32) {
        self.seek(progress.clamp(0.0, 1.0) * self.cached_duration);
    }

    /// Update animations to the current time
    fn update_animations_to_time(&mut self) {
        for entry in &mut self.entries {
            if let TimelineEntryKind::Animation(anim) = &mut entry.kind {
                let local_time = self.current_time - entry.start_time;
                if local_time >= 0.0 && local_time <= anim.duration {
                    anim.seek(local_time / anim.duration);
                } else if local_time > anim.duration {
                    anim.seek(1.0);
                } else {
                    anim.seek(0.0);
                }
            }
        }
    }

    /// Update the timeline by a time delta (in seconds)
    /// Returns true if still active
    pub fn update(&mut self, dt: f32) -> bool {
        if self.status != TimelineStatus::Running {
            return self.status == TimelineStatus::Paused;
        }

        // Apply speed
        let scaled_dt = dt * self.speed;

        // Apply direction
        let effective_dt = match self.effective_direction() {
            PlayDirection::Forward | PlayDirection::Alternate => scaled_dt,
            PlayDirection::Reverse | PlayDirection::AlternateReverse => -scaled_dt,
        };

        self.current_time += effective_dt;

        // Update active animations
        for entry in &mut self.entries {
            let local_time = self.current_time - entry.start_time;

            match &mut entry.kind {
                TimelineEntryKind::Animation(anim) => {
                    if local_time >= 0.0 {
                        // Animation should be active
                        if anim.state.status == AnimationStatus::Idle {
                            anim.play();
                        }
                        // Calculate how much time to advance this animation
                        let anim_dt = if local_time < anim.duration {
                            scaled_dt.abs()
                        } else {
                            0.0
                        };
                        if anim_dt > 0.0 {
                            anim.update(anim_dt);
                        }
                    }
                }
                TimelineEntryKind::Timeline(timeline) => {
                    if local_time >= 0.0 {
                        if timeline.status == TimelineStatus::Idle {
                            timeline.play();
                        }
                        timeline.update(scaled_dt.abs());
                    }
                }
            }
        }

        // Check for completion
        let at_end = match self.effective_direction() {
            PlayDirection::Forward | PlayDirection::Alternate => {
                self.current_time >= self.cached_duration
            }
            PlayDirection::Reverse | PlayDirection::AlternateReverse => {
                self.current_time <= 0.0
            }
        };

        if at_end {
            self.current_iteration += 1;

            if self.repeat == 0 || self.current_iteration < self.repeat {
                // Start next iteration
                if self.yoyo {
                    // Reverse direction but don't reset time
                    // Just continue from current position
                } else {
                    // Reset to start
                    self.current_time = match self.effective_direction() {
                        PlayDirection::Forward | PlayDirection::Alternate => 0.0,
                        PlayDirection::Reverse | PlayDirection::AlternateReverse => {
                            self.cached_duration
                        }
                    };
                    // Reset animations
                    for entry in &mut self.entries {
                        if let TimelineEntryKind::Animation(anim) = &mut entry.kind {
                            anim.reset();
                        }
                    }
                }
                true
            } else {
                // Complete
                self.status = TimelineStatus::Completed;
                // Clamp to end
                self.current_time = match self.effective_direction() {
                    PlayDirection::Forward | PlayDirection::Alternate => self.cached_duration,
                    PlayDirection::Reverse | PlayDirection::AlternateReverse => 0.0,
                };
                false
            }
        } else {
            true
        }
    }

    /// Get the effective play direction considering yoyo
    fn effective_direction(&self) -> PlayDirection {
        if self.yoyo && self.current_iteration % 2 == 1 {
            match self.direction {
                PlayDirection::Forward => PlayDirection::Reverse,
                PlayDirection::Reverse => PlayDirection::Forward,
                PlayDirection::Alternate => PlayDirection::AlternateReverse,
                PlayDirection::AlternateReverse => PlayDirection::Alternate,
            }
        } else {
            self.direction
        }
    }

    /// Get current values for all properties
    pub fn get_values(&self) -> HashMap<String, AnimatableValue> {
        let mut values = HashMap::new();

        for entry in &self.entries {
            match &entry.kind {
                TimelineEntryKind::Animation(anim) => {
                    if let Some(value) = anim.current_value() {
                        values.insert(anim.property.clone(), value);
                    }
                }
                TimelineEntryKind::Timeline(timeline) => {
                    values.extend(timeline.get_values());
                }
            }
        }

        values
    }

    /// Get the current status
    pub fn status(&self) -> TimelineStatus {
        self.status
    }

    /// Check if the timeline is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, TimelineStatus::Running | TimelineStatus::Paused)
    }

    /// Check if the timeline is complete
    pub fn is_complete(&self) -> bool {
        self.status == TimelineStatus::Completed
    }

    /// Get all entries
    pub fn entries(&self) -> &[TimelineEntry] {
        &self.entries
    }

    /// Get an entry by ID
    pub fn get_entry(&self, id: EntryId) -> Option<&TimelineEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Get a mutable entry by ID
    pub fn get_entry_mut(&mut self, id: EntryId) -> Option<&mut TimelineEntry> {
        self.entries.iter_mut().find(|e| e.id == id)
    }

    /// Remove an entry by ID
    pub fn remove_entry(&mut self, id: EntryId) -> bool {
        if let Some(pos) = self.entries.iter().position(|e| e.id == id) {
            self.entries.remove(pos);
            self.update_duration();
            true
        } else {
            false
        }
    }
}

/// Builder for creating timelines with a fluent API
#[allow(dead_code)]
pub struct TimelineBuilder {
    timeline: Timeline,
    cursor: f32,
}

#[allow(dead_code)]
impl TimelineBuilder {
    pub fn new() -> Self {
        Self {
            timeline: Timeline::new(),
            cursor: 0.0,
        }
    }

    /// Add an animation at the current cursor position
    pub fn add(mut self, animation: Animation) -> Self {
        let duration = animation.total_duration();
        self.timeline.add(animation, TimelinePosition::At(self.cursor));
        self.cursor += duration;
        self
    }

    /// Add an animation at the current cursor, then advance cursor
    pub fn then(self, animation: Animation) -> Self {
        self.add(animation)
    }

    /// Add an animation at the current cursor without advancing
    pub fn with(mut self, animation: Animation) -> Self {
        self.timeline.add(animation, TimelinePosition::At(self.cursor));
        self
    }

    /// Move the cursor to a specific time
    pub fn at(mut self, time: f32) -> Self {
        self.cursor = time;
        self
    }

    /// Add a delay
    pub fn delay(mut self, duration: f32) -> Self {
        self.cursor += duration;
        self
    }

    /// Add a label at the current position
    pub fn label(mut self, name: impl Into<String>) -> Self {
        self.timeline.labels.insert(name.into(), self.cursor);
        self
    }

    /// Set the speed
    pub fn speed(mut self, speed: f32) -> Self {
        self.timeline.speed = speed;
        self
    }

    /// Set the repeat count
    pub fn repeat(mut self, count: u32) -> Self {
        self.timeline.repeat = count;
        self
    }

    /// Set to repeat infinitely
    pub fn infinite(mut self) -> Self {
        self.timeline.repeat = 0;
        self
    }

    /// Enable yoyo mode
    pub fn yoyo(mut self) -> Self {
        self.timeline.yoyo = true;
        self
    }

    /// Build the timeline
    pub fn build(self) -> Timeline {
        self.timeline
    }
}

impl Default for TimelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::Easing;

    fn test_animation(property: &str, duration: f32) -> Animation {
        Animation::new(property)
            .from(0.0_f32)
            .to(1.0_f32)
            .duration(duration)
            .build()
    }

    #[test]
    fn test_timeline_add() {
        let mut timeline = Timeline::new();

        let id1 = timeline.add(test_animation("opacity", 1.0), TimelinePosition::At(0.0));
        let id2 = timeline.add(test_animation("x", 0.5), TimelinePosition::AfterPrevious);

        assert_eq!(timeline.entries.len(), 2);
        assert!((timeline.duration() - 1.5).abs() < 0.001);

        let entry1 = timeline.get_entry(id1).unwrap();
        assert!((entry1.start_time - 0.0).abs() < 0.001);

        let entry2 = timeline.get_entry(id2).unwrap();
        assert!((entry2.start_time - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_timeline_with_previous() {
        let mut timeline = Timeline::new();

        timeline.add(test_animation("opacity", 1.0), TimelinePosition::At(0.0));
        timeline.add(test_animation("x", 0.5), TimelinePosition::WithPrevious);

        // Both should start at 0
        assert!((timeline.entries[0].start_time - 0.0).abs() < 0.001);
        assert!((timeline.entries[1].start_time - 0.0).abs() < 0.001);

        // Duration should be max of both
        assert!((timeline.duration() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_timeline_update() {
        let mut timeline = Timeline::new();

        timeline.add(
            Animation::new("opacity")
                .from(0.0_f32)
                .to(1.0_f32)
                .duration(1.0)
                .easing(Easing::Linear)
                .build(),
            TimelinePosition::At(0.0),
        );

        timeline.play();
        timeline.update(0.5);

        let values = timeline.get_values();
        let opacity = values.get("opacity").unwrap().as_float().unwrap();
        assert!((opacity - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_timeline_seek() {
        let mut timeline = Timeline::new();

        timeline.add(
            Animation::new("opacity")
                .from(0.0_f32)
                .to(1.0_f32)
                .duration(1.0)
                .easing(Easing::Linear)
                .build(),
            TimelinePosition::At(0.0),
        );

        timeline.seek(0.75);

        let values = timeline.get_values();
        let opacity = values.get("opacity").unwrap().as_float().unwrap();
        assert!((opacity - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_timeline_builder() {
        let timeline = TimelineBuilder::new()
            .add(test_animation("opacity", 0.5))
            .delay(0.2)
            .add(test_animation("x", 0.3))
            .speed(2.0)
            .build();

        assert!((timeline.duration() - 1.0).abs() < 0.001);
        assert!((timeline.speed - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_timeline_complete() {
        let mut timeline = Timeline::new();

        timeline.add(test_animation("opacity", 0.5), TimelinePosition::At(0.0));

        timeline.play();
        let active = timeline.update(1.0); // More than duration

        assert!(!active);
        assert_eq!(timeline.status(), TimelineStatus::Completed);
    }

    #[test]
    fn test_timeline_repeat() {
        let mut timeline = Timeline::new();
        timeline.repeat = 2;

        timeline.add(test_animation("opacity", 0.5), TimelinePosition::At(0.0));

        timeline.play();

        // First iteration
        timeline.update(0.5);
        assert_eq!(timeline.status(), TimelineStatus::Running);

        // Should still be running (second iteration)
        timeline.update(0.4);
        assert_eq!(timeline.status(), TimelineStatus::Running);

        // Should complete after second iteration
        timeline.update(0.2);
        assert_eq!(timeline.status(), TimelineStatus::Completed);
    }

    #[test]
    fn test_timeline_labels() {
        let mut timeline = Timeline::new();

        timeline.add_labeled(
            test_animation("opacity", 1.0),
            TimelinePosition::At(0.0),
            "fade_in",
        );

        assert!(timeline.labels.contains_key("fade_in"));
        assert!((timeline.labels["fade_in"] - 0.0).abs() < 0.001);
    }
}
