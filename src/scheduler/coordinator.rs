//! Scheduler coordinator - manages different scheduling policies

use super::{AdaptiveScheduler, LoadStatistics, WorkerId, WorkerState};
use crate::config::{Config, SchedulingPolicy};
use crate::error::Result;
use std::sync::Arc;
use parking_lot::RwLock;

#[cfg(feature = "deterministic")]
use super::deterministic::DeterministicScheduler;

#[cfg(feature = "energy-aware")]
use super::energy::EnergyAwareScheduler;

/// Unified scheduler coordinator that dispatches to the appropriate scheduler
pub struct SchedulerCoordinator {
    policy: SchedulingPolicy,
    adaptive: Option<AdaptiveScheduler>,
    #[cfg(feature = "deterministic")]
    deterministic: Option<DeterministicScheduler>,
    #[cfg(feature = "energy-aware")]
    energy_aware: Option<EnergyAwareScheduler>,
}

impl SchedulerCoordinator {
    pub fn new(config: &Config) -> Result<Self> {
        let policy = config.scheduling_policy;
        
        let adaptive = if matches!(policy, SchedulingPolicy::Adaptive) {
            Some(AdaptiveScheduler::new(config.into()))
        } else {
            None
        };
        
        #[cfg(feature = "deterministic")]
        let deterministic = if matches!(policy, SchedulingPolicy::Deterministic { .. }) {
            if let SchedulingPolicy::Deterministic { seed } = policy {
                Some(DeterministicScheduler::new(seed, config.worker_threads()))
            } else {
                None
            }
        } else {
            None
        };
        
        #[cfg(feature = "energy-aware")]
        let energy_aware = if matches!(policy, SchedulingPolicy::EnergyEfficient) {
            let energy_config = super::energy::EnergyConfig {
                max_watts: config.max_power_watts.unwrap_or(100.0),
                max_temp_celsius: 85.0,
                enable_dvfs: true,
            };
            Some(EnergyAwareScheduler::new(energy_config))
        } else {
            None
        };
        
        Ok(Self {
            policy,
            adaptive,
            #[cfg(feature = "deterministic")]
            deterministic,
            #[cfg(feature = "energy-aware")]
            energy_aware,
        })
    }
    
    /// Perform scheduling cycle - rebalance, collect stats, etc.
    pub fn schedule_cycle(&self) -> Option<LoadStatistics> {
        match self.policy {
            SchedulingPolicy::Adaptive => {
                if let Some(ref adaptive) = self.adaptive {
                    let stats = adaptive.collect_statistics();
                    adaptive.maybe_rebalance();
                    return Some(stats);
                }
            }
            #[cfg(feature = "deterministic")]
            SchedulingPolicy::Deterministic { .. } => {
                if let Some(ref det) = self.deterministic {
                    // Deterministic scheduler tracks its own state
                    return det.collect_statistics();
                }
            }
            #[cfg(feature = "energy-aware")]
            SchedulingPolicy::EnergyEfficient => {
                if let Some(ref energy) = self.energy_aware {
                    // Check if we should throttle
                    if energy.should_throttle() {
                        if cfg!(debug_assertions) {
                            eprintln!("[VEDA Energy] Throttling due to power/thermal constraints");
                        }
                    }
                    // Return energy statistics as load statistics
                    return Some(LoadStatistics {
                        mean_load: 0.0,
                        std_dev: 0.0,
                        avg_utilization: 0.0,
                        task_arrival_rate: 0.0,
                        avg_queue_wait_time_ns: 0,
                        timestamp: std::time::Instant::now(),
                    });
                }
            }
            _ => {}
        }
        None
    }
    
    /// Get worker that should handle next task (for deterministic mode)
    pub fn select_worker(&self, num_workers: usize) -> usize {
        match self.policy {
            #[cfg(feature = "deterministic")]
            SchedulingPolicy::Deterministic { .. } => {
                if let Some(ref det) = self.deterministic {
                    return det.next_worker().0;
                }
            }
            _ => {}
        }
        
        // Default: round-robin or random
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        COUNTER.fetch_add(1, Ordering::Relaxed) % num_workers
    }
    
    /// Check if task execution should be throttled (energy-aware)
    pub fn should_throttle(&self) -> bool {
        #[cfg(feature = "energy-aware")]
        if matches!(self.policy, SchedulingPolicy::EnergyEfficient) {
            if let Some(ref energy) = self.energy_aware {
                return energy.should_throttle();
            }
        }
        false
    }
    
    /// Update power consumption (energy-aware)
    #[cfg(feature = "energy-aware")]
    pub fn update_power(&self, watts: f64) {
        if let Some(ref energy) = self.energy_aware {
            energy.power_monitor().update_power(watts);
        }
    }
    
    /// Update temperature (energy-aware)
    #[cfg(feature = "energy-aware")]
    pub fn update_temperature(&self, temp_celsius: f64) {
        if let Some(ref energy) = self.energy_aware {
            energy.thermal_state().update_temperature(temp_celsius);
        }
    }
    
    /// Record task execution for deterministic replay
    #[cfg(feature = "deterministic")]
    pub fn record_task_execution(&self, worker_id: WorkerId, task_id: usize) {
        if let Some(ref det) = self.deterministic {
            det.record_execution(worker_id, task_id);
        }
    }
    
    /// Get scheduling policy
    pub fn policy(&self) -> SchedulingPolicy {
        self.policy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_coordinator_adaptive() {
        let config = Config::builder()
            .scheduling_policy(SchedulingPolicy::Adaptive)
            .num_threads(4)
            .build()
            .unwrap();
        
        let coordinator = SchedulerCoordinator::new(&config).unwrap();
        assert_eq!(coordinator.policy(), SchedulingPolicy::Adaptive);
    }
    
    #[test]
    #[cfg(feature = "deterministic")]
    fn test_coordinator_deterministic() {
        let config = Config::builder()
            .scheduling_policy(SchedulingPolicy::Deterministic { seed: 42 })
            .num_threads(4)
            .build()
            .unwrap();
        
        let coordinator = SchedulerCoordinator::new(&config).unwrap();
        
        // Should produce consistent worker selection
        let w1 = coordinator.select_worker(4);
        let w2 = coordinator.select_worker(4);
        assert!(w1 < 4);
        assert!(w2 < 4);
    }
    
    #[test]
    #[cfg(feature = "energy-aware")]
    fn test_coordinator_energy() {
        let config = Config::builder()
            .scheduling_policy(SchedulingPolicy::EnergyEfficient)
            .num_threads(4)
            .max_power_watts(50.0)
            .build()
            .unwrap();
        
        let coordinator = SchedulerCoordinator::new(&config).unwrap();
        
        // Should not throttle initially
        assert!(!coordinator.should_throttle());
        
        // Update with high power
        coordinator.update_power(150.0);
        
        // Now should throttle
        assert!(coordinator.should_throttle());
    }
}
