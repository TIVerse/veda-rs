use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

static TASK_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(u64);

impl TaskId {
    fn next() -> Self {
        TaskId(TASK_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

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

pub(crate) struct Task {
    pub(crate) id: TaskId,
    pub(crate) func: Box<dyn FnOnce() + Send + 'static>,
    pub(crate) priority: Priority,
    pub(crate) spawn_time: Instant,
}

impl Task {
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
