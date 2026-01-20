//! Interactive tutorials and learning guides
//!
//! This module provides a system for creating and running interactive
//! tutorials that guide users through OxideKit development.

mod runner;
mod step;
mod tutorial;

pub use runner::TutorialRunner;
pub use step::{TutorialStep, StepAction, StepValidation};
pub use tutorial::{Tutorial, getting_started_tutorial, component_tutorial};

use crate::DocsResult;
use std::path::Path;

/// Load a tutorial from a file
pub fn load_tutorial(path: &Path) -> DocsResult<Tutorial> {
    Tutorial::load(path)
}

/// List all available tutorials in a directory
pub fn list_tutorials(dir: &Path) -> DocsResult<Vec<Tutorial>> {
    Tutorial::list_in_directory(dir)
}

/// Run a tutorial interactively
pub fn run_tutorial(tutorial: &Tutorial) -> DocsResult<TutorialRunner> {
    TutorialRunner::new(tutorial.clone())
}
