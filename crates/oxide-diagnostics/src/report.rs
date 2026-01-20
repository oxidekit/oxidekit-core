//! Auto-Reporting
//!
//! Optional automatic reporting of diagnostics to a configured endpoint.
//! This feature requires explicit opt-in and user consent.

use crate::{DiagnosticsBundle, DiagnosticsConfig};
use serde::{Deserialize, Serialize};

/// Report status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportStatus {
    /// Report was sent successfully
    Sent { report_id: String },
    /// Report is pending (queued)
    Pending,
    /// Report failed to send
    Failed { error: String },
    /// Reporting is disabled
    Disabled,
    /// No consent given
    NoConsent,
}

/// Auto-reporter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    /// Whether auto-reporting is enabled
    pub enabled: bool,

    /// Endpoint URL
    pub endpoint: String,

    /// API key (optional)
    pub api_key: Option<String>,

    /// Retry count on failure
    pub retry_count: u32,

    /// Timeout in milliseconds
    pub timeout_ms: u64,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: String::new(),
            api_key: None,
            retry_count: 3,
            timeout_ms: 30000,
        }
    }
}

/// Auto-reporter for sending diagnostics
pub struct AutoReporter {
    config: ReportConfig,
    client: reqwest::Client,
}

impl AutoReporter {
    /// Create a new auto-reporter
    pub fn new(config: ReportConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { config, client }
    }

    /// Create from diagnostics config
    pub fn from_diagnostics_config(config: &DiagnosticsConfig) -> Option<Self> {
        if !config.auto_report {
            return None;
        }

        let endpoint = config.endpoint.clone()?;

        Some(Self::new(ReportConfig {
            enabled: true,
            endpoint,
            api_key: None,
            retry_count: 3,
            timeout_ms: 30000,
        }))
    }

    /// Check if reporting is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled && !self.config.endpoint.is_empty()
    }

    /// Send a diagnostics bundle
    pub async fn send_bundle(&self, bundle: &DiagnosticsBundle) -> ReportStatus {
        if !self.is_enabled() {
            return ReportStatus::Disabled;
        }

        // Check for consent
        if !bundle.metadata.auto_report_consent {
            return ReportStatus::NoConsent;
        }

        let mut last_error = String::new();

        for attempt in 0..=self.config.retry_count {
            match self.try_send(bundle).await {
                Ok(report_id) => {
                    return ReportStatus::Sent { report_id };
                }
                Err(e) => {
                    last_error = e.to_string();
                    tracing::warn!(
                        attempt = attempt + 1,
                        max_attempts = self.config.retry_count + 1,
                        error = %e,
                        "Failed to send diagnostics report, retrying..."
                    );

                    // Exponential backoff
                    if attempt < self.config.retry_count {
                        let delay = std::time::Duration::from_millis(100 * 2u64.pow(attempt));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        ReportStatus::Failed { error: last_error }
    }

    /// Try to send a bundle once
    async fn try_send(&self, bundle: &DiagnosticsBundle) -> Result<String, reqwest::Error> {
        let mut request = self.client.post(&self.config.endpoint).json(bundle);

        // Add API key header if configured
        if let Some(ref api_key) = self.config.api_key {
            request = request.header("X-API-Key", api_key);
        }

        let response = request.send().await?;

        // Check for success
        let response = response.error_for_status()?;

        // Try to get report ID from response
        #[derive(Deserialize)]
        struct ReportResponse {
            id: Option<String>,
        }

        let report_response: ReportResponse = response.json().await.unwrap_or(ReportResponse {
            id: Some(bundle.id.to_string()),
        });

        Ok(report_response.id.unwrap_or_else(|| bundle.id.to_string()))
    }

    /// Queue a bundle for later sending (non-blocking)
    pub fn queue_bundle(&self, bundle: DiagnosticsBundle) -> ReportStatus {
        if !self.is_enabled() {
            return ReportStatus::Disabled;
        }

        if !bundle.metadata.auto_report_consent {
            return ReportStatus::NoConsent;
        }

        // In a real implementation, this would queue to a background task
        // For now, we just mark as pending
        tracing::info!(
            bundle_id = %bundle.id,
            "Queued diagnostics bundle for sending"
        );

        ReportStatus::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_config_default() {
        let config = ReportConfig::default();
        assert!(!config.enabled);
        assert!(config.endpoint.is_empty());
    }

    #[test]
    fn test_reporter_disabled() {
        let reporter = AutoReporter::new(ReportConfig::default());
        assert!(!reporter.is_enabled());
    }

    #[test]
    fn test_reporter_enabled() {
        let config = ReportConfig {
            enabled: true,
            endpoint: "https://example.com/report".to_string(),
            api_key: None,
            retry_count: 3,
            timeout_ms: 30000,
        };

        let reporter = AutoReporter::new(config);
        assert!(reporter.is_enabled());
    }
}
