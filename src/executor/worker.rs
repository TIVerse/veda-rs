//! Worker thread implementation.
//!
//! Each worker maintains a local queue and can steal from other workers
//! when idle. This implements the Chase-Lev work-stealing algorithm.

use super::task::Task;
use crossbeam_deque::{Injector, Stealer, Worker as WorkerQueue};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Unique identifier for workers
pub type WorkerId = usize;

/// Worker thread state and statistics
pub struct WorkerState {
    pub tasks_executed: AtomicU64,
    pub tasks_stolen: AtomicU64,
    pub idle_time_ns: AtomicU64,
}

impl WorkerState {
    fn new() -> Self {
        Self {
            tasks_executed: AtomicU64::new(0),
            tasks_stolen: AtomicU64::new(0),
            idle_time_ns: AtomicU64::new(0),
        }
    }
}

/// Internal worker thread context
pub(crate) struct Worker {
    pub id: WorkerId,
    pub local_queue: WorkerQueue<Task>,
    pub state: Arc<WorkerState>,
}

impl Worker {
    pub fn new(id: WorkerId) -> Self {
        Self {
            id,
            local_queue: WorkerQueue::new_fifo(),
            state: Arc::new(WorkerState::new()),
        }
    }
    
    /// Main worker loop - tries to find and execute tasks
    pub fn run(
        &self,
        stealers: Vec<Stealer<Task>>,
        injector: Arc<Injector<Task>>,
        shutdown: Arc<AtomicBool>,
    ) {
        let mut backoff_count = 0;
        
        loop {
            if shutdown.load(Ordering::Acquire) {
                break;
            }
            
            // Try to get work in priority order:
            // 1. Local queue (best cache locality)
            // 2. Global injector (fairness)
            // 3. Steal from other workers
            
            if let Some(task) = self.find_task(&stealers, &injector) {
                backoff_count = 0;
                self.execute_task(task);
            } else {
                // No work found - back off
                if self.backoff(&mut backoff_count) {
                    break; // Shutdown signaled during backoff
                }
            }
        }
    }
    
    fn find_task(&self, stealers: &[Stealer<Task>], injector: &Injector<Task>) -> Option<Task> {
        // Try local queue first
        if let Some(task) = self.local_queue.pop() {
            return Some(task);
        }
        
        // Try stealing from global injector
        loop {
            match injector.steal_batch_and_pop(&self.local_queue) {
                crossbeam_deque::Steal::Success(task) => {
                    self.state.tasks_stolen.fetch_add(1, Ordering::Relaxed);
                    return Some(task);
                }
                crossbeam_deque::Steal::Empty => break,
                crossbeam_deque::Steal::Retry => continue,
            }
        }
        
        // Try stealing from other workers (randomized to reduce contention)
        self.try_steal_from_workers(stealers)
    }
    
    fn try_steal_from_workers(&self, stealers: &[Stealer<Task>]) -> Option<Task> {
        use rand::seq::SliceRandom;
        use rand::thread_rng;
        
        if stealers.is_empty() {
            return None;
        }
        
        // Create a randomized order to reduce contention
        let mut indices: Vec<usize> = (0..stealers.len()).collect();
        indices.shuffle(&mut thread_rng());
        
        for &idx in &indices {
            // Don't try to steal from ourselves
            if idx == self.id {
                continue;
            }
            
            loop {
                match stealers[idx].steal_batch_and_pop(&self.local_queue) {
                    crossbeam_deque::Steal::Success(task) => {
                        self.state.tasks_stolen.fetch_add(1, Ordering::Relaxed);
                        return Some(task);
                    }
                    crossbeam_deque::Steal::Empty => break,
                    crossbeam_deque::Steal::Retry => continue,
                }
            }
        }
        
        None
    }
    
    fn execute_task(&self, task: Task) {
        let task_id = task.id;
        
        // Catch panics to prevent worker thread from dying
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            task.execute();
        }));
        
        if result.is_err() {
            // Task panicked, but we keep the worker alive
            // TODO: integrate with telemetry to track panics
            eprintln!("VEDA: Task {:?} panicked", task_id);
        }
        
        self.state.tasks_executed.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Exponential backoff when no work is available
    fn backoff(&self, count: &mut u32) -> bool {
        const MAX_SPINS: u32 = 10;
        const MAX_YIELDS: u32 = 20;
        
        *count += 1;
        
        if *count <= MAX_SPINS {
            // Spin a bit
            let spins = (*count).min(6);
            for _ in 0..(1 << spins) {
                std::hint::spin_loop();
            }
        } else if *count <= MAX_YIELDS {
            // Yield to OS scheduler
            thread::yield_now();
        } else {
            // Park the thread for a short duration
            thread::park_timeout(Duration::from_micros(100));
        }
        
        false // Not shutting down
    }
}
