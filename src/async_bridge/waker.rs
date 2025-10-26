//! Custom waker implementation for VEDA async tasks.

use std::sync::Arc;
use std::task::{Wake, Waker};

/// Custom waker for VEDA async tasks
pub struct VedaWaker {
    task_id: usize,
}

impl VedaWaker {
    /// Create a new waker for a task
    pub fn new(task_id: usize) -> Self {
        Self { task_id }
    }

    /// Get the task ID
    pub fn task_id(&self) -> usize {
        self.task_id
    }

    /// Create a standard Waker from this VedaWaker
    pub fn into_waker(self) -> Waker {
        Arc::new(self).into()
    }
}

impl Wake for VedaWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref()
    }

    fn wake_by_ref(self: &Arc<Self>) {
        // In a full implementation, this would reschedule the task
        // For now, this is a placeholder
        eprintln!("Waking task {}", self.task_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_veda_waker() {
        let waker = VedaWaker::new(42);
        assert_eq!(waker.task_id(), 42);

        let std_waker = waker.into_waker();
        std_waker.wake();
    }
}
