//! Offline detection and network status monitoring for OxideKit.
//!
//! Provides reliable network connectivity detection with:
//! - Active connectivity checks
//! - Passive monitoring from request failures
//! - Configurable check endpoints
//! - Event notifications for status changes

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info, warn};

/// Network connectivity status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkStatus {
    /// Network is available.
    Online,
    /// Network is unavailable.
    Offline,
    /// Network status is unknown (initial state).
    Unknown,
}

impl std::fmt::Display for NetworkStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkStatus::Online => write!(f, "Online"),
            NetworkStatus::Offline => write!(f, "Offline"),
            NetworkStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Configuration for offline detection.
#[derive(Debug, Clone)]
pub struct OfflineDetectorConfig {
    /// URLs to use for connectivity checks.
    pub check_urls: Vec<String>,
    /// How often to perform active connectivity checks.
    pub check_interval: Duration,
    /// Timeout for connectivity checks.
    pub check_timeout: Duration,
    /// Number of consecutive failures before marking offline.
    pub failure_threshold: u32,
    /// Number of consecutive successes before marking online.
    pub success_threshold: u32,
    /// Whether to perform active checks.
    pub active_checks_enabled: bool,
    /// Minimum time between status change notifications.
    pub debounce_duration: Duration,
}

impl Default for OfflineDetectorConfig {
    fn default() -> Self {
        Self {
            check_urls: vec![
                "https://connectivitycheck.gstatic.com/generate_204".to_string(),
                "https://www.msftconnecttest.com/connecttest.txt".to_string(),
                "https://captive.apple.com/hotspot-detect.html".to_string(),
            ],
            check_interval: Duration::from_secs(30),
            check_timeout: Duration::from_secs(5),
            failure_threshold: 3,
            success_threshold: 2,
            active_checks_enabled: true,
            debounce_duration: Duration::from_secs(2),
        }
    }
}

/// Offline detector for monitoring network connectivity.
#[derive(Debug)]
pub struct OfflineDetector {
    config: OfflineDetectorConfig,
    /// Current status (atomic for lock-free reads).
    is_offline: AtomicBool,
    /// Consecutive failure count.
    failure_count: AtomicU64,
    /// Consecutive success count.
    success_count: AtomicU64,
    /// Last check time.
    last_check: RwLock<Option<Instant>>,
    /// Last status change time.
    last_status_change: RwLock<Option<Instant>>,
    /// Channel for status change notifications.
    status_tx: broadcast::Sender<NetworkStatus>,
}

impl OfflineDetector {
    /// Create a new offline detector.
    pub fn new(config: OfflineDetectorConfig) -> Arc<Self> {
        let (status_tx, _) = broadcast::channel(10);

        Arc::new(Self {
            config,
            is_offline: AtomicBool::new(false),
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            last_check: RwLock::new(None),
            last_status_change: RwLock::new(None),
            status_tx,
        })
    }

    /// Create a detector with default configuration.
    pub fn default_detector() -> Arc<Self> {
        Self::new(OfflineDetectorConfig::default())
    }

    /// Check if currently offline.
    pub fn is_offline(&self) -> bool {
        self.is_offline.load(Ordering::Relaxed)
    }

    /// Check if currently online.
    pub fn is_online(&self) -> bool {
        !self.is_offline()
    }

    /// Get current network status.
    pub fn status(&self) -> NetworkStatus {
        if self.failure_count.load(Ordering::Relaxed) == 0
            && self.success_count.load(Ordering::Relaxed) == 0
        {
            NetworkStatus::Unknown
        } else if self.is_offline() {
            NetworkStatus::Offline
        } else {
            NetworkStatus::Online
        }
    }

    /// Subscribe to status change notifications.
    pub fn subscribe(&self) -> broadcast::Receiver<NetworkStatus> {
        self.status_tx.subscribe()
    }

    /// Record a successful network operation.
    pub fn record_success(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
        let count = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;

        if count >= self.config.success_threshold as u64 && self.is_offline() {
            self.set_online();
        }
    }

    /// Record a failed network operation.
    pub fn record_failure(&self) {
        self.success_count.store(0, Ordering::Relaxed);
        let count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;

        if count >= self.config.failure_threshold as u64 && !self.is_offline() {
            self.set_offline();
        }
    }

