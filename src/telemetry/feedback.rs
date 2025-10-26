//! Feedback controller for adaptive runtime adjustments based on metrics.

use super::metrics::{Metrics, MetricsSnapshot};
use crate::scheduler::adaptive::AdaptiveScheduler;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;

/// Configuration for feedback controller
#[derive(Debug, Clone)]
pub struct FeedbackConfig {
    /// Minimum task rate (tasks/sec) to maintain
    pub min_task_rate: f64,
    
    /// Maximum acceptable P99 latency in nanoseconds
    pub max_latency_ns: u64,
    
    /// Maximum power consumption in watts (if energy-aware)
    pub max_power_watts: Option<f64>,
    
    /// Update interval for feedback loop
    pub update_interval: Duration,
    
    /// Size of history buffer for trend analysis
    pub history_size: usize,
}

impl Default for FeedbackConfig {
    fn default() -> Self {
        Self {
            min_task_rate: 100.0,
            max_latency_ns: 10_000_000, // 10ms
            max_power_watts: None,
            update_interval: Duration::from_millis(100),
            history_size: 100,
        }
    }
}

/// Feedback controller for adaptive runtime
pub struct FeedbackController {
    metrics: Arc<Metrics>,
    config: FeedbackConfig,
    history: RwLock<VecDeque<MetricsSnapshot>>,
    last_update: RwLock<Instant>,
}

impl FeedbackController {
    /// Create a new feedback controller
    pub fn new(metrics: Arc<Metrics>, config: FeedbackConfig) -> Self {
        let history_size = config.history_size;
        Self {
            metrics,
            config,
            history: RwLock::new(VecDeque::with_capacity(history_size)),
            last_update: RwLock::new(Instant::now()),
        }
    }
    
    /// Update the feedback loop and make adjustments
    pub fn update(&self) -> FeedbackAction {
        let now = Instant::now();
        let elapsed = {
            let last = *self.last_update.read();
            now.duration_since(last)
        };
        
        // Only update at specified interval
        if elapsed < self.config.update_interval {
            return FeedbackAction::None;
        }
        
        // Get current metrics
        let snapshot = self.metrics.snapshot();
        
        // Add to history
        {
            let mut history = self.history.write();
            history.push_back(snapshot.clone());
            if history.len() > self.config.history_size {
                history.pop_front();
            }
        }
        
        // Update timestamp
        *self.last_update.write() = now;
        
        // Analyze and determine action
        self.analyze(&snapshot)
    }
    
    /// Analyze metrics and determine necessary action
    fn analyze(&self, current: &MetricsSnapshot) -> FeedbackAction {
        // Check latency
        if current.p99_latency_ns > self.config.max_latency_ns {
            return FeedbackAction::ReduceLoad {
                reason: "High latency detected".to_string(),
            };
        }
        
        // Check task rate
        let task_rate = current.tasks_per_second();
        if task_rate < self.config.min_task_rate && current.tasks_executed > 100 {
            return FeedbackAction::IncreaseParallelism {
                reason: "Low throughput detected".to_string(),
            };
        }
        
        // Check utilization
        let utilization = current.utilization();
        if utilization < 0.3 && current.tasks_executed > 100 {
            return FeedbackAction::OptimizeResources {
                reason: "Low utilization detected".to_string(),
            };
        }
        
        FeedbackAction::None
    }
    
    /// Compute rate of change for a metric
    pub fn compute_delta(&self) -> Option<MetricsDelta> {
        let history = self.history.read();
        
        if history.len() < 2 {
            return None;
        }
        
        let current = history.back()?;
        let previous = history.get(history.len().saturating_sub(10))?;
        
        let time_delta = current.timestamp.duration_since(previous.timestamp).as_secs_f64();
        if time_delta == 0.0 {
            return None;
        }
        
        Some(MetricsDelta {
            task_rate_change: ((current.tasks_executed - previous.tasks_executed) as f64 / time_delta)
                - previous.tasks_per_second(),
            latency_p99_change: current.p99_latency_ns as i64 - previous.p99_latency_ns as i64,
            utilization_change: current.utilization() - previous.utilization(),
        })
    }
    
    /// Get recent trends
    pub fn trends(&self) -> Trends {
        let delta = self.compute_delta();
        
        Trends {
            task_rate_trend: delta.as_ref().map(|d| classify_trend(d.task_rate_change)),
            latency_trend: delta.as_ref().map(|d| classify_trend(d.latency_p99_change as f64)),
            utilization_trend: delta.as_ref().map(|d| classify_trend(d.utilization_change)),
        }
    }
}

/// Actions that can be taken based on feedback
#[derive(Debug, Clone, PartialEq)]
pub enum FeedbackAction {
    /// No action needed
    None,
    
    /// Increase parallelism to improve throughput
    IncreaseParallelism { reason: String },
    
    /// Reduce load to improve latency
    ReduceLoad { reason: String },
    
    /// Optimize resource usage
    OptimizeResources { reason: String },
}

/// Delta between two metric snapshots
#[derive(Debug, Clone)]
pub struct MetricsDelta {
    pub task_rate_change: f64,
    pub latency_p99_change: i64,
    pub utilization_change: f64,
}

/// Trend classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trend {
    Increasing,
    Stable,
    Decreasing,
}

/// Trends for various metrics
#[derive(Debug, Clone)]
pub struct Trends {
    pub task_rate_trend: Option<Trend>,
    pub latency_trend: Option<Trend>,
    pub utilization_trend: Option<Trend>,
}

fn classify_trend(value: f64) -> Trend {
    const THRESHOLD: f64 = 0.05; // 5% change threshold
    
    if value > THRESHOLD {
        Trend::Increasing
    } else if value < -THRESHOLD {
        Trend::Decreasing
    } else {
        Trend::Stable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feedback_controller() {
        let metrics = Arc::new(Metrics::new());
        let config = FeedbackConfig::default();
        let controller = FeedbackController::new(metrics, config);
        
        let action = controller.update();
        assert_eq!(action, FeedbackAction::None);
    }
    
    #[test]
    fn test_trend_classification() {
        assert_eq!(classify_trend(0.1), Trend::Increasing);
        assert_eq!(classify_trend(-0.1), Trend::Decreasing);
        assert_eq!(classify_trend(0.01), Trend::Stable);
    }
}
