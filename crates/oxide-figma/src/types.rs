//! Figma API data types
//!
//! These types represent the Figma API response structures.
//! They are used for parsing Figma files and nodes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A Figma file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FigmaFile {
    /// File name
    pub name: String,

    /// Document node
    pub document: DocumentNode,

    /// Component definitions
    #[serde(default)]
    pub components: HashMap<String, Component>,

    /// Component sets (variants)
    #[serde(default)]
    pub component_sets: HashMap<String, ComponentSet>,

    /// Style definitions
    #[serde(default)]
    pub styles: HashMap<String, Style>,

    /// Last modified timestamp
    #[serde(default)]
    pub last_modified: String,

    /// Thumbnail URL
    #[serde(default)]
    pub thumbnail_url: String,

    /// File version
    #[serde(default)]
    pub version: String,

    /// Schema version
    #[serde(default)]
    pub schema_version: u32,
}

/// Document node (root of Figma file)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentNode {
    /// Node ID
    pub id: String,

    /// Node name
    pub name: String,

    /// Node type
    #[serde(rename = "type")]
    pub node_type: String,

    /// Child nodes (pages)
    #[serde(default)]
    pub children: Vec<Node>,
}

/// A Figma node (frame, component, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Node ID
    pub id: String,

    /// Node name
    pub name: String,

    /// Node type
    #[serde(rename = "type")]
    pub node_type: NodeType,

    /// Visibility
    #[serde(default = "default_true")]
    pub visible: bool,

    /// Child nodes
    #[serde(default)]
    pub children: Vec<Node>,

    /// Absolute bounding box
    #[serde(default, rename = "absoluteBoundingBox")]
    pub absolute_bounding_box: Option<Rectangle>,

    /// Relative bounding box
    #[serde(default, rename = "relativeTransform")]
    pub relative_transform: Option<Transform>,

    /// Size constraints
    #[serde(default)]
    pub constraints: Option<LayoutConstraint>,

    /// Layout mode (for frames with auto-layout)
    #[serde(default, rename = "layoutMode")]
    pub layout_mode: Option<LayoutMode>,

    /// Primary axis sizing mode
    #[serde(default, rename = "primaryAxisSizingMode")]
    pub primary_axis_sizing_mode: Option<AxisSizingMode>,

    /// Counter axis sizing mode
    #[serde(default, rename = "counterAxisSizingMode")]
    pub counter_axis_sizing_mode: Option<AxisSizingMode>,

    /// Primary axis alignment
    #[serde(default, rename = "primaryAxisAlignItems")]
    pub primary_axis_align_items: Option<AlignItems>,

    /// Counter axis alignment
    #[serde(default, rename = "counterAxisAlignItems")]
    pub counter_axis_align_items: Option<AlignItems>,

    /// Padding
    #[serde(default, rename = "paddingLeft")]
    pub padding_left: f32,
    #[serde(default, rename = "paddingRight")]
    pub padding_right: f32,
    #[serde(default, rename = "paddingTop")]
    pub padding_top: f32,
    #[serde(default, rename = "paddingBottom")]
    pub padding_bottom: f32,

    /// Item spacing (gap)
    #[serde(default, rename = "itemSpacing")]
    pub item_spacing: f32,

    /// Corner radius
    #[serde(default, rename = "cornerRadius")]
    pub corner_radius: f32,

    /// Individual corner radii
    #[serde(default, rename = "rectangleCornerRadii")]
    pub rectangle_corner_radii: Option<[f32; 4]>,

    /// Fills
    #[serde(default)]
    pub fills: Vec<Paint>,

    /// Strokes
    #[serde(default)]
    pub strokes: Vec<Paint>,

    /// Stroke weight
    #[serde(default, rename = "strokeWeight")]
    pub stroke_weight: f32,

    /// Effects (shadows, blur)
    #[serde(default)]
    pub effects: Vec<Effect>,

    /// Blend mode
    #[serde(default, rename = "blendMode")]
    pub blend_mode: Option<BlendMode>,

    /// Opacity
    #[serde(default = "default_one")]
    pub opacity: f32,

    /// Style references
    #[serde(default)]
    pub styles: Option<StyleReferences>,

    /// Characters (for text nodes)
    #[serde(default)]
    pub characters: Option<String>,

    /// Text style
    #[serde(default)]
    pub style: Option<TypeStyle>,

    /// Character style overrides
    #[serde(default, rename = "characterStyleOverrides")]
    pub character_style_overrides: Vec<u32>,

    /// Style override table
    #[serde(default, rename = "styleOverrideTable")]
    pub style_override_table: HashMap<String, TypeStyle>,

    /// Component ID (if this is an instance)
    #[serde(default, rename = "componentId")]
    pub component_id: Option<String>,

    /// Component properties (for instances)
    #[serde(default, rename = "componentProperties")]
    pub component_properties: HashMap<String, ComponentProperty>,

    /// Exported images
    #[serde(default, rename = "exportSettings")]
    pub export_settings: Vec<ExportSetting>,

    /// Whether this is a mask
    #[serde(default, rename = "isMask")]
    pub is_mask: bool,

    /// Layout grow (flex-grow)
    #[serde(default, rename = "layoutGrow")]
    pub layout_grow: f32,

    /// Layout align (self alignment)
    #[serde(default, rename = "layoutAlign")]
    pub layout_align: Option<LayoutAlign>,

    /// Min width
    #[serde(default, rename = "minWidth")]
    pub min_width: Option<f32>,

    /// Max width
    #[serde(default, rename = "maxWidth")]
    pub max_width: Option<f32>,

    /// Min height
    #[serde(default, rename = "minHeight")]
    pub min_height: Option<f32>,

    /// Max height
    #[serde(default, rename = "maxHeight")]
    pub max_height: Option<f32>,
}

