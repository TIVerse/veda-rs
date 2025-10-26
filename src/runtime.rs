//! Global runtime management.
//!
//! VEDA uses a global runtime that can be initialized once and accessed
//! from anywhere in the program.

use crate::config::Config;
use crate::error::{Error, Result};
use crate::executor::CpuPool;
use parking_lot::RwLock;
use std::sync::Arc;

/// The global VEDA runtime.
pub struct Runtime {
    pub(crate) pool: Arc<CpuPool>,
    config: Config,
}

impl Runtime {
    /// Create a new runtime with the given configuration
    pub fn new(config: Config) -> Result<Self> {
        config.validate()?;
        
        let pool = CpuPool::new(&config)?;
        
        Ok(Self {
            pool: Arc::new(pool),
            config,
        })
    }
    
    /// Get the runtime configuration
    pub fn config(&self) -> &Config {
        &self.config
    }
}

// Global runtime state
static RUNTIME: RwLock<Option<Arc<Runtime>>> = RwLock::new(None);

/// Initialize the VEDA runtime with default configuration.
///
/// # Errors
///
/// Returns an error if the runtime is already initialized.
///
/// # Examples
///
/// ```no_run
/// use veda;
///
/// veda::init().unwrap();
///
/// // Now you can use parallel operations
/// ```
pub fn init() -> Result<()> {
    init_with_config(Config::default())
}

/// Initialize the VEDA runtime with custom configuration.
///
/// # Errors
///
/// Returns an error if the runtime is already initialized or if the
/// configuration is invalid.
///
/// # Examples
///
/// ```no_run
/// use veda::{Config, SchedulingPolicy};
///
/// let config = Config::builder()
///     .num_threads(4)
///     .scheduling_policy(SchedulingPolicy::Adaptive)
///     .build()
///     .unwrap();
///
/// veda::init_with_config(config).unwrap();
/// ```
pub fn init_with_config(config: Config) -> Result<()> {
    let mut runtime = RUNTIME.write();
    
    if runtime.is_some() {
        return Err(Error::AlreadyInitialized);
    }
    
    let rt = Runtime::new(config)?;
    *runtime = Some(Arc::new(rt));
    
    Ok(())
}

/// Get a reference to the current runtime.
///
/// # Panics
///
/// Panics if the runtime has not been initialized.
pub(crate) fn current_runtime() -> Arc<Runtime> {
    RUNTIME
        .read()
        .as_ref()
        .expect("VEDA runtime not initialized - call veda::init() first")
        .clone()
}

/// Execute a function with access to the current runtime.
///
/// # Panics
///
/// Panics if the runtime has not been initialized.
pub(crate) fn with_current_runtime<F, R>(f: F) -> R
where
    F: FnOnce(&Runtime) -> R,
{
    let rt = current_runtime();
    f(&rt)
}

/// Shutdown the global runtime.
///
/// This will wait for all pending tasks to complete.
pub fn shutdown() {
    let mut runtime = RUNTIME.write();
    *runtime = None;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_runtime_init() {
        shutdown(); // Clean up any existing runtime
        
        let result = init();
        assert!(result.is_ok());
        
        // Should fail to initialize twice
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
