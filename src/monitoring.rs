//! Monitoring and metrics collection for diesel-gaussdb
//!
//! This module provides comprehensive monitoring capabilities including
//! performance metrics, health checks, and diagnostic tools.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Global metrics collector for diesel-gaussdb
#[derive(Debug, Default)]
pub struct GaussDBMetrics {
    /// Total number of connections established
    pub connections_established: AtomicU64,
    /// Total number of connection failures
    pub connection_failures: AtomicU64,
    /// Total number of queries executed
    pub queries_executed: AtomicU64,
    /// Total number of query failures
    pub query_failures: AtomicU64,
    /// Total query execution time in microseconds
    pub total_query_time_us: AtomicU64,
    /// Total number of transactions started
    pub transactions_started: AtomicU64,
    /// Total number of transactions committed
    pub transactions_committed: AtomicU64,
    /// Total number of transactions rolled back
    pub transactions_rolled_back: AtomicU64,
}

impl GaussDBMetrics {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Record a successful connection
    pub fn record_connection_success(&self) {
        self.connections_established.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record a connection failure
    pub fn record_connection_failure(&self) {
        self.connection_failures.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record a successful query execution
    pub fn record_query_success(&self, duration: Duration) {
        self.queries_executed.fetch_add(1, Ordering::Relaxed);
        self.total_query_time_us.fetch_add(
            duration.as_micros() as u64, 
            Ordering::Relaxed
        );
    }
    
    /// Record a query failure
    pub fn record_query_failure(&self) {
        self.query_failures.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record transaction start
    pub fn record_transaction_start(&self) {
        self.transactions_started.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record transaction commit
    pub fn record_transaction_commit(&self) {
        self.transactions_committed.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record transaction rollback
    pub fn record_transaction_rollback(&self) {
        self.transactions_rolled_back.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Get current metrics snapshot
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            connections_established: self.connections_established.load(Ordering::Relaxed),
            connection_failures: self.connection_failures.load(Ordering::Relaxed),
            queries_executed: self.queries_executed.load(Ordering::Relaxed),
            query_failures: self.query_failures.load(Ordering::Relaxed),
            total_query_time_us: self.total_query_time_us.load(Ordering::Relaxed),
            transactions_started: self.transactions_started.load(Ordering::Relaxed),
            transactions_committed: self.transactions_committed.load(Ordering::Relaxed),
            transactions_rolled_back: self.transactions_rolled_back.load(Ordering::Relaxed),
        }
    }
    
    /// Calculate average query time in microseconds
    pub fn average_query_time_us(&self) -> f64 {
        let total_time = self.total_query_time_us.load(Ordering::Relaxed);
        let total_queries = self.queries_executed.load(Ordering::Relaxed);
        
        if total_queries > 0 {
            total_time as f64 / total_queries as f64
        } else {
            0.0
        }
    }
    
    /// Calculate connection success rate
    pub fn connection_success_rate(&self) -> f64 {
        let successes = self.connections_established.load(Ordering::Relaxed);
        let failures = self.connection_failures.load(Ordering::Relaxed);
        let total = successes + failures;
        
        if total > 0 {
            successes as f64 / total as f64
        } else {
            0.0
        }
    }
    
    /// Calculate query success rate
    pub fn query_success_rate(&self) -> f64 {
        let successes = self.queries_executed.load(Ordering::Relaxed);
        let failures = self.query_failures.load(Ordering::Relaxed);
        let total = successes + failures;
        
        if total > 0 {
            successes as f64 / total as f64
        } else {
            0.0
        }
    }
}

/// Snapshot of metrics at a point in time
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub connections_established: u64,
    pub connection_failures: u64,
    pub queries_executed: u64,
    pub query_failures: u64,
    pub total_query_time_us: u64,
    pub transactions_started: u64,
    pub transactions_committed: u64,
    pub transactions_rolled_back: u64,
}

impl MetricsSnapshot {
    /// Convert to a HashMap for easy serialization
    pub fn to_map(&self) -> HashMap<String, u64> {
        let mut map = HashMap::new();
        map.insert("connections_established".to_string(), self.connections_established);
        map.insert("connection_failures".to_string(), self.connection_failures);
        map.insert("queries_executed".to_string(), self.queries_executed);
        map.insert("query_failures".to_string(), self.query_failures);
        map.insert("total_query_time_us".to_string(), self.total_query_time_us);
        map.insert("transactions_started".to_string(), self.transactions_started);
        map.insert("transactions_committed".to_string(), self.transactions_committed);
        map.insert("transactions_rolled_back".to_string(), self.transactions_rolled_back);
        map
    }
}

/// Global metrics instance
static GLOBAL_METRICS: std::sync::OnceLock<Arc<GaussDBMetrics>> = std::sync::OnceLock::new();

/// Get the global metrics instance
pub fn global_metrics() -> Arc<GaussDBMetrics> {
    GLOBAL_METRICS.get_or_init(|| Arc::new(GaussDBMetrics::new())).clone()
}

/// Health check status
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheck {
    pub status: HealthStatus,
    pub message: String,
    pub timestamp: Instant,
    pub details: HashMap<String, String>,
}

impl HealthCheck {
    /// Create a healthy status
    pub fn healthy(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Healthy,
            message: message.into(),
            timestamp: Instant::now(),
            details: HashMap::new(),
        }
    }
    
    /// Create a degraded status
    pub fn degraded(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Degraded,
            message: message.into(),
            timestamp: Instant::now(),
            details: HashMap::new(),
        }
    }
    
    /// Create an unhealthy status
    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            message: message.into(),
            timestamp: Instant::now(),
            details: HashMap::new(),
        }
    }
    
    /// Add detail information
    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.details.insert(key.into(), value.into());
        self
    }
}

