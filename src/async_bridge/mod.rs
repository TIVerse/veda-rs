pub mod executor_bridge;
pub mod spawn;
pub mod waker;

#[cfg(feature = "async")]
pub mod par_stream;

#[cfg(feature = "async")]
pub use par_stream::{ParFilter, ParForEach, ParMap, ParStreamExt};

pub use executor_bridge::AsyncBridge;
pub use spawn::{block_on, spawn_async};
pub use waker::VedaWaker;

use futures::Future;
use std::pin::Pin;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
