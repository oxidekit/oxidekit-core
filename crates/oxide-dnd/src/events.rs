//! Drag and Drop Events
//!
//! Defines all event types for the drag and drop system, including
//! position updates, drag lifecycle events, and drop events.

use serde::{Deserialize, Serialize};
use std::any::Any;

/// Unique identifier for a drag operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DragId(pub u64);

impl DragId {
    /// Create a new drag ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// 2D point representing a position
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    /// Create a new point
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Zero point
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Calculate distance to another point
    pub fn distance_to(&self, other: &Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Calculate the delta between two points
    pub fn delta(&self, other: &Point) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    /// Add another point
    pub fn add(&self, other: &Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    /// Subtract another point
    pub fn sub(&self, other: &Point) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl From<(f32, f32)> for Point {
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}

impl From<Point> for (f32, f32) {
    fn from(point: Point) -> Self {
        (point.x, point.y)
    }
}

/// Size of an element
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    /// Create a new size
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Zero size
    pub fn zero() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
        }
    }
}

impl From<(f32, f32)> for Size {
    fn from((width, height): (f32, f32)) -> Self {
        Self { width, height }
    }
}

/// Rectangle representing bounds
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    /// Create a new rectangle
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Create from position and size
    pub fn from_point_size(point: Point, size: Size) -> Self {
        Self {
            x: point.x,
            y: point.y,
            width: size.width,
            height: size.height,
        }
    }

    /// Get the center point
    pub fn center(&self) -> Point {
        Point {
            x: self.x + self.width / 2.0,
            y: self.y + self.height / 2.0,
        }
    }

    /// Get the top-left corner
    pub fn origin(&self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }

    /// Get the size
    pub fn size(&self) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    /// Check if a point is inside the rectangle
    pub fn contains(&self, point: &Point) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }

    /// Check if this rectangle intersects another
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    /// Get the intersection area with another rectangle
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = (self.x + self.width).min(other.x + other.width);
        let bottom = (self.y + self.height).min(other.y + other.height);

        if right > x && bottom > y {
            Some(Rect::new(x, y, right - x, bottom - y))
        } else {
            None
        }
    }

    /// Calculate the area of the rectangle
    pub fn area(&self) -> f32 {
        self.width * self.height
    }

    /// Get the right edge
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// Get the bottom edge
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Get the left edge
    pub fn left(&self) -> f32 {
        self.x
    }

    /// Get the top edge
    pub fn top(&self) -> f32 {
        self.y
    }
}

/// Type identifier for drag data
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DragType(pub String);

impl DragType {
    /// Create a new drag type
    pub fn new(type_name: impl Into<String>) -> Self {
        Self(type_name.into())
    }

    /// Text drag type
    pub fn text() -> Self {
        Self::new("text")
    }

    /// File drag type
    pub fn file() -> Self {
        Self::new("file")
    }

    /// Image drag type
    pub fn image() -> Self {
        Self::new("image")
    }

    /// Custom drag type
    pub fn custom(name: &str) -> Self {
        Self::new(format!("custom:{}", name))
    }
}

impl<S: Into<String>> From<S> for DragType {
    fn from(s: S) -> Self {
        Self::new(s)
    }
}

/// Data payload for drag operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DragData {
    /// Simple text data
    Text(String),
    /// File path
    File(String),
    /// Image URL or path
    Image(String),
    /// Numeric identifier
    Id(u64),
    /// JSON-serialized custom data
    Json(String),
    /// Multiple items (for multi-drag)
    Multiple(Vec<DragData>),
    /// Empty/no data
    Empty,
}

impl DragData {
    /// Create text drag data
    pub fn text(s: impl Into<String>) -> Self {
        Self::Text(s.into())
    }

    /// Create file drag data
    pub fn file(path: impl Into<String>) -> Self {
        Self::File(path.into())
    }

    /// Create image drag data
    pub fn image(url: impl Into<String>) -> Self {
        Self::Image(url.into())
    }

    /// Create ID drag data
    pub fn id(id: u64) -> Self {
        Self::Id(id)
    }

    /// Create JSON drag data from serializable value
    pub fn json<T: Serialize>(value: &T) -> Result<Self, serde_json::Error> {
        Ok(Self::Json(serde_json::to_string(value)?))
    }

