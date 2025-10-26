use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

pub use crate::scheduler::priority::Priority;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(u64);

impl TaskId {
    pub fn new() -> Self {
        static TASK_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
        TaskId(TASK_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

pub(crate) struct Task {
    pub(crate) id: TaskId,
    pub(crate) func: Box<dyn FnOnce() + Send + 'static>,
    pub(crate) priority: Priority,
    pub(crate) spawn_time: Instant,
    pub(crate) deadline: Option<Instant>,
}

impl Task {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        Task {
            id: TaskId::new(),
            func: Box::new(f),
            priority: Priority::Normal,
            spawn_time: Instant::now(),
            deadline: None,
        }
    }

    pub fn with_priority<F>(f: F, priority: Priority) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        Task {
            id: TaskId::new(),
            func: Box::new(f),
            priority,
            spawn_time: Instant::now(),
            deadline: None,
        }
    }

    pub fn with_deadline(mut self, deadline: Instant) -> Self {
        self.deadline = Some(deadline);
        self
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
            .field("deadline", &self.deadline)
            .finish()
    }
}
