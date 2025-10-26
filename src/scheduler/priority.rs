use crate::executor::Task;
use parking_lot::Mutex;
use std::cmp::Ordering as CmpOrdering;
use std::collections::BinaryHeap;
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
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

#[derive(Debug)]
pub struct PriorityTask {
    pub task: Task,
    pub priority: Priority,
    pub deadline: Option<Instant>,
    pub enqueue_time: Instant,
}

impl PriorityTask {
    pub fn new(task: Task, priority: Priority) -> Self {
        Self {
            task,
            priority,
            deadline: None,
            enqueue_time: Instant::now(),
        }
    }

    pub fn with_deadline(mut self, deadline: Instant) -> Self {
        self.deadline = Some(deadline);
        self
    }
}

impl PartialEq for PriorityTask {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.deadline == other.deadline
    }
}

impl Eq for PriorityTask {}

impl PartialOrd for PriorityTask {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityTask {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        match (self.deadline, other.deadline) {
            (Some(a), Some(b)) => {
                let deadline_cmp = b.cmp(&a);
                if deadline_cmp != CmpOrdering::Equal {
                    return deadline_cmp;
                }
            }
            (Some(_), None) => return CmpOrdering::Greater,
            (None, Some(_)) => return CmpOrdering::Less,
            (None, None) => {}
        }

        let priority_cmp = other.priority.cmp(&self.priority);
        if priority_cmp != CmpOrdering::Equal {
            return priority_cmp;
        }

        other.enqueue_time.cmp(&self.enqueue_time)
    }
}

pub struct PriorityQueue {
    heap: Arc<Mutex<BinaryHeap<PriorityTask>>>,
}

impl PriorityQueue {
    pub fn new() -> Self {
        Self {
            heap: Arc::new(Mutex::new(BinaryHeap::new())),
        }
    }

    pub fn push(&self, task: Task, priority: Priority) {
        let priority_task = PriorityTask::new(task, priority);
        self.heap.lock().push(priority_task);
    }

    pub fn push_with_deadline(&self, task: Task, priority: Priority, deadline: Instant) {
        let priority_task = PriorityTask::new(task, priority).with_deadline(deadline);
        self.heap.lock().push(priority_task);
    }

    pub fn pop(&self) -> Option<Task> {
        self.heap.lock().pop().map(|pt| pt.task)
    }

    pub fn is_empty(&self) -> bool {
        self.heap.lock().is_empty()
    }

    pub fn len(&self) -> usize {
        self.heap.lock().len()
    }

    pub fn peek(&self) -> Option<Priority> {
        self.heap.lock().peek().map(|pt| pt.priority)
    }
}

impl Default for PriorityQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for PriorityQueue {
    fn clone(&self) -> Self {
        Self {
            heap: Arc::clone(&self.heap),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_task() -> Task {
        Task::new(Box::new(|| {}))
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Realtime < Priority::High);
        assert!(Priority::High < Priority::Normal);
        assert!(Priority::Normal < Priority::Low);
        assert!(Priority::Low < Priority::Background);
    }

    #[test]
    fn test_priority_queue() {
        let queue = PriorityQueue::new();

        queue.push(dummy_task(), Priority::Low);
        queue.push(dummy_task(), Priority::Realtime);
        queue.push(dummy_task(), Priority::Normal);

        assert_eq!(queue.peek(), Some(Priority::Realtime));
        queue.pop();
        assert_eq!(queue.peek(), Some(Priority::Normal));
        queue.pop();
        assert_eq!(queue.peek(), Some(Priority::Low));
    }

    #[test]
    fn test_deadline_priority() {
        let queue = PriorityQueue::new();
        let now = Instant::now();

        queue.push_with_deadline(
            dummy_task(),
            Priority::Normal,
            now + std::time::Duration::from_secs(10),
        );

        queue.push_with_deadline(
            dummy_task(),
            Priority::Normal,
            now + std::time::Duration::from_secs(1),
        );

        let task1 = queue.pop();
        assert!(task1.is_some());

        let task2 = queue.pop();
        assert!(task2.is_some());
    }
}