    /// Parse JSON data to a type
    pub fn parse_json<T: for<'a> Deserialize<'a>>(&self) -> Option<T> {
        match self {
            Self::Json(s) => serde_json::from_str(s).ok(),
            _ => None,
        }
    }

    /// Get as text if applicable
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s),
            _ => None,
        }
    }

    /// Get as file path if applicable
    pub fn as_file(&self) -> Option<&str> {
        match self {
            Self::File(s) => Some(s),
            _ => None,
        }
    }

    /// Get as ID if applicable
    pub fn as_id(&self) -> Option<u64> {
        match self {
            Self::Id(id) => Some(*id),
            _ => None,
        }
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Get item count (for multi-drag)
    pub fn count(&self) -> usize {
        match self {
            Self::Multiple(items) => items.len(),
            Self::Empty => 0,
            _ => 1,
        }
    }
}

impl Default for DragData {
    fn default() -> Self {
        Self::Empty
    }
}

/// Mouse button that initiated the drag
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DragButton {
    Left,
    Right,
    Middle,
}

impl Default for DragButton {
    fn default() -> Self {
        Self::Left
    }
}

/// Modifier keys held during drag
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DragModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

impl DragModifiers {
    /// No modifiers
    pub fn none() -> Self {
        Self::default()
    }

    /// Check if any modifier is pressed
    pub fn any(&self) -> bool {
        self.shift || self.ctrl || self.alt || self.meta
    }
}

/// Event emitted when a drag operation starts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DragStartEvent {
    /// Unique drag operation ID
    pub drag_id: DragId,
    /// Position where drag started
    pub start_position: Point,
    /// Current position (same as start at drag start)
    pub position: Point,
    /// The element being dragged
    pub element_id: String,
    /// Size of the dragged element
    pub element_size: Size,
    /// Offset from top-left of element to mouse position
    pub offset: Point,
    /// Type of drag data
    pub drag_type: DragType,
    /// The drag data payload
    pub data: DragData,
    /// Mouse button used
    pub button: DragButton,
    /// Modifier keys
    pub modifiers: DragModifiers,
    /// Timestamp
    pub timestamp: f64,
}

impl DragStartEvent {
    /// Create a new drag start event
    pub fn new(
        drag_id: DragId,
        element_id: impl Into<String>,
        position: Point,
        element_size: Size,
        offset: Point,
    ) -> Self {
        Self {
            drag_id,
            start_position: position,
            position,
            element_id: element_id.into(),
            element_size,
            offset,
            drag_type: DragType::custom("default"),
            data: DragData::Empty,
            button: DragButton::Left,
            modifiers: DragModifiers::none(),
            timestamp: 0.0,
        }
    }

    /// Set the drag type
    pub fn with_type(mut self, drag_type: DragType) -> Self {
        self.drag_type = drag_type;
        self
    }

    /// Set the drag data
    pub fn with_data(mut self, data: DragData) -> Self {
        self.data = data;
        self
    }

    /// Set the button
    pub fn with_button(mut self, button: DragButton) -> Self {
        self.button = button;
        self
    }

    /// Set the modifiers
    pub fn with_modifiers(mut self, modifiers: DragModifiers) -> Self {
        self.modifiers = modifiers;
        self
    }

    /// Set the timestamp
    pub fn with_timestamp(mut self, timestamp: f64) -> Self {
        self.timestamp = timestamp;
        self
    }
}

/// Event emitted during drag movement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DragEvent {
    /// Unique drag operation ID
    pub drag_id: DragId,
    /// Original start position
    pub start_position: Point,
    /// Current position
    pub position: Point,
    /// Change since last event
    pub delta: Point,
    /// Total change since drag start
    pub total_delta: Point,
    /// Current velocity (pixels per second)
    pub velocity: Point,
    /// Modifier keys
    pub modifiers: DragModifiers,
    /// Timestamp
    pub timestamp: f64,
}

impl DragEvent {
    /// Create a new drag event
    pub fn new(drag_id: DragId, start_position: Point, position: Point, delta: Point) -> Self {
        Self {
            drag_id,
            start_position,
            position,
            delta,
            total_delta: position.sub(&start_position),
            velocity: Point::zero(),
            modifiers: DragModifiers::none(),
            timestamp: 0.0,
        }
    }

    /// Set velocity
    pub fn with_velocity(mut self, velocity: Point) -> Self {
        self.velocity = velocity;
        self
    }

