//! Task representation and execution.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Global task ID counter
static TASK_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Unique identifier for a task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(u64);

impl TaskId {
    fn next() -> Self {
        TaskId(TASK_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Priority level for task scheduling
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Realtime = 0,
    High = 1,
    Normal = 2,
    Low = 3,
    Background = 4,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

/// Internal task representation
pub(crate) struct Task {
    pub(crate) id: TaskId,
    pub(crate) func: Box<dyn FnOnce() + Send + 'static>,
    pub(crate) priority: Priority,
    pub(crate) spawn_time: Instant,
}

impl Task {
    /// Create a new task with normal priority
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        Task {
            id: TaskId::next(),
            func: Box::new(f),
            priority: Priority::Normal,
            spawn_time: Instant::now(),
        }
    }
    
    /// Create a task with specific priority
    pub fn with_priority<F>(f: F, priority: Priority) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        Task {
            id: TaskId::next(),
            func: Box::new(f),
            priority,
            spawn_time: Instant::now(),
        }
    }
    
    /// Execute the task
    pub fn execute(self) {
        (self.func)();
    }
}

impl std::fmt::Debug for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Task")
            .field("id", &self.id)
            .field("priority", &self.priority)
            .field("spawn_time", &self.spawn_time)
            .finish()
    }
}