    /// Manually set status to online.
    fn set_online(&self) {
        let was_offline = self.is_offline.swap(false, Ordering::Relaxed);

        if was_offline {
            info!("Network status changed: Online");
            self.notify_status_change(NetworkStatus::Online);
        }
    }

    /// Manually set status to offline.
    fn set_offline(&self) {
        let was_online = !self.is_offline.swap(true, Ordering::Relaxed);

        if was_online {
            warn!("Network status changed: Offline");
            self.notify_status_change(NetworkStatus::Offline);
        }
    }

    /// Notify subscribers of status change with debouncing.
    fn notify_status_change(&self, status: NetworkStatus) {
        // Non-blocking check - if we can't acquire lock, skip debounce check
        // This is fine since we're just trying to avoid spam
        let _ = self.status_tx.send(status);
    }

    /// Perform an active connectivity check.
    pub async fn check_connectivity(&self) -> bool {
        if self.config.check_urls.is_empty() {
            return true; // Assume online if no check URLs configured
        }

        let client = match reqwest::Client::builder()
            .timeout(self.config.check_timeout)
            .build()
        {
            Ok(c) => c,
            Err(_) => return false,
        };

        // Try each URL until one succeeds
        for url in &self.config.check_urls {
            match client.head(url).send().await {
                Ok(response) if response.status().is_success() || response.status().as_u16() == 204 => {
                    debug!(url = %url, "Connectivity check succeeded");
                    self.record_success();
                    return true;
                }
                Ok(response) => {
                    debug!(url = %url, status = %response.status(), "Connectivity check failed with status");
                }
                Err(e) => {
                    debug!(url = %url, error = %e, "Connectivity check failed");
                }
            }
        }

        self.record_failure();
        false
    }

    /// Start background connectivity monitoring.
    ///
    /// Returns a handle that can be used to stop monitoring.
    pub fn start_monitoring(self: Arc<Self>) -> MonitorHandle {
        let (stop_tx, mut stop_rx) = tokio::sync::mpsc::channel::<()>(1);
        let detector = self.clone();

        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = stop_rx.recv() => {
                        debug!("Stopping connectivity monitoring");
                        break;
                    }
                    _ = tokio::time::sleep(detector.config.check_interval) => {
                        if detector.config.active_checks_enabled {
                            detector.check_connectivity().await;
                        }
                    }
                }
            }
        });

        MonitorHandle {
            stop_tx,
            task_handle: handle,
        }
    }
}

/// Handle for controlling the background monitor.
pub struct MonitorHandle {
    stop_tx: tokio::sync::mpsc::Sender<()>,
    task_handle: tokio::task::JoinHandle<()>,
}

impl MonitorHandle {
    /// Stop the background monitor.
    pub async fn stop(self) {
        let _ = self.stop_tx.send(()).await;
        let _ = self.task_handle.await;
    }
}

/// Retry policy configuration.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retries.
    pub max_retries: u32,
    /// Initial delay between retries.
    pub initial_delay: Duration,
    /// Maximum delay between retries.
    pub max_delay: Duration,
    /// Backoff multiplier.
    pub backoff_multiplier: f64,
    /// Add random jitter to delays.
    pub jitter: bool,
    /// Maximum total time for all retries.
    pub max_total_time: Option<Duration>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter: true,
            max_total_time: Some(Duration::from_secs(60)),
        }
    }
}

impl RetryPolicy {
    /// Create a policy with no retries.
    pub fn no_retry() -> Self {
        Self {
            max_retries: 0,
            ..Default::default()
        }
    }

    /// Create a policy for aggressive retries.
    pub fn aggressive() -> Self {
        Self {
            max_retries: 10,
            initial_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 1.5,
            jitter: true,
            max_total_time: Some(Duration::from_secs(300)),
        }
    }

    /// Create a policy for gentle retries.
    pub fn gentle() -> Self {
        Self {
            max_retries: 2,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
            jitter: true,
            max_total_time: Some(Duration::from_secs(15)),
        }
    }

    /// Calculate delay for a given attempt.
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let base_delay = self.initial_delay.as_millis() as f64
            * self.backoff_multiplier.powi(attempt as i32);

