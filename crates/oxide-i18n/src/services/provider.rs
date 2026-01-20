//! Translation provider traits and implementations
//!
//! Provides a unified interface for different translation services.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Error type for provider operations
#[derive(Debug, Error)]
pub enum ProviderError {
    /// API error
    #[error("API error: {message} (status: {status})")]
    ApiError { status: u16, message: String },

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded, retry after {retry_after} seconds")]
    RateLimited { retry_after: u64 },

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    /// Unsupported language pair
    #[error("Unsupported language pair: {source_lang} -> {target_lang}")]
    UnsupportedLanguage { source_lang: String, target_lang: String },

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Other error
    #[error("{0}")]
    Other(String),
}

/// Result type for provider operations
pub type ProviderResult<T> = Result<T, ProviderError>;

/// A translation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationRequest {
    /// Text to translate
    pub text: String,
    /// Source language code
    pub source_lang: String,
    /// Target language code
    pub target_lang: String,
    /// Optional context for better translation
    pub context: Option<String>,
    /// Formality level (for supported providers)
    pub formality: Option<Formality>,
    /// Glossary ID (for supported providers)
    pub glossary_id: Option<String>,
}

/// Formality level for translations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Formality {
    /// Less formal
    Informal,
    /// Neutral (default)
    Neutral,
    /// More formal
    Formal,
}

/// A translation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResponse {
    /// Translated text
    pub translation: String,
    /// Detected source language (if auto-detected)
    pub detected_language: Option<String>,
    /// Provider-specific metadata
    pub metadata: HashMap<String, String>,
}

/// Configuration for a translation provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider name
    pub provider: String,
    /// API key
    #[serde(skip_serializing)]
    pub api_key: Option<String>,
    /// API endpoint URL
    pub endpoint: Option<String>,
    /// Additional options
    #[serde(default)]
    pub options: HashMap<String, String>,
}

impl ProviderConfig {
    /// Create a new config
    pub fn new(provider: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            api_key: None,
            endpoint: None,
            options: HashMap::new(),
        }
    }

    /// Set API key
    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Set endpoint
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    /// Add option
    pub fn with_option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }
}

/// Trait for translation providers
#[async_trait]
pub trait TranslationProvider: Send + Sync {
    /// Get provider name
    fn name(&self) -> &str;

    /// Get supported languages
    fn supported_languages(&self) -> Vec<String>;

    /// Translate a single text
    async fn translate(&self, request: TranslationRequest) -> ProviderResult<TranslationResponse>;

    /// Translate multiple texts (default: sequential calls)
    async fn translate_batch(
        &self,
        requests: Vec<TranslationRequest>,
    ) -> ProviderResult<Vec<TranslationResponse>> {
        let mut results = Vec::with_capacity(requests.len());
        for request in requests {
            results.push(self.translate(request).await?);
        }
        Ok(results)
    }

    /// Check if a language pair is supported
    fn supports_language_pair(&self, source: &str, target: &str) -> bool {
        let supported = self.supported_languages();
        supported.contains(&source.to_string()) && supported.contains(&target.to_string())
    }
}

// =============================================================================
// Mock Provider (for testing)
// =============================================================================

/// Mock translation provider for testing
pub struct MockProvider {
    /// Prefix to add to translations
    prefix: String,
    /// Simulated delay in ms
    delay_ms: u64,
}

impl MockProvider {
    /// Create a new mock provider
    pub fn new() -> Self {
        Self {
            prefix: String::new(),
            delay_ms: 0,
        }
    }

    /// Set a delay for simulating network latency
    pub fn with_delay(mut self, ms: u64) -> Self {
        self.delay_ms = ms;
        self
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TranslationProvider for MockProvider {
    fn name(&self) -> &str {
        "mock"
    }

    fn supported_languages(&self) -> Vec<String> {
        vec![
            "en".to_string(),
            "de".to_string(),
            "fr".to_string(),
            "es".to_string(),
            "it".to_string(),
            "pt".to_string(),
            "ja".to_string(),
            "zh".to_string(),
        ]
    }

    async fn translate(&self, request: TranslationRequest) -> ProviderResult<TranslationResponse> {
        if self.delay_ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(self.delay_ms)).await;
        }

        // Simple mock: prefix with target language
        let translation = format!("[{}] {}", request.target_lang, request.text);

        Ok(TranslationResponse {
            translation,
            detected_language: Some(request.source_lang),
            metadata: HashMap::new(),
        })
    }
}

