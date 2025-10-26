//! Adaptive task scheduling subsystem.
//!
//! The scheduler is responsible for deciding which tasks run where and when,
//! implementing various scheduling policies including adaptive load balancing,
//! work stealing, priority scheduling, and energy-aware scheduling.

pub mod adaptive;
pub mod work_stealing;
pub mod priority;

#[cfg(feature = "energy-aware")]
pub mod energy;

#[cfg(feature = "deterministic")]
pub mod deterministic;

pub use adaptive::{AdaptiveScheduler, LoadEstimator};
pub use work_stealing::WorkStealingQueue;
pub use priority::{Priority, PriorityQueue};

#[cfg(feature = "energy-aware")]
pub use energy::{EnergyAwareScheduler, PowerMonitor};

#[cfg(feature = "deterministic")]
pub use deterministic::DeterministicScheduler;

use crate::executor::Task;
use crate::error::Result;
use std::time::Instant;

/// Unique identifier for a worker thread
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "deterministic", derive(serde::Serialize, serde::Deserialize))]
pub struct WorkerId(pub usize);

/// Worker state for load tracking
#[derive(Debug, Clone)]
pub struct WorkerState {
    pub id: WorkerId,
    pub tasks_executed: u64,
    pub tasks_stolen: u64,
    pub idle_time_ns: u64,
    pub busy_time_ns: u64,
    pub current_task: Option<usize>,
}

impl WorkerState {
    pub fn new(id: WorkerId) -> Self {
        Self {
            id,
            tasks_executed: 0,
            tasks_stolen: 0,
            idle_time_ns: 0,
            busy_time_ns: 0,
            current_task: None,
        }
    }
    
    pub fn utilization(&self) -> f64 {
        let total = self.idle_time_ns + self.busy_time_ns;
        if total == 0 {
            return 0.0;
        }
        self.busy_time_ns as f64 / total as f64
    }
}

/// Load statistics for the scheduler
#[derive(Debug, Clone)]
pub struct LoadStatistics {
    pub mean_load: f64,
    pub std_dev: f64,
    pub avg_utilization: f64,
    pub task_arrival_rate: f64,
    pub avg_queue_wait_time_ns: u64,
    pub timestamp: Instant,
}

impl LoadStatistics {
    pub fn coefficient_of_variation(&self) -> f64 {
        if self.mean_load == 0.0 {
            0.0
        } else {
            self.std_dev / self.mean_load
        }
    }
}
