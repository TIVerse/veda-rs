//! Bridge between VEDA and external async executors.

use crate::executor::CpuPool;
use std::sync::Arc;
use futures::Future;

/// Bridge for integrating with async executors like Tokio
pub struct AsyncBridge {
    pool: Arc<CpuPool>,
}

impl AsyncBridge {
    /// Create a new async bridge
    pub fn new(pool: Arc<CpuPool>) -> Self {
        Self { pool }
    }
    
    /// Execute a future on the VEDA thread pool
    pub fn execute<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.pool.execute(move || {
            futures::executor::block_on(future);
        });
    }
    
    /// Spawn a future and get a handle to its result
    pub fn spawn<F, T>(&self, future: F) -> async_channel::Receiver<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let (sender, receiver) = async_channel::bounded(1);
        
        self.pool.execute(move || {
            let result = futures::executor::block_on(future);
            let _ = futures::executor::block_on(sender.send(result));
        });
        
        receiver
    }
    
    /// Get reference to the underlying thread pool
    pub fn pool(&self) -> &Arc<CpuPool> {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    
    #[test]
    fn test_async_bridge() {
        let config = Config::default();
        let pool = CpuPool::new(&config).unwrap();
        let bridge = AsyncBridge::new(Arc::new(pool));
        
        let receiver = bridge.spawn(async { 42 });
        let result = futures::executor::block_on(receiver.recv()).unwrap();
        assert_eq!(result, 42);
    }
}