// =============================================================================
// DeepL Provider
// =============================================================================

/// DeepL translation provider
#[cfg(feature = "services")]
pub struct DeepLProvider {
    /// API key
    api_key: String,
    /// API endpoint
    endpoint: String,
    /// HTTP client
    client: reqwest::Client,
}

#[cfg(feature = "services")]
impl DeepLProvider {
    /// Create a new DeepL provider
    pub fn new(api_key: impl Into<String>) -> Self {
        let api_key = api_key.into();
        // Use free API endpoint if key ends with :fx
        let endpoint = if api_key.ends_with(":fx") {
            "https://api-free.deepl.com/v2".to_string()
        } else {
            "https://api.deepl.com/v2".to_string()
        };

        Self {
            api_key,
            endpoint,
            client: reqwest::Client::new(),
        }
    }

    /// Create with custom endpoint
    pub fn with_endpoint(api_key: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            endpoint: endpoint.into(),
            client: reqwest::Client::new(),
        }
    }
}

#[cfg(feature = "services")]
#[async_trait]
impl TranslationProvider for DeepLProvider {
    fn name(&self) -> &str {
        "deepl"
    }

    fn supported_languages(&self) -> Vec<String> {
        vec![
            "bg", "cs", "da", "de", "el", "en", "es", "et", "fi", "fr",
            "hu", "id", "it", "ja", "ko", "lt", "lv", "nb", "nl", "pl",
            "pt", "ro", "ru", "sk", "sl", "sv", "tr", "uk", "zh",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }

    async fn translate(&self, request: TranslationRequest) -> ProviderResult<TranslationResponse> {
        let mut params = vec![
            ("text", request.text.clone()),
            ("target_lang", request.target_lang.to_uppercase()),
        ];

        if !request.source_lang.is_empty() && request.source_lang != "auto" {
            params.push(("source_lang", request.source_lang.to_uppercase()));
        }

        if let Some(formality) = request.formality {
            let formality_str = match formality {
                Formality::Informal => "less",
                Formality::Neutral => "default",
                Formality::Formal => "more",
            };
            params.push(("formality", formality_str.to_string()));
        }

        if let Some(glossary_id) = &request.glossary_id {
            params.push(("glossary_id", glossary_id.clone()));
        }

        let response = self
            .client
            .post(format!("{}/translate", self.endpoint))
            .header("Authorization", format!("DeepL-Auth-Key {}", self.api_key))
            .form(&params)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if response.status() == 429 {
            return Err(ProviderError::RateLimited { retry_after: 60 });
        }

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            return Err(ProviderError::ApiError { status, message });
        }

        #[derive(Deserialize)]
        struct DeepLResponse {
            translations: Vec<DeepLTranslation>,
        }

        #[derive(Deserialize)]
        struct DeepLTranslation {
            text: String,
            detected_source_language: Option<String>,
        }

        let body: DeepLResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::SerializationError(e.to_string()))?;

        let first = body.translations.into_iter().next().ok_or_else(|| {
            ProviderError::ApiError {
                status: 200,
                message: "No translation returned".to_string(),
            }
        })?;

        Ok(TranslationResponse {
            translation: first.text,
            detected_language: first.detected_source_language,
            metadata: HashMap::new(),
        })
    }
}

// Stub for non-feature builds
#[cfg(not(feature = "services"))]
pub struct DeepLProvider;

#[cfg(not(feature = "services"))]
impl DeepLProvider {
    pub fn new(_api_key: impl Into<String>) -> Self {
        Self
    }
}

#[cfg(not(feature = "services"))]
#[async_trait]
impl TranslationProvider for DeepLProvider {
    fn name(&self) -> &str {
        "deepl"
    }

    fn supported_languages(&self) -> Vec<String> {
        vec![]
    }

    async fn translate(&self, _request: TranslationRequest) -> ProviderResult<TranslationResponse> {
        Err(ProviderError::ConfigError(
            "DeepL provider requires 'services' feature".to_string(),
        ))
    }
}

// =============================================================================
// LibreTranslate Provider
// =============================================================================

/// LibreTranslate provider (self-hosted or public)
#[cfg(feature = "services")]
pub struct LibreTranslateProvider {
    /// API endpoint
    endpoint: String,
    /// API key (optional for some instances)
    api_key: Option<String>,
    /// HTTP client
    client: reqwest::Client,
}