fn default_true() -> bool {
    true
}

fn default_one() -> f32 {
    1.0
}

/// Node type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NodeType {
    Document,
    Canvas,
    Frame,
    Group,
    Section,
    Vector,
    BooleanOperation,
    Star,
    Line,
    Ellipse,
    RegularPolygon,
    Rectangle,
    Text,
    Slice,
    Component,
    ComponentSet,
    Instance,
    Sticky,
    ShapeWithText,
    Connector,
    Widget,
    #[serde(other)]
    Unknown,
}

/// Layout mode for auto-layout frames
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LayoutMode {
    None,
    Horizontal,
    Vertical,
}

/// Axis sizing mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AxisSizingMode {
    Fixed,
    Auto,
}

/// Alignment for layout items
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AlignItems {
    Min,
    Center,
    Max,
    SpaceBetween,
    Baseline,
}

/// Layout alignment for individual items
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LayoutAlign {
    Inherit,
    Stretch,
    Min,
    Center,
    Max,
}

/// Rectangle bounding box
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// 2D transform matrix
pub type Transform = [[f32; 3]; 2];

/// Layout constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConstraint {
    pub vertical: ConstraintType,
    pub horizontal: ConstraintType,
}

/// Constraint type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConstraintType {
    Top,
    Bottom,
    Left,
    Right,
    TopBottom,
    LeftRight,
    Center,
    Scale,
}

/// Paint (fill or stroke)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paint {
    /// Paint type
    #[serde(rename = "type")]
    pub paint_type: PaintType,

    /// Visibility
    #[serde(default = "default_true")]
    pub visible: bool,

    /// Opacity
    #[serde(default = "default_one")]
    pub opacity: f32,

    /// Color (for solid fills)
    #[serde(default)]
    pub color: Option<Color>,

    /// Blend mode
    #[serde(default, rename = "blendMode")]
    pub blend_mode: Option<BlendMode>,

    /// Gradient handles (for gradients)
    #[serde(default, rename = "gradientHandlePositions")]
    pub gradient_handle_positions: Vec<Vector>,

    /// Gradient stops
    #[serde(default, rename = "gradientStops")]
    pub gradient_stops: Vec<ColorStop>,

    /// Image ref (for image fills)
    #[serde(default, rename = "imageRef")]
    pub image_ref: Option<String>,

    /// Scale mode (for images)
    #[serde(default, rename = "scaleMode")]
    pub scale_mode: Option<ScaleMode>,
}

/// Paint type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaintType {
    Solid,
    GradientLinear,
    GradientRadial,
    GradientAngular,
    GradientDiamond,
    Image,
    Emoji,
    Video,
}

