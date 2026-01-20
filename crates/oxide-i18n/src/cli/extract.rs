//! Extract command implementation

use crate::error::I18nResult;
use crate::extractor::{ExtractorConfig, KeyExtractor};
use std::fs;
use std::path::Path;

/// Run the extract command
pub fn run(
    source: Option<String>,
    output: &str,
    extensions: &str,
    human_readable: bool,
    ai_schema: bool,
) -> I18nResult<()> {
    let source_dir = source.as_deref().unwrap_or(".");

    println!("Extracting translation keys from: {}", source_dir);

    // Parse extensions
    let ext_list: Vec<String> = extensions.split(',').map(|s| s.trim().to_string()).collect();

    // Create extractor with config
    let config = ExtractorConfig {
        extensions: ext_list.clone(),
        ..Default::default()
    };

    let extractor = KeyExtractor::with_config(config)?;

    // Run extraction
    let report = extractor.extract(source_dir)?;

    // Print summary
    println!();
    println!("Extraction complete:");
    println!("  Files scanned: {}", report.stats.total_files);
    println!("    - .oui files: {}", report.stats.oui_files);
    println!("    - .rs files: {}", report.stats.rs_files);
    println!("  Unique keys: {}", report.stats.unique_keys);
    println!("  Total usages: {}", report.stats.total_usages);
    println!("  Plural keys: {}", report.stats.plural_keys);

    if !report.errors.is_empty() {
        println!();
        println!("Errors ({}):", report.errors.len());
        for err in &report.errors {
            println!("  {}: {}", err.file.display(), err.message);
        }
    }

    // Ensure output directory exists
    if let Some(parent) = Path::new(output).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    // Save JSON report
    report.save_json(output)?;
    println!();
    println!("Keys saved to: {}", output);

    // Generate human-readable output if requested
    if human_readable {
        let readable_path = output.replace(".json", ".txt");
        let content = report.generate_keys_file();
        fs::write(&readable_path, content)?;
        println!("Human-readable keys saved to: {}", readable_path);
    }

    // Generate AI schema if requested
    if ai_schema {
        let schema_path = output.replace(".json", ".ai.json");
        let schema = report.to_ai_schema();
        let content = serde_json::to_string_pretty(&schema)?;
        fs::write(&schema_path, content)?;
        println!("AI schema saved to: {}", schema_path);
    }

    println!();
    println!("Extracted keys:");
    for key in report.sorted_keys() {
        let info = report.keys.get(key).unwrap();
        let plural_marker = if info.is_plural { " [plural]" } else { "" };
        println!("  {}{}", key, plural_marker);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_extract_command() {
        let dir = TempDir::new().unwrap();

        // Create source file
        let rs_content = r#"
let text = t!("auth.login");
let text2 = t!("auth.welcome", name = "Test");
"#;
        let mut rs_file = fs::File::create(dir.path().join("test.rs")).unwrap();
        rs_file.write_all(rs_content.as_bytes()).unwrap();

        let output_path = dir.path().join("keys.json");

        run(
            Some(dir.path().to_string_lossy().to_string()),
            output_path.to_str().unwrap(),
            "rs",
            false,
            false,
        )
        .unwrap();

        assert!(output_path.exists());

        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("auth.login"));
        assert!(content.contains("auth.welcome"));
    }
}
