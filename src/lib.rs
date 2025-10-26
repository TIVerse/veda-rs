// VEDA - parallel runtime for Rust
// High-performance work-stealing task scheduler with intuitive parallel iterator API
#![allow(dead_code)]
pub mod config;
pub mod error;
pub mod executor;
pub mod iter;
pub mod prelude;
pub mod runtime;
pub mod runtime_manager;
pub mod scheduler;
pub mod scope;
pub mod telemetry;
pub mod util;

#[cfg(feature = "async")]
pub mod async_bridge;

#[cfg(feature = "gpu")]
pub mod gpu;

#[cfg(feature = "custom-allocators")]
pub mod memory;

pub use config::{Config, ConfigBuilder, SchedulingPolicy};
pub use error::{Error, Result};
pub use executor::task::Priority;
pub use iter::{IntoParallelIterator, ParallelIterator, ParallelSlice};
pub use runtime::{
    init, init_thread_local, init_thread_local_with_config, init_with_config, set_lazy_init,
    shutdown,
};
pub use runtime_manager::RuntimeManager;
pub use util::{BackpressureConfig, BackpressureController};

#[cfg(feature = "telemetry")]
pub use telemetry::{JsonExporter, Metrics, MetricsExporter, MetricsSnapshot};

#[cfg(feature = "async")]
pub use async_bridge::{block_on, spawn_async, ParStreamExt};

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

        let result: Vec<i32> = (0i32..10i32).into_par_iter().map(|x| x * 2).collect();

        assert_eq!(result.len(), 10);
        assert!(result.contains(&0));
        assert!(result.contains(&18));

        shutdown();
    }

    #[test]
    fn test_scope_simple() {
        shutdown();
        init().unwrap();

        use parking_lot::Mutex;
        use std::sync::Arc;

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
