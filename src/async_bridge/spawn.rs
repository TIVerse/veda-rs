//! Async task spawning within VEDA runtime.

use crate::runtime;
use crate::error::Result;
use futures::Future;
use std::sync::Arc;
use parking_lot::Mutex;
use async_channel::{Sender, Receiver, bounded};

/// Spawn an async task in the VEDA runtime
///
/// The future will be executed on the VEDA thread pool.
pub fn spawn_async<F, T>(future: F) -> JoinHandle<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let (sender, receiver) = bounded(1);
    
    runtime::with_current_runtime(|rt| {
        let task_future = async move {
            let result = future.await;
            let _ = sender.send(result).await;
        };
        
        // Execute the future on a worker thread
        rt.pool.execute(move || {
            futures::executor::block_on(task_future);
        });
    });
    
    JoinHandle { receiver }
}

/// Block on a future in the current thread
///
/// This is a convenience wrapper around futures::executor::block_on
pub fn block_on<F>(future: F) -> F::Output
where
    F: Future,
{
    futures::executor::block_on(future)
}

/// Handle for joining on an async task
pub struct JoinHandle<T> {
    receiver: Receiver<T>,
}

impl<T> JoinHandle<T> {
    /// Wait for the task to complete and get the result
    pub async fn join(self) -> Result<T> {
        self.receiver.recv().await
            .map_err(|_| crate::error::Error::async_error("Task was cancelled"))
    }
    
    /// Try to get the result without blocking
    pub fn try_join(&self) -> Option<T> {
        self.receiver.try_recv().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_spawn_async() {
        crate::runtime::shutdown();
        crate::runtime::init().unwrap();
        
        let handle = spawn_async(async { 42 });
        let result = block_on(handle.join()).unwrap();
        assert_eq!(result, 42);
        
        crate::runtime::shutdown();
    }
    
    #[test]
    fn test_block_on() {
        let result = block_on(async {
            let x = 10;
            let y = 32;
            x + y
        });
        
        assert_eq!(result, 42);
    }
}
