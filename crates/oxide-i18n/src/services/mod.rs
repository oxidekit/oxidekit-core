//! Translation service integrations
//!
//! Optional integrations with external translation services:
//! - DeepL
//! - Google Translate
//! - LibreTranslate (self-hosted)
//! - Custom API endpoints

mod provider;

pub use provider::{
    TranslationProvider, TranslationRequest, TranslationResponse,
    ProviderConfig, ProviderError, ProviderResult,
    DeepLProvider, LibreTranslateProvider, MockProvider,
};

use crate::formats::{TranslationEntry, TranslationFile, TranslationState, TranslationValue};
use std::collections::HashMap;

/// Manager for translation services
pub struct TranslationService {
    /// Active provider
    provider: Box<dyn TranslationProvider>,
    /// Cache of translations
    cache: HashMap<String, String>,
    /// Enable caching
    cache_enabled: bool,
    /// Maximum concurrent requests
    max_concurrent: usize,
}

impl TranslationService {
    /// Create with a provider
    pub fn new(provider: Box<dyn TranslationProvider>) -> Self {
        Self {
            provider,
            cache: HashMap::new(),
            cache_enabled: true,
            max_concurrent: 5,
        }
    }

    /// Enable or disable caching
    pub fn set_cache_enabled(&mut self, enabled: bool) {
        self.cache_enabled = enabled;
    }

    /// Set max concurrent requests
    pub fn set_max_concurrent(&mut self, max: usize) {
        self.max_concurrent = max;
    }

    /// Translate a single text
    pub async fn translate(
        &mut self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> ProviderResult<String> {
        // Check cache
        if self.cache_enabled {
            let cache_key = format!("{}:{}:{}:{}", source_lang, target_lang, text, self.provider.name());
            if let Some(cached) = self.cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        let request = TranslationRequest {
            text: text.to_string(),
            source_lang: source_lang.to_string(),
            target_lang: target_lang.to_string(),
            context: None,
            formality: None,
            glossary_id: None,
        };

        let response = self.provider.translate(request).await?;

        // Cache result
        if self.cache_enabled {
            let cache_key = format!("{}:{}:{}:{}", source_lang, target_lang, text, self.provider.name());
            self.cache.insert(cache_key, response.translation.clone());
        }

        Ok(response.translation)
    }

    /// Translate multiple texts
    pub async fn translate_batch(
        &mut self,
        texts: &[&str],
        source_lang: &str,
        target_lang: &str,
    ) -> ProviderResult<Vec<String>> {
        let requests: Vec<TranslationRequest> = texts
            .iter()
            .map(|text| TranslationRequest {
                text: text.to_string(),
                source_lang: source_lang.to_string(),
                target_lang: target_lang.to_string(),
                context: None,
                formality: None,
                glossary_id: None,
            })
            .collect();

        let responses = self.provider.translate_batch(requests).await?;
        Ok(responses.into_iter().map(|r| r.translation).collect())
    }

    /// Pre-translate untranslated entries in a file
    pub async fn pre_translate(
        &mut self,
        file: &mut TranslationFile,
        mark_as_needs_review: bool,
    ) -> ProviderResult<usize> {
        let untranslated: Vec<(usize, String)> = file
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.target.is_none())
            .filter_map(|(i, e)| e.source.as_string().map(|s| (i, s.to_string())))
            .collect();

        if untranslated.is_empty() {
            return Ok(0);
        }

        let texts: Vec<&str> = untranslated.iter().map(|(_, s)| s.as_str()).collect();
        let translations = self
            .translate_batch(&texts, &file.source_locale, &file.target_locale)
            .await?;

        for ((idx, _), translation) in untranslated.iter().zip(translations.iter()) {
            file.entries[*idx].target = Some(TranslationValue::Simple(translation.clone()));
            if mark_as_needs_review {
                file.entries[*idx].state = TranslationState::NeedsReview;
            }
        }

        Ok(untranslated.len())
    }

    /// Get provider name
    pub fn provider_name(&self) -> &str {
        self.provider.name()
    }

    /// Check if provider supports a language pair
    pub fn supports_language(&self, lang: &str) -> bool {
        self.provider.supported_languages().contains(&lang.to_string())
    }

    /// Clear the translation cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider() {
        let provider = MockProvider::new();
        let mut service = TranslationService::new(Box::new(provider));

        let result = service.translate("Hello", "en", "de").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "[de] Hello");
    }
}
