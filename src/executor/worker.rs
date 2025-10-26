// worker thread stuff
use super::task::Task;
use crate::scheduler::priority::PriorityQueue;
use crossbeam_deque::{Injector, Stealer, Worker as WorkerQueue};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[cfg(feature = "telemetry")]
use crate::telemetry::Metrics;

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
    #[cfg(feature = "telemetry")]
    pub metrics: Option<Arc<Metrics>>,
}

impl Worker {
    pub fn new(id: WorkerId) -> Self {
        Self {
            id,
            local_queue: WorkerQueue::new_fifo(),
            state: Arc::new(WorkerState::new()),
            #[cfg(feature = "telemetry")]
            metrics: None,
        }
    }

    #[cfg(feature = "telemetry")]
    pub fn with_metrics(mut self, metrics: Arc<Metrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    // main loop
    pub fn run(
        &self,
        stealers: Vec<Stealer<Task>>,
        injector: Arc<Injector<Task>>,
        priority_queue: Arc<PriorityQueue>,
        shutdown: Arc<AtomicBool>,
        pending_tasks: Arc<AtomicUsize>,
    ) {
        let mut backoff_cnt = 0;

        loop {
            if shutdown.load(Ordering::Acquire) {
                break;
            }

            // Priority: local -> priority queue -> global -> steal
            if let Some(task) = self.find_task(&stealers, &injector, &priority_queue) {
                backoff_cnt = 0;
                let start = Instant::now();
                self.execute_task(task);
                pending_tasks.fetch_sub(1, Ordering::Relaxed);

                // Record execution time for metrics
                #[cfg(feature = "telemetry")]
                if let Some(ref _metrics) = self.metrics {
                    let _duration_ms = start.elapsed().as_millis() as u64;
                    // Future: feed this to backpressure controller
                }
            } else {
                // nothing to do, backoff
                if self.backoff(&mut backoff_cnt) {
                    break;
                }
            }
        }
    }

    fn find_task(
        &self,
        stealers: &[Stealer<Task>],
        injector: &Injector<Task>,
        priority_queue: &PriorityQueue,
    ) -> Option<Task> {
        // 1. Check local queue first (best cache locality)
        if let Some(task) = self.local_queue.pop() {
            return Some(task);
        }

        // 2. Check priority queue for high-priority/deadline tasks
        if let Some(task) = priority_queue.pop() {
            self.state.tasks_stolen.fetch_add(1, Ordering::Relaxed);
            #[cfg(feature = "telemetry")]
            if let Some(ref metrics) = self.metrics {
                metrics.record_task_stolen();
            }
            return Some(task);
        }

        // 3. Check global injector queue
        loop {
            match injector.steal_batch_and_pop(&self.local_queue) {
                crossbeam_deque::Steal::Success(task) => {
                    self.state.tasks_stolen.fetch_add(1, Ordering::Relaxed);
                    #[cfg(feature = "telemetry")]
                    if let Some(ref metrics) = self.metrics {
                        metrics.record_task_stolen();
                    }
                    return Some(task);
                }
                crossbeam_deque::Steal::Empty => break,
                crossbeam_deque::Steal::Retry => continue,
            }
        }

        // 4. Steal from other workers
        self.try_steal_from_workers(stealers)
    }

    fn try_steal_from_workers(&self, stealers: &[Stealer<Task>]) -> Option<Task> {
        use rand::seq::SliceRandom;
        use rand::thread_rng;

        if stealers.is_empty() {
            return None;
        }

        let mut indices: Vec<usize> = (0..stealers.len()).collect();
        indices.shuffle(&mut thread_rng());

        for &idx in &indices {
            if idx == self.id {
                continue;
            }

            loop {
                match stealers[idx].steal_batch_and_pop(&self.local_queue) {
                    crossbeam_deque::Steal::Success(task) => {
                        self.state.tasks_stolen.fetch_add(1, Ordering::Relaxed);
                        #[cfg(feature = "telemetry")]
                        if let Some(ref metrics) = self.metrics {
                            metrics.record_task_stolen();
                        }
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
        let start = Instant::now();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            task.execute();
        }));

        let duration_ns = start.elapsed().as_nanos() as u64;

        match result {
            Ok(_) =>
            {
                #[cfg(feature = "telemetry")]
                if let Some(ref metrics) = self.metrics {
                    metrics.record_task_execution(duration_ns);
                }
            }
            Err(_) => {
                eprintln!("task {:?} panicked", tid);
                #[cfg(feature = "telemetry")]
                if let Some(ref metrics) = self.metrics {
                    metrics.record_task_panic();
                }
            }
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
