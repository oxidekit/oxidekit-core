//! Team workflow CLI command implementations
//!
//! Handles export, import, coverage, quality, lock, and freeze commands.

use crate::formats::{self, Format, TranslationFile, TranslationState};
use crate::reports::{CoverageReport, QualityConfig, QualityReport};
use crate::workflow::{LockManager, LockStatus, StringFreeze};
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Run the export command
pub fn run_export(
    locale: &str,
    output: Option<&str>,
    format: &str,
    translations_dir: &str,
    untranslated_only: bool,
    include_context: bool,
) -> Result<()> {
    let translations_path = PathBuf::from(translations_dir);

    // Load the source (English) file
    let source_file = find_translation_file(&translations_path, "en")?;
    let mut source = formats::load_file(&source_file)?;

    // Try to load existing translations
    if let Ok(target_file) = find_translation_file(&translations_path, locale) {
        let existing = formats::load_file(&target_file)?;

        // Merge existing translations
        for entry in &mut source.entries {
            if let Some(existing_entry) = existing.entries.iter().find(|e| e.key == entry.key) {
                entry.target = existing_entry.target.clone();
                entry.state = existing_entry.state;
            }
        }
    }

    // Set target locale
    source.target_locale = locale.to_string();

    // Filter if untranslated only
    if untranslated_only {
        source.entries.retain(|e| e.target.is_none());
    }

    // Clear context if not needed
    if !include_context {
        for entry in &mut source.entries {
            entry.metadata.context = None;
            entry.metadata.notes = None;
        }
    }

    // Determine output format
    let export_format = match format.to_lowercase().as_str() {
        "xliff" | "xlf" => Format::Xliff,
        "json" => Format::Json,
        "po" => Format::Po,
        "toml" => Format::Toml,
        _ => Format::Xliff,
    };

    // Determine output path
    let output_path = match output {
        Some(p) => PathBuf::from(p),
        None => {
            let filename = format!("{}.{}", locale, export_format.extension());
            translations_path.join("exports").join(filename)
        }
    };

    // Create parent directory if needed
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Export
    formats::save_file(&source, &output_path, export_format)?;

    println!(
        "Exported {} keys for locale '{}' to {}",
        source.entries.len(),
        locale,
        output_path.display()
    );

    if untranslated_only {
        println!("(untranslated keys only)");
    }

    Ok(())
}

/// Run the import command
pub fn run_import(
    file: &str,
    translations_dir: &str,
    needs_review: bool,
    overwrite: bool,
    dry_run: bool,
) -> Result<()> {
    let input_path = PathBuf::from(file);
    let translations_path = PathBuf::from(translations_dir);

    // Load imported file
    let imported = formats::load_file(&input_path)?;
    let locale = &imported.target_locale;

    // Try to load existing translations
    let target_file = translations_path.join(format!("{}.toml", locale));
    let mut target = if target_file.exists() {
        formats::load_file(&target_file)?
    } else {
        TranslationFile::new(&imported.source_locale, locale)
    };

    let mut added = 0;
    let mut updated = 0;
    let mut skipped = 0;

    for imported_entry in &imported.entries {
        if imported_entry.target.is_none() {
            continue; // Skip entries without translations
        }

        if let Some(existing) = target.get_entry_mut(&imported_entry.key) {
            if existing.target.is_some() && !overwrite {
                skipped += 1;
                if dry_run {
                    println!("  SKIP (exists): {}", imported_entry.key);
                }
                continue;
            }

            existing.target = imported_entry.target.clone();
            existing.state = if needs_review {
                TranslationState::NeedsReview
            } else {
                imported_entry.state
            };
            updated += 1;

            if dry_run {
                println!("  UPDATE: {}", imported_entry.key);
            }
        } else {
            let mut new_entry = imported_entry.clone();
            if needs_review {
                new_entry.state = TranslationState::NeedsReview;
            }
            target.add_entry(new_entry);
            added += 1;

            if dry_run {
                println!("  ADD: {}", imported_entry.key);
            }
        }
    }

    if dry_run {
        println!("\nDry run - no changes made");
        println!("Would add: {}, update: {}, skip: {}", added, updated, skipped);
    } else {
        // Save the updated file
        formats::save_file(&target, &target_file, Format::Toml)?;
        println!(
            "Imported translations for '{}': {} added, {} updated, {} skipped",
            locale, added, updated, skipped
        );
    }

    Ok(())
}

