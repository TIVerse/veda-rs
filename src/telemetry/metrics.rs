//! Metrics collection for runtime monitoring.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use hdrhistogram::Histogram;
use parking_lot::RwLock;

/// Runtime metrics collector
#[derive(Debug)]
pub struct Metrics {
    // Task counters
    tasks_executed: AtomicU64,
    tasks_stolen: AtomicU64,
    tasks_panicked: AtomicU64,
    
    // Timing metrics
    idle_time_ns: AtomicU64,
    busy_time_ns: AtomicU64,
    
    // Latency histogram (protected by RwLock for interior mutability)
    latency_histogram: RwLock<Histogram<u64>>,
    
    // Memory metrics
    memory_allocated: AtomicU64,
    
    // Creation time
    start_time: Instant,
}

impl Metrics {
    /// Create a new metrics collector
    pub fn new() -> Self {
        // Create histogram with 3 significant figures and max value of 1 hour in nanoseconds
        let histogram = Histogram::new_with_max(3_600_000_000_000, 3)
            .expect("Failed to create histogram");
        
        Self {
            tasks_executed: AtomicU64::new(0),
            tasks_stolen: AtomicU64::new(0),
            tasks_panicked: AtomicU64::new(0),
            idle_time_ns: AtomicU64::new(0),
            busy_time_ns: AtomicU64::new(0),
            latency_histogram: RwLock::new(histogram),
            memory_allocated: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }
    
    /// Record a task execution with duration
    pub fn record_task_execution(&self, duration_ns: u64) {
        self.tasks_executed.fetch_add(1, Ordering::Relaxed);
        
        // Record latency in histogram
        if let Some(mut hist) = self.latency_histogram.try_write() {
            let _ = hist.record(duration_ns);
        }
    }
    
    /// Record a stolen task
    pub fn record_task_stolen(&self) {
        self.tasks_stolen.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record a task panic
    pub fn record_task_panic(&self) {
        self.tasks_panicked.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record idle time
    pub fn record_idle_time(&self, duration_ns: u64) {
        self.idle_time_ns.fetch_add(duration_ns, Ordering::Relaxed);
    }
    
    /// Record busy time
    pub fn record_busy_time(&self, duration_ns: u64) {
        self.busy_time_ns.fetch_add(duration_ns, Ordering::Relaxed);
    }
    
    /// Record memory allocation
    pub fn record_allocation(&self, bytes: usize) {
        self.memory_allocated.fetch_add(bytes as u64, Ordering::Relaxed);
    }
    
    /// Get a snapshot of current metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        let histogram = self.latency_histogram.read();
        
        MetricsSnapshot {
            timestamp: Instant::now(),
            uptime: self.start_time.elapsed(),
            tasks_executed: self.tasks_executed.load(Ordering::Relaxed),
            tasks_stolen: self.tasks_stolen.load(Ordering::Relaxed),
            tasks_panicked: self.tasks_panicked.load(Ordering::Relaxed),
            idle_time_ns: self.idle_time_ns.load(Ordering::Relaxed),
            busy_time_ns: self.busy_time_ns.load(Ordering::Relaxed),
            avg_latency_ns: if histogram.len() > 0 {
                histogram.mean() as u64
            } else {
                0
            },
            p50_latency_ns: histogram.value_at_quantile(0.50),
            p95_latency_ns: histogram.value_at_quantile(0.95),
            p99_latency_ns: histogram.value_at_quantile(0.99),
            max_latency_ns: histogram.max(),
            memory_allocated: self.memory_allocated.load(Ordering::Relaxed),
        }
    }
    
    /// Reset all metrics
    pub fn reset(&self) {
        self.tasks_executed.store(0, Ordering::Relaxed);
        self.tasks_stolen.store(0, Ordering::Relaxed);
        self.tasks_panicked.store(0, Ordering::Relaxed);
        self.idle_time_ns.store(0, Ordering::Relaxed);
        self.busy_time_ns.store(0, Ordering::Relaxed);
        self.memory_allocated.store(0, Ordering::Relaxed);
        
        if let Some(mut hist) = self.latency_histogram.try_write() {
            hist.reset();
        }
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of metrics at a point in time
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub timestamp: Instant,
    pub uptime: std::time::Duration,
    pub tasks_executed: u64,
    pub tasks_stolen: u64,
    pub tasks_panicked: u64,
    pub idle_time_ns: u64,
    pub busy_time_ns: u64,
    pub avg_latency_ns: u64,
    pub p50_latency_ns: u64,
    pub p95_latency_ns: u64,
    pub p99_latency_ns: u64,
    pub max_latency_ns: u64,
    pub memory_allocated: u64,
}

impl MetricsSnapshot {
    /// Calculate overall utilization (0.0 to 1.0)
    pub fn utilization(&self) -> f64 {
        let total_time = self.idle_time_ns + self.busy_time_ns;
        if total_time == 0 {
            return 0.0;
        }
        self.busy_time_ns as f64 / total_time as f64
    }
    
    /// Calculate tasks per second
    pub fn tasks_per_second(&self) -> f64 {
        let seconds = self.uptime.as_secs_f64();
        if seconds == 0.0 {
            return 0.0;
        }
        self.tasks_executed as f64 / seconds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metrics_basic() {
        let metrics = Metrics::new();
        
        metrics.record_task_execution(1000);
        metrics.record_task_execution(2000);
        metrics.record_task_stolen();
        
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.tasks_executed, 2);
        assert_eq!(snapshot.tasks_stolen, 1);
        assert!(snapshot.avg_latency_ns > 0);
    }
    
    #[test]
    fn test_metrics_reset() {
        let metrics = Metrics::new();
        
        metrics.record_task_execution(1000);
        assert_eq!(metrics.snapshot().tasks_executed, 1);
        
        metrics.reset();
        assert_eq!(metrics.snapshot().tasks_executed, 0);
    }
    
    #[test]
    fn test_utilization() {
        let mut snapshot = MetricsSnapshot {
            timestamp: Instant::now(),
            uptime: std::time::Duration::from_secs(1),
            tasks_executed: 0,
            tasks_stolen: 0,
            tasks_panicked: 0,
            idle_time_ns: 1_000_000_000,
            busy_time_ns: 1_000_000_000,
            avg_latency_ns: 0,
            p50_latency_ns: 0,
            p95_latency_ns: 0,
            p99_latency_ns: 0,
            max_latency_ns: 0,
            memory_allocated: 0,
        };
        
        assert_eq!(snapshot.utilization(), 0.5);
        
        snapshot.busy_time_ns = 3_000_000_000;
        assert_eq!(snapshot.utilization(), 0.75);
    }
}
