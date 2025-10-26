//! CPU thread pool implementation.
//!
//! Manages a pool of worker threads that execute tasks using work-stealing.

use super::task::Task;
use super::worker::{Worker, WorkerId};
use crate::config::Config;
use crate::error::{Error, Result};
use crossbeam_deque::{Injector, Stealer};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

/// CPU thread pool for executing tasks
pub struct CpuPool {
    workers: Vec<WorkerHandle>,
    injector: Arc<Injector<Task>>,
    stealers: Vec<Stealer<Task>>,
    shutdown: Arc<AtomicBool>,
    num_threads: usize,
}

struct WorkerHandle {
    id: WorkerId,
    thread: Option<JoinHandle<()>>,
    unparker: thread::Thread,
}

impl CpuPool {
    /// Create a new CPU pool with the given configuration
    pub fn new(config: &Config) -> Result<Self> {
        let num_threads = config.worker_threads();
        
        if num_threads == 0 {
            return Err(Error::config("num_threads must be > 0"));
        }
        
        let injector = Arc::new(Injector::new());
        let shutdown = Arc::new(AtomicBool::new(false));
        
        // Create workers
        let mut workers = Vec::with_capacity(num_threads);
        let mut stealers = Vec::with_capacity(num_threads);
        
        for id in 0..num_threads {
            let worker = Worker::new(id);
            stealers.push(worker.local_queue.stealer());
            workers.push(worker);
        }
        
        // Start worker threads
        let mut handles = Vec::with_capacity(num_threads);
        
        for worker in workers {
            let id = worker.id;
            let stealers_clone = stealers.clone();
            let injector_clone = injector.clone();
            let shutdown_clone = shutdown.clone();
            let name = format!("{}-{}", config.thread_name_prefix, id);
            
            let mut builder = thread::Builder::new().name(name);
            
            if let Some(stack_size) = config.stack_size {
                builder = builder.stack_size(stack_size);
            }
            
            let thread = builder.spawn(move || {
                worker.run(stealers_clone, injector_clone, shutdown_clone);
            }).map_err(|e| Error::executor(format!("failed to spawn worker thread: {}", e)))?;
            
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
            stealers,
            shutdown,
            num_threads,
        })
    }
    
    /// Submit a task to the pool
    pub fn submit(&self, task: Task) {
        self.injector.push(task);
        
        // Unpark a random worker to handle the task
        // This is a simple strategy - could be improved with better scheduling
        if let Some(worker) = self.workers.get(self.num_threads / 2) {
            worker.unparker.unpark();
        }
    }
    
    /// Execute a function on the pool
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let task = Task::new(f);
        self.submit(task);
    }
    
    /// Get number of worker threads
    pub fn num_threads(&self) -> usize {
        self.num_threads
    }
    
    /// Shutdown the pool and wait for all workers to finish
    pub fn shutdown(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        
        // Unpark all workers so they can see the shutdown signal
        for worker in &self.workers {
            worker.unparker.unpark();
        }
        
        // Wait for all threads to finish
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