/// Perform basic health checks
pub fn perform_health_check() -> HealthCheck {
    let metrics = global_metrics();
    let snapshot = metrics.snapshot();
    
    // Check connection success rate
    let conn_success_rate = metrics.connection_success_rate();
    if conn_success_rate < 0.8 && snapshot.connections_established > 10 {
        return HealthCheck::degraded("Low connection success rate")
            .with_detail("success_rate", format!("{:.2}", conn_success_rate))
            .with_detail("total_connections", snapshot.connections_established.to_string());
    }
    
    // Check query success rate
    let query_success_rate = metrics.query_success_rate();
    if query_success_rate < 0.9 && snapshot.queries_executed > 100 {
        return HealthCheck::degraded("Low query success rate")
            .with_detail("success_rate", format!("{:.2}", query_success_rate))
            .with_detail("total_queries", snapshot.queries_executed.to_string());
    }
    
    // Check average query time
    let avg_query_time = metrics.average_query_time_us();
    if avg_query_time > 10_000.0 && snapshot.queries_executed > 10 {
        return HealthCheck::degraded("High average query time")
            .with_detail("avg_time_us", format!("{:.2}", avg_query_time))
            .with_detail("total_queries", snapshot.queries_executed.to_string());
    }
    
    HealthCheck::healthy("All systems operational")
        .with_detail("connections", snapshot.connections_established.to_string())
        .with_detail("queries", snapshot.queries_executed.to_string())
        .with_detail("avg_query_time_us", format!("{:.2}", avg_query_time))
}

/// Query performance tracker
pub struct QueryTracker {
    start_time: Instant,
    metrics: Arc<GaussDBMetrics>,
}

impl QueryTracker {
    /// Start tracking a query
    pub fn start() -> Self {
        Self {
            start_time: Instant::now(),
            metrics: global_metrics(),
        }
    }
    
    /// Finish tracking with success
    pub fn finish_success(self) {
        let duration = self.start_time.elapsed();
        self.metrics.record_query_success(duration);
    }
    
    /// Finish tracking with failure
    pub fn finish_failure(self) {
        self.metrics.record_query_failure();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    
    #[test]
    fn test_metrics_recording() {
        let metrics = GaussDBMetrics::new();
        
        // Test connection metrics
        metrics.record_connection_success();
        metrics.record_connection_failure();
        
        assert_eq!(metrics.connections_established.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.connection_failures.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.connection_success_rate(), 0.5);
    }
    
    #[test]
    fn test_query_metrics() {
        let metrics = GaussDBMetrics::new();
        
        // Test query metrics
        metrics.record_query_success(Duration::from_millis(10));
        metrics.record_query_success(Duration::from_millis(20));
        metrics.record_query_failure();
        
        assert_eq!(metrics.queries_executed.load(Ordering::Relaxed), 2);
        assert_eq!(metrics.query_failures.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.query_success_rate(), 2.0 / 3.0);
        assert_eq!(metrics.average_query_time_us(), 15_000.0);
    }
    
    #[test]
    fn test_health_check() {
        let health = perform_health_check();
        assert_eq!(health.status, HealthStatus::Healthy);
        assert!(!health.message.is_empty());
    }
    
    #[test]
    fn test_query_tracker() {
        let tracker = QueryTracker::start();
        thread::sleep(Duration::from_millis(1));
        tracker.finish_success();
        
        let metrics = global_metrics();
        assert!(metrics.queries_executed.load(Ordering::Relaxed) >= 1);
    }
}