/// Run the coverage command
pub fn run_coverage(
    translations_dir: &str,
    format: &str,
    output: Option<&str>,
    locale: Option<&str>,
    show_keys: bool,
) -> Result<()> {
    let translations_path = PathBuf::from(translations_dir);

    // Load all translation files
    let mut files = Vec::new();
    for entry in fs::read_dir(&translations_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |e| e == "toml" || e == "json") {
            if let Ok(file) = formats::load_file(&path) {
                // Filter by locale if specified
                if let Some(target_locale) = locale {
                    if file.target_locale != target_locale {
                        continue;
                    }
                }
                files.push(file);
            }
        }
    }

    if files.is_empty() {
        return Err(anyhow!("No translation files found in {}", translations_dir));
    }

    // Generate report
    let report = CoverageReport::from_files("en", &files);

    // Output based on format
    let output_content = match format.to_lowercase().as_str() {
        "json" => report.to_json(),
        "markdown" | "md" => report.to_markdown(),
        _ => format_coverage_human(&report, show_keys),
    };

    if let Some(output_path) = output {
        fs::write(output_path, &output_content)?;
        println!("Coverage report written to {}", output_path);
    } else {
        println!("{}", output_content);
    }

    // Exit with error if not release ready
    if !report.is_release_ready() {
        std::process::exit(1);
    }

    Ok(())
}

/// Format coverage report for human reading
fn format_coverage_human(report: &CoverageReport, show_keys: bool) -> String {
    let mut output = String::new();

    output.push_str("\nTranslation Coverage Report\n");
    output.push_str(&"=".repeat(60));
    output.push('\n');

    // Overall stats
    output.push_str(&format!(
        "\nOverall: {}/{} locales complete ({:.1}% avg)\n",
        report.overall.complete_locales,
        report.overall.total_locales,
        report.overall.average_translation_percentage
    ));

    if report.is_release_ready() {
        output.push_str("Status: RELEASE READY\n");
    } else {
        output.push_str(&format!(
            "Status: NOT READY ({} blockers)\n",
            report.blockers().len()
        ));
    }

    // Locale breakdown
    output.push_str("\nBy Locale:\n");
    output.push_str(&"-".repeat(60));
    output.push('\n');

    for locale in &report.by_locale {
        let bar_len = (locale.translation_percentage / 5.0) as usize;
        let bar = format!("[{}{}]", "#".repeat(bar_len), " ".repeat(20 - bar_len));

        output.push_str(&format!(
            "{:8} {} {:5.1}%",
            locale.locale, bar, locale.translation_percentage
        ));

        if !locale.missing_keys.is_empty() {
            output.push_str(&format!(" ({} missing)", locale.missing_keys.len()));
        }

        if locale.needs_review_keys > 0 {
            output.push_str(&format!(" ({} review)", locale.needs_review_keys));
        }

        output.push('\n');
    }

    // Show missing keys if requested
    if show_keys && !report.by_key.is_empty() {
        output.push_str("\nKeys with Issues:\n");
        output.push_str(&"-".repeat(60));
        output.push('\n');

        for key in report.by_key.iter().take(20) {
            output.push_str(&format!("  {} - missing in: {}\n", key.key, key.missing_locales.join(", ")));
        }

        if report.by_key.len() > 20 {
            output.push_str(&format!("  ... and {} more\n", report.by_key.len() - 20));
        }
    }

    output
}

/// Run the quality command
pub fn run_quality(
    locale: &str,
    translations_dir: &str,
    format: &str,
    output: Option<&str>,
    strict: bool,
) -> Result<()> {
    let translations_path = PathBuf::from(translations_dir);
    let file_path = find_translation_file(&translations_path, locale)?;

    let file = formats::load_file(&file_path)?;
    let config = QualityConfig::default();
    let report = QualityReport::from_file(&file, &config);

    // Output based on format
    let output_content = match format.to_lowercase().as_str() {
        "json" => serde_json::to_string_pretty(&report).unwrap_or_default(),
        "markdown" | "md" => report.to_markdown(),
        _ => format_quality_human(&report),
    };

    if let Some(output_path) = output {
        fs::write(output_path, &output_content)?;
        println!("Quality report written to {}", output_path);
    } else {
        println!("{}", output_content);
    }

    // Exit with error if not passing
    if !report.is_passing() || (strict && report.summary.warning_count > 0) {
        std::process::exit(1);
    }

    Ok(())
}

