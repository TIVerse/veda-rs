//! Task execution infrastructure.
//!
//! This module provides the core task execution primitives including
//! worker threads, task queues, and the CPU thread pool.

pub mod cpu_pool;
pub mod panic_handler;
pub mod task;
pub mod worker;

pub use cpu_pool::CpuPool;
pub use panic_handler::{PanicHandler, PanicStrategy};
pub use task::Priority;

pub(crate) use task::Task;
