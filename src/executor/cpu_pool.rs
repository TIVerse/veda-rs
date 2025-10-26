use super::task::{Priority, Task};
use super::worker::{Worker, WorkerId};
use crate::config::Config;
use crate::error::{Error, Result};
use crate::scheduler::priority::PriorityQueue;
use crate::util::{BackpressureConfig, BackpressureController};
use crossbeam_deque::{Injector, Stealer};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Instant;

#[cfg(feature = "telemetry")]
use crate::telemetry::Metrics;

#[cfg(target_os = "linux")]
fn pin_thread_to_core(core_id: usize) {
    unsafe {
        let mut cpuset: libc::cpu_set_t = std::mem::zeroed();
        libc::CPU_SET(core_id, &mut cpuset);
        let result = libc::sched_setaffinity(
            0, // current thread
            std::mem::size_of::<libc::cpu_set_t>(),
            &cpuset,
        );
        if result != 0 {
            eprintln!(
                "Failed to pin thread {} to core {}",
                std::thread::current().name().unwrap_or("unknown"),
                core_id
            );
        }
    }
}

pub struct CpuPool {
    workers: Vec<WorkerHandle>,
    injector: Arc<Injector<Task>>,
    priority_queue: Arc<PriorityQueue>,
    stealers: Vec<Stealer<Task>>,
    shutdown: Arc<AtomicBool>,
    num_threads: usize,
    pending_tasks: Arc<AtomicUsize>,
    backpressure: Arc<BackpressureController>,
    #[cfg(feature = "telemetry")]
    pub(crate) metrics: Arc<Metrics>,
}

struct WorkerHandle {
    id: WorkerId,
    thread: Option<JoinHandle<()>>,
    unparker: thread::Thread,
}

impl CpuPool {
    pub fn new(config: &Config) -> Result<Self> {
        let num_threads = config.worker_threads();
        if num_threads == 0 {
            return Err(Error::config("need at least 1 thread"));
        }

        let injector = Arc::new(Injector::new());
        let priority_queue = Arc::new(PriorityQueue::new());
        let shutdown = Arc::new(AtomicBool::new(false));
        let pending_tasks = Arc::new(AtomicUsize::new(0));

        // Initialize backpressure controller
        let backpressure = Arc::new(BackpressureController::new(BackpressureConfig {
            max_queue_size: 10_000,
            target_latency_ms: 100,
            rate_limit_per_sec: None,
            backoff_factor: 0.5,
        }));

        #[cfg(feature = "telemetry")]
        let metrics = Arc::new(Metrics::new());

        let mut workers = Vec::with_capacity(num_threads);
        let mut stealers = Vec::with_capacity(num_threads);

        for id in 0..num_threads {
            let worker = Worker::new(id);
            stealers.push(worker.local_queue.stealer());
            workers.push(worker);
        }

        let mut handles = Vec::with_capacity(num_threads);

        for worker in workers {
            let id = worker.id;
            let stealers_clone = stealers.clone();
            let injector_clone = injector.clone();
            let priority_queue_clone = priority_queue.clone();
            let shutdown_clone = shutdown.clone();
            let pending_clone = pending_tasks.clone();
            let name = format!("{}-{}", config.thread_name_prefix, id);

            #[cfg(feature = "telemetry")]
            let worker = worker.with_metrics(metrics.clone());

            let mut builder = thread::Builder::new().name(name);

            if let Some(stack_size) = config.stack_size {
                builder = builder.stack_size(stack_size);
            }

            let pin_workers = config.pin_workers;
            let thread = builder
                .spawn(move || {
                    // Pin worker to core if requested
                    #[cfg(target_os = "linux")]
                    if pin_workers {
                        pin_thread_to_core(id);
                    }

                    worker.run(
                        stealers_clone,
                        injector_clone,
                        priority_queue_clone,
                        shutdown_clone,
                        pending_clone,
                    );
                })
                .map_err(|e| Error::executor(format!("spawn failed: {}", e)))?;

            let unparker = thread.thread().clone();

            handles.push(WorkerHandle {
                id,
                thread: Some(thread),
                unparker,
            });
        }

        Ok(Self {
            workers: handles,
            injector,
            priority_queue,
            stealers,
            shutdown,
            num_threads,
            pending_tasks,
            backpressure,
            #[cfg(feature = "telemetry")]
            metrics,
        })
    }

    pub fn submit(&self, task: Task) {
        self.submit_with_priority(task, Priority::Normal);
    }

    pub fn submit_with_priority(&self, task: Task, priority: Priority) {
        // Apply backpressure control
        if !self.backpressure.on_enqueue() {
            // Queue is full, task rejected
            if cfg!(debug_assertions) {
                eprintln!("[VEDA] Task rejected due to backpressure (queue full)");
            }
            return;
        }

        self.pending_tasks.fetch_add(1, Ordering::Relaxed);

        if priority == Priority::Normal {
            // Normal priority goes to injector for work stealing
            self.injector.push(task);
        } else {
            // High/Low/Realtime/Background go to priority queue
            self.priority_queue.push(task, priority);
        }

        // Wake up a worker
        if let Some(worker) = self.workers.get(self.num_threads / 2) {
            worker.unparker.unpark();
        }
    }

    pub fn submit_with_deadline(&self, task: Task, priority: Priority, deadline: Instant) {
        self.pending_tasks.fetch_add(1, Ordering::Relaxed);
        self.priority_queue
            .push_with_deadline(task, priority, deadline);

        // Wake up urgently for deadline tasks
        if let Some(worker) = self.workers.first() {
            worker.unparker.unpark();
        }
    }

    pub fn pending_tasks(&self) -> usize {
        self.pending_tasks.load(Ordering::Relaxed)
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let task = Task::new(f);
        self.submit(task);
    }

    pub fn num_threads(&self) -> usize {
        self.num_threads
    }

    pub fn shutdown(&mut self) {
        self.shutdown.store(true, Ordering::Release);

        // wake everyone up to check shutdown flag
        for worker in &self.workers {
            worker.unparker.unpark();
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                let _ = thread.join();
            }
        }
    }
}

impl Drop for CpuPool {
    fn drop(&mut self) {
        self.shutdown();
    }
}
