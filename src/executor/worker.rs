// worker thread stuff
use super::task::Task;
use crossbeam_deque::{Injector, Stealer, Worker as WorkerQueue};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub type WorkerId = usize;

// stats for each worker
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
    
    // main loop
    pub fn run(
        &self,
        stealers: Vec<Stealer<Task>>,
        injector: Arc<Injector<Task>>,
        shutdown: Arc<AtomicBool>,
    ) {
        let mut backoff_cnt = 0;
        
        loop {
            if shutdown.load(Ordering::Acquire) {
                break;
            }
            
            // try local first, then global, then steal
            if let Some(task) = self.find_task(&stealers, &injector) {
                backoff_cnt = 0;
                self.execute_task(task);
            } else {
                // nothing to do, backoff
                if self.backoff(&mut backoff_cnt) {
                    break;
                }
            }
        }
    }
    
    fn find_task(&self, stealers: &[Stealer<Task>], injector: &Injector<Task>) -> Option<Task> {
        if let Some(task) = self.local_queue.pop() {
            return Some(task);
        }
        
        // check global queue
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
        
        // steal from others
        self.try_steal_from_workers(stealers)
    }
    
    fn try_steal_from_workers(&self, stealers: &[Stealer<Task>]) -> Option<Task> {
        use rand::seq::SliceRandom;
        use rand::thread_rng;
        
        if stealers.is_empty() { return None; }
        
        let mut indices: Vec<usize> = (0..stealers.len()).collect();
        indices.shuffle(&mut thread_rng());
        
        for &idx in &indices {
            if idx == self.id { continue; }
            
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
        let tid = task.id;
        
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            task.execute();
        }));
        
        if result.is_err() {
            // TODO: better panic handling
            eprintln!("task {:?} panicked", tid);
        }
        
        self.state.tasks_executed.fetch_add(1, Ordering::Relaxed);
    }
    
    fn backoff(&self, count: &mut u32) -> bool {
        const MAX_SPINS: u32 = 10;
        const MAX_YIELDS: u32 = 20;
        
        *count += 1;
        
        if *count <= MAX_SPINS {
            let spins = (*count).min(6);
            for _ in 0..(1 << spins) {
                std::hint::spin_loop();
            }
        } else if *count <= MAX_YIELDS {
            thread::yield_now();
        } else {
            thread::park_timeout(Duration::from_micros(100));
        }
        
        false
    }
}
