//! Work-stealing queue implementation for efficient load distribution.
//!
//! Uses crossbeam's lock-free deque for high-performance work stealing.

use crossbeam_deque::{Injector, Stealer, Worker};
use crate::executor::Task;
use std::sync::Arc;

/// Work-stealing queue for task distribution
pub struct WorkStealingQueue {
    /// Global injector queue for external task submission
    pub(crate) injector: Arc<Injector<Task>>,
    
    /// Per-worker local queues
    pub(crate) workers: Vec<Worker<Task>>,
    
    /// Stealers for each worker (used by other workers to steal)
    pub(crate) stealers: Vec<Stealer<Task>>,
}

impl WorkStealingQueue {
    /// Create a new work-stealing queue with the specified number of workers
    pub fn new(num_workers: usize) -> Self {
        let injector = Arc::new(Injector::new());
        let mut workers = Vec::with_capacity(num_workers);
        let mut stealers = Vec::with_capacity(num_workers);
        
        for _ in 0..num_workers {
            let worker = Worker::new_fifo();
            stealers.push(worker.stealer());
            workers.push(worker);
        }
        
        Self {
            injector,
            workers,
            stealers,
        }
    }
    
    /// Push a task to a specific worker's local queue
    pub fn push_local(&self, worker_id: usize, task: Task) {
        if let Some(worker) = self.workers.get(worker_id) {
            worker.push(task);
        } else {
            // Fallback to global queue if worker_id is invalid
            self.injector.push(task);
        }
    }
    
    /// Push a task to the global injector queue
    pub fn push_global(&self, task: Task) {
        self.injector.push(task);
    }
    
    /// Get a reference to the global injector
    pub fn injector(&self) -> &Arc<Injector<Task>> {
        &self.injector
    }
    
    /// Get a reference to a worker's local queue
    pub fn worker(&self, id: usize) -> Option<&Worker<Task>> {
        self.workers.get(id)
    }
    
    /// Get all stealers (for work stealing)
    pub fn stealers(&self) -> &[Stealer<Task>] {
        &self.stealers
    }
    
    /// Get number of workers
    pub fn num_workers(&self) -> usize {
        self.workers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam_deque::Steal;
    
    fn dummy_task() -> Task {
        Task::new(Box::new(|| {}))
    }
    
    #[test]
    fn test_work_stealing_queue_creation() {
        let queue = WorkStealingQueue::new(4);
        assert_eq!(queue.num_workers(), 4);
        assert_eq!(queue.stealers().len(), 4);
    }
    
    #[test]
    fn test_push_and_steal() {
        let queue = WorkStealingQueue::new(2);
        
        // Push to worker 0
        queue.push_local(0, dummy_task());
        
        // Worker 0 should be able to pop it
        let worker = queue.worker(0).unwrap();
        assert!(worker.pop().is_some());
    }
    
    #[test]
    fn test_global_queue() {
        let queue = WorkStealingQueue::new(2);
        
        // Push to global queue
        queue.push_global(dummy_task());
        
        // Should be able to steal from injector
        match queue.injector().steal() {
            Steal::Success(_) => {},
            _ => panic!("Expected to steal from injector"),
        }
    }
}
