//! Core crypto primitives and shared types.
//!
//! This module provides fundamental building blocks for crypto operations:
//! - Hex, Base58, and Bech32 encoding/decoding
//! - Big integer handling
//! - Chain-agnostic transaction models
//! - Serialization helpers
//! - Redaction utilities for safe logging
//!
//! # Rules
//!
//! This module has strict rules:
//! - **No network access** - Pure computational functions only
//! - **No key storage** - Keys are handled by the `keys` module
//! - **No signing** - Signing is handled by chain-specific modules

use crate::{CryptoError, CryptoResult};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

// ============================================================================
// Hex Encoding
// ============================================================================

/// A hex-encoded string with validation.
///
/// Ensures the string contains only valid hexadecimal characters.
/// Optionally includes "0x" prefix for Ethereum compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HexString {
    inner: String,
}

impl HexString {
    /// Create a new HexString from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            inner: bytes.iter().map(|b| format!("{:02x}", b)).collect(),
        }
    }

    /// Create a new HexString with "0x" prefix (Ethereum style).
    pub fn from_bytes_prefixed(bytes: &[u8]) -> Self {
        Self {
            inner: format!("0x{}", bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>()),
        }
    }

    /// Parse a hex string, with or without "0x" prefix.
    pub fn parse(s: &str) -> CryptoResult<Self> {
        let s = s.trim();
        let hex_str = s.strip_prefix("0x").unwrap_or(s);

        // Validate hex characters
        if !hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(CryptoError::InvalidHex(
                "contains non-hexadecimal characters".to_string(),
            ));
        }

        if hex_str.len() % 2 != 0 {
            return Err(CryptoError::InvalidHex(
                "odd number of characters".to_string(),
            ));
        }

        Ok(Self {
            inner: hex_str.to_lowercase(),
        })
    }

    /// Convert to bytes.
    pub fn to_bytes(&self) -> CryptoResult<Vec<u8>> {
        let hex_str = self.inner.strip_prefix("0x").unwrap_or(&self.inner);

        (0..hex_str.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&hex_str[i..i + 2], 16)
                    .map_err(|_| CryptoError::InvalidHex("decode failed".to_string()))
            })
            .collect()
    }

    /// Get the hex string without prefix.
    pub fn as_str(&self) -> &str {
        self.inner.strip_prefix("0x").unwrap_or(&self.inner)
    }

    /// Get the hex string with "0x" prefix.
    pub fn as_prefixed(&self) -> String {
        if self.inner.starts_with("0x") {
            self.inner.clone()
        } else {
            format!("0x{}", self.inner)
        }
    }

    /// Get the length in bytes.
    pub fn byte_len(&self) -> usize {
        self.as_str().len() / 2
    }

    /// Check if the hex string is empty.
    pub fn is_empty(&self) -> bool {
        self.as_str().is_empty()
    }
}

impl fmt::Display for HexString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl FromStr for HexString {
    type Err = CryptoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

// ============================================================================
// Base58 Encoding
// ============================================================================

/// A Base58-encoded string (Bitcoin style).
///
/// Base58 uses an alphabet that avoids visually similar characters
/// (0, O, I, l) to reduce transcription errors.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Base58String {
    inner: String,
}

/// The Base58 alphabet (Bitcoin).
const BASE58_ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