    /// Set modifiers
    pub fn with_modifiers(mut self, modifiers: DragModifiers) -> Self {
        self.modifiers = modifiers;
        self
    }

    /// Set timestamp
    pub fn with_timestamp(mut self, timestamp: f64) -> Self {
        self.timestamp = timestamp;
        self
    }
}

/// Event emitted when drag ends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DragEndEvent {
    /// Unique drag operation ID
    pub drag_id: DragId,
    /// Original start position
    pub start_position: Point,
    /// Final position
    pub position: Point,
    /// Total movement
    pub total_delta: Point,
    /// Final velocity (for animations)
    pub velocity: Point,
    /// Whether drag was cancelled
    pub cancelled: bool,
    /// Whether drop was successful
    pub dropped: bool,
    /// Modifier keys at end
    pub modifiers: DragModifiers,
    /// Timestamp
    pub timestamp: f64,
}

impl DragEndEvent {
    /// Create a new drag end event
    pub fn new(
        drag_id: DragId,
        start_position: Point,
        position: Point,
        velocity: Point,
        dropped: bool,
    ) -> Self {
        Self {
            drag_id,
            start_position,
            position,
            total_delta: position.sub(&start_position),
            velocity,
            cancelled: false,
            dropped,
            modifiers: DragModifiers::none(),
            timestamp: 0.0,
        }
    }

    /// Create a cancelled drag end event
    pub fn cancelled(drag_id: DragId, start_position: Point, position: Point) -> Self {
        Self {
            drag_id,
            start_position,
            position,
            total_delta: position.sub(&start_position),
            velocity: Point::zero(),
            cancelled: true,
            dropped: false,
            modifiers: DragModifiers::none(),
            timestamp: 0.0,
        }
    }

    /// Set timestamp
    pub fn with_timestamp(mut self, timestamp: f64) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Set modifiers
    pub fn with_modifiers(mut self, modifiers: DragModifiers) -> Self {
        self.modifiers = modifiers;
        self
    }
}

/// Event emitted when dragged item enters a drop zone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DragEnterEvent {
    /// Unique drag operation ID
    pub drag_id: DragId,
    /// The drop zone being entered
    pub drop_zone_id: String,
    /// Current drag position
    pub position: Point,
    /// Type of the dragged data
    pub drag_type: DragType,
    /// The drag data
    pub data: DragData,
    /// Timestamp
    pub timestamp: f64,
}

impl DragEnterEvent {
    /// Create a new drag enter event
    pub fn new(
        drag_id: DragId,
        drop_zone_id: impl Into<String>,
        position: Point,
        drag_type: DragType,
        data: DragData,
    ) -> Self {
        Self {
            drag_id,
            drop_zone_id: drop_zone_id.into(),
            position,
            drag_type,
            data,
            timestamp: 0.0,
        }
    }

    /// Set timestamp
    pub fn with_timestamp(mut self, timestamp: f64) -> Self {
        self.timestamp = timestamp;
        self
    }
}

/// Event emitted while dragged item is over a drop zone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DragOverEvent {
    /// Unique drag operation ID
    pub drag_id: DragId,
    /// The drop zone being hovered
    pub drop_zone_id: String,
    /// Current drag position
    pub position: Point,
    /// Position within the drop zone
    pub local_position: Point,
    /// Type of the dragged data
    pub drag_type: DragType,
    /// Timestamp
    pub timestamp: f64,
}

impl DragOverEvent {
    /// Create a new drag over event
    pub fn new(
        drag_id: DragId,
        drop_zone_id: impl Into<String>,
        position: Point,
        local_position: Point,
        drag_type: DragType,
    ) -> Self {
        Self {
            drag_id,
            drop_zone_id: drop_zone_id.into(),
            position,
            local_position,
            drag_type,
            timestamp: 0.0,
        }
    }

    /// Set timestamp
    pub fn with_timestamp(mut self, timestamp: f64) -> Self {
        self.timestamp = timestamp;
        self
    }
}

/// Event emitted when dragged item leaves a drop zone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DragLeaveEvent {
    /// Unique drag operation ID
    pub drag_id: DragId,
    /// The drop zone being left
    pub drop_zone_id: String,
    /// Position when leaving
    pub position: Point,
    /// Timestamp
    pub timestamp: f64,
}

