//! Dialog and modal system for OxideKit
//!
//! Provides alert, confirm, prompt dialogs and modal management.

pub mod alert;
pub mod confirm;
pub mod dialog;
pub mod options;
pub mod prompt;

pub use alert::*;
pub use confirm::*;
pub use dialog::*;
pub use options::*;
pub use prompt::*;
