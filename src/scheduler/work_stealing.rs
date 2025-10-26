use crate::executor::Task;
use crossbeam_deque::{Injector, Stealer, Worker};
use std::sync::Arc;

pub struct WorkStealingQueue {
    pub(crate) injector: Arc<Injector<Task>>,
    pub(crate) workers: Vec<Worker<Task>>,
    pub(crate) stealers: Vec<Stealer<Task>>,
}

impl WorkStealingQueue {
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

    pub fn push_local(&self, worker_id: usize, task: Task) {
        if let Some(worker) = self.workers.get(worker_id) {
            worker.push(task);
        } else {
            self.injector.push(task);
        }
    }

    pub fn push_global(&self, task: Task) {
        self.injector.push(task);
    }

    pub fn injector(&self) -> &Arc<Injector<Task>> {
        &self.injector
    }

    pub fn worker(&self, id: usize) -> Option<&Worker<Task>> {
        self.workers.get(id)
    }

    pub fn stealers(&self) -> &[Stealer<Task>] {
        &self.stealers
    }

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

        queue.push_local(0, dummy_task());
        let worker = queue.worker(0).unwrap();
        assert!(worker.pop().is_some());
    }

    #[test]
    fn test_global_queue() {
        let queue = WorkStealingQueue::new(2);

        queue.push_global(dummy_task());
        match queue.injector().steal() {
            Steal::Success(_) => {}
            _ => panic!("Expected to steal from injector"),
        }
    }
}
