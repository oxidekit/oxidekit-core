//! Item measurement and caching for variable height items.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Strategy for measuring item sizes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MeasureStrategy {
    /// All items have the same size
    Fixed,
    /// Items are measured on render
    #[default]
    OnRender,
    /// Items are pre-measured
    PreMeasured,
    /// Estimate first, measure later
    Estimate,
}

/// Context for measuring items
#[derive(Debug, Clone)]
pub struct MeasureContext {
    /// Container width
    pub container_width: f32,
    /// Container height
    pub container_height: f32,
    /// Device pixel ratio
    pub device_pixel_ratio: f32,
}

impl Default for MeasureContext {
    fn default() -> Self {
        Self {
            container_width: 0.0,
            container_height: 0.0,
            device_pixel_ratio: 1.0,
        }
    }
}

/// Measured dimensions for an item
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ItemMeasurement {
    /// Width
    pub width: f32,
    /// Height
    pub height: f32,
}

impl ItemMeasurement {
    /// Create a new measurement
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Create from height only (for list items)
    pub fn from_height(height: f32) -> Self {
        Self {
            width: 0.0,
            height,
        }
    }
}

/// Cache for item measurements
#[derive(Debug, Clone, Default)]
pub struct ItemMeasureCache {
    /// Cached measurements by item index
    measurements: HashMap<usize, ItemMeasurement>,
    /// Default measurement for unmeasured items
    default_measurement: Option<ItemMeasurement>,
    /// Estimated item height
    estimated_height: f32,
}

impl ItemMeasureCache {
    /// Create a new cache
    pub fn new() -> Self {
        Self {
            measurements: HashMap::new(),
            default_measurement: None,
            estimated_height: 48.0,
        }
    }

    /// Create cache with estimated height
    pub fn with_estimated_height(height: f32) -> Self {
        Self {
            measurements: HashMap::new(),
            default_measurement: Some(ItemMeasurement::from_height(height)),
            estimated_height: height,
        }
    }

    /// Set default measurement
    pub fn set_default(&mut self, measurement: ItemMeasurement) {
        self.default_measurement = Some(measurement);
    }

    /// Get measurement for item
    pub fn get(&self, index: usize) -> Option<&ItemMeasurement> {
        self.measurements.get(&index)
    }

    /// Get measurement or default
    pub fn get_or_default(&self, index: usize) -> ItemMeasurement {
        self.measurements
            .get(&index)
            .copied()
            .or(self.default_measurement)
            .unwrap_or(ItemMeasurement::from_height(self.estimated_height))
    }

    /// Set measurement for item
    pub fn set(&mut self, index: usize, measurement: ItemMeasurement) {
        self.measurements.insert(index, measurement);
    }

    /// Remove measurement for item
    pub fn remove(&mut self, index: usize) -> Option<ItemMeasurement> {
        self.measurements.remove(&index)
    }

    /// Clear all measurements
    pub fn clear(&mut self) {
        self.measurements.clear();
    }

    /// Get number of cached measurements
    pub fn len(&self) -> usize {
        self.measurements.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.measurements.is_empty()
    }

    /// Calculate total height for range
    pub fn total_height(&self, range: std::ops::Range<usize>) -> f32 {
        range
            .map(|i| self.get_or_default(i).height)
            .sum()
    }

    /// Get estimated height
    pub fn estimated_height(&self) -> f32 {
        self.estimated_height
    }

    /// Update estimated height from measurements
    pub fn update_estimate(&mut self) {
        if !self.measurements.is_empty() {
            let sum: f32 = self.measurements.values().map(|m| m.height).sum();
            self.estimated_height = sum / self.measurements.len() as f32;
        }
    }
}