/// Format quality report for human reading
fn format_quality_human(report: &QualityReport) -> String {
    let mut output = String::new();

    output.push_str("\nTranslation Quality Report\n");
    output.push_str(&"=".repeat(60));
    output.push('\n');

    output.push_str(&format!("Locale: {}\n", report.locale));
    output.push_str(&format!("Score:  {:.0}%\n", report.summary.quality_score));
    output.push_str(&format!(
        "Status: {}\n\n",
        if report.is_passing() { "PASSING" } else { "FAILING" }
    ));

    output.push_str(&format!("Entries Checked: {}\n", report.summary.total_checked));
    output.push_str(&format!("Errors:          {}\n", report.summary.error_count));
    output.push_str(&format!("Warnings:        {}\n", report.summary.warning_count));
    output.push_str(&format!("Info:            {}\n", report.summary.info_count));

    if !report.issues.is_empty() {
        output.push_str("\nIssues:\n");
        output.push_str(&"-".repeat(60));
        output.push('\n');

        for issue in &report.issues {
            let level = match issue.level {
                crate::reports::QualityLevel::Error => "ERROR",
                crate::reports::QualityLevel::Warning => "WARN ",
                crate::reports::QualityLevel::Info => "INFO ",
            };

            output.push_str(&format!("[{}] {}: {}\n", level, issue.key, issue.message));
        }
    }

    output
}

/// Run lock acquire command
pub fn run_lock_acquire(key: &str, duration: u32, note: Option<&str>, translations_dir: &str) -> Result<()> {
    let lock_file = PathBuf::from(translations_dir).join(".locks.json");
    let user = get_current_user();

    let mut manager = LockManager::new(&lock_file, &user)?;
    let duration = chrono::Duration::hours(duration as i64);

    let mut lock = manager.lock_with_duration(key, duration)?;

    if let Some(note_text) = note {
        lock.note = Some(note_text.to_string());
    }

    println!("Locked '{}' for {} hours", key, duration.num_hours());
    println!("Lock expires at: {}", lock.expires_at.format("%Y-%m-%d %H:%M:%S UTC"));

    Ok(())
}

/// Run lock release command
pub fn run_lock_release(key: &str, translations_dir: &str) -> Result<()> {
    let lock_file = PathBuf::from(translations_dir).join(".locks.json");
    let user = get_current_user();

    let mut manager = LockManager::new(&lock_file, &user)?;
    manager.unlock(key)?;

    println!("Released lock on '{}'", key);

    Ok(())
}

/// Run lock list command
pub fn run_lock_list(mine_only: bool, translations_dir: &str) -> Result<()> {
    let lock_file = PathBuf::from(translations_dir).join(".locks.json");
    let user = get_current_user();

    let manager = LockManager::new(&lock_file, &user)?;
    let locks: Vec<_> = if mine_only {
        manager.my_locks()
    } else {
        manager.active_locks()
    };

    if locks.is_empty() {
        println!("No active locks");
        return Ok(());
    }

    println!("\nActive Locks:");
    println!("{:-<60}", "");

    for lock in locks {
        println!(
            "  {} - {} (by {}, expires {})",
            lock.key_pattern,
            lock.note.as_deref().unwrap_or(""),
            lock.owner,
            lock.expires_at.format("%H:%M:%S")
        );
    }

    Ok(())
}

/// Run lock status command
pub fn run_lock_status(key: &str, translations_dir: &str) -> Result<()> {
    let lock_file = PathBuf::from(translations_dir).join(".locks.json");
    let user = get_current_user();

    let manager = LockManager::new(&lock_file, &user)?;
    let status = manager.status(key);

    match status {
        LockStatus::Unlocked => println!("'{}' is not locked", key),
        LockStatus::LockedBySelf(lock) => {
            println!("'{}' is locked by you", key);
            println!("Expires: {}", lock.expires_at.format("%Y-%m-%d %H:%M:%S UTC"));
        }
        LockStatus::LockedByOther(lock) => {
            println!("'{}' is locked by {}", key, lock.owner);
            println!("Expires: {}", lock.expires_at.format("%Y-%m-%d %H:%M:%S UTC"));
            if let Some(note) = &lock.note {
                println!("Note: {}", note);
            }
        }
        LockStatus::Expired(lock) => {
            println!("'{}' has an expired lock (was held by {})", key, lock.owner);
        }
    }

    Ok(())
}

