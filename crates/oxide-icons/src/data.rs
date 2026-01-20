//! Icon data types and registry
//!
//! Core types for icon representation and management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

/// Supported icon sets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IconSet {
    /// Material Design Icons
    Material,
    /// Lucide (Feather) Icons
    Lucide,
    /// Heroicons
    Heroicons,
    /// Phosphor Icons
    Phosphor,
    /// Custom/inline icons
    Custom,
}

impl Default for IconSet {
    fn default() -> Self {
        IconSet::Material
    }
}

/// SVG path command
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PathCommand {
    /// Move to (x, y)
    MoveTo(f32, f32),
    /// Line to (x, y)
    LineTo(f32, f32),
    /// Horizontal line to x
    HorizontalLineTo(f32),
    /// Vertical line to y
    VerticalLineTo(f32),
    /// Cubic bezier curve (x1, y1, x2, y2, x, y)
    CurveTo(f32, f32, f32, f32, f32, f32),
    /// Smooth cubic bezier (x2, y2, x, y)
    SmoothCurveTo(f32, f32, f32, f32),
    /// Quadratic bezier (x1, y1, x, y)
    QuadraticCurveTo(f32, f32, f32, f32),
    /// Smooth quadratic bezier (x, y)
    SmoothQuadraticCurveTo(f32, f32),
    /// Arc (rx, ry, x_rotation, large_arc, sweep, x, y)
    Arc(f32, f32, f32, bool, bool, f32, f32),
    /// Close path
    ClosePath,
}

/// SVG path with commands and optional styling
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SvgPath {
    /// Path commands
    pub commands: Vec<PathCommand>,
    /// Fill color (CSS color string)
    pub fill: Option<String>,
    /// Stroke color (CSS color string)
    pub stroke: Option<String>,
    /// Stroke width
    pub stroke_width: Option<f32>,
    /// Fill rule ("nonzero" or "evenodd")
    pub fill_rule: Option<String>,
}

impl SvgPath {
    /// Create a new SVG path from commands
    pub fn new(commands: Vec<PathCommand>) -> Self {
        Self {
            commands,
            fill: None,
            stroke: None,
            stroke_width: None,
            fill_rule: None,
        }
    }

    /// Parse path commands from an SVG path data string
    pub fn from_d(d: &str) -> Option<Self> {
        let commands = parse_path_data(d)?;
        Some(Self::new(commands))
    }

