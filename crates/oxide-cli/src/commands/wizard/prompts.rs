//! Interactive prompt utilities
//!
//! Wrapper utilities for dialoguer prompts with consistent styling.

use anyhow::Result;
use console::{style, Style, Term};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select, FuzzySelect};

/// Custom theme for OxideKit CLI prompts
pub fn oxide_theme() -> ColorfulTheme {
    ColorfulTheme::default()
}

/// Prompt for text input with optional default value
pub fn text_input(prompt: &str, default: Option<&str>) -> Result<String> {
    let theme = oxide_theme();

    let mut input = Input::<String>::with_theme(&theme)
        .with_prompt(prompt);

    if let Some(default_val) = default {
        input = input.default(default_val.to_string());
    }

    let result = input
        .interact_text()
        .map_err(|e| anyhow::anyhow!("Input error: {}", e))?;

    Ok(result)
}

/// Prompt for text input with validation
pub fn validated_input<F>(prompt: &str, default: Option<&str>, validator: F) -> Result<String>
where
    F: Fn(&String) -> Result<(), String> + Clone,
{
    let theme = oxide_theme();

    let mut input = Input::<String>::with_theme(&theme)
        .with_prompt(prompt)
        .validate_with(move |input: &String| -> Result<(), String> {
            validator(input)
        });

    if let Some(default_val) = default {
        input = input.default(default_val.to_string());
    }

    let result = input
        .interact_text()
        .map_err(|e| anyhow::anyhow!("Input error: {}", e))?;

    Ok(result)
}

/// Prompt for yes/no confirmation
pub fn confirm(prompt: &str, default: bool) -> Result<bool> {
    let theme = oxide_theme();

    let result = Confirm::with_theme(&theme)
        .with_prompt(prompt)
        .default(default)
        .interact()
        .map_err(|e| anyhow::anyhow!("Confirmation error: {}", e))?;

    Ok(result)
}

/// Prompt for single selection from a list
pub fn select<T: ToString + Clone>(prompt: &str, items: &[T], default: usize) -> Result<usize> {
    let theme = oxide_theme();

    let items_str: Vec<String> = items.iter().map(|i| i.to_string()).collect();

    let selection = Select::with_theme(&theme)
        .with_prompt(prompt)
        .items(&items_str)
        .default(default)
        .interact()
        .map_err(|e| anyhow::anyhow!("Selection error: {}", e))?;

    Ok(selection)
}

/// Prompt for single selection with fuzzy search
pub fn fuzzy_select<T: ToString + Clone>(prompt: &str, items: &[T], default: usize) -> Result<usize> {
    let theme = oxide_theme();

    let items_str: Vec<String> = items.iter().map(|i| i.to_string()).collect();

    let selection = FuzzySelect::with_theme(&theme)
        .with_prompt(prompt)
        .items(&items_str)
        .default(default)
        .interact()
        .map_err(|e| anyhow::anyhow!("Selection error: {}", e))?;

    Ok(selection)
}

/// Prompt for multiple selections from a list
pub fn multi_select<T: ToString + Clone>(
    prompt: &str,
    items: &[T],
    defaults: &[bool],
) -> Result<Vec<usize>> {
    let theme = oxide_theme();

    let items_str: Vec<String> = items.iter().map(|i| i.to_string()).collect();

    let selections = MultiSelect::with_theme(&theme)
        .with_prompt(prompt)
        .items(&items_str)
        .defaults(defaults)
        .interact()
        .map_err(|e| anyhow::anyhow!("Multi-selection error: {}", e))?;

    Ok(selections)
}

/// Display a styled header
pub fn display_header(title: &str) {
    let term = Term::stdout();
    let _ = term.clear_line();
    println!();
    println!("{}", style(title).cyan().bold());
    println!("{}", style("=".repeat(title.len())).cyan());
    println!();
}

/// Display a styled section
pub fn display_section(title: &str) {
    println!();
    println!("{}", style(format!("  {}", title)).bold());
    println!();
}

/// Display an info message
pub fn info(message: &str) {
    println!("  {} {}", style("i").blue().bold(), message);
}

/// Display a success message
pub fn success(message: &str) {
    println!("  {} {}", style("v").green().bold(), message);
}

/// Display a warning message
pub fn warning(message: &str) {
    println!("  {} {}", style("!").yellow().bold(), message);
}

/// Display an error message
pub fn error(message: &str) {
    eprintln!("  {} {}", style("x").red().bold(), message);
}

/// Display a list item
pub fn list_item(item: &str, description: &str) {
    println!(
        "    {} {} - {}",
        style("-").dim(),
        style(item).cyan(),
        style(description).dim()
    );
}

/// Display a progress step
pub fn step(number: usize, total: usize, description: &str) {
    println!(
        "  {} {} {}",
        style(format!("[{}/{}]", number, total)).dim(),
        style(">").cyan(),
        description
    );
}

/// Clear the terminal screen
pub fn clear_screen() -> Result<()> {
    let term = Term::stdout();
    term.clear_screen()
        .map_err(|e| anyhow::anyhow!("Terminal error: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oxide_theme_creation() {
        let theme = oxide_theme();
        // Just ensure it doesn't panic
        let _ = theme;
    }
}