impl Base58String {
    /// Create a new Base58String from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        if bytes.is_empty() {
            return Self { inner: String::new() };
        }

        // Count leading zeros
        let zeros = bytes.iter().take_while(|&&b| b == 0).count();

        // Convert bytes to base58
        let mut digits: Vec<u8> = vec![0; bytes.len() * 2];
        let mut digit_len = 1;

        for &byte in bytes.iter() {
            let mut carry = byte as u32;
            for digit in digits.iter_mut().take(digit_len) {
                carry += (*digit as u32) << 8;
                *digit = (carry % 58) as u8;
                carry /= 58;
            }
            while carry > 0 {
                digits[digit_len] = (carry % 58) as u8;
                digit_len += 1;
                carry /= 58;
            }
        }

        // Build result string
        let mut result = String::with_capacity(zeros + digit_len);

        // Add leading '1's for each leading zero byte
        for _ in 0..zeros {
            result.push('1');
        }

        // Add the encoded digits in reverse
        for &digit in digits[..digit_len].iter().rev() {
            result.push(BASE58_ALPHABET[digit as usize] as char);
        }

        Self { inner: result }
    }

    /// Parse a Base58 string.
    pub fn parse(s: &str) -> CryptoResult<Self> {
        let s = s.trim();

        // Validate characters
        for c in s.chars() {
            if !BASE58_ALPHABET.contains(&(c as u8)) {
                return Err(CryptoError::InvalidBase58(format!(
                    "invalid character: '{}'",
                    c
                )));
            }
        }

        Ok(Self { inner: s.to_string() })
    }

    /// Convert to bytes.
    pub fn to_bytes(&self) -> CryptoResult<Vec<u8>> {
        if self.inner.is_empty() {
            return Ok(vec![]);
        }

        // Count leading '1's (zeros in output)
        let zeros = self.inner.chars().take_while(|&c| c == '1').count();

        // Decode base58
        let mut bytes: Vec<u8> = vec![0; self.inner.len()];
        let mut byte_len = 1;

        for c in self.inner.chars() {
            let idx = BASE58_ALPHABET
                .iter()
                .position(|&x| x == c as u8)
                .ok_or_else(|| CryptoError::InvalidBase58("invalid character".to_string()))?;

            let mut carry = idx;
            for byte in bytes.iter_mut().take(byte_len) {
                carry += (*byte as usize) * 58;
                *byte = (carry & 0xff) as u8;
                carry >>= 8;
            }
            while carry > 0 {
                bytes[byte_len] = (carry & 0xff) as u8;
                byte_len += 1;
                carry >>= 8;
            }
        }

        // Build result with leading zeros
        let mut result = vec![0u8; zeros];
        result.extend(bytes[..byte_len].iter().rev());

        Ok(result)
    }

    /// Get the base58 string.
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl fmt::Display for Base58String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl FromStr for Base58String {
    type Err = CryptoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

// ============================================================================
// Bech32 Encoding
// ============================================================================

/// A Bech32-encoded string (SegWit addresses, etc.).
///
/// Bech32 is a checksummed base-32 encoding format used for
/// Bitcoin SegWit addresses and other blockchain applications.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Bech32String {
    /// Human-readable part (e.g., "bc" for Bitcoin mainnet)
    pub hrp: String,
    /// The data part
    pub data: Vec<u8>,
    /// The variant (bech32 or bech32m)
    pub variant: Bech32Variant,
}

/// Bech32 encoding variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Bech32Variant {
    /// Original bech32 (BIP-173)
    Bech32,
    /// Modified bech32m (BIP-350)
    Bech32m,
}

/// The Bech32 alphabet.
const BECH32_ALPHABET: &[u8] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

impl Bech32String {
    /// Create a new Bech32String.
    pub fn new(hrp: &str, data: Vec<u8>, variant: Bech32Variant) -> CryptoResult<Self> {
        // Validate HRP
        if hrp.is_empty() || hrp.len() > 83 {
            return Err(CryptoError::InvalidBech32("invalid HRP length".to_string()));
        }

        for c in hrp.chars() {
            if !c.is_ascii() || c.is_ascii_uppercase() {
                return Err(CryptoError::InvalidBech32(
                    "HRP must be lowercase ASCII".to_string(),
                ));
            }
        }

        Ok(Self {
            hrp: hrp.to_string(),
            data,
            variant,
        })
    }

