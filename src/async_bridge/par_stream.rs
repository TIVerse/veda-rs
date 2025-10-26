//! Parallel async stream processing.

use crate::runtime;
use futures::{Stream, Future};
use std::pin::Pin;
use std::sync::Arc;
use async_channel::{bounded, Sender, Receiver};
use parking_lot::Mutex;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Extension trait for parallel stream processing
pub trait ParStreamExt: Stream + Sized {
    /// Process stream items in parallel
    fn par_for_each<F, Fut>(self, f: F) -> ParForEach<Self, F>
    where
        F: Fn(Self::Item) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
        Self::Item: Send + 'static;
    
    /// Map stream items in parallel
    fn par_map<F, Fut, R>(self, f: F) -> ParMap<Self, F>
    where
        F: Fn(Self::Item) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: Send + 'static,
        Self::Item: Send + 'static;
    
    /// Filter stream items in parallel
    fn par_filter<F, Fut>(self, f: F) -> ParFilter<Self, F>
    where
        F: Fn(&Self::Item) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = bool> + Send + 'static,
        Self::Item: Send + 'static;
}

impl<S> ParStreamExt for S
where
    S: Stream + Send + 'static,
    S::Item: Send + 'static,
{
    fn par_for_each<F, Fut>(self, f: F) -> ParForEach<Self, F>
    where
        F: Fn(Self::Item) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        ParForEach {
            stream: self,
            func: Arc::new(f),
            buffer_size: 1000,
            max_concurrency: num_cpus::get(),
        }
    }
    
    fn par_map<F, Fut, R>(self, f: F) -> ParMap<Self, F>
    where
        F: Fn(Self::Item) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: Send + 'static,
    {
        ParMap {
            stream: self,
            func: Arc::new(f),
            buffer_size: 1000,
            max_concurrency: num_cpus::get(),
        }
    }
    
    fn par_filter<F, Fut>(self, f: F) -> ParFilter<Self, F>
    where
        F: Fn(&Self::Item) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = bool> + Send + 'static,
    {
        ParFilter {
            stream: self,
            func: Arc::new(f),
            buffer_size: 1000,
            max_concurrency: num_cpus::get(),
        }
    }
}

pub struct ParForEach<S, F> {
    stream: S,
    func: Arc<F>,
    buffer_size: usize,
    max_concurrency: usize,
}

impl<S, F, Fut> ParForEach<S, F>
where
    S: Stream + Send + 'static,
    S::Item: Send + 'static,
    F: Fn(S::Item) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    pub async fn execute(self) {
        use futures::StreamExt;
        use crate::runtime;
        
        let (tx, rx): (Sender<S::Item>, Receiver<S::Item>) = bounded(self.buffer_size);
        let func = self.func.clone();
        
        // Spawn consumer tasks on VEDA runtime
        let completion_signals: Vec<_> = (0..self.max_concurrency)
            .map(|_| {
                let rx = rx.clone();
                let func = func.clone();
                let (done_tx, done_rx) = bounded(1);
                
                runtime::with_current_runtime(|rt| {
                    rt.pool.execute(move || {
                        futures::executor::block_on(async {
                            while let Ok(item) = rx.recv().await {
                                func(item).await;
                            }
                            let _ = done_tx.try_send(());
                        });
                    });
                });
                
                done_rx
            })
            .collect();
        
        // Producer: feed stream items
        let mut stream = Box::pin(self.stream);
        while let Some(item) = stream.next().await {
            if tx.send(item).await.is_err() {
                break;
            }
        }
        
        drop(tx); // Signal completion
        
        // Wait for all consumers
        for done_rx in completion_signals {
            let _ = done_rx.recv().await;
        }
    }
}

pub struct ParMap<S, F> {
    stream: S,
    func: Arc<F>,
    buffer_size: usize,
    max_concurrency: usize,
}

