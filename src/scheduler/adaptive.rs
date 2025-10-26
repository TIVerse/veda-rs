use super::{LoadStatistics, WorkerId, WorkerState};
use crate::config::Config;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use parking_lot::RwLock;

pub struct AdaptiveScheduler {
    worker_states: Vec<Arc<RwLock<WorkerState>>>,
    load_estimator: LoadEstimator,
    config: SchedulerConfig,
    last_rebalance: RwLock<Instant>,
}

#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    pub imbalance_threshold: f64,
    pub target_latency_ns: u64,
    pub min_workers: usize,
    pub max_workers: usize,
}

impl From<&Config> for SchedulerConfig {
    fn from(config: &Config) -> Self {
        let num_threads = config.worker_threads();
        Self {
            imbalance_threshold: config.imbalance_threshold,
            target_latency_ns: 1_000_000, // 1ms default
            min_workers: 1,
            max_workers: num_threads,
        }
    }
}

impl AdaptiveScheduler {
    pub fn new(config: SchedulerConfig) -> Self {
        let num_workers = config.max_workers;
        let worker_states = (0..num_workers)
            .map(|i| Arc::new(RwLock::new(WorkerState::new(WorkerId(i)))))
            .collect();
            
        Self {
            worker_states,
            load_estimator: LoadEstimator::new(),
            config,
            last_rebalance: RwLock::new(Instant::now()),
        }
    }
    
    pub fn maybe_rebalance(&self) -> bool {
        let stats = self.collect_statistics();
        
        if self.detect_imbalance(&stats) {
            self.rebalance(&stats);
            *self.last_rebalance.write() = Instant::now();
            true
        } else {
            false
        }
    }
    
    pub fn collect_statistics(&self) -> LoadStatistics {
        let worker_loads: Vec<f64> = self.worker_states
            .iter()
            .map(|state| {
                let s = state.read();
                s.tasks_executed as f64
            })
            .collect();
            
        let mean = if worker_loads.is_empty() {
            0.0
        } else {
            worker_loads.iter().sum::<f64>() / worker_loads.len() as f64
        };
        
        let variance = if worker_loads.is_empty() {
            0.0
        } else {
            worker_loads.iter()
                .map(|&load| {
                    let diff = load - mean;
                    diff * diff
                })
                .sum::<f64>() / worker_loads.len() as f64
        };
        
        let std_dev = variance.sqrt();
        
        let avg_utilization = if self.worker_states.is_empty() {
            0.0
        } else {
            self.worker_states.iter()
                .map(|state| state.read().utilization())
                .sum::<f64>() / self.worker_states.len() as f64
        };
        
        LoadStatistics {
            mean_load: mean,
            std_dev,
            avg_utilization,
            task_arrival_rate: self.load_estimator.arrival_rate(),
            avg_queue_wait_time_ns: 0,
            timestamp: Instant::now(),
        }
    }
    
    fn detect_imbalance(&self, stats: &LoadStatistics) -> bool {
        let cv = stats.coefficient_of_variation();
        cv > self.config.imbalance_threshold
    }
    
    fn rebalance(&self, _stats: &LoadStatistics) {}
    
    pub fn compute_optimal_workers(&self, stats: &LoadStatistics) -> usize {
        if stats.avg_utilization < 0.5 {
            self.worker_states.len().max(self.config.min_workers)
        } else if stats.avg_utilization > 0.9 {
            (self.worker_states.len() * 5 / 4).min(self.config.max_workers)
        } else {
            self.worker_states.len()
        }
    }
    
    pub fn worker_state(&self, id: WorkerId) -> Option<Arc<RwLock<WorkerState>>> {
        self.worker_states.get(id.0).cloned()
    }
    
    pub fn num_workers(&self) -> usize {
        self.worker_states.len()
    }
}

pub struct LoadEstimator {
    estimate: AtomicU64,
    last_update: RwLock<Instant>,
    task_count: AtomicU64,
}

impl LoadEstimator {
    pub fn new() -> Self {
        Self {
            estimate: AtomicU64::new(0),
            last_update: RwLock::new(Instant::now()),
            task_count: AtomicU64::new(0),
        }
    }
    
    pub fn update(&self, current_load: f64) {
        let alpha = 0.3;
        let old_estimate = f64::from_bits(self.estimate.load(Ordering::Relaxed));
        let new_estimate = alpha * current_load + (1.0 - alpha) * old_estimate;
        self.estimate.store(new_estimate.to_bits(), Ordering::Relaxed);
        *self.last_update.write() = Instant::now();
    }
    
    pub fn estimate(&self) -> f64 {
        f64::from_bits(self.estimate.load(Ordering::Relaxed))
    }
    
    pub fn record_task(&self) {
        self.task_count.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn arrival_rate(&self) -> f64 {
        let elapsed = self.last_update.read().elapsed().as_secs_f64();
        if elapsed == 0.0 {
            return 0.0;
        }
        let count = self.task_count.load(Ordering::Relaxed);
        count as f64 / elapsed
    }
}

impl Default for LoadEstimator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_load_estimator() {
        let estimator = LoadEstimator::new();
        estimator.update(1.0);
        assert!(estimator.estimate() > 0.0);
        
        estimator.update(0.5);
        let estimate = estimator.estimate();
        assert!(estimate > 0.0 && estimate < 1.0);
    }
    
    #[test]
    fn test_adaptive_scheduler_creation() {
        let config = SchedulerConfig {
            imbalance_threshold: 0.2,
            target_latency_ns: 1_000_000,
            min_workers: 1,
            max_workers: 4,
        };
        
        let scheduler = AdaptiveScheduler::new(config);
        assert_eq!(scheduler.num_workers(), 4);
    }
    
    #[test]
    fn test_collect_statistics() {
        let config = SchedulerConfig {
            imbalance_threshold: 0.2,
            target_latency_ns: 1_000_000,
            min_workers: 1,
            max_workers: 4,
        };
        
        let scheduler = AdaptiveScheduler::new(config);
        let stats = scheduler.collect_statistics();
        
        assert_eq!(stats.mean_load, 0.0);
        assert!(stats.avg_utilization >= 0.0 && stats.avg_utilization <= 1.0);
    }
}
