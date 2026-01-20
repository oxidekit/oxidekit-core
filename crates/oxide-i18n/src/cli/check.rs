//! Check command implementation

use crate::error::{I18nError, I18nResult};
use crate::extractor::{ExtractionReport, KeyExtractor};
use crate::format::TranslationFile;
use crate::validation::{ValidationReport, Validator, ValidatorConfig};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Run the check command
#[allow(clippy::too_many_arguments)]
pub fn run(
    translations_dir: &str,
    source: Option<String>,
    base: &str,
    locale: Option<String>,
    strict: bool,
    format: &str,
    output: Option<String>,
    in_progress: Option<String>,
    skip_orphaned: bool,
    skip_placeholders: bool,
    skip_plurals: bool,
    length_threshold: Option<u32>,
) -> I18nResult<bool> {
    let trans_path = Path::new(translations_dir);

    if !trans_path.exists() {
        return Err(I18nError::FileNotFound {
            path: trans_path.to_path_buf(),
        });
    }

    println!("Checking translations in: {}", translations_dir);

    // Load all translation files
    let translations = load_translations(trans_path)?;

    if translations.is_empty() {
        println!("No translation files found in {}", translations_dir);
        return Ok(true);
    }

    println!("Found {} locale(s): {}", translations.len(), translations.keys().cloned().collect::<Vec<_>>().join(", "));

    // Extract keys from source
    let source_dir = source.as_deref().unwrap_or(".");
    println!("Extracting keys from: {}", source_dir);

    let extractor = KeyExtractor::new()?;
    let extraction_report = extractor.extract(source_dir)?;

    println!("Found {} unique keys in source", extraction_report.stats.unique_keys);

    // Configure validator
    let in_progress_locales: Vec<String> = in_progress
        .map(|s| s.split(',').map(|l| l.trim().to_string()).collect())
        .unwrap_or_default();

    let locales = locale.map(|l| vec![l]);

    let config = ValidatorConfig {
        base_locale: base.to_string(),
        locales,
        check_orphaned: !skip_orphaned,
        check_placeholders: !skip_placeholders,
        check_plurals: !skip_plurals,
        length_warning_threshold: length_threshold.map(|t| t as f32 / 100.0),
        in_progress_locales,
    };

    // Run validation
    let validator = Validator::with_config(config);
    let report = validator.validate(&translations, &extraction_report);

    // Output results
    let output_content = match format {
        "json" => serde_json::to_string_pretty(&report)?,
        _ => report.to_human_readable(),
    };

    if let Some(output_path) = output {
        fs::write(&output_path, &output_content)?;
        println!("Report saved to: {}", output_path);
    } else {
        println!();
        println!("{}", output_content);
    }

    // Determine exit status
    let passed = if strict {
        report.passed_strict
    } else {
        report.passed
    };

    if passed {
        println!("i18n check passed!");
    } else {
        println!("i18n check failed!");
        if strict {
            println!("(Strict mode: warnings are also treated as failures)");
        }
    }

    Ok(passed)
}

/// Load all translation files from a directory
fn load_translations(dir: &Path) -> I18nResult<HashMap<String, TranslationFile>> {
    let mut translations = HashMap::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map(|e| e == "toml").unwrap_or(false) {
            let file_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default();

            if !file_name.starts_with('_') && !file_name.is_empty() {
                let file = TranslationFile::load(&path)?;
                translations.insert(file_name.to_string(), file);
            }
        }
    }

    Ok(translations)
}