/// Color (RGBA)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        format!(
            "#{:02X}{:02X}{:02X}",
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8
        )
    }

    /// Convert to rgba string
    pub fn to_rgba(&self) -> String {
        format!(
            "rgba({}, {}, {}, {:.2})",
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            self.a
        )
    }
}

/// 2D vector
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
}

/// Gradient color stop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorStop {
    pub position: f32,
    pub color: Color,
}

/// Scale mode for images
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScaleMode {
    Fill,
    Fit,
    Tile,
    Stretch,
}

/// Blend mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BlendMode {
    PassThrough,
    Normal,
    Darken,
    Multiply,
    LinearBurn,
    ColorBurn,
    Lighten,
    Screen,
    LinearDodge,
    ColorDodge,
    Overlay,
    SoftLight,
    HardLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

/// Effect (shadow, blur, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
    /// Effect type
    #[serde(rename = "type")]
    pub effect_type: EffectType,

    /// Visibility
    #[serde(default = "default_true")]
    pub visible: bool,

    /// Radius
    #[serde(default)]
    pub radius: f32,

    /// Color (for shadows)
    #[serde(default)]
    pub color: Option<Color>,

    /// Blend mode
    #[serde(default, rename = "blendMode")]
    pub blend_mode: Option<BlendMode>,

    /// Offset (for shadows)
    #[serde(default)]
    pub offset: Option<Vector>,

    /// Spread (for shadows)
    #[serde(default)]
    pub spread: f32,
}

/// Effect type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EffectType {
    InnerShadow,
    DropShadow,
    LayerBlur,
    BackgroundBlur,
}

/// Style references
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StyleReferences {
    #[serde(default)]
    pub fill: Option<String>,
    #[serde(default)]
    pub stroke: Option<String>,
    #[serde(default)]
    pub effect: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub grid: Option<String>,
}

/// Type style (text formatting)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TypeStyle {
    /// Font family
    #[serde(default, rename = "fontFamily")]
    pub font_family: String,

    /// Font post-script name
    #[serde(default, rename = "fontPostScriptName")]
    pub font_post_script_name: Option<String>,

    /// Font weight
    #[serde(default = "default_font_weight", rename = "fontWeight")]
    pub font_weight: u16,

    /// Font size
    #[serde(default = "default_font_size", rename = "fontSize")]
    pub font_size: f32,

    /// Text alignment
    #[serde(default, rename = "textAlignHorizontal")]
    pub text_align_horizontal: Option<TextAlign>,

    /// Vertical alignment
    #[serde(default, rename = "textAlignVertical")]
    pub text_align_vertical: Option<TextAlignVertical>,

    /// Letter spacing
    #[serde(default, rename = "letterSpacing")]
    pub letter_spacing: f32,

    /// Line height (px or %)
    #[serde(default, rename = "lineHeightPx")]
    pub line_height_px: f32,

    /// Line height percent
    #[serde(default, rename = "lineHeightPercent")]
    pub line_height_percent: f32,

    /// Line height unit
    #[serde(default, rename = "lineHeightUnit")]
    pub line_height_unit: Option<LineHeightUnit>,

    /// Text decoration
    #[serde(default, rename = "textDecoration")]
    pub text_decoration: Option<TextDecoration>,

    /// Text case
    #[serde(default, rename = "textCase")]
    pub text_case: Option<TextCase>,

    /// Italic
    #[serde(default)]
    pub italic: bool,
}

fn default_font_weight() -> u16 {
    400
}

fn default_font_size() -> f32 {
    14.0
}

/// Text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TextAlign {
    Left,
    Right,
    Center,
    Justified,
}

/// Vertical text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TextAlignVertical {
    Top,
    Center,
    Bottom,
}

/// Line height unit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LineHeightUnit {
    Pixels,
    FontSizePercent,
    Intrinsic,
}

/// Text decoration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TextDecoration {
    None,
    Underline,
    Strikethrough,
}

/// Text case transformation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TextCase {
    Original,
    Upper,
    Lower,
    Title,
    SmallCaps,
    SmallCapsForced,
}

