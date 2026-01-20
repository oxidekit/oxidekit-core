//! Prometheus metrics exporter
//!
//! Feature-gated behind `prometheus` feature flag.

use crate::{Metric, MetricType, MetricValue, MetricsRegistry};
use std::collections::HashMap;
use std::net::SocketAddr;

/// Prometheus exporter configuration
#[derive(Debug, Clone)]
pub struct PrometheusConfig {
    /// Address to bind the metrics server to
    pub bind_address: SocketAddr,
    /// Path to expose metrics on (default: /metrics)
    pub metrics_path: String,
    /// Include help text in output
    pub include_help: bool,
    /// Include type annotations
    pub include_type: bool,
    /// Additional labels to add to all metrics
    pub extra_labels: HashMap<String, String>,
}

impl Default for PrometheusConfig {
    fn default() -> Self {
        Self {
            bind_address: ([127, 0, 0, 1], 9090).into(),
            metrics_path: "/metrics".to_string(),
            include_help: true,
            include_type: true,
            extra_labels: HashMap::new(),
        }
    }
}

/// Prometheus metrics exporter
pub struct PrometheusExporter {
    config: PrometheusConfig,
    registry: &'static MetricsRegistry,
}

impl PrometheusExporter {
    /// Create a new Prometheus exporter with default config
    pub fn new() -> Self {
        Self::with_config(PrometheusConfig::default())
    }

    /// Create a new Prometheus exporter with custom config
    pub fn with_config(config: PrometheusConfig) -> Self {
        Self {
            config,
            registry: MetricsRegistry::global(),
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &PrometheusConfig {
        &self.config
    }

    /// Generate Prometheus-formatted metrics output
    pub fn render(&self) -> String {
        let mut output = String::new();
        let metrics = self.registry.all_metrics();
        let extra_labels = &self.config.extra_labels;

        for metric in metrics {
            self.render_metric(&metric, extra_labels, &mut output);
        }

        output
    }

    /// Render a single metric
    fn render_metric(
        &self,
        metric: &Metric,
        extra_labels: &HashMap<String, String>,
        output: &mut String,
    ) {
        let name = sanitize_metric_name(&metric.metadata.name);

        // Add HELP comment
        if self.config.include_help && !metric.metadata.description.is_empty() {
            output.push_str(&format!(
                "# HELP {} {}\n",
                name, metric.metadata.description
            ));
        }

        // Add TYPE comment
        if self.config.include_type {
            let type_str = match metric.metadata.metric_type {
                MetricType::Counter => "counter",
                MetricType::Gauge => "gauge",
                MetricType::Histogram => "histogram",
                MetricType::Summary => "summary",
            };
            output.push_str(&format!("# TYPE {} {}\n", name, type_str));
        }

        // Build labels string
        let mut all_labels = metric.metadata.labels.clone();
        all_labels.extend(extra_labels.clone());
        let labels_str = format_labels(&all_labels);

        // Render the value
        match &metric.value {
            MetricValue::Counter(v) => {
                output.push_str(&format!("{}{} {}\n", name, labels_str, v));
            }
            MetricValue::Gauge(v) => {
                output.push_str(&format!("{}{} {}\n", name, labels_str, format_float(*v)));
            }
            MetricValue::Histogram(hist) => {
                // Render histogram buckets
                let mut cumulative = 0u64;
                for (i, &bound) in hist.buckets.iter().enumerate() {
                    cumulative += hist.counts[i];
                    let bucket_labels = format_labels_with_le(&all_labels, bound);
                    output.push_str(&format!(
                        "{}_bucket{} {}\n",
                        name, bucket_labels, cumulative
                    ));
                }
                // +Inf bucket
                cumulative += hist.counts.last().copied().unwrap_or(0);
                let inf_labels = format_labels_with_le(&all_labels, f64::INFINITY);
                output.push_str(&format!("{}_bucket{} {}\n", name, inf_labels, cumulative));

                // Sum and count
                output.push_str(&format!(
                    "{}_sum{} {}\n",
                    name,
                    labels_str,
                    format_float(hist.sum)
                ));
                output.push_str(&format!("{}_count{} {}\n", name, labels_str, hist.count));
            }
            MetricValue::Summary(summary) => {
                // Render quantiles
                for (q, v) in summary.quantiles.iter().zip(summary.values.iter()) {
                    let quantile_labels = format_labels_with_quantile(&all_labels, *q);
                    output.push_str(&format!("{}{} {}\n", name, quantile_labels, format_float(*v)));
                }
                output.push_str(&format!(
                    "{}_sum{} {}\n",
                    name,
                    labels_str,
                    format_float(summary.sum)
                ));
                output.push_str(&format!("{}_count{} {}\n", name, labels_str, summary.count));
            }
        }
    }

    /// Start the HTTP server to expose metrics
    #[cfg(feature = "prometheus")]
    pub async fn start_server(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use hyper::server::conn::http1;
        use hyper::service::service_fn;
        use hyper::{Request, Response, StatusCode};
        use hyper_util::rt::TokioIo;
        use std::convert::Infallible;
        use std::sync::Arc;
        use tokio::net::TcpListener;

        let config = Arc::new(self.config.clone());
        let registry = self.registry;

        let listener = TcpListener::bind(&self.config.bind_address).await?;
        tracing::info!(
            "Prometheus metrics server listening on {}",
            self.config.bind_address
        );

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let config = config.clone();

            tokio::spawn(async move {
                let service = service_fn(move |req: Request<hyper::body::Incoming>| {
                    let config = config.clone();
                    async move {
                        if req.uri().path() == config.metrics_path {
                            // Generate metrics
                            let exporter = PrometheusExporter {
                                config: (*config).clone(),
                                registry,
                            };
                            let body = exporter.render();

                            Ok::<_, Infallible>(
                                Response::builder()
                                    .status(StatusCode::OK)
                                    .header("Content-Type", "text/plain; version=0.0.4; charset=utf-8")
                                    .body(body)
                                    .unwrap(),
                            )
                        } else if req.uri().path() == "/health" {
                            Ok(Response::builder()
                                .status(StatusCode::OK)
                                .body("OK".to_string())
                                .unwrap())
                        } else {
                            Ok(Response::builder()
                                .status(StatusCode::NOT_FOUND)
                                .body("Not Found".to_string())
                                .unwrap())
                        }
                    }
                });

                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                    tracing::error!("Error serving connection: {:?}", err);
                }
            });
        }
    }