impl DragLeaveEvent {
    /// Create a new drag leave event
    pub fn new(drag_id: DragId, drop_zone_id: impl Into<String>, position: Point) -> Self {
        Self {
            drag_id,
            drop_zone_id: drop_zone_id.into(),
            position,
            timestamp: 0.0,
        }
    }

    /// Set timestamp
    pub fn with_timestamp(mut self, timestamp: f64) -> Self {
        self.timestamp = timestamp;
        self
    }
}

/// Event emitted when item is dropped
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropEvent {
    /// Unique drag operation ID
    pub drag_id: DragId,
    /// The drop zone where item was dropped
    pub drop_zone_id: String,
    /// Drop position
    pub position: Point,
    /// Position within the drop zone
    pub local_position: Point,
    /// Type of the dropped data
    pub drag_type: DragType,
    /// The dropped data
    pub data: DragData,
    /// Modifier keys at drop
    pub modifiers: DragModifiers,
    /// Timestamp
    pub timestamp: f64,
}

impl DropEvent {
    /// Create a new drop event
    pub fn new(
        drag_id: DragId,
        drop_zone_id: impl Into<String>,
        position: Point,
        local_position: Point,
        drag_type: DragType,
        data: DragData,
    ) -> Self {
        Self {
            drag_id,
            drop_zone_id: drop_zone_id.into(),
            position,
            local_position,
            drag_type,
            data,
            modifiers: DragModifiers::none(),
            timestamp: 0.0,
        }
    }

    /// Set modifiers
    pub fn with_modifiers(mut self, modifiers: DragModifiers) -> Self {
        self.modifiers = modifiers;
        self
    }

    /// Set timestamp
    pub fn with_timestamp(mut self, timestamp: f64) -> Self {
        self.timestamp = timestamp;
        self
    }
}

/// Union type for all drag events
#[derive(Debug, Clone)]
pub enum DragEventKind {
    Start(DragStartEvent),
    Drag(DragEvent),
    End(DragEndEvent),
    Enter(DragEnterEvent),
    Over(DragOverEvent),
    Leave(DragLeaveEvent),
    Drop(DropEvent),
}

impl DragEventKind {
    /// Get the drag ID from any event type
    pub fn drag_id(&self) -> DragId {
        match self {
            DragEventKind::Start(e) => e.drag_id,
            DragEventKind::Drag(e) => e.drag_id,
            DragEventKind::End(e) => e.drag_id,
            DragEventKind::Enter(e) => e.drag_id,
            DragEventKind::Over(e) => e.drag_id,
            DragEventKind::Leave(e) => e.drag_id,
            DragEventKind::Drop(e) => e.drag_id,
        }
    }

    /// Get the position from any event type
    pub fn position(&self) -> Point {
        match self {
            DragEventKind::Start(e) => e.position,
            DragEventKind::Drag(e) => e.position,
            DragEventKind::End(e) => e.position,
            DragEventKind::Enter(e) => e.position,
            DragEventKind::Over(e) => e.position,
            DragEventKind::Leave(e) => e.position,
            DragEventKind::Drop(e) => e.position,
        }
    }

