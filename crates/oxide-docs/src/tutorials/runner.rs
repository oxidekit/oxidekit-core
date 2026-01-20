//! Tutorial runner for executing interactive tutorials

use crate::{DocsError, DocsResult};
use std::io::{self, Write};
use std::path::PathBuf;
use tracing::{debug, info};

use super::{StepAction, Tutorial, TutorialStep};

/// State of a tutorial session
#[derive(Debug, Clone)]
pub struct TutorialState {
    /// Current step index
    pub current_step: usize,
    /// Completed step IDs
    pub completed_steps: Vec<String>,
    /// Working directory for the tutorial
    pub working_dir: Option<PathBuf>,
    /// Whether the tutorial is complete
    pub is_complete: bool,
    /// Any errors encountered
    pub errors: Vec<String>,
}

impl Default for TutorialState {
    fn default() -> Self {
        Self {
            current_step: 0,
            completed_steps: Vec::new(),
            working_dir: None,
            is_complete: false,
            errors: Vec::new(),
        }
    }
}

/// Runner for executing tutorials interactively
pub struct TutorialRunner {
    tutorial: Tutorial,
    state: TutorialState,
}

impl TutorialRunner {
    /// Create a new tutorial runner
    pub fn new(tutorial: Tutorial) -> DocsResult<Self> {
        Ok(Self {
            tutorial,
            state: TutorialState::default(),
        })
    }

    /// Set the working directory
    pub fn set_working_dir(&mut self, dir: PathBuf) {
        self.state.working_dir = Some(dir);
    }

    /// Get the current state
    pub fn state(&self) -> &TutorialState {
        &self.state
    }

    /// Get the tutorial
    pub fn tutorial(&self) -> &Tutorial {
        &self.tutorial
    }

    /// Get the current step
    pub fn current_step(&self) -> Option<&TutorialStep> {
        self.tutorial.steps.get(self.state.current_step)
    }

    /// Get the progress as a percentage
    pub fn progress(&self) -> f32 {
        if self.tutorial.steps.is_empty() {
            return 100.0;
        }
        (self.state.current_step as f32 / self.tutorial.steps.len() as f32) * 100.0
    }

    /// Move to the next step
    pub fn next_step(&mut self) -> DocsResult<bool> {
        // Validate current step if needed
        if let Some(step) = self.current_step() {
            if let Some(ref validation) = step.validation {
                if !validation.validate(self.state.working_dir.as_ref()) {
                    return Err(DocsError::Tutorial(format!(
                        "Step '{}' validation failed: {}",
                        step.title,
                        validation.description()
                    )));
                }
            }

            // Mark as completed
            self.state.completed_steps.push(step.id.clone());
        }

        // Move to next
        self.state.current_step += 1;

        if self.state.current_step >= self.tutorial.steps.len() {
            self.state.is_complete = true;
            info!("Tutorial '{}' completed!", self.tutorial.title);
            return Ok(false);
        }

        Ok(true)
    }

    /// Go back to the previous step
    pub fn previous_step(&mut self) -> bool {
        if self.state.current_step > 0 {
            self.state.current_step -= 1;
            self.state.is_complete = false;
            true
        } else {
            false
        }
    }

    /// Jump to a specific step by ID
    pub fn goto_step(&mut self, step_id: &str) -> DocsResult<()> {
        let position = self
            .tutorial
            .steps
            .iter()
            .position(|s| s.id == step_id)
            .ok_or_else(|| DocsError::Tutorial(format!("Step not found: {}", step_id)))?;

        self.state.current_step = position;
        self.state.is_complete = false;
        Ok(())
    }

    /// Execute the current step's action
    pub fn execute_action(&mut self) -> DocsResult<ActionResult> {
        let step = self
            .current_step()
            .ok_or_else(|| DocsError::Tutorial("No current step".to_string()))?;

        let action = match &step.action {
            Some(a) => a.clone(),
            None => return Ok(ActionResult::NoAction),
        };

        debug!("Executing action: {:?}", action);

        match action {
            StepAction::RunCommand { command, working_dir } => {
                self.run_command(&command, working_dir.as_deref())
            }
            StepAction::CreateFile { path, content } => {
                self.create_file(&path, &content)
            }
            StepAction::ModifyFile { path, find, replace } => {
                self.modify_file(&path, &find, &replace)
            }
            StepAction::CopyFile { from, to } => {
                self.copy_file(&from, &to)
            }
            StepAction::OpenFile { path, line } => {
                self.open_file(&path, line)
            }
            StepAction::DisplayInfo { message } => {
                Ok(ActionResult::Info(message))
            }
            StepAction::WaitForConfirmation { message } => {
                Ok(ActionResult::WaitingForConfirmation(message))
            }
            StepAction::RunCode { language, code } => {
                self.run_code(&language, &code)
            }
        }
    }

    /// Run a shell command
    fn run_command(&self, command: &str, working_dir: Option<&str>) -> DocsResult<ActionResult> {
        let dir = working_dir
            .map(PathBuf::from)
            .or_else(|| self.state.working_dir.clone());

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

        let output = cmd.output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(ActionResult::CommandOutput {
                success: true,
                stdout,
                stderr: String::new(),
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Ok(ActionResult::CommandOutput {
                success: false,
                stdout: String::new(),
                stderr,
            })
        }
    }

    /// Create a file
    fn create_file(&self, path: &str, content: &str) -> DocsResult<ActionResult> {
        let full_path = self.resolve_path(path);

        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&full_path, content)?;

        Ok(ActionResult::FileCreated(full_path))
    }

