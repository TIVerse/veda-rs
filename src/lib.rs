//! VEDA - Versatile Execution and Dynamic Adaptation
//!
//! A next-generation parallel runtime library for Rust that combines adaptive
//! scheduling, heterogeneous compute support, and comprehensive observability.
//!
//! # Quick Start
//!
//! ```no_run
//! use veda::prelude::*;
//!
//! // Initialize the runtime
//! veda::init().unwrap();
//!
//! // Use parallel iterators (Rayon-compatible API)
//! let sum: i32 = (0..1000)
//!     .into_par_iter()
//!     .map(|x| x * 2)
//!     .sum();
//!
//! println!("Sum: {}", sum);
//! ```
//!
//! # Features
//!
//! - **Adaptive Thread Pools**: Dynamic worker scaling based on load
//! - **Work Stealing**: Efficient task distribution with randomized stealing
//! - **Rayon Compatibility**: Drop-in replacement for Rayon's parallel iterators
//! - **Scoped Parallelism**: Safe task spawning with lifetime guarantees
//! - **Telemetry**: Rich metrics and observability (optional)
//! - **GPU Support**: Automatic CPU/GPU task distribution (optional)
//! - **Async Integration**: Seamless async/await support (optional)
//! - **Deterministic Mode**: Reproducible execution for testing (optional)

// Lint configuration
#![warn(missing_docs, missing_debug_implementations)]
#![allow(dead_code)] // During development

// Core modules - always available
pub mod config;
pub mod error;
pub mod executor;
pub mod iter;
pub mod prelude;
pub mod runtime;
pub mod scheduler;
pub mod scope;
pub mod telemetry;
pub mod util;

// Optional modules
#[cfg(feature = "async")]
pub mod async_bridge;

#[cfg(feature = "gpu")]
pub mod gpu;

#[cfg(feature = "custom-allocators")]
pub mod memory;

// Re-export key types at crate root
pub use config::{Config, ConfigBuilder, SchedulingPolicy};
pub use error::{Error, Result};
pub use iter::{IntoParallelIterator, ParallelIterator};
pub use runtime::{init, init_with_config, shutdown};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parallel_sum() {
        shutdown();
        init().unwrap();
        
        let sum: i32 = (0i32..100i32).into_par_iter().sum();
        assert_eq!(sum, 4950);
        
        shutdown();
    }
    
    #[test]
    fn test_parallel_map() {
        shutdown();
        init().unwrap();
        
        let result: Vec<i32> = (0i32..10i32)
            .into_par_iter()
            .map(|x| x * 2)
            .collect();
        
        assert_eq!(result.len(), 10);
        assert!(result.contains(&0));
        assert!(result.contains(&18));
        
        shutdown();
    }
    
    #[test]
    fn test_scope_simple() {
        shutdown();
        init().unwrap();
        
        use std::sync::Arc;
        use parking_lot::Mutex;
        
        let counter = Arc::new(Mutex::new(0));
        
        scope::scope(|s| {
            for _ in 0..10 {
                let counter = counter.clone();
                s.spawn(move || {
                    *counter.lock() += 1;
                });
            }
        });
        
        assert_eq!(*counter.lock(), 10);
        
        shutdown();
    }
}
