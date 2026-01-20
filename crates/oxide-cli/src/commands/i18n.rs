//! `oxide i18n` commands - internationalization management

use anyhow::Result;
use oxide_i18n::cli::{check, extract};

/// Run the i18n extract command
pub fn run_extract(
    source: Option<String>,
    output: &str,
    extensions: &str,
    human_readable: bool,
    ai_schema: bool,
) -> Result<()> {
    extract::run(source, output, extensions, human_readable, ai_schema)?;
    Ok(())
}

/// Run the i18n check command
#[allow(clippy::too_many_arguments)]
pub fn run_check(
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
) -> Result<()> {
    let passed = check::run(
        translations_dir,
        source,
        base,
        locale,
        strict,
        format,
        output,
        in_progress,
        skip_orphaned,
        skip_placeholders,
        skip_plurals,
        length_threshold,
    )?;

    if !passed {
        std::process::exit(1);
    }

    Ok(())
}

/// Run the i18n init command
pub fn run_init(
    translations_dir: &str,
    default_locale: &str,
    locales: Option<String>,
) -> Result<()> {
    check::run_init(translations_dir, default_locale, locales)?;
    Ok(())
}

/// Run the i18n add command
pub fn run_add(
    locale: &str,
    translations_dir: &str,
    copy_from: Option<String>,
) -> Result<()> {
    check::run_add(locale, translations_dir, copy_from)?;
    Ok(())
}

/// Run the i18n status command
pub fn run_status(translations_dir: &str, detailed: bool) -> Result<()> {
    check::run_status(translations_dir, detailed)?;
    Ok(())
}
