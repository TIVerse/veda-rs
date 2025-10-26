use crate::config::Config;
use crate::error::{Error, Result};
use crate::executor::CpuPool;
use parking_lot::RwLock;
use std::sync::Arc;
use std::sync::{Mutex, OnceLock};
use std::collections::HashMap;
use std::thread::ThreadId;

pub struct Runtime {
    pub(crate) pool: Arc<CpuPool>,
    config: Config,
}

impl Runtime {
    pub fn new(config: Config) -> Result<Self> {
        config.validate()?;
        
        let pool = CpuPool::new(&config)?;
        
        Ok(Self {
            pool: Arc::new(pool),
            config,
        })
    }
    
    pub fn config(&self) -> &Config {
        &self.config
    }
}

// Global runtime for simple API
static GLOBAL_RUNTIME: RwLock<Option<Arc<Runtime>>> = RwLock::new(None);

// Thread-local runtime for isolated tests
thread_local! {
    static THREAD_RUNTIME: std::cell::RefCell<Option<Arc<Runtime>>> = std::cell::RefCell::new(None);
}

// Track which threads have thread-local runtimes
static THREAD_RUNTIME_MAP: OnceLock<Mutex<HashMap<ThreadId, bool>>> = OnceLock::new();

fn get_thread_runtime_map() -> &'static Mutex<HashMap<ThreadId, bool>> {
    THREAD_RUNTIME_MAP.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn init() -> Result<()> {
    init_with_config(Config::default())
}

pub fn init_with_config(config: Config) -> Result<()> {
    let thread_id = std::thread::current().id();
    
    // Check if this thread already has a thread-local runtime
    let has_thread_local = get_thread_runtime_map().lock().unwrap()
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
        let mut runtime = GLOBAL_RUNTIME.write();
        
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
    get_thread_runtime_map().lock().unwrap().insert(thread_id, true);
    
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
    let thread_id = std::thread::current().id();
    let has_thread_local = get_thread_runtime_map().lock().unwrap()
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
        GLOBAL_RUNTIME
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
    let has_thread_local = get_thread_runtime_map().lock().unwrap()
        .get(&thread_id)
        .copied()
        .unwrap_or(false);
    
    if has_thread_local {
        THREAD_RUNTIME.with(|rt_cell| {
            *rt_cell.borrow_mut() = None;
        });
        get_thread_runtime_map().lock().unwrap().remove(&thread_id);
    } else {
        let mut runtime = GLOBAL_RUNTIME.write();
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
        
        let config = Config::builder()
            .num_threads(2)
            .build()
            .unwrap();
        
        init_with_config(config).unwrap();
        
        let rt = current_runtime();
        assert_eq!(rt.pool.num_threads(), 2);
        
        shutdown();
    }
}
