//! Deterministic scheduler for reproducible execution.

use super::WorkerId;
use crate::executor::Task;
use rand_pcg::Pcg64;
use rand::SeedableRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use parking_lot::Mutex;

/// Deterministic scheduler for reproducible execution
pub struct DeterministicScheduler {
    seed: u64,
    rng: Mutex<Pcg64>,
    logical_clock: AtomicU64,
    num_workers: usize,
    trace: Option<Mutex<ExecutionTrace>>,
}

impl DeterministicScheduler {
    /// Create a new deterministic scheduler with the given seed
    pub fn new(seed: u64, num_workers: usize) -> Self {
        Self {
            seed,
            rng: Mutex::new(Pcg64::seed_from_u64(seed)),
            logical_clock: AtomicU64::new(0),
            num_workers,
            trace: Some(Mutex::new(ExecutionTrace::new())),
        }
    }
    
    /// Schedule a task to a worker deterministically
    pub fn schedule_task(&self, task_id: usize) -> WorkerId {
        let worker_id = {
            let mut rng = self.rng.lock();
            rng.gen_range(0..self.num_workers)
        };
        
        let timestamp = self.tick();
        
        if let Some(ref trace) = self.trace {
            trace.lock().record(TraceEvent::TaskScheduled {
                task_id,
                worker_id: WorkerId(worker_id),
                timestamp,
            });
        }
        
        WorkerId(worker_id)
    }
    
    /// Record task start
    pub fn record_task_start(&self, task_id: usize, worker_id: WorkerId) {
        let timestamp = self.tick();
        if let Some(ref trace) = self.trace {
            trace.lock().record(TraceEvent::TaskStarted {
                task_id,
                worker_id,
                timestamp,
            });
        }
    }
    
    /// Record task completion
    pub fn record_task_completion(&self, task_id: usize, worker_id: WorkerId, duration_ns: u64) {
        let timestamp = self.tick();
        if let Some(ref trace) = self.trace {
            trace.lock().record(TraceEvent::TaskCompleted {
                task_id,
                worker_id,
                timestamp,
                duration_ns,
            });
        }
    }
    
    /// Increment logical clock
    fn tick(&self) -> u64 {
        self.logical_clock.fetch_add(1, Ordering::SeqCst)
    }
    
    /// Get the execution trace
    pub fn trace(&self) -> Option<ExecutionTrace> {
        self.trace.as_ref().map(|t| t.lock().clone())
    }
    
    /// Get the seed
    pub fn seed(&self) -> u64 {
        self.seed
    }
    
    /// Get the next worker in a deterministic manner
    pub fn next_worker(&self) -> WorkerId {
        let worker_id = {
            let mut rng = self.rng.lock();
            rng.gen_range(0..self.num_workers)
        };
        WorkerId(worker_id)
    }
    
    /// Record task execution
    pub fn record_execution(&self, worker_id: WorkerId, task_id: usize) {
        self.record_task_start(task_id, worker_id);
    }
    
    /// Collect statistics (returns None for deterministic mode)
    pub fn collect_statistics(&self) -> Option<super::LoadStatistics> {
        None // Deterministic mode doesn't collect load statistics
    }
}

/// Execution trace for deterministic replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    events: Vec<TraceEvent>,
    #[serde(skip, default = "Instant::now")]
    start_time: Instant,
}

impl ExecutionTrace {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            start_time: Instant::now(),
        }
    }
    
    /// Record an event
    pub fn record(&mut self, event: TraceEvent) {
        self.events.push(event);
    }
    
    /// Get all events
    pub fn events(&self) -> &[TraceEvent] {
        &self.events
    }
    
    /// Save trace to JSON file
    pub fn save(&self, path: &std::path::Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, json)
    }
    
    /// Load trace from JSON file
    pub fn load(path: &std::path::Path) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

impl Default for ExecutionTrace {
    fn default() -> Self {
        Self::new()
    }
}

/// Events that can be recorded in an execution trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraceEvent {
    TaskScheduled {
        task_id: usize,
        worker_id: WorkerId,
        timestamp: u64,
    },
    TaskStarted {
        task_id: usize,
        worker_id: WorkerId,
        timestamp: u64,
        },
    TaskCompleted {
        task_id: usize,
        worker_id: WorkerId,
        timestamp: u64,
        duration_ns: u64,
    },
    TaskStolen {
        task_id: usize,
        from: WorkerId,
        to: WorkerId,
        timestamp: u64,
    },
    WorkerIdle {
        worker_id: WorkerId,
        timestamp: u64,
    },
    WorkerBusy {
        worker_id: WorkerId,
        timestamp: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_deterministic_scheduler() {
        let scheduler = DeterministicScheduler::new(42, 4);
        
        let worker1 = scheduler.schedule_task(0);
        let worker2 = scheduler.schedule_task(1);
        
        assert!(worker1.0 < 4);
        assert!(worker2.0 < 4);
    }
    
    #[test]
    fn test_deterministic_reproducibility() {
        let scheduler1 = DeterministicScheduler::new(123, 4);
        let scheduler2 = DeterministicScheduler::new(123, 4);
        
        let mut results1 = Vec::new();
        let mut results2 = Vec::new();
        
        for i in 0..100 {
            results1.push(scheduler1.schedule_task(i));
            results2.push(scheduler2.schedule_task(i));
        }
        
        assert_eq!(results1, results2);
    }
    
    #[test]
    fn test_execution_trace() {
        let scheduler = DeterministicScheduler::new(42, 4);
        
        scheduler.schedule_task(0);
        scheduler.record_task_start(0, WorkerId(0));
        scheduler.record_task_completion(0, WorkerId(0), 1000);
        
        let trace = scheduler.trace().unwrap();
        assert_eq!(trace.events().len(), 3);
    }
}