    /// Get timestamp from any event type
    pub fn timestamp(&self) -> f64 {
        match self {
            DragEventKind::Start(e) => e.timestamp,
            DragEventKind::Drag(e) => e.timestamp,
            DragEventKind::End(e) => e.timestamp,
            DragEventKind::Enter(e) => e.timestamp,
            DragEventKind::Over(e) => e.timestamp,
            DragEventKind::Leave(e) => e.timestamp,
            DragEventKind::Drop(e) => e.timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_operations() {
        let p1 = Point::new(10.0, 20.0);
        let p2 = Point::new(5.0, 5.0);

        let sum = p1.add(&p2);
        assert_eq!(sum.x, 15.0);
        assert_eq!(sum.y, 25.0);

        let diff = p1.sub(&p2);
        assert_eq!(diff.x, 5.0);
        assert_eq!(diff.y, 15.0);

        let distance = p1.distance_to(&p2);
        assert!((distance - 15.811_388).abs() < 0.001);
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10.0, 10.0, 100.0, 100.0);

        assert!(rect.contains(&Point::new(50.0, 50.0)));
        assert!(rect.contains(&Point::new(10.0, 10.0)));
        assert!(rect.contains(&Point::new(110.0, 110.0)));
        assert!(!rect.contains(&Point::new(5.0, 50.0)));
        assert!(!rect.contains(&Point::new(150.0, 50.0)));
    }

    #[test]
    fn test_rect_intersection() {
        let r1 = Rect::new(0.0, 0.0, 100.0, 100.0);
        let r2 = Rect::new(50.0, 50.0, 100.0, 100.0);

        let intersection = r1.intersection(&r2).unwrap();
        assert_eq!(intersection.x, 50.0);
        assert_eq!(intersection.y, 50.0);
        assert_eq!(intersection.width, 50.0);
        assert_eq!(intersection.height, 50.0);
    }

    #[test]
    fn test_rect_no_intersection() {
        let r1 = Rect::new(0.0, 0.0, 50.0, 50.0);
        let r2 = Rect::new(100.0, 100.0, 50.0, 50.0);

        assert!(r1.intersection(&r2).is_none());
    }

    #[test]
    fn test_drag_data() {
        let text_data = DragData::text("hello");
        assert_eq!(text_data.as_text(), Some("hello"));
        assert_eq!(text_data.count(), 1);

        let id_data = DragData::id(42);
        assert_eq!(id_data.as_id(), Some(42));

        let empty = DragData::Empty;
        assert!(empty.is_empty());
        assert_eq!(empty.count(), 0);
    }

    #[test]
    fn test_drag_data_json() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestData {
            value: i32,
        }

        let data = TestData { value: 42 };
        let drag_data = DragData::json(&data).unwrap();

        let parsed: TestData = drag_data.parse_json().unwrap();
        assert_eq!(parsed, data);
    }

    #[test]
    fn test_drag_type() {
        let custom = DragType::custom("my-type");
        assert_eq!(custom.0, "custom:my-type");

        let text = DragType::text();
        assert_eq!(text.0, "text");
    }

    #[test]
    fn test_drag_modifiers() {
        let none = DragModifiers::none();
        assert!(!none.any());

        let with_shift = DragModifiers {
            shift: true,
            ..Default::default()
        };
        assert!(with_shift.any());
    }

    #[test]
    fn test_drag_start_event() {
        let event = DragStartEvent::new(
            DragId::new(1),
            "element-1",
            Point::new(100.0, 200.0),
            Size::new(50.0, 30.0),
            Point::new(10.0, 10.0),
        )
        .with_type(DragType::text())
        .with_data(DragData::text("hello"));

        assert_eq!(event.drag_id.0, 1);
        assert_eq!(event.element_id, "element-1");
        assert_eq!(event.drag_type.0, "text");
    }

    #[test]
    fn test_drag_event() {
        let event = DragEvent::new(
            DragId::new(1),
            Point::new(0.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(10.0, 10.0),
        )
        .with_velocity(Point::new(500.0, 500.0));

        assert_eq!(event.total_delta.x, 100.0);
        assert_eq!(event.total_delta.y, 100.0);
        assert_eq!(event.velocity.x, 500.0);
    }

    #[test]
    fn test_drag_end_event() {
        let event = DragEndEvent::new(
            DragId::new(1),
            Point::new(0.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(200.0, 200.0),
            true,
        );

        assert!(!event.cancelled);
        assert!(event.dropped);

        let cancelled =
            DragEndEvent::cancelled(DragId::new(1), Point::new(0.0, 0.0), Point::new(50.0, 50.0));

        assert!(cancelled.cancelled);
        assert!(!cancelled.dropped);
    }

    #[test]
    fn test_drop_event() {
        let event = DropEvent::new(
            DragId::new(1),
            "drop-zone-1",
            Point::new(100.0, 100.0),
            Point::new(50.0, 50.0),
            DragType::text(),
            DragData::text("dropped"),
        );

        assert_eq!(event.drop_zone_id, "drop-zone-1");
        assert_eq!(event.data.as_text(), Some("dropped"));
    }

    #[test]
    fn test_drag_event_kind() {
        let start = DragEventKind::Start(DragStartEvent::new(
            DragId::new(42),
            "element",
            Point::new(10.0, 20.0),
            Size::new(100.0, 50.0),
            Point::zero(),
        ));

        assert_eq!(start.drag_id().0, 42);
        assert_eq!(start.position().x, 10.0);
    }
}
