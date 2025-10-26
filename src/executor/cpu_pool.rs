use super::task::Task;
use super::worker::{Worker, WorkerId};
use crate::config::Config;
use crate::error::{Error, Result};
use crossbeam_deque::{Injector, Stealer};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

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
    pub fn new(config: &Config) -> Result<Self> {
        let num_threads = config.worker_threads();
        if num_threads == 0 {
            return Err(Error::config("need at least 1 thread"));
        }
        
        let injector = Arc::new(Injector::new());
        let shutdown = Arc::new(AtomicBool::new(false));
        
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
            let shutdown_clone = shutdown.clone();
            let name = format!("{}-{}", config.thread_name_prefix, id);
            
            let mut builder = thread::Builder::new().name(name);
            
            if let Some(stack_size) = config.stack_size {
                builder = builder.stack_size(stack_size);
            }
            
            let thread = builder.spawn(move || {
                worker.run(stealers_clone, injector_clone, shutdown_clone);
            }).map_err(|e| Error::executor(format!("spawn failed: {}", e)))?;
            
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
    
    pub fn submit(&self, task: Task) {
        self.injector.push(task);
        
        // wake up a worker
        if let Some(worker) = self.workers.get(self.num_threads / 2) {
            worker.unparker.unpark();
        }
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