/// Component definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    /// Component key
    pub key: String,

    /// Component name
    pub name: String,

    /// Description
    #[serde(default)]
    pub description: String,

    /// Component set ID (if part of a variant set)
    #[serde(default, rename = "componentSetId")]
    pub component_set_id: Option<String>,

    /// Documentation links
    #[serde(default, rename = "documentationLinks")]
    pub documentation_links: Vec<DocumentationLink>,
}

/// Component set (variant container)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentSet {
    /// Component set key
    pub key: String,

    /// Component set name
    pub name: String,

    /// Description
    #[serde(default)]
    pub description: String,

    /// Documentation links
    #[serde(default, rename = "documentationLinks")]
    pub documentation_links: Vec<DocumentationLink>,
}

/// Documentation link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationLink {
    pub uri: String,
}

/// Component property
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentProperty {
    #[serde(rename = "type")]
    pub property_type: ComponentPropertyType,
    pub value: serde_json::Value,
    #[serde(default, rename = "preferredValues")]
    pub preferred_values: Vec<serde_json::Value>,
}

/// Component property type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComponentPropertyType {
    Boolean,
    InstanceSwap,
    Text,
    Variant,
}

/// Style definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Style {
    /// Style key
    pub key: String,

    /// Style name
    pub name: String,

    /// Style type
    #[serde(rename = "styleType")]
    pub style_type: StyleType,

    /// Description
    #[serde(default)]
    pub description: String,
}

/// Style type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StyleType {
    Fill,
    Stroke,
    Text,
    Effect,
    Grid,
}

/// Export setting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSetting {
    pub suffix: String,
    pub format: ExportFormat,
    #[serde(default)]
    pub constraint: Option<ExportConstraint>,
}

/// Export format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExportFormat {
    Jpg,
    Png,
    Svg,
    Pdf,
}

/// Export constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConstraint {
    #[serde(rename = "type")]
    pub constraint_type: ExportConstraintType,
    pub value: f32,
}

/// Export constraint type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExportConstraintType {
    Scale,
    Width,
    Height,
}

/// Variables response from Figma API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariablesResponse {
    pub status: u32,
    pub error: bool,
    pub meta: VariablesMeta,
}

/// Variables metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariablesMeta {
    #[serde(default)]
    pub variables: HashMap<String, Variable>,
    #[serde(default, rename = "variableCollections")]
    pub variable_collections: HashMap<String, VariableCollection>,
}

/// A Figma variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub id: String,
    pub name: String,
    pub key: String,
    #[serde(rename = "variableCollectionId")]
    pub variable_collection_id: String,
    #[serde(rename = "resolvedType")]
    pub resolved_type: VariableType,
    #[serde(default, rename = "valuesByMode")]
    pub values_by_mode: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub scopes: Vec<VariableScope>,
    #[serde(default, rename = "codeSyntax")]
    pub code_syntax: HashMap<String, String>,
}

/// Variable type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VariableType {
    Boolean,
    Float,
    String,
    Color,
}

/// Variable scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VariableScope {
    AllScopes,
    TextContent,
    CornerRadius,
    WidthHeight,
    Gap,
    AllFills,
    FrameFill,
    ShapeFill,
    TextFill,
    StrokeColor,
    StrokeFloat,
    EffectColor,
    EffectFloat,
    Opacity,
    FontFamily,
    FontStyle,
    FontWeight,
    FontSize,
    LineHeight,
    LetterSpacing,
    ParagraphSpacing,
    ParagraphIndent,
}

/// Variable collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableCollection {
    pub id: String,
    pub name: String,
    pub key: String,
    #[serde(default, rename = "variableIds")]
    pub variable_ids: Vec<String>,
    #[serde(default)]
    pub modes: Vec<VariableMode>,
    #[serde(default, rename = "defaultModeId")]
    pub default_mode_id: String,
}

/// Variable mode (e.g., light/dark)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableMode {
    #[serde(rename = "modeId")]
    pub mode_id: String,
    pub name: String,
}

/// Image fills response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageFillsResponse {
    pub error: bool,
    #[serde(default)]
    pub images: HashMap<String, String>,
    pub status: Option<u32>,
}

/// Images response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagesResponse {
    pub err: Option<String>,
    #[serde(default)]
    pub images: HashMap<String, Option<String>>,
}
