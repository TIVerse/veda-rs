//! Telemetry and observability subsystem.
//!
//! Provides metrics collection, tracing, and export capabilities for
//! monitoring runtime performance and behavior.

#[cfg(feature = "telemetry")]
pub mod metrics;

#[cfg(feature = "telemetry")]
pub mod tracing;

#[cfg(feature = "telemetry")]
pub mod export;

#[cfg(feature = "telemetry")]
pub mod feedback;

#[cfg(feature = "telemetry")]
pub use metrics::{Metrics, MetricsSnapshot};

#[cfg(feature = "telemetry")]
pub use tracing::{Span, SpanGuard, TracingSystem};

#[cfg(feature = "telemetry")]
pub use export::{MetricsExporter, JsonExporter};

#[cfg(feature = "telemetry")]
pub use feedback::FeedbackController;

// Stub implementations when telemetry is disabled
#[cfg(not(feature = "telemetry"))]
pub mod metrics {
    use std::time::Instant;
    
    #[derive(Debug, Clone)]
    pub struct Metrics;
    
    impl Metrics {
        pub fn new() -> Self { Self }
        pub fn record_task_execution(&self, _: u64) {}
        pub fn record_task_stolen(&self) {}
        pub fn record_task_panic(&self) {}
        pub fn snapshot(&self) -> MetricsSnapshot { MetricsSnapshot::default() }
    }
    
    #[derive(Debug, Clone, Default)]
    pub struct MetricsSnapshot {
        pub timestamp: Option<Instant>,
        pub tasks_executed: u64,
        pub tasks_stolen: u64,
        pub tasks_panicked: u64,
        pub avg_latency_ns: u64,
        pub p50_latency_ns: u64,
        pub p99_latency_ns: u64,
    }
}