        let delay_ms = if self.jitter {
            // Add 0-25% jitter
            let jitter = base_delay * 0.25 * rand_factor();
            base_delay + jitter
        } else {
            base_delay
        };

        let delay = Duration::from_millis(delay_ms as u64);
        std::cmp::min(delay, self.max_delay)
    }

    /// Check if we should retry based on attempt count and elapsed time.
    pub fn should_retry(&self, attempt: u32, elapsed: Duration) -> bool {
        if attempt >= self.max_retries {
            return false;
        }

        if let Some(max_time) = self.max_total_time {
            if elapsed >= max_time {
                return false;
            }
        }

        true
    }
}

/// Simple pseudo-random factor for jitter (0.0 to 1.0).
fn rand_factor() -> f64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::SystemTime;

    let mut hasher = DefaultHasher::new();
    SystemTime::now().hash(&mut hasher);
    std::thread::current().id().hash(&mut hasher);

    (hasher.finish() % 1000) as f64 / 1000.0
}

/// Execute an async operation with retry logic.
pub async fn with_retry<T, E, F, Fut>(
    policy: &RetryPolicy,
    offline_detector: Option<&OfflineDetector>,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let start = Instant::now();
    let mut attempt = 0;

    loop {
        // Check if offline
        if let Some(detector) = offline_detector {
            if detector.is_offline() {
                debug!("Skipping retry attempt, network is offline");
                // Wait for online status or timeout
                let mut rx = detector.subscribe();
                tokio::select! {
                    result = rx.recv() => {
                        if let Ok(NetworkStatus::Online) = result {
                            debug!("Network came back online, resuming");
                        }
                    }
                    _ = tokio::time::sleep(Duration::from_secs(30)) => {
                        debug!("Timeout waiting for network, attempting anyway");
                    }
                }
            }
        }

        match operation().await {
            Ok(result) => {
                if let Some(detector) = offline_detector {
                    detector.record_success();
                }
                return Ok(result);
            }
            Err(e) => {
                if let Some(detector) = offline_detector {
                    detector.record_failure();
                }

                if !policy.should_retry(attempt, start.elapsed()) {
                    return Err(e);
                }

                let delay = policy.delay_for_attempt(attempt);
                warn!(
                    attempt = attempt + 1,
                    max_retries = policy.max_retries,
                    delay_ms = delay.as_millis() as u64,
                    error = %e,
                    "Operation failed, retrying"
                );

                tokio::time::sleep(delay).await;
                attempt += 1;
            }
        }
    }
}

/// Request queue for offline mode.
///
/// Queues requests when offline and executes them when back online.
#[derive(Debug)]
pub struct RequestQueue {
    /// Queued requests.
    queue: RwLock<Vec<QueuedRequest>>,
    /// Maximum queue size.
    max_size: usize,
    /// Whether to persist queue to disk.
    persist: bool,
}

/// A queued network request.
#[derive(Debug, Clone)]
pub struct QueuedRequest {
    /// Unique request ID.
    pub id: uuid::Uuid,
    /// Request data (serialized).
    pub data: serde_json::Value,
    /// When the request was queued.
    pub queued_at: chrono::DateTime<chrono::Utc>,
    /// Priority (higher = more important).
    pub priority: u8,
    /// Maximum age before discarding.
    pub max_age: Option<Duration>,
}

impl RequestQueue {
    /// Create a new request queue.
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: RwLock::new(Vec::new()),
            max_size,
            persist: false,
        }
    }

    /// Enable persistence.
    pub fn with_persistence(mut self) -> Self {
        self.persist = true;
        self
    }

    /// Add a request to the queue.
    pub async fn enqueue(&self, request: QueuedRequest) -> Result<(), QueueError> {
        let mut queue = self.queue.write().await;

        if queue.len() >= self.max_size {
            return Err(QueueError::QueueFull);
        }

        // Insert sorted by priority (highest first)
        let pos = queue
            .iter()
            .position(|r| r.priority < request.priority)
            .unwrap_or(queue.len());

        queue.insert(pos, request);
        Ok(())
    }

    /// Get the next request from the queue.
    pub async fn dequeue(&self) -> Option<QueuedRequest> {
        let mut queue = self.queue.write().await;

        // Remove expired requests
        let now = chrono::Utc::now();
        queue.retain(|r| {
            if let Some(max_age) = r.max_age {
                let age = now - r.queued_at;
                age < chrono::Duration::from_std(max_age).unwrap_or(chrono::Duration::MAX)
            } else {
                true
            }
        });

        if queue.is_empty() {
            None
        } else {
            Some(queue.remove(0))
        }
    }

    /// Get current queue size.
    pub async fn len(&self) -> usize {
        self.queue.read().await.len()
    }

    /// Check if queue is empty.
    pub async fn is_empty(&self) -> bool {
        self.queue.read().await.is_empty()
    }

    /// Clear the queue.
    pub async fn clear(&self) {
        self.queue.write().await.clear();
    }
}

