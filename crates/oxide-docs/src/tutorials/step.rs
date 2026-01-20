//! Tutorial step definitions

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A single step in a tutorial
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialStep {
    /// Step identifier
    pub id: String,
    /// Step title
    pub title: String,
    /// Step content (markdown)
    pub content: String,
    /// Action to perform (optional)
    pub action: Option<StepAction>,
    /// Validation to check completion
    pub validation: Option<StepValidation>,
    /// Hint for if the user gets stuck
    pub hint: Option<String>,
    /// Whether this step is optional
    pub optional: bool,
}

impl TutorialStep {
    /// Create a new tutorial step
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            action: None,
            validation: None,
            hint: None,
            optional: false,
        }
    }

    /// Set an action for this step
    pub fn with_action(mut self, action: StepAction) -> Self {
        self.action = Some(action);
        self
    }

    /// Set validation for this step
    pub fn with_validation(mut self, validation: StepValidation) -> Self {
        self.validation = Some(validation);
        self
    }

    /// Set a hint for this step
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Mark this step as optional
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Check if this step requires user action
    pub fn requires_action(&self) -> bool {
        self.action.is_some()
    }

    /// Check if this step has validation
    pub fn has_validation(&self) -> bool {
        self.validation.is_some()
    }
}

/// Action to perform in a tutorial step
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StepAction {
    /// Run a shell command
    RunCommand {
        command: String,
        working_dir: Option<String>,
    },
    /// Create a file with content
    CreateFile {
        path: String,
        content: String,
    },
    /// Modify an existing file
    ModifyFile {
        path: String,
        find: String,
        replace: String,
    },
    /// Copy a file
    CopyFile {
        from: String,
        to: String,
    },
    /// Open a file in editor
    OpenFile {
        path: String,
        line: Option<u32>,
    },
    /// Display information (no action needed)
    DisplayInfo {
        message: String,
    },
    /// Wait for user confirmation
    WaitForConfirmation {
        message: String,
    },
    /// Run a code snippet
    RunCode {
        language: String,
        code: String,
    },
}

impl StepAction {
    /// Get a human-readable description of the action
    pub fn description(&self) -> String {
        match self {
            Self::RunCommand { command, .. } => format!("Run command: {}", command),
            Self::CreateFile { path, .. } => format!("Create file: {}", path),
            Self::ModifyFile { path, .. } => format!("Modify file: {}", path),
            Self::CopyFile { from, to } => format!("Copy {} to {}", from, to),
            Self::OpenFile { path, .. } => format!("Open file: {}", path),
            Self::DisplayInfo { .. } => "Read information".to_string(),
            Self::WaitForConfirmation { .. } => "Confirm completion".to_string(),
            Self::RunCode { language, .. } => format!("Run {} code", language),
        }
    }

    /// Check if this action can run automatically
    pub fn is_automatic(&self) -> bool {
        matches!(
            self,
            Self::RunCommand { .. }
                | Self::CreateFile { .. }
                | Self::ModifyFile { .. }
                | Self::CopyFile { .. }
                | Self::RunCode { .. }
        )
    }
}

/// Validation to check if a step is completed
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StepValidation {
    /// Check if a file exists
    FileExists {
        path: String,
    },
    /// Check if a file contains specific content
    FileContains {
        path: String,
        content: String,
    },
    /// Check if a directory exists
    DirectoryExists {
        path: String,
    },
    /// Check if a command succeeds
    CommandSucceeds {
        command: String,
        working_dir: Option<String>,
    },
    /// Run a custom validation script
    CustomScript {
        script: String,
    },
    /// Always passes (manual steps)
    Manual,
}

impl StepValidation {
    /// Get a description of what this validation checks
    pub fn description(&self) -> String {
        match self {
            Self::FileExists { path } => format!("File exists: {}", path),
            Self::FileContains { path, .. } => format!("File {} contains expected content", path),
            Self::DirectoryExists { path } => format!("Directory exists: {}", path),
            Self::CommandSucceeds { command, .. } => format!("Command succeeds: {}", command),
            Self::CustomScript { .. } => "Custom validation".to_string(),
            Self::Manual => "Manual verification".to_string(),
        }
    }

    /// Validate the step (basic implementation)
    pub fn validate(&self, working_dir: Option<&PathBuf>) -> bool {
        match self {
            Self::FileExists { path } => {
                let full_path = working_dir
                    .map(|d| d.join(path))
                    .unwrap_or_else(|| PathBuf::from(path));
                full_path.exists()
            }
            Self::FileContains { path, content } => {
                let full_path = working_dir
                    .map(|d| d.join(path))
                    .unwrap_or_else(|| PathBuf::from(path));
                if let Ok(file_content) = std::fs::read_to_string(full_path) {
                    file_content.contains(content)
                } else {
                    false
                }
            }
            Self::DirectoryExists { path } => {
                let full_path = working_dir
                    .map(|d| d.join(path))
                    .unwrap_or_else(|| PathBuf::from(path));
                full_path.is_dir()
            }
            Self::CommandSucceeds { command, working_dir: cmd_dir } => {
                let dir = cmd_dir
                    .as_ref()
                    .map(PathBuf::from)
                    .or_else(|| working_dir.cloned());

                let mut cmd = if cfg!(target_os = "windows") {
                    let mut c = std::process::Command::new("cmd");
                    c.args(["/C", command]);
                    c
                } else {
                    let mut c = std::process::Command::new("sh");
                    c.args(["-c", command]);
                    c
                };

                if let Some(d) = dir {
                    cmd.current_dir(d);
                }

                cmd.output().map(|o| o.status.success()).unwrap_or(false)
            }
            Self::CustomScript { .. } => {
                // Custom scripts need specialized handling
                true
            }
            Self::Manual => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_creation() {
        let step = TutorialStep::new("test", "Test Step", "Test content")
            .with_hint("This is a hint")
            .optional();

        assert_eq!(step.id, "test");
        assert!(step.optional);
        assert!(step.hint.is_some());
    }

    #[test]
    fn test_action_description() {
        let action = StepAction::RunCommand {
            command: "cargo build".to_string(),
            working_dir: None,
        };

        assert!(action.description().contains("cargo build"));
        assert!(action.is_automatic());
    }

    #[test]
    fn test_validation() {
        let validation = StepValidation::Manual;
        assert!(validation.validate(None));
    }
}
