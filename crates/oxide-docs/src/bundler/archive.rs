//! Archive creation and extraction for documentation bundles

use crate::{DocsError, DocsResult};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};
use tracing::{debug, info};
use walkdir::WalkDir;

/// Create a compressed tar.gz archive from a documentation bundle
pub fn create_archive(source_dir: &Path, output_path: &Path) -> DocsResult<PathBuf> {
    let output_path = if output_path.extension().map_or(true, |ext| ext != "gz") {
        output_path.with_extension("tar.gz")
    } else {
        output_path.to_path_buf()
    };

    info!("Creating archive: {:?}", output_path);

    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = File::create(&output_path)?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut archive = Builder::new(encoder);

    // Add all files from source directory
    for entry in WalkDir::new(source_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let rel_path = path
            .strip_prefix(source_dir)
            .unwrap_or(path);

        if path.is_file() {
            debug!("Adding to archive: {:?}", rel_path);
            archive.append_path_with_name(path, rel_path)?;
        }
    }

    archive.finish()?;

    info!(
        "Archive created: {:?} ({} bytes)",
        output_path,
        std::fs::metadata(&output_path)?.len()
    );

    Ok(output_path)
}

/// Extract a compressed tar.gz archive to a directory
pub fn extract_archive(archive_path: &Path, extract_to: &Path) -> DocsResult<()> {
    info!("Extracting archive: {:?} to {:?}", archive_path, extract_to);

    if !archive_path.exists() {
        return Err(DocsError::BundleNotFound(archive_path.to_path_buf()));
    }

    // Create target directory
    std::fs::create_dir_all(extract_to)?;

    let file = File::open(archive_path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);

    // Extract with safety checks
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;

        // Security: prevent path traversal attacks
        let dest_path = extract_to.join(&path);
        if !dest_path.starts_with(extract_to) {
            return Err(DocsError::InvalidBundle(format!(
                "Invalid path in archive: {:?}",
                path
            )));
        }

        // Create parent directories
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        entry.unpack(&dest_path)?;
        debug!("Extracted: {:?}", dest_path);
    }

    info!("Archive extracted successfully");
    Ok(())
}

/// Verify archive integrity without extracting
pub fn verify_archive(archive_path: &Path) -> DocsResult<bool> {
    if !archive_path.exists() {
        return Ok(false);
    }

    let file = File::open(archive_path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);

    // Try to iterate through all entries
    let mut has_manifest = false;
    for entry in archive.entries()? {
        let entry = entry?;
        let path = entry.path()?;

        if path.file_name().map_or(false, |n| n == "manifest.json") {
            has_manifest = true;
        }
    }

    Ok(has_manifest)
}

/// Get archive info without extracting
pub struct ArchiveInfo {
    pub file_count: usize,
    pub total_size: u64,
    pub has_manifest: bool,
    pub compressed_size: u64,
}

pub fn get_archive_info(archive_path: &Path) -> DocsResult<ArchiveInfo> {
    let compressed_size = std::fs::metadata(archive_path)?.len();

    let file = File::open(archive_path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);

    let mut file_count = 0;
    let mut total_size = 0u64;
    let mut has_manifest = false;

    for entry in archive.entries()? {
        let entry = entry?;
        file_count += 1;
        total_size += entry.size();

        let path = entry.path()?;
        if path.file_name().map_or(false, |n| n == "manifest.json") {
            has_manifest = true;
        }
    }

    Ok(ArchiveInfo {
        file_count,
        total_size,
        has_manifest,
        compressed_size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_and_extract_archive() {
        let source_dir = TempDir::new().unwrap();
        let archive_dir = TempDir::new().unwrap();
        let extract_dir = TempDir::new().unwrap();

        // Create some test files
        std::fs::write(source_dir.path().join("test.txt"), "Hello, World!").unwrap();
        std::fs::create_dir(source_dir.path().join("subdir")).unwrap();
        std::fs::write(source_dir.path().join("subdir/nested.txt"), "Nested content").unwrap();

        // Create archive
        let archive_path = archive_dir.path().join("test.tar.gz");
        let result = create_archive(source_dir.path(), &archive_path);
        assert!(result.is_ok());

        // Extract archive
        let result = extract_archive(&archive_path, extract_dir.path());
        assert!(result.is_ok());

        // Verify contents
        assert!(extract_dir.path().join("test.txt").exists());
        assert!(extract_dir.path().join("subdir/nested.txt").exists());

        let content = std::fs::read_to_string(extract_dir.path().join("test.txt")).unwrap();
        assert_eq!(content, "Hello, World!");
    }
}
