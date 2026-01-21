//! Chart legend.

use serde::{Deserialize, Serialize};

/// Legend position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LegendPosition {
    /// Top of chart
    Top,
    /// Bottom of chart
    #[default]
    Bottom,
    /// Left of chart
    Left,
    /// Right of chart
    Right,
}

/// A legend item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegendItem {
    /// Label
    pub label: String,
    /// Color
    pub color: String,
    /// Is visible
    pub visible: bool,
}

impl LegendItem {
    /// Create new item
    pub fn new(label: impl Into<String>, color: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            color: color.into(),
            visible: true,
        }
    }

    /// Toggle visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}

/// Legend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Legend {
    /// Position
    pub position: LegendPosition,
    /// Items
    pub items: Vec<LegendItem>,
    /// Visible
    pub visible: bool,
    /// Interactive (clickable)
    pub interactive: bool,
}

impl Default for Legend {
    fn default() -> Self {
        Self {
            position: LegendPosition::Bottom,
            items: Vec::new(),
            visible: true,
            interactive: true,
        }
    }
}

impl Legend {
    /// Create legend at position
    pub fn at(position: LegendPosition) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Create top legend
    pub fn top() -> Self {
        Self::at(LegendPosition::Top)
    }

    /// Create bottom legend
    pub fn bottom() -> Self {
        Self::at(LegendPosition::Bottom)
    }

    /// Create left legend
    pub fn left() -> Self {
        Self::at(LegendPosition::Left)
    }

    /// Create right legend
    pub fn right() -> Self {
        Self::at(LegendPosition::Right)
    }

    /// Add item
    pub fn add_item(&mut self, item: LegendItem) {
        self.items.push(item);
    }

    /// Set items
    pub fn items(mut self, items: Vec<LegendItem>) -> Self {
        self.items = items;
        self
    }

    /// Set interactive
    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    /// Hide legend
    pub fn hide(mut self) -> Self {
        self.visible = false;
        self
    }

    /// Toggle item visibility
    pub fn toggle_item(&mut self, index: usize) {
        if let Some(item) = self.items.get_mut(index) {
            item.toggle();
        }
    }
}