    /// Parse a Bech32 string.
    pub fn parse(s: &str) -> CryptoResult<Self> {
        let s = s.trim().to_lowercase();

        // Find separator
        let sep_pos = s
            .rfind('1')
            .ok_or_else(|| CryptoError::InvalidBech32("no separator found".to_string()))?;

        if sep_pos == 0 {
            return Err(CryptoError::InvalidBech32("empty HRP".to_string()));
        }

        let hrp = &s[..sep_pos];
        let data_part = &s[sep_pos + 1..];

        if data_part.len() < 6 {
            return Err(CryptoError::InvalidBech32("data too short".to_string()));
        }

        // Decode data
        let mut data = Vec::with_capacity(data_part.len() - 6);
        for c in data_part[..data_part.len() - 6].chars() {
            let idx = BECH32_ALPHABET
                .iter()
                .position(|&x| x == c as u8)
                .ok_or_else(|| CryptoError::InvalidBech32("invalid character".to_string()))?;
            data.push(idx as u8);
        }

        // TODO: Verify checksum and determine variant
        // For now, assume bech32m
        Ok(Self {
            hrp: hrp.to_string(),
            data,
            variant: Bech32Variant::Bech32m,
        })
    }

    /// Encode to string.
    pub fn encode(&self) -> String {
        let mut result = self.hrp.clone();
        result.push('1');

        for &byte in &self.data {
            result.push(BECH32_ALPHABET[byte as usize] as char);
        }

        // TODO: Add checksum
        result
    }

    /// Get the human-readable part.
    pub fn hrp(&self) -> &str {
        &self.hrp
    }

    /// Get the data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

impl fmt::Display for Bech32String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.encode())
    }
}

// ============================================================================
// Big Integer
// ============================================================================

/// A big unsigned integer for crypto operations.
///
/// Wraps a byte array to represent arbitrary-precision integers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BigUint {
    /// Big-endian bytes
    bytes: Vec<u8>,
}

impl BigUint {
    /// Create from bytes (big-endian).
    pub fn from_bytes_be(bytes: &[u8]) -> Self {
        // Strip leading zeros
        let start = bytes.iter().position(|&b| b != 0).unwrap_or(bytes.len());
        Self {
            bytes: bytes[start..].to_vec(),
        }
    }

    /// Create from a u64.
    pub fn from_u64(value: u64) -> Self {
        Self::from_bytes_be(&value.to_be_bytes())
    }

    /// Create from a u128.
    pub fn from_u128(value: u128) -> Self {
        Self::from_bytes_be(&value.to_be_bytes())
    }

    /// Parse from decimal string.
    pub fn from_dec_str(s: &str) -> CryptoResult<Self> {
        let s = s.trim();
        if !s.chars().all(|c| c.is_ascii_digit()) {
            return Err(CryptoError::InvalidHex(
                "invalid decimal string".to_string(),
            ));
        }

        // Simple decimal parsing
        let mut result = vec![0u8];
        for c in s.chars() {
            let digit = (c as u8) - b'0';

            // Multiply by 10
            let mut carry = 0u16;
            for byte in result.iter_mut().rev() {
                let val = (*byte as u16) * 10 + carry;
                *byte = (val & 0xff) as u8;
                carry = val >> 8;
            }
            while carry > 0 {
                result.insert(0, (carry & 0xff) as u8);
                carry >>= 8;
            }

            // Add digit
            let mut carry = digit as u16;
            for byte in result.iter_mut().rev() {
                let val = (*byte as u16) + carry;
                *byte = (val & 0xff) as u8;
                carry = val >> 8;
            }
            while carry > 0 {
                result.insert(0, (carry & 0xff) as u8);
                carry >>= 8;
            }
        }

        Ok(Self::from_bytes_be(&result))
    }

    /// Parse from hex string.
    pub fn from_hex_str(s: &str) -> CryptoResult<Self> {
        let hex = HexString::parse(s)?;
        Ok(Self::from_bytes_be(&hex.to_bytes()?))
    }