impl<S, F, Fut, R> ParMap<S, F>
where
    S: Stream + Send + 'static,
    S::Item: Send + 'static,
    F: Fn(S::Item) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: Send + 'static,
{
    pub async fn collect(self) -> Vec<R> {
        use futures::StreamExt;
        use crate::runtime;
        
        let results = Arc::new(Mutex::new(Vec::new()));
        let (tx, rx): (Sender<(usize, S::Item)>, Receiver<(usize, S::Item)>) = bounded(self.buffer_size);
        let func = self.func.clone();
        
        // Spawn consumer tasks on VEDA runtime
        let completion_signals: Vec<_> = (0..self.max_concurrency)
            .map(|_| {
                let rx = rx.clone();
                let func = func.clone();
                let results = results.clone();
                let (done_tx, done_rx) = bounded(1);
                
                runtime::with_current_runtime(|rt| {
                    rt.pool.execute(move || {
                        futures::executor::block_on(async {
                            while let Ok((idx, item)) = rx.recv().await {
                                let result = func(item).await;
                                results.lock().push((idx, result));
                            }
                            let _ = done_tx.try_send(());
                        });
                    });
                });
                
                done_rx
            })
            .collect();
        
        // Producer: feed stream items with index
        let mut stream = Box::pin(self.stream);
        let mut idx = 0;
        while let Some(item) = stream.next().await {
            if tx.send((idx, item)).await.is_err() {
                break;
            }
            idx += 1;
        }
        
        drop(tx);
        
        // Wait for all consumers
        for done_rx in completion_signals {
            let _ = done_rx.recv().await;
        }
        
        // Sort by index and extract values
        let mut result_vec = match Arc::try_unwrap(results) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => {
                let mut guard = arc.lock();
                // Move items out instead of cloning
                std::mem::take(&mut *guard)
            }
        };
        result_vec.sort_by_key(|(idx, _)| *idx);
        result_vec.into_iter().map(|(_, r)| r).collect()
    }
}

pub struct ParFilter<S, F> {
    stream: S,
    func: Arc<F>,
    buffer_size: usize,
    max_concurrency: usize,
}

impl<S, F, Fut> ParFilter<S, F>
where
    S: Stream + Send + 'static,
    S::Item: Send + Clone + 'static,
    F: Fn(&S::Item) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = bool> + Send + 'static,
{
    pub async fn collect(self) -> Vec<S::Item> {
        use futures::StreamExt;
        use crate::runtime;
        
        let results = Arc::new(Mutex::new(Vec::new()));
        let (tx, rx): (Sender<S::Item>, Receiver<S::Item>) = bounded(self.buffer_size);
        let func = self.func.clone();
        
        // Spawn consumer tasks on VEDA runtime
        let completion_signals: Vec<_> = (0..self.max_concurrency)
            .map(|_| {
                let rx = rx.clone();
                let func = func.clone();
                let results = results.clone();
                let (done_tx, done_rx) = bounded(1);
                
                runtime::with_current_runtime(|rt| {
                    rt.pool.execute(move || {
                        futures::executor::block_on(async {
                            while let Ok(item) = rx.recv().await {
                                if func(&item).await {
                                    results.lock().push(item);
                                }
                            }
                            let _ = done_tx.try_send(());
                        });
                    });
                });
                
                done_rx
            })
            .collect();
        
        // Producer: feed stream items
        let mut stream = Box::pin(self.stream);
        while let Some(item) = stream.next().await {
            if tx.send(item).await.is_err() {
                break;
            }
        }
        
        drop(tx);
        
        // Wait for all consumers
        for done_rx in completion_signals {
            let _ = done_rx.recv().await;
        }
        
        // Extract results
        match Arc::try_unwrap(results) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => {
                let mut guard = arc.lock();
                std::mem::take(&mut *guard)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream;
    
    #[test]
    fn test_par_for_each() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        
        crate::runtime::shutdown();
        crate::runtime::init().unwrap();
        
        let counter = Arc::new(AtomicUsize::new(0));
        let items = stream::iter(0..10);
        
        let counter_clone = counter.clone();
        let fut = items.par_for_each(move |_| {
            let c = counter_clone.clone();
            async move {
                c.fetch_add(1, Ordering::Relaxed);
            }
        }).execute();
        
        futures::executor::block_on(fut);
        assert_eq!(counter.load(Ordering::Relaxed), 10);
        
        crate::runtime::shutdown();
    }
    
    #[test]
    fn test_par_map() {
        crate::runtime::shutdown();
        crate::runtime::init().unwrap();
        
        let items = stream::iter(0..10);
        let fut = items.par_map(|x| async move { x * 2 }).collect();
        
        let results = futures::executor::block_on(fut);
        assert_eq!(results.len(), 10);
        assert!(results.contains(&0));
        assert!(results.contains(&18));
        
        crate::runtime::shutdown();
    }
    
    #[test]
    fn test_par_filter() {
        crate::runtime::shutdown();
        crate::runtime::init().unwrap();
        
        let items = stream::iter(0..10);
        let fut = items.par_filter(|x| async move { *x % 2 == 0 }).collect();
        
        let results = futures::executor::block_on(fut);
        assert_eq!(results.len(), 5);
        assert!(results.contains(&0));
        assert!(results.contains(&8));
        
        crate::runtime::shutdown();
    }
}