/// Run a quick status check (for `oxide i18n status`)
pub fn run_status(translations_dir: &str, detailed: bool) -> I18nResult<()> {
    let trans_path = Path::new(translations_dir);

    if !trans_path.exists() {
        return Err(I18nError::FileNotFound {
            path: trans_path.to_path_buf(),
        });
    }

    let translations = load_translations(trans_path)?;

    if translations.is_empty() {
        println!("No translation files found in {}", translations_dir);
        return Ok(());
    }

    println!("=== i18n Status ===");
    println!();

    // Find the locale with the most keys (likely the base)
    let base_locale = translations
        .iter()
        .max_by_key(|(_, f)| f.keys().len())
        .map(|(name, _)| name.as_str())
        .unwrap_or("en");

    let base_keys = translations
        .get(base_locale)
        .map(|f| f.keys().len())
        .unwrap_or(0);

    println!("Base locale: {} ({} keys)", base_locale, base_keys);
    println!();
    println!("Locales:");

    let mut locales: Vec<_> = translations.iter().collect();
    locales.sort_by_key(|(name, _)| name.as_str());

    for (name, file) in locales {
        let key_count = file.keys().len();
        let completion = if base_keys > 0 {
            (key_count as f32 / base_keys as f32) * 100.0
        } else {
            100.0
        };

        let status_icon = if completion >= 100.0 {
            "complete"
        } else if completion >= 80.0 {
            "mostly complete"
        } else if completion >= 50.0 {
            "partial"
        } else {
            "incomplete"
        };

        println!(
            "  {} - {} keys ({:.1}%) [{}]",
            name, key_count, completion, status_icon
        );

        if detailed && name != base_locale {
            // Show missing keys
            if let Some(base_file) = translations.get(base_locale) {
                let base_key_set: std::collections::HashSet<_> = base_file.keys().into_iter().collect();
                let locale_key_set: std::collections::HashSet<_> = file.keys().into_iter().collect();

                let missing: Vec<_> = base_key_set
                    .difference(&locale_key_set)
                    .take(10)
                    .collect();

                if !missing.is_empty() {
                    println!("    Missing keys (first 10):");
                    for key in &missing {
                        println!("      - {}", key);
                    }
                    let remaining = base_key_set.difference(&locale_key_set).count() - missing.len();
                    if remaining > 0 {
                        println!("      ... and {} more", remaining);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Initialize i18n for a project
pub fn run_init(
    translations_dir: &str,
    default_locale: &str,
    locales: Option<String>,
) -> I18nResult<()> {
    let trans_path = Path::new(translations_dir);

    // Create directory if it doesn't exist
    if !trans_path.exists() {
        fs::create_dir_all(trans_path)?;
        println!("Created directory: {}", translations_dir);
    }

    // Create default locale file
    let default_file = trans_path.join(format!("{}.toml", default_locale));

    if !default_file.exists() {
        let content = format!(
            r#"# {} translations
# Add your translations here

[_meta]
locale = "{}"
language_name = ""

# Example translations:
# [common]
# welcome = "Welcome!"
# loading = "Loading..."
"#,
            default_locale, default_locale
        );

        fs::write(&default_file, content)?;
        println!("Created: {}", default_file.display());
    } else {
        println!("Already exists: {}", default_file.display());
    }

    // Create additional locale files
    if let Some(locale_list) = locales {
        for locale in locale_list.split(',').map(|s| s.trim()) {
            if locale == default_locale {
                continue;
            }

            let locale_file = trans_path.join(format!("{}.toml", locale));

            if !locale_file.exists() {
                let content = format!(
                    r#"# {} translations
# Translate the strings from {} here

[_meta]
locale = "{}"
language_name = ""

# Copy keys from {}.toml and translate them here
"#,
                    locale, default_locale, locale, default_locale
                );

                fs::write(&locale_file, content)?;
                println!("Created: {}", locale_file.display());
            } else {
                println!("Already exists: {}", locale_file.display());
            }
        }
    }

    println!();
    println!("i18n initialized successfully!");
    println!();
    println!("Next steps:");
    println!("  1. Add your translations to {}/{}.toml", translations_dir, default_locale);
    println!("  2. Use t!(\"key.name\") in your Rust code");
    println!("  3. Use t(\"key.name\") in your .oui files");
    println!("  4. Run `oxide i18n extract` to find all keys");
    println!("  5. Run `oxide i18n check` to validate translations");

    Ok(())
}

/// Add a new locale
pub fn run_add(
    locale: &str,
    translations_dir: &str,
    copy_from: Option<String>,
) -> I18nResult<()> {
    let trans_path = Path::new(translations_dir);

    if !trans_path.exists() {
        return Err(I18nError::FileNotFound {
            path: trans_path.to_path_buf(),
        });
    }

    let locale_file = trans_path.join(format!("{}.toml", locale));

    if locale_file.exists() {
        println!("Locale file already exists: {}", locale_file.display());
        return Ok(());
    }

    let content = if let Some(source_locale) = copy_from {
        // Copy from existing locale
        let source_file = trans_path.join(format!("{}.toml", source_locale));

        if !source_file.exists() {
            return Err(I18nError::FileNotFound { path: source_file });
        }

        let mut content = fs::read_to_string(&source_file)?;

        // Update metadata
        content = content.replace(
            &format!("locale = \"{}\"", source_locale),
            &format!("locale = \"{}\"", locale),
        );

        // Add a comment at the top
        format!(
            "# {} translations (copied from {})\n# TODO: Translate all strings\n\n{}",
            locale, source_locale, content
        )
    } else {
        // Create empty template
        format!(
            r#"# {} translations

[_meta]
locale = "{}"
language_name = ""

# Add your translations here
# Copy keys from the base locale and translate them
"#,
            locale, locale
        )
    };

    fs::write(&locale_file, content)?;
    println!("Created: {}", locale_file.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_load_translations() {
        let dir = TempDir::new().unwrap();

        let en_content = r#"
[auth]
login = "Sign in"
"#;
        let mut en_file = fs::File::create(dir.path().join("en.toml")).unwrap();
        en_file.write_all(en_content.as_bytes()).unwrap();

        let translations = load_translations(dir.path()).unwrap();

        assert!(translations.contains_key("en"));
        assert_eq!(translations.len(), 1);
    }

    #[test]
    fn test_init() {
        let dir = TempDir::new().unwrap();
        let i18n_dir = dir.path().join("i18n");

        run_init(
            i18n_dir.to_str().unwrap(),
            "en",
            Some("nl,fr".to_string()),
        )
        .unwrap();

        assert!(i18n_dir.join("en.toml").exists());
        assert!(i18n_dir.join("nl.toml").exists());
        assert!(i18n_dir.join("fr.toml").exists());
    }
}
