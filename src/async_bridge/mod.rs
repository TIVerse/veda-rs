//! Async/await integration for VEDA runtime.
//!
//! Provides bridges between VEDA's work-stealing executor and async runtimes
//! like Tokio, enabling seamless use of async code within parallel workloads.

pub mod spawn;
pub mod executor_bridge;
pub mod waker;

pub use spawn::{spawn_async, block_on};
pub use executor_bridge::AsyncBridge;
pub use waker::VedaWaker;

use futures::Future;
use std::pin::Pin;

/// Type alias for boxed futures
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