/// Run lock release all command
pub fn run_lock_release_all(translations_dir: &str) -> Result<()> {
    let lock_file = PathBuf::from(translations_dir).join(".locks.json");
    let user = get_current_user();

    let mut manager = LockManager::new(&lock_file, &user)?;
    let count = manager.unlock_all_mine()?;

    println!("Released {} lock(s)", count);

    Ok(())
}

/// Run freeze start command
pub fn run_freeze_start(version: &str, translations_dir: &str) -> Result<()> {
    let freeze_file = PathBuf::from(translations_dir).join(".freeze.json");
    let user = get_current_user();

    let mut freeze = StringFreeze::new(&freeze_file, &user)?;
    freeze.start_freeze(version)?;

    println!("Started string freeze for version {}", version);
    println!("Current phase: {:?}", freeze.phase());

    Ok(())
}

/// Run freeze advance command
pub fn run_freeze_advance(reason: Option<&str>, translations_dir: &str) -> Result<()> {
    let freeze_file = PathBuf::from(translations_dir).join(".freeze.json");
    let user = get_current_user();

    let mut freeze = StringFreeze::new(&freeze_file, &user)?;
    let new_phase = freeze.advance_phase(reason.map(String::from))?;

    println!("Advanced to phase: {:?}", new_phase);

    Ok(())
}

/// Run freeze status command
pub fn run_freeze_status(translations_dir: &str) -> Result<()> {
    let freeze_file = PathBuf::from(translations_dir).join(".freeze.json");
    let user = get_current_user();

    let freeze = StringFreeze::new(&freeze_file, &user)?;
    let status = freeze.status();

    println!("\nString Freeze Status");
    println!("{:-<40}", "");
    println!("Version: {}", status.version);
    println!("Phase:   {:?}", status.phase);

    if let Some(started) = status.started_at {
        println!("Started: {}", started.format("%Y-%m-%d %H:%M:%S UTC"));
    }

    if !status.exceptions.is_empty() {
        println!("\nExceptions:");
        for key in &status.exceptions {
            println!("  - {}", key);
        }
    }

    println!("\nAllowed actions in this phase:");
    println!("  - Add strings:    {}", if status.phase.allows_additions() { "yes" } else { "NO" });
    println!("  - Modify strings: {}", if status.phase.allows_modifications() { "yes" } else { "NO" });
    println!("  - Remove strings: {}", if status.phase.allows_removals() { "yes" } else { "NO" });
    println!("  - Translations:   {}", if status.phase.allows_translations() { "yes" } else { "NO" });

    Ok(())
}

/// Run freeze exception command
pub fn run_freeze_exception(key: &str, translations_dir: &str) -> Result<()> {
    let freeze_file = PathBuf::from(translations_dir).join(".freeze.json");
    let user = get_current_user();

    let mut freeze = StringFreeze::new(&freeze_file, &user)?;
    freeze.add_exception(key)?;

    println!("Added exception for '{}'", key);

    Ok(())
}

/// Run freeze reset command
pub fn run_freeze_reset(translations_dir: &str) -> Result<()> {
    let freeze_file = PathBuf::from(translations_dir).join(".freeze.json");
    let user = get_current_user();

    let mut freeze = StringFreeze::new(&freeze_file, &user)?;
    freeze.reset()?;

    println!("Reset to development phase");

    Ok(())
}

/// Find a translation file for a locale
fn find_translation_file(dir: &Path, locale: &str) -> Result<PathBuf> {
    // Try common extensions
    for ext in &["toml", "json"] {
        let path = dir.join(format!("{}.{}", locale, ext));
        if path.exists() {
            return Ok(path);
        }
    }

    Err(anyhow!("No translation file found for locale '{}' in {}", locale, dir.display()))
}

/// Get the current user (from environment or git)
fn get_current_user() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_find_translation_file() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        // Create a test file
        fs::write(path.join("en.toml"), "[test]\nkey = \"value\"").unwrap();

        let result = find_translation_file(path, "en");
        assert!(result.is_ok());

        let result = find_translation_file(path, "de");
        assert!(result.is_err());
    }
}