    /// Convert commands to SVG path data string
    pub fn to_d(&self) -> String {
        self.commands
            .iter()
            .map(|cmd| match cmd {
                PathCommand::MoveTo(x, y) => format!("M{} {}", x, y),
                PathCommand::LineTo(x, y) => format!("L{} {}", x, y),
                PathCommand::HorizontalLineTo(x) => format!("H{}", x),
                PathCommand::VerticalLineTo(y) => format!("V{}", y),
                PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                    format!("C{} {} {} {} {} {}", x1, y1, x2, y2, x, y)
                }
                PathCommand::SmoothCurveTo(x2, y2, x, y) => format!("S{} {} {} {}", x2, y2, x, y),
                PathCommand::QuadraticCurveTo(x1, y1, x, y) => format!("Q{} {} {} {}", x1, y1, x, y),
                PathCommand::SmoothQuadraticCurveTo(x, y) => format!("T{} {}", x, y),
                PathCommand::Arc(rx, ry, rot, large, sweep, x, y) => {
                    format!(
                        "A{} {} {} {} {} {} {}",
                        rx,
                        ry,
                        rot,
                        if *large { 1 } else { 0 },
                        if *sweep { 1 } else { 0 },
                        x,
                        y
                    )
                }
                PathCommand::ClosePath => "Z".to_string(),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Icon data containing SVG paths and metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IconData {
    /// Icon name
    pub name: String,
    /// Viewbox (x, y, width, height)
    pub viewbox: (i32, i32, u32, u32),
    /// SVG paths
    pub paths: Vec<SvgPath>,
    /// Icon categories
    #[serde(default)]
    pub categories: Vec<String>,
    /// Icon tags for search
    #[serde(default)]
    pub tags: Vec<String>,
}

impl IconData {
    /// Create new icon data
    pub fn new(name: impl Into<String>, viewbox: (i32, i32, u32, u32), paths: Vec<SvgPath>) -> Self {
        Self {
            name: name.into(),
            viewbox,
            paths,
            categories: Vec::new(),
            tags: Vec::new(),
        }
    }

    /// Parse icon data from SVG string
    pub fn from_svg(svg: &str) -> Option<Self> {
        // Simple SVG parsing - extract viewBox and path d attributes
        // This is a basic implementation; a real one would use an XML parser

        // Extract viewBox
        let viewbox = if let Some(start) = svg.find("viewBox=\"") {
            let start = start + 9;
            if let Some(end) = svg[start..].find('"') {
                let vb_str = &svg[start..start + end];
                let parts: Vec<i32> = vb_str
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();
                if parts.len() == 4 {
                    (parts[0], parts[1], parts[2] as u32, parts[3] as u32)
                } else {
                    (0, 0, 24, 24)
                }
            } else {
                (0, 0, 24, 24)
            }
        } else {
            (0, 0, 24, 24)
        };

        // Extract path d attributes
        let mut paths = Vec::new();
        let mut pos = 0;
        while let Some(path_start) = svg[pos..].find("<path") {
            let path_start = pos + path_start;
            if let Some(path_end) = svg[path_start..].find('>') {
                let path_tag = &svg[path_start..path_start + path_end];
                if let Some(d_start) = path_tag.find(" d=\"") {
                    let d_start = d_start + 4;
                    if let Some(d_end) = path_tag[d_start..].find('"') {
                        let d = &path_tag[d_start..d_start + d_end];
                        if let Some(svg_path) = SvgPath::from_d(d) {
                            paths.push(svg_path);
                        }
                    }
                }
                pos = path_start + path_end;
            } else {
                break;
            }
        }

        Some(Self {
            name: String::new(),
            viewbox,
            paths,
            categories: Vec::new(),
            tags: Vec::new(),
        })
    }

    /// Get the SVG width from viewbox
    pub fn width(&self) -> u32 {
        self.viewbox.2
    }

    /// Get the SVG height from viewbox
    pub fn height(&self) -> u32 {
        self.viewbox.3
    }
}

/// Global icon registry
pub struct IconRegistry {
    icons: RwLock<HashMap<(IconSet, String), IconData>>,
}

impl IconRegistry {
    /// Create a new icon registry
    pub fn new() -> Self {
        Self {
            icons: RwLock::new(HashMap::new()),
        }
    }

    /// Get the global icon registry instance
    pub fn global() -> &'static IconRegistry {
        static REGISTRY: OnceLock<IconRegistry> = OnceLock::new();
        REGISTRY.get_or_init(IconRegistry::new)
    }

    /// Register an icon
    pub fn register(&self, set: IconSet, name: impl Into<String>, data: IconData) {
        let mut icons = self.icons.write().unwrap();
        icons.insert((set, name.into()), data);
    }

    /// Register multiple icons from an icon set
    pub fn register_set(&self, set: IconSet, icons: impl IntoIterator<Item = (String, IconData)>) {
        let mut registry = self.icons.write().unwrap();
        for (name, data) in icons {
            registry.insert((set, name), data);
        }
    }

    /// Get an icon by set and name
    pub fn get_icon(&self, set: IconSet, name: &str) -> Option<IconData> {
        let icons = self.icons.read().unwrap();
        icons.get(&(set, name.to_string())).cloned()
    }

    /// Get an icon from a specific set (alias for get_icon)
    pub fn get_from_set(&self, set: &IconSet, name: &str) -> Option<IconData> {
        self.get_icon(*set, name)
    }

    /// Get an icon by name, searching all sets (returns first match)
    pub fn get(&self, name: &str) -> Option<IconData> {
        self.find_icon(name).map(|(_, data)| data)
    }

    /// Get an icon by name, searching all sets
    pub fn find_icon(&self, name: &str) -> Option<(IconSet, IconData)> {
        let icons = self.icons.read().unwrap();
        for ((set, icon_name), data) in icons.iter() {
            if icon_name == name {
                return Some((*set, data.clone()));
            }
        }
        None
    }

    /// List all registered icon names for a set
    pub fn list_icons(&self, set: IconSet) -> Vec<String> {
        let icons = self.icons.read().unwrap();
        icons
            .keys()
            .filter(|(s, _)| *s == set)
            .map(|(_, name)| name.clone())
            .collect()
    }

    /// Check if an icon is registered
    pub fn has_icon(&self, set: IconSet, name: &str) -> bool {
        let icons = self.icons.read().unwrap();
        icons.contains_key(&(set, name.to_string()))
    }

    /// Get the number of registered icons
    pub fn count(&self) -> usize {
        let icons = self.icons.read().unwrap();
        icons.len()
    }

    /// Clear all registered icons
    pub fn clear(&self) {
        let mut icons = self.icons.write().unwrap();
        icons.clear();
    }
}

impl Default for IconRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse SVG path data string into commands
fn parse_path_data(d: &str) -> Option<Vec<PathCommand>> {
    let mut commands = Vec::new();
    let mut chars = d.chars().peekable();
    let mut current_cmd = None;

    while let Some(c) = chars.next() {
        if c.is_whitespace() || c == ',' {
            continue;
        }

        if c.is_alphabetic() {
            current_cmd = Some(c);
            continue;
        }

        // Parse numbers
        let mut num_str = String::new();
        if c == '-' || c == '.' || c.is_ascii_digit() {
            num_str.push(c);
        }

        while let Some(&next) = chars.peek() {
            if next.is_ascii_digit() || next == '.' || (next == '-' && num_str.is_empty()) {
                num_str.push(chars.next().unwrap());
            } else {
                break;
            }
        }

        if !num_str.is_empty() {
            let _num: f32 = num_str.parse().ok()?;
            // Simplified parsing - a full implementation would track state
        }

        // Simplified: just add basic commands
        match current_cmd {
            Some('M') | Some('m') => {
                // Would parse x, y coordinates
                commands.push(PathCommand::MoveTo(0.0, 0.0));
            }
            Some('L') | Some('l') => {
                commands.push(PathCommand::LineTo(0.0, 0.0));
            }
            Some('Z') | Some('z') => {
                commands.push(PathCommand::ClosePath);
            }
            _ => {}
        }
    }

    Some(commands)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_set_default() {
        assert_eq!(IconSet::default(), IconSet::Material);
    }

    #[test]
    fn test_svg_path_to_d() {
        let path = SvgPath::new(vec![
            PathCommand::MoveTo(0.0, 0.0),
            PathCommand::LineTo(10.0, 10.0),
            PathCommand::ClosePath,
        ]);
        assert_eq!(path.to_d(), "M0 0 L10 10 Z");
    }

    #[test]
    fn test_icon_registry() {
        let registry = IconRegistry::new();
        let data = IconData::new(
            "test",
            (0, 0, 24, 24),
            vec![SvgPath::new(vec![PathCommand::MoveTo(0.0, 0.0)])],
        );

        registry.register(IconSet::Material, "test", data.clone());
        assert!(registry.has_icon(IconSet::Material, "test"));
        assert!(!registry.has_icon(IconSet::Lucide, "test"));

        let retrieved = registry.get_icon(IconSet::Material, "test");
        assert!(retrieved.is_some());
    }
}