#[cfg(feature = "services")]
impl LibreTranslateProvider {
    /// Create for a LibreTranslate instance
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            api_key: None,
            client: reqwest::Client::new(),
        }
    }

    /// Create with API key
    pub fn with_api_key(endpoint: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            api_key: Some(api_key.into()),
            client: reqwest::Client::new(),
        }
    }
}

#[cfg(feature = "services")]
#[async_trait]
impl TranslationProvider for LibreTranslateProvider {
    fn name(&self) -> &str {
        "libretranslate"
    }

    fn supported_languages(&self) -> Vec<String> {
        // Common languages supported by most LibreTranslate instances
        vec![
            "en", "de", "fr", "es", "it", "pt", "ru", "ar", "zh", "ja",
            "ko", "hi", "tr", "pl", "nl", "sv", "da", "fi", "no", "cs",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }

    async fn translate(&self, request: TranslationRequest) -> ProviderResult<TranslationResponse> {
        let mut body = serde_json::json!({
            "q": request.text,
            "source": request.source_lang,
            "target": request.target_lang,
        });

        if let Some(ref api_key) = self.api_key {
            body["api_key"] = serde_json::Value::String(api_key.clone());
        }

        let response = self
            .client
            .post(format!("{}/translate", self.endpoint))
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if response.status() == 429 {
            return Err(ProviderError::RateLimited { retry_after: 60 });
        }

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            return Err(ProviderError::ApiError { status, message });
        }

        #[derive(Deserialize)]
        struct LibreTranslateResponse {
            #[serde(rename = "translatedText")]
            translated_text: String,
            #[serde(rename = "detectedLanguage")]
            detected_language: Option<DetectedLanguage>,
        }

        #[derive(Deserialize)]
        struct DetectedLanguage {
            language: String,
        }

        let result: LibreTranslateResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::SerializationError(e.to_string()))?;

        Ok(TranslationResponse {
            translation: result.translated_text,
            detected_language: result.detected_language.map(|d| d.language),
            metadata: HashMap::new(),
        })
    }
}

// Stub for non-feature builds
#[cfg(not(feature = "services"))]
pub struct LibreTranslateProvider;

#[cfg(not(feature = "services"))]
impl LibreTranslateProvider {
    pub fn new(_endpoint: impl Into<String>) -> Self {
        Self
    }

    pub fn with_api_key(_endpoint: impl Into<String>, _api_key: impl Into<String>) -> Self {
        Self
    }
}

#[cfg(not(feature = "services"))]
#[async_trait]
impl TranslationProvider for LibreTranslateProvider {
    fn name(&self) -> &str {
        "libretranslate"
    }

    fn supported_languages(&self) -> Vec<String> {
        vec![]
    }

    async fn translate(&self, _request: TranslationRequest) -> ProviderResult<TranslationResponse> {
        Err(ProviderError::ConfigError(
            "LibreTranslate provider requires 'services' feature".to_string(),
        ))
    }
}

// We need to add async-trait to support async trait methods
// This is a workaround since we're using basic async traits
use std::future::Future;
use std::pin::Pin;

/// Async trait support (simplified version)
#[doc(hidden)]
pub mod async_trait {
    pub use async_trait::async_trait;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider_translate() {
        let provider = MockProvider::new();
        let request = TranslationRequest {
            text: "Hello".to_string(),
            source_lang: "en".to_string(),
            target_lang: "de".to_string(),
            context: None,
            formality: None,
            glossary_id: None,
        };

        let response = provider.translate(request).await.unwrap();
        assert_eq!(response.translation, "[de] Hello");
    }

    #[tokio::test]
    async fn test_mock_provider_batch() {
        let provider = MockProvider::new();
        let requests = vec![
            TranslationRequest {
                text: "Hello".to_string(),
                source_lang: "en".to_string(),
                target_lang: "de".to_string(),
                context: None,
                formality: None,
                glossary_id: None,
            },
            TranslationRequest {
                text: "World".to_string(),
                source_lang: "en".to_string(),
                target_lang: "de".to_string(),
                context: None,
                formality: None,
                glossary_id: None,
            },
        ];

        let responses = provider.translate_batch(requests).await.unwrap();
        assert_eq!(responses.len(), 2);
    }
}
