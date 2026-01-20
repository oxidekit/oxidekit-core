//! Platform-specific behaviors for OxideKit
//!
//! Provides platform detection, keyboard handling, navigation idioms.

pub mod detect;
pub mod keyboard;
pub mod navigation;
pub mod safe_area;
pub mod scroll;

pub use detect::*;
pub use keyboard::*;
pub use navigation::*;
pub use safe_area::*;
pub use scroll::*;
