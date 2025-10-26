//! Metrics export functionality for various formats.

use super::metrics::MetricsSnapshot;
use crate::error::Result;

/// Trait for exporting metrics to different formats
pub trait MetricsExporter: Send + Sync {
    /// Export a metrics snapshot
    fn export(&self, snapshot: &MetricsSnapshot) -> Result<()>;
}

/// Export metrics to JSON format
pub struct JsonExporter {
    output_path: std::path::PathBuf,
}

impl JsonExporter {
    /// Create a new JSON exporter
    pub fn new(output_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            output_path: output_path.into(),
        }
    }
}

impl MetricsExporter for JsonExporter {
    fn export(&self, snapshot: &MetricsSnapshot) -> Result<()> {
        let serializable = SerializableSnapshot::from(snapshot);
        let json = serde_json::to_string_pretty(&serializable).map_err(|e| {
            crate::error::Error::telemetry(format!("JSON serialization failed: {}", e))
        })?;

        std::fs::write(&self.output_path, json)
            .map_err(|e| crate::error::Error::telemetry(format!("Failed to write file: {}", e)))?;

        Ok(())
    }
}

/// Serializable version of MetricsSnapshot
#[derive(Debug, Clone)]
#[cfg_attr(feature = "telemetry", derive(serde::Serialize))]
struct SerializableSnapshot {
    timestamp_secs: u64,
    uptime_secs: f64,
    tasks_executed: u64,
    tasks_stolen: u64,
    tasks_panicked: u64,
    idle_time_ms: u64,
    busy_time_ms: u64,
    avg_latency_us: f64,
    p50_latency_us: f64,
    p95_latency_us: f64,
    p99_latency_us: f64,
    max_latency_us: f64,
    memory_allocated_mb: f64,
    utilization: f64,
    tasks_per_second: f64,
}

impl From<&MetricsSnapshot> for SerializableSnapshot {
    fn from(snapshot: &MetricsSnapshot) -> Self {
        Self {
            timestamp_secs: snapshot.timestamp.elapsed().as_secs(),
            uptime_secs: snapshot.uptime.as_secs_f64(),
            tasks_executed: snapshot.tasks_executed,
            tasks_stolen: snapshot.tasks_stolen,
            tasks_panicked: snapshot.tasks_panicked,
            idle_time_ms: snapshot.idle_time_ns / 1_000_000,
            busy_time_ms: snapshot.busy_time_ns / 1_000_000,
            avg_latency_us: snapshot.avg_latency_ns as f64 / 1_000.0,
            p50_latency_us: snapshot.p50_latency_ns as f64 / 1_000.0,
            p95_latency_us: snapshot.p95_latency_ns as f64 / 1_000.0,
            p99_latency_us: snapshot.p99_latency_ns as f64 / 1_000.0,
            max_latency_us: snapshot.max_latency_ns as f64 / 1_000.0,
            memory_allocated_mb: snapshot.memory_allocated as f64 / (1024.0 * 1024.0),
            utilization: snapshot.utilization(),
            tasks_per_second: snapshot.tasks_per_second(),
        }
    }
}

/// Export metrics to console (stdout)
pub struct ConsoleExporter {
    verbose: bool,
}

impl ConsoleExporter {
    /// Create a new console exporter
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }
}

impl MetricsExporter for ConsoleExporter {
    fn export(&self, snapshot: &MetricsSnapshot) -> Result<()> {
        println!("=== VEDA Runtime Metrics ===");
        println!("Uptime: {:.2}s", snapshot.uptime.as_secs_f64());
        println!("Tasks executed: {}", snapshot.tasks_executed);
        println!("Tasks stolen: {}", snapshot.tasks_stolen);
        println!("Tasks panicked: {}", snapshot.tasks_panicked);
        println!("Utilization: {:.1}%", snapshot.utilization() * 100.0);
        println!("Tasks/sec: {:.2}", snapshot.tasks_per_second());

        if self.verbose {
            println!("\nLatency:");
            println!(
                "  Average: {:.2}μs",
                snapshot.avg_latency_ns as f64 / 1_000.0
            );
            println!("  P50: {:.2}μs", snapshot.p50_latency_ns as f64 / 1_000.0);
            println!("  P95: {:.2}μs", snapshot.p95_latency_ns as f64 / 1_000.0);
            println!("  P99: {:.2}μs", snapshot.p99_latency_ns as f64 / 1_000.0);
            println!("  Max: {:.2}μs", snapshot.max_latency_ns as f64 / 1_000.0);

            println!("\nMemory:");
            println!(
                "  Allocated: {:.2}MB",
                snapshot.memory_allocated as f64 / (1024.0 * 1024.0)
            );
        }

        println!("===========================");

        Ok(())
    }
}

impl Default for ConsoleExporter {
    fn default() -> Self {
        Self::new(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn dummy_snapshot() -> MetricsSnapshot {
        MetricsSnapshot {
            timestamp: Instant::now(),
            uptime: std::time::Duration::from_secs(10),
            tasks_executed: 1000,
            tasks_stolen: 50,
            tasks_panicked: 1,
            idle_time_ns: 1_000_000_000,
            busy_time_ns: 9_000_000_000,
            avg_latency_ns: 1000,
            p50_latency_ns: 900,
            p95_latency_ns: 1500,
            p99_latency_ns: 2000,
            max_latency_ns: 5000,
            memory_allocated: 1024 * 1024, // 1MB
        }
    }

    #[test]
    fn test_console_exporter() {
        let exporter = ConsoleExporter::new(false);
        let snapshot = dummy_snapshot();

        // Should not panic
        assert!(exporter.export(&snapshot).is_ok());
    }

    #[test]
    fn test_json_exporter() {
        use std::env::temp_dir;

        let path = temp_dir().join("veda_metrics_test.json");
        let exporter = JsonExporter::new(&path);
        let snapshot = dummy_snapshot();

        assert!(exporter.export(&snapshot).is_ok());
        assert!(path.exists());

        // Clean up
        let _ = std::fs::remove_file(&path);
    }
}
