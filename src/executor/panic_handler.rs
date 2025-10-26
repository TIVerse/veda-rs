//! Panic isolation and recovery for tasks.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Strategy for handling panicked tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanicStrategy {
    /// Abort the entire program on panic
    Abort,
    /// Isolate the panic and continue
    Isolate,
    /// Log and continue
    LogAndContinue,
}

impl Default for PanicStrategy {
    fn default() -> Self {
        PanicStrategy::LogAndContinue
    }
}

/// Panic handler for task execution
pub struct PanicHandler {
    strategy: PanicStrategy,
    panic_count: AtomicUsize,
}

impl PanicHandler {
    /// Create a new panic handler
    pub fn new(strategy: PanicStrategy) -> Self {
        Self {
            strategy,
            panic_count: AtomicUsize::new(0),
        }
    }
    
    /// Execute a closure with panic handling
    pub fn execute<F, R>(&self, f: F) -> Result<R, PanicInfo>
    where
        F: FnOnce() -> R + std::panic::UnwindSafe,
    {
        match catch_unwind(AssertUnwindSafe(f)) {
            Ok(result) => Ok(result),
            Err(panic_payload) => {
                self.panic_count.fetch_add(1, Ordering::Relaxed);
                
                let panic_info = PanicInfo::from_payload(panic_payload);
                
                match self.strategy {
                    PanicStrategy::Abort => {
                        eprintln!("VEDA: Task panicked (abort strategy)");
                        std::process::abort();
                    }
                    PanicStrategy::Isolate => {
                        // Just return the error
                    }
                    PanicStrategy::LogAndContinue => {
                        eprintln!("VEDA: Task panicked: {}", panic_info.message);
                        if let Some(location) = &panic_info.location {
                            eprintln!("  at {}", location);
                        }
                    }
                }
                
                Err(panic_info)
            }
        }
    }
    
    /// Get total number of panics
    pub fn panic_count(&self) -> usize {
        self.panic_count.load(Ordering::Relaxed)
    }
    
    /// Reset panic counter
    pub fn reset_count(&self) {
        self.panic_count.store(0, Ordering::Relaxed);
    }
    
    /// Get the strategy
    pub fn strategy(&self) -> PanicStrategy {
        self.strategy
    }
}

impl Default for PanicHandler {
    fn default() -> Self {
        Self::new(PanicStrategy::default())
    }
}

/// Information about a panic
#[derive(Debug, Clone)]
pub struct PanicInfo {
    pub message: String,
    pub location: Option<String>,
}

impl PanicInfo {
    /// Create panic info from a panic payload
    fn from_payload(payload: Box<dyn std::any::Any + Send>) -> Self {
        let message = if let Some(s) = payload.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };
        
        Self {
            message,
            location: None, // Could be enhanced with backtrace
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_panic_handler_isolate() {
        let handler = PanicHandler::new(PanicStrategy::Isolate);
        
        let result = handler.execute(|| {
            panic!("test panic");
        });
        
        assert!(result.is_err());
        assert_eq!(handler.panic_count(), 1);
    }
    
    #[test]
    fn test_panic_handler_success() {
        let handler = PanicHandler::new(PanicStrategy::Isolate);
        
        let result = handler.execute(|| 42);
        
        assert_eq!(result.unwrap(), 42);
        assert_eq!(handler.panic_count(), 0);
    }
    
    #[test]
    fn test_panic_counter() {
        let handler = PanicHandler::new(PanicStrategy::LogAndContinue);
        
        for _ in 0..5 {
            let _ = handler.execute(|| {
                panic!("test");
            });
        }
        
        assert_eq!(handler.panic_count(), 5);
        
        handler.reset_count();
        assert_eq!(handler.panic_count(), 0);
    }
}