    /// Get bytes (big-endian).
    pub fn to_bytes_be(&self) -> Vec<u8> {
        if self.bytes.is_empty() {
            vec![0]
        } else {
            self.bytes.clone()
        }
    }

    /// Get bytes with specific length (zero-padded).
    pub fn to_bytes_be_padded(&self, len: usize) -> CryptoResult<Vec<u8>> {
        let bytes = self.to_bytes_be();
        if bytes.len() > len {
            return Err(CryptoError::InvalidLength {
                expected: len,
                actual: bytes.len(),
            });
        }

        let mut result = vec![0u8; len];
        result[len - bytes.len()..].copy_from_slice(&bytes);
        Ok(result)
    }

    /// Convert to u64 if it fits.
    pub fn to_u64(&self) -> Option<u64> {
        if self.bytes.len() > 8 {
            return None;
        }

        let mut result = 0u64;
        for &byte in &self.bytes {
            result = (result << 8) | (byte as u64);
        }
        Some(result)
    }

    /// Check if zero.
    pub fn is_zero(&self) -> bool {
        self.bytes.is_empty() || self.bytes.iter().all(|&b| b == 0)
    }

    /// Get hex representation.
    pub fn to_hex(&self) -> HexString {
        HexString::from_bytes(&self.to_bytes_be())
    }
}

impl fmt::Display for BigUint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", self.to_hex())
    }
}

impl Default for BigUint {
    fn default() -> Self {
        Self { bytes: vec![] }
    }
}

// ============================================================================
// Encoding Trait
// ============================================================================

/// Trait for types that can be encoded/decoded.
pub trait Encoding: Sized {
    /// Encode to bytes.
    fn encode(&self) -> Vec<u8>;

    /// Decode from bytes.
    fn decode(bytes: &[u8]) -> CryptoResult<Self>;

    /// Encode to hex string.
    fn to_hex(&self) -> HexString {
        HexString::from_bytes(&self.encode())
    }

    /// Decode from hex string.
    fn from_hex(hex: &str) -> CryptoResult<Self> {
        let bytes = HexString::parse(hex)?.to_bytes()?;
        Self::decode(&bytes)
    }
}

// ============================================================================
// Redaction
// ============================================================================

/// A string that redacts its content in Display/Debug.
///
/// Useful for logging sensitive data without exposing it.
#[derive(Clone, PartialEq, Eq)]
pub struct RedactedString {
    value: String,
    redacted: String,
}

impl Serialize for RedactedString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize only the value, not the redacted representation
        serializer.serialize_str(&self.value)
    }
}

impl<'de> Deserialize<'de> for RedactedString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Self::new(value))
    }
}

impl RedactedString {
    /// Create a new redacted string.
    pub fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        let redacted = if value.is_empty() {
            "[empty]".to_string()
        } else if value.len() <= 8 {
            "[redacted]".to_string()
        } else {
            format!(
                "{}...{}",
                &value[..4],
                &value[value.len() - 4..]
            )
        };

        Self { value, redacted }
    }

    /// Get the actual value.
    ///
    /// # Security
    ///
    /// Only use this when you actually need the value.
    /// Never log the result.
    pub fn reveal(&self) -> &str {
        &self.value
    }

    /// Get the redacted representation.
    pub fn redacted(&self) -> &str {
        &self.redacted
    }
}

impl fmt::Display for RedactedString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.redacted)
    }
}

impl fmt::Debug for RedactedString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RedactedString({})", self.redacted)
    }
}

impl<T: Into<String>> From<T> for RedactedString {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl Default for RedactedString {
    fn default() -> Self {
        Self {
            value: String::new(),
            redacted: "[empty]".to_string(),
        }
    }
}

/// Redact a byte slice for logging.
pub fn redact_bytes(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        "[empty]".to_string()
    } else if bytes.len() <= 8 {
        format!("[{} bytes]", bytes.len())
    } else {
        format!(
            "{:02x}{:02x}...{:02x}{:02x} ({} bytes)",
            bytes[0],
            bytes[1],
            bytes[bytes.len() - 2],
            bytes[bytes.len() - 1],
            bytes.len()
        )
    }
}

