use crate::config::Config;
use crate::error::{Error, Result};
use crate::executor::CpuPool;
use crate::scheduler::SchedulerCoordinator;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::{Mutex, OnceLock};
use std::thread::ThreadId;

pub struct Runtime {
    pub(crate) pool: Arc<CpuPool>,
    pub(crate) scheduler: Arc<SchedulerCoordinator>,
    config: Config,
}

impl Runtime {
    pub fn new(config: Config) -> Result<Self> {
        config.validate()?;

        let pool = CpuPool::new(&config)?;
        let scheduler = SchedulerCoordinator::new(&config)?;

        Ok(Self {
            pool: Arc::new(pool),
            scheduler: Arc::new(scheduler),
            config,
        })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}

// Global runtime for simple API
static GLOBAL_RUNTIME: OnceLock<RwLock<Option<Arc<Runtime>>>> = OnceLock::new();

fn get_global_runtime() -> &'static RwLock<Option<Arc<Runtime>>> {
    GLOBAL_RUNTIME.get_or_init(|| RwLock::new(None))
}

// Thread-local runtime for isolated tests
thread_local! {
    static THREAD_RUNTIME: std::cell::RefCell<Option<Arc<Runtime>>> = std::cell::RefCell::new(None);
}

// Track which threads have thread-local runtimes
static THREAD_RUNTIME_MAP: OnceLock<Mutex<HashMap<ThreadId, bool>>> = OnceLock::new();

fn get_thread_runtime_map() -> &'static Mutex<HashMap<ThreadId, bool>> {
    THREAD_RUNTIME_MAP.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Lazy initialization flag
static LAZY_INIT_ENABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// Enable or disable lazy initialization (default: enabled)
pub fn set_lazy_init(enabled: bool) {
    LAZY_INIT_ENABLED.store(enabled, std::sync::atomic::Ordering::Release);
}

/// Initialize runtime lazily if not already initialized
fn ensure_runtime_initialized() {
    if !LAZY_INIT_ENABLED.load(std::sync::atomic::Ordering::Acquire) {
        return;
    }

    let thread_id = std::thread::current().id();
    let has_thread_local = get_thread_runtime_map()
        .lock()
        .unwrap()
        .get(&thread_id)
        .copied()
        .unwrap_or(false);

    if has_thread_local {
        // Check thread-local
        let has_runtime = THREAD_RUNTIME.with(|rt| rt.borrow().is_some());
        if !has_runtime {
            let _ = init_thread_local();
        }
    } else {
        // Check global
        let runtime = get_global_runtime().read();
        if runtime.is_none() {
            drop(runtime); // Release read lock
            let _ = init(); // This will acquire write lock
        }
    }
}

pub fn init() -> Result<()> {
    init_with_config(Config::default())
}

pub fn init_with_config(config: Config) -> Result<()> {
    let thread_id = std::thread::current().id();

    // Check if this thread already has a thread-local runtime
    let has_thread_local = get_thread_runtime_map()
        .lock()
        .unwrap()
        .get(&thread_id)
        .copied()
        .unwrap_or(false);

    if has_thread_local {
        // Use thread-local runtime
        let has_existing = THREAD_RUNTIME.with(|rt| rt.borrow().is_some());
        if has_existing {
            return Err(Error::AlreadyInitialized);
        }

        let rt = Runtime::new(config)?;
        THREAD_RUNTIME.with(|rt_cell| {
            *rt_cell.borrow_mut() = Some(Arc::new(rt));
        });

        Ok(())
    } else {
        // Use global runtime
        let mut runtime = get_global_runtime().write();

        if runtime.is_some() {
            return Err(Error::AlreadyInitialized);
        }

        let rt = Runtime::new(config)?;
        *runtime = Some(Arc::new(rt));

        Ok(())
    }
}

/// Initialize runtime in thread-local mode (for tests)
pub fn init_thread_local() -> Result<()> {
    init_thread_local_with_config(Config::default())
}

/// Initialize runtime in thread-local mode with config (for tests)
pub fn init_thread_local_with_config(config: Config) -> Result<()> {
    let thread_id = std::thread::current().id();
    get_thread_runtime_map()
        .lock()
        .unwrap()
        .insert(thread_id, true);

    let has_existing = THREAD_RUNTIME.with(|rt| rt.borrow().is_some());
    if has_existing {
        return Err(Error::AlreadyInitialized);
    }

    let rt = Runtime::new(config)?;
    THREAD_RUNTIME.with(|rt_cell| {
        *rt_cell.borrow_mut() = Some(Arc::new(rt));
    });

    Ok(())
}

pub(crate) fn current_runtime() -> Arc<Runtime> {
    // Lazy initialize if enabled
    ensure_runtime_initialized();

    let thread_id = std::thread::current().id();
    let has_thread_local = get_thread_runtime_map()
        .lock()
        .unwrap()
        .get(&thread_id)
        .copied()
        .unwrap_or(false);

    if has_thread_local {
        THREAD_RUNTIME.with(|rt| {
            rt.borrow()
                .as_ref()
                .expect("VEDA runtime not initialized - call veda::init() first")
                .clone()
        })
    } else {
        get_global_runtime()
            .read()
            .as_ref()
            .expect("VEDA runtime not initialized - call veda::init() first")
            .clone()
    }
}

pub(crate) fn with_current_runtime<F, R>(f: F) -> R
where
    F: FnOnce(&Runtime) -> R,
{
    let rt = current_runtime();
    f(&rt)
}

pub fn shutdown() {
    let thread_id = std::thread::current().id();
    let has_thread_local = get_thread_runtime_map()
        .lock()
        .unwrap()
        .get(&thread_id)
        .copied()
        .unwrap_or(false);

    if has_thread_local {
        THREAD_RUNTIME.with(|rt_cell| {
            *rt_cell.borrow_mut() = None;
        });
        get_thread_runtime_map().lock().unwrap().remove(&thread_id);
    } else {
        let mut runtime = get_global_runtime().write();
        *runtime = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_init() {
        shutdown();

        let result = init();
        assert!(result.is_ok());

        let result2 = init();
        assert!(result2.is_err());

        shutdown();
    }

    #[test]
    fn test_custom_config() {
        shutdown();

        let config = Config::builder().num_threads(2).build().unwrap();

        init_with_config(config).unwrap();

        let rt = current_runtime();
        assert_eq!(rt.pool.num_threads(), 2);

        shutdown();
    }
}
