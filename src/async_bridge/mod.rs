pub mod spawn;
pub mod executor_bridge;
pub mod waker;

pub use spawn::{spawn_async, block_on};
pub use executor_bridge::AsyncBridge;
pub use waker::VedaWaker;

use futures::Future;
use std::pin::Pin;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
