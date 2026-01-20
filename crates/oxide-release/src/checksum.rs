//! Checksum calculation and verification

use crate::error::{ReleaseError, ReleaseResult};
use sha2::{Digest, Sha256, Sha512};
use std::io::Read;
use std::path::Path;

/// Checksum algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksumAlgorithm {
    /// SHA-256
    Sha256,
    /// SHA-512
    Sha512,
}

impl Default for ChecksumAlgorithm {
    fn default() -> Self {
        Self::Sha256
    }
}

/// A checksum value
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Checksum {
    /// The algorithm used
    pub algorithm: String,
    /// The hex-encoded hash value
    pub value: String,
}

impl Checksum {
    /// Create a new checksum
    pub fn new(algorithm: ChecksumAlgorithm, value: impl Into<String>) -> Self {
        Self {
            algorithm: match algorithm {
                ChecksumAlgorithm::Sha256 => "sha256".to_string(),
                ChecksumAlgorithm::Sha512 => "sha512".to_string(),
            },
            value: value.into(),
        }
    }

    /// Verify the checksum against a file
    pub fn verify(&self, path: &Path) -> ReleaseResult<bool> {
        let actual = match self.algorithm.as_str() {
            "sha256" => calculate_sha256(path)?,
            "sha512" => calculate_sha512(path)?,
            _ => {
                return Err(ReleaseError::config(format!(
                    "Unknown checksum algorithm: {}",
                    self.algorithm
                )))
            }
        };

        if actual == self.value {
            Ok(true)
        } else {
            Err(ReleaseError::ChecksumMismatch {
                expected: self.value.clone(),
                actual,
            })
        }
    }
}

impl std::fmt::Display for Checksum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.algorithm, self.value)
    }
}

/// Calculate SHA-256 checksum of a file
pub fn calculate_sha256(path: &Path) -> ReleaseResult<String> {
    calculate_hash::<Sha256>(path)
}

/// Calculate SHA-512 checksum of a file
pub fn calculate_sha512(path: &Path) -> ReleaseResult<String> {
    calculate_hash::<Sha512>(path)
}

/// Calculate hash using the specified digest algorithm
fn calculate_hash<D: Digest + Default>(path: &Path) -> ReleaseResult<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = D::default();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hex::encode(hasher.finalize()))
}

/// Calculate checksum of bytes
pub fn calculate_sha256_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::default();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Calculate checksum of a string
pub fn calculate_sha256_str(data: &str) -> String {
    calculate_sha256_bytes(data.as_bytes())
}

/// Generate a checksums file (SHA256SUMS format)
pub fn generate_checksums_file(files: &[(impl AsRef<Path>, impl AsRef<str>)]) -> ReleaseResult<String> {
    let mut content = String::new();

    for (path, name) in files {
        let checksum = calculate_sha256(path.as_ref())?;
        content.push_str(&format!("{}  {}\n", checksum, name.as_ref()));
    }

    Ok(content)
}

/// Verify a checksums file
pub fn verify_checksums_file(checksums_content: &str, base_path: &Path) -> ReleaseResult<Vec<(String, bool)>> {
    let mut results = Vec::new();

    for line in checksums_content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse "checksum  filename" format (two spaces)
        let parts: Vec<&str> = line.splitn(2, "  ").collect();
        if parts.len() != 2 {
            // Try single space
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() != 2 {
                continue;
            }
        }

        let expected = parts[0].trim();
        let filename = parts[1].trim();
        let file_path = base_path.join(filename);

        let ok = if file_path.exists() {
            match calculate_sha256(&file_path) {
                Ok(actual) => actual == expected,
                Err(_) => false,
            }
        } else {
            false
        };

        results.push((filename.to_string(), ok));
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_sha256_calculation() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"hello world").unwrap();

        let checksum = calculate_sha256(file.path()).unwrap();
        assert_eq!(
            checksum,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_sha256_bytes() {
        let checksum = calculate_sha256_bytes(b"hello world");
        assert_eq!(
            checksum,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_checksum_verify() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"hello world").unwrap();

        let checksum = Checksum::new(
            ChecksumAlgorithm::Sha256,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
        );

        assert!(checksum.verify(file.path()).unwrap());
    }

    #[test]
    fn test_checksum_verify_failure() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"hello world").unwrap();

        let checksum = Checksum::new(ChecksumAlgorithm::Sha256, "invalid");

        assert!(checksum.verify(file.path()).is_err());
    }
}