    /// Modify a file
    fn modify_file(&self, path: &str, find: &str, replace: &str) -> DocsResult<ActionResult> {
        let full_path = self.resolve_path(path);

        let content = std::fs::read_to_string(&full_path)?;
        let new_content = content.replace(find, replace);
        std::fs::write(&full_path, new_content)?;

        Ok(ActionResult::FileModified(full_path))
    }

    /// Copy a file
    fn copy_file(&self, from: &str, to: &str) -> DocsResult<ActionResult> {
        let from_path = self.resolve_path(from);
        let to_path = self.resolve_path(to);

        if let Some(parent) = to_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::copy(&from_path, &to_path)?;

        Ok(ActionResult::FileCopied { from: from_path, to: to_path })
    }

    /// Open a file (just returns the path, actual opening depends on environment)
    fn open_file(&self, path: &str, line: Option<u32>) -> DocsResult<ActionResult> {
        let full_path = self.resolve_path(path);
        Ok(ActionResult::OpenFile { path: full_path, line })
    }

    /// Run code (placeholder - would need language runtime integration)
    fn run_code(&self, language: &str, code: &str) -> DocsResult<ActionResult> {
        // For now, just return the code as info
        Ok(ActionResult::Info(format!(
            "Would run {} code:\n{}",
            language, code
        )))
    }

    /// Resolve a path relative to working directory
    fn resolve_path(&self, path: &str) -> PathBuf {
        if let Some(ref dir) = self.state.working_dir {
            dir.join(path)
        } else {
            PathBuf::from(path)
        }
    }

    /// Run the tutorial in interactive mode (terminal)
    pub fn run_interactive(&mut self) -> DocsResult<()> {
        println!("\n{}", "=".repeat(60));
        println!("Tutorial: {}", self.tutorial.title);
        println!("{}", "=".repeat(60));
        println!("\n{}\n", self.tutorial.description);
        println!("Steps: {}", self.tutorial.steps.len());
        println!("Difficulty: {}", self.tutorial.difficulty.display_name());
        println!("Est. time: {} minutes\n", self.tutorial.estimated_minutes);

        loop {
            let step = match self.current_step() {
                Some(s) => s.clone(),
                None => break,
            };

            println!("\n{}", "-".repeat(60));
            println!(
                "Step {}/{}: {}",
                self.state.current_step + 1,
                self.tutorial.steps.len(),
                step.title
            );
            println!("{}", "-".repeat(60));
            println!("\n{}\n", step.content);

            // Show action if present
            if let Some(ref action) = step.action {
                println!("Action: {}", action.description());
            }

            // Interactive prompt
            print!("\n[n]ext, [p]revious, [r]un action, [h]int, [q]uit: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            match input.trim().to_lowercase().as_str() {
                "n" | "next" | "" => {
                    if !self.next_step()? {
                        println!("\nCongratulations! Tutorial completed!");
                        break;
                    }
                }
                "p" | "previous" => {
                    if !self.previous_step() {
                        println!("Already at the first step.");
                    }
                }
                "r" | "run" => {
                    match self.execute_action() {
                        Ok(result) => println!("\nResult: {:?}", result),
                        Err(e) => println!("\nError: {}", e),
                    }
                }
                "h" | "hint" => {
                    if let Some(ref hint) = step.hint {
                        println!("\nHint: {}", hint);
                    } else {
                        println!("\nNo hint available for this step.");
                    }
                }
                "q" | "quit" => {
                    println!("\nExiting tutorial. Progress saved.");
                    break;
                }
                _ => {
                    println!("Unknown command. Try: n, p, r, h, q");
                }
            }
        }

        Ok(())
    }
}

/// Result of executing an action
#[derive(Debug, Clone)]
pub enum ActionResult {
    /// No action was needed
    NoAction,
    /// Information message
    Info(String),
    /// Waiting for user confirmation
    WaitingForConfirmation(String),
    /// Command output
    CommandOutput {
        success: bool,
        stdout: String,
        stderr: String,
    },
    /// File was created
    FileCreated(PathBuf),
    /// File was modified
    FileModified(PathBuf),
    /// File was copied
    FileCopied {
        from: PathBuf,
        to: PathBuf,
    },
    /// File should be opened
    OpenFile {
        path: PathBuf,
        line: Option<u32>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tutorials::tutorial::getting_started_tutorial;

    #[test]
    fn test_runner_creation() {
        let tutorial = getting_started_tutorial();
        let runner = TutorialRunner::new(tutorial).unwrap();

        assert_eq!(runner.state().current_step, 0);
        assert!(!runner.state().is_complete);
    }

    #[test]
    fn test_progress_calculation() {
        let tutorial = getting_started_tutorial();
        let runner = TutorialRunner::new(tutorial).unwrap();

        assert_eq!(runner.progress(), 0.0);
    }

    #[test]
    fn test_next_step() {
        let tutorial = getting_started_tutorial();
        let mut runner = TutorialRunner::new(tutorial).unwrap();

        let initial = runner.state().current_step;
        runner.next_step().unwrap();
        assert_eq!(runner.state().current_step, initial + 1);
    }
}