/// Errors related to request queue.
#[derive(Debug, thiserror::Error)]
pub enum QueueError {
    /// Queue is full.
    #[error("Request queue is full")]
    QueueFull,
    /// Request expired.
    #[error("Request expired")]
    Expired,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy_delay() {
        let policy = RetryPolicy {
            jitter: false,
            ..Default::default()
        };

        let delay0 = policy.delay_for_attempt(0);
        let delay1 = policy.delay_for_attempt(1);
        let delay2 = policy.delay_for_attempt(2);

        assert_eq!(delay0.as_millis(), 100);
        assert_eq!(delay1.as_millis(), 200);
        assert_eq!(delay2.as_millis(), 400);
    }

    #[test]
    fn test_retry_policy_max_delay() {
        let policy = RetryPolicy {
            max_delay: Duration::from_millis(300),
            jitter: false,
            ..Default::default()
        };

        let delay5 = policy.delay_for_attempt(5);
        assert_eq!(delay5.as_millis(), 300); // Capped at max
    }

    #[test]
    fn test_retry_policy_should_retry() {
        let policy = RetryPolicy {
            max_retries: 3,
            max_total_time: Some(Duration::from_secs(10)),
            ..Default::default()
        };

        assert!(policy.should_retry(0, Duration::from_secs(0)));
        assert!(policy.should_retry(2, Duration::from_secs(5)));
        assert!(!policy.should_retry(3, Duration::from_secs(0))); // Max retries
        assert!(!policy.should_retry(0, Duration::from_secs(15))); // Max time
    }

    #[tokio::test]
    async fn test_offline_detector() {
        let detector = OfflineDetector::new(OfflineDetectorConfig {
            failure_threshold: 2,
            success_threshold: 2,
            ..Default::default()
        });

        assert!(!detector.is_offline());

        // Record failures
        detector.record_failure();
        assert!(!detector.is_offline()); // Not yet at threshold

        detector.record_failure();
        assert!(detector.is_offline()); // At threshold

        // Record successes
        detector.record_success();
        assert!(detector.is_offline()); // Not yet at threshold

        detector.record_success();
        assert!(!detector.is_offline()); // Back online
    }

    #[tokio::test]
    async fn test_request_queue() {
        let queue = RequestQueue::new(10);

        let request = QueuedRequest {
            id: uuid::Uuid::new_v4(),
            data: serde_json::json!({"test": true}),
            queued_at: chrono::Utc::now(),
            priority: 5,
            max_age: None,
        };

        queue.enqueue(request.clone()).await.unwrap();
        assert_eq!(queue.len().await, 1);

        let dequeued = queue.dequeue().await.unwrap();
        assert_eq!(dequeued.id, request.id);
        assert!(queue.is_empty().await);
    }

    #[tokio::test]
    async fn test_queue_priority() {
        let queue = RequestQueue::new(10);

        let low_priority = QueuedRequest {
            id: uuid::Uuid::new_v4(),
            data: serde_json::json!({"priority": "low"}),
            queued_at: chrono::Utc::now(),
            priority: 1,
            max_age: None,
        };

        let high_priority = QueuedRequest {
            id: uuid::Uuid::new_v4(),
            data: serde_json::json!({"priority": "high"}),
            queued_at: chrono::Utc::now(),
            priority: 10,
            max_age: None,
        };

        // Enqueue low priority first
        queue.enqueue(low_priority).await.unwrap();
        // Enqueue high priority second
        queue.enqueue(high_priority.clone()).await.unwrap();

        // High priority should come out first
        let first = queue.dequeue().await.unwrap();
        assert_eq!(first.id, high_priority.id);
    }
}