    /// Get the bind address
    pub fn bind_address(&self) -> SocketAddr {
        self.config.bind_address
    }

    /// Get the metrics URL
    pub fn metrics_url(&self) -> String {
        format!(
            "http://{}{}",
            self.config.bind_address, self.config.metrics_path
        )
    }
}

impl Default for PrometheusExporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Sanitize a metric name for Prometheus (alphanumeric and underscores only)
fn sanitize_metric_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect()
}

/// Format labels as Prometheus label string
fn format_labels(labels: &HashMap<String, String>) -> String {
    if labels.is_empty() {
        String::new()
    } else {
        let pairs: Vec<String> = labels
            .iter()
            .map(|(k, v)| format!("{}=\"{}\"", k, escape_label_value(v)))
            .collect();
        format!("{{{}}}", pairs.join(","))
    }
}

/// Format labels with an additional `le` (less than or equal) label for histograms
fn format_labels_with_le(labels: &HashMap<String, String>, le: f64) -> String {
    let mut all_labels = labels.clone();
    let le_str = if le.is_infinite() {
        "+Inf".to_string()
    } else {
        format_float(le)
    };
    all_labels.insert("le".to_string(), le_str);
    format_labels(&all_labels)
}

/// Format labels with an additional `quantile` label for summaries
fn format_labels_with_quantile(labels: &HashMap<String, String>, quantile: f64) -> String {
    let mut all_labels = labels.clone();
    all_labels.insert("quantile".to_string(), format_float(quantile));
    format_labels(&all_labels)
}

/// Escape special characters in label values
fn escape_label_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

/// Format a float for Prometheus output
fn format_float(v: f64) -> String {
    if v.is_nan() {
        "NaN".to_string()
    } else if v.is_infinite() {
        if v.is_sign_positive() {
            "+Inf".to_string()
        } else {
            "-Inf".to_string()
        }
    } else if v.fract() == 0.0 {
        format!("{:.1}", v) // Ensure at least one decimal place
    } else {
        format!("{}", v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_metric_name("oxide_frame_time"), "oxide_frame_time");
        assert_eq!(sanitize_metric_name("oxide-frame-time"), "oxide_frame_time");
        assert_eq!(sanitize_metric_name("oxide.frame.time"), "oxide_frame_time");
    }

    #[test]
    fn test_format_labels() {
        let mut labels = HashMap::new();
        labels.insert("app".to_string(), "oxidekit".to_string());
        labels.insert("env".to_string(), "dev".to_string());

        let result = format_labels(&labels);
        assert!(result.contains("app=\"oxidekit\""));
        assert!(result.contains("env=\"dev\""));
    }

    #[test]
    fn test_escape_label_value() {
        assert_eq!(escape_label_value("hello"), "hello");
        assert_eq!(escape_label_value("hello\"world"), "hello\\\"world");
        assert_eq!(escape_label_value("line\nbreak"), "line\\nbreak");
    }

    #[test]
    fn test_format_float() {
        assert_eq!(format_float(1.5), "1.5");
        assert_eq!(format_float(1.0), "1.0");
        assert_eq!(format_float(f64::NAN), "NaN");
        assert_eq!(format_float(f64::INFINITY), "+Inf");
        assert_eq!(format_float(f64::NEG_INFINITY), "-Inf");
    }

    #[test]
    fn test_render_metrics() {
        let exporter = PrometheusExporter::new();
        let output = exporter.render();

        // Should contain standard metrics
        assert!(output.contains("oxide_frame_fps"));
        assert!(output.contains("# TYPE"));
        assert!(output.contains("# HELP"));
    }
}