// ============================================================================
// Common Types
// ============================================================================

/// A 32-byte hash (e.g., SHA256, Keccak256).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Hash32(#[serde(with = "hex_bytes")] pub [u8; 32]);

impl Hash32 {
    /// Create from bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Parse from hex string.
    pub fn from_hex(s: &str) -> CryptoResult<Self> {
        let bytes = HexString::parse(s)?.to_bytes()?;
        if bytes.len() != 32 {
            return Err(CryptoError::InvalidLength {
                expected: 32,
                actual: bytes.len(),
            });
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }

    /// Get as bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert to hex string.
    pub fn to_hex(&self) -> HexString {
        HexString::from_bytes(&self.0)
    }

    /// Zero hash.
    pub fn zero() -> Self {
        Self([0u8; 32])
    }
}

impl fmt::Display for Hash32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Serde helper for hex-encoded bytes.
mod hex_bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8; 32], s: S) -> Result<S::Ok, S::Error> {
        let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        s.serialize_str(&hex)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 32], D::Error> {
        let s = String::deserialize(d)?;
        let s = s.strip_prefix("0x").unwrap_or(&s);

        if s.len() != 64 {
            return Err(serde::de::Error::custom("expected 64 hex characters"));
        }

        let mut result = [0u8; 32];
        for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
            let s = std::str::from_utf8(chunk).map_err(serde::de::Error::custom)?;
            result[i] = u8::from_str_radix(s, 16).map_err(serde::de::Error::custom)?;
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_string_roundtrip() {
        let bytes = vec![0xde, 0xad, 0xbe, 0xef];
        let hex = HexString::from_bytes(&bytes);
        assert_eq!(hex.as_str(), "deadbeef");
        assert_eq!(hex.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn test_hex_string_prefixed() {
        let hex = HexString::parse("0xDEADBEEF").unwrap();
        assert_eq!(hex.as_str(), "deadbeef");
        assert_eq!(hex.as_prefixed(), "0xdeadbeef");
    }

    #[test]
    fn test_hex_string_invalid() {
        assert!(HexString::parse("not hex").is_err());
        assert!(HexString::parse("123").is_err()); // Odd length
    }

    #[test]
    fn test_base58_roundtrip() {
        let bytes = vec![0, 0, 0xde, 0xad, 0xbe, 0xef];
        let b58 = Base58String::from_bytes(&bytes);
        let decoded = b58.to_bytes().unwrap();
        assert_eq!(decoded, bytes);
    }

    #[test]
    fn test_bech32_parse() {
        let b32 = Bech32String::parse("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4").unwrap();
        assert_eq!(b32.hrp(), "bc");
    }

    #[test]
    fn test_big_uint_from_u64() {
        let num = BigUint::from_u64(0xdeadbeef);
        assert_eq!(num.to_u64(), Some(0xdeadbeef));
    }

    #[test]
    fn test_big_uint_from_dec() {
        let num = BigUint::from_dec_str("1000000000000000000").unwrap();
        assert!(!num.is_zero());
    }

    #[test]
    fn test_redacted_string() {
        let secret = RedactedString::new("my_super_secret_key");
        assert!(!secret.to_string().contains("super"));
        assert_eq!(secret.reveal(), "my_super_secret_key");
    }

    #[test]
    fn test_redact_bytes() {
        let bytes = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let redacted = redact_bytes(&bytes);
        assert!(redacted.contains("10 bytes"));
        assert!(!redacted.contains("05"));
    }

    #[test]
    fn test_hash32() {
        let hash = Hash32::from_hex(
            "0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
        ).unwrap();
        assert_eq!(hash.as_bytes()[0], 0xde);
    }
}
