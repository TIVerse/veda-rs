//! Core parallel iterator traits.
//!
//! These traits provide the foundation for data parallelism in VEDA,
//! with an API compatible with Rayon.

use crate::runtime::with_current_runtime;
use std::sync::Arc;

/// Conversion into a parallel iterator.
pub trait IntoParallelIterator {
    type Item: Send;
    type Iter: ParallelIterator<Item = Self::Item>;
    
    fn into_par_iter(self) -> Self::Iter;
}

/// A parallel iterator - the core abstraction for data parallelism.
pub trait ParallelIterator: Send + Sized {
    type Item: Send;
    
    /// Apply a function to each element in parallel.
    fn for_each<F>(self, f: F)
    where
        F: Fn(Self::Item) + Sync + Send + 'static;
    
    /// Map each element to a new value in parallel.
    fn map<F, R>(self, f: F) -> Map<Self, F>
    where
        F: Fn(Self::Item) -> R + Sync + Send,
        R: Send;
    
    /// Sum all elements (requires addition).
    fn sum<S>(self) -> S
    where
        S: Send + std::iter::Sum<Self::Item> + std::iter::Sum<S>,
        Self::Item: std::ops::Add<Output = Self::Item> + Clone;
    
    /// Collect into a container.
    fn collect<C>(self) -> C
    where
        C: std::iter::FromIterator<Self::Item>,
        Self::Item: Clone;
}

/// Map adapter for parallel iterators
pub struct Map<I, F> {
    base: I,
    map_fn: F,
}

impl<I, F, R> ParallelIterator for Map<I, F>
where
    I: ParallelIterator,
    F: Fn(I::Item) -> R + Sync + Send + 'static,
    R: Send + 'static,
{
    type Item = R;
    
    fn for_each<G>(self, consumer: G)
    where
        G: Fn(Self::Item) + Sync + Send + 'static,
    {
        let map_fn = Arc::new(self.map_fn);
        self.base.for_each(move |item| {
            let result = map_fn(item);
            consumer(result);
        });
    }
    
    fn map<G, S>(self, f: G) -> Map<Self, G>
    where
        G: Fn(Self::Item) -> S + Sync + Send,
        S: Send,
    {
        Map {
            base: self,
            map_fn: f,
        }
    }
    
    fn sum<S>(self) -> S
    where
        S: Send + std::iter::Sum<Self::Item> + std::iter::Sum<S>,
        Self::Item: std::ops::Add<Output = Self::Item> + Clone,
    {
        use parking_lot::Mutex;
        use std::sync::Arc;
        
        let partial_sums = Arc::new(Mutex::new(Vec::new()));
        let ps_clone = partial_sums.clone();
        
        self.for_each(move |item| {
            ps_clone.lock().push(item);
        });
        
        let sums = match Arc::try_unwrap(partial_sums) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => {
                let guard = arc.lock();
                (*guard).clone()
            },
        };
        
        sums.into_iter().sum()
    }
    
    fn collect<C>(self) -> C
    where
        C: std::iter::FromIterator<Self::Item>,
        Self::Item: Clone,
    {
        use parking_lot::Mutex;
        use std::sync::Arc;
        
        let results = Arc::new(Mutex::new(Vec::new()));
        let res_clone = results.clone();
        
        self.for_each(move |item| {
            res_clone.lock().push(item);
        });
        
        let items = match Arc::try_unwrap(results) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => {
                let guard = arc.lock();
                (*guard).clone()
            },
        };
        
        items.into_iter().collect()
    }
}

/// Parallel iterator over a range
pub struct RangeIter<T> {
    range: std::ops::Range<T>,
}

// Trait for calculating range length - handles different integer types
trait RangeLenCalc {
    fn range_len(&self) -> usize;
}

impl RangeLenCalc for std::ops::Range<i32> {
    fn range_len(&self) -> usize {
        (self.end - self.start) as usize
    }
}

impl RangeLenCalc for std::ops::Range<i64> {
    fn range_len(&self) -> usize {
        (self.end - self.start) as usize
    }
}

impl RangeLenCalc for std::ops::Range<u32> {
    fn range_len(&self) -> usize {
        (self.end - self.start) as usize
    }
}

impl RangeLenCalc for std::ops::Range<u64> {
    fn range_len(&self) -> usize {
        (self.end - self.start) as usize
    }
}

impl RangeLenCalc for std::ops::Range<usize> {
    fn range_len(&self) -> usize {
        self.end - self.start
    }
}

impl<T> ParallelIterator for RangeIter<T>
where
    T: Send + Copy + std::cmp::PartialOrd + std::cmp::Ord + std::ops::Add<Output = T> + From<u8> + 'static,
    std::ops::Range<T>: RangeLenCalc,
{
    type Item = T;
    
    fn for_each<F>(self, f: F)
    where
        F: Fn(Self::Item) + Sync + Send + 'static,
    {
        let range = self.range;
        let f = Arc::new(f);
        
        // Simple chunking strategy
        with_current_runtime(|rt| {
            let num_threads = rt.pool.num_threads();
            let start = range.start;
            let end = range.end;
            
            let len = range.range_len();
            let chunk_size = (len / num_threads).max(1);
            
            let mut current = start;
            let mut handles = Vec::new();
            
            while current < end {
                // Calculate chunk end, being careful about overflow
                let chunk_end = {
                    let mut tmp = current;
                    for _ in 0..chunk_size {
                        tmp = tmp + T::from(1);
                        if tmp >= end {
                            break;
                        }
                    }
                    std::cmp::min(tmp, end)
                };
                
                let f_clone = f.clone();
                let (tx, rx) = crossbeam_channel::bounded(0);
                
                rt.pool.execute(move || {
                    let mut i = current;
                    while i < chunk_end {
                        f_clone(i);
                        i = i + T::from(1);
                    }
                    let _ = tx.send(());
                });
                
                handles.push(rx);
                current = chunk_end;
            }
            
            for handle in handles {
                let _ = handle.recv();
            }
        });
    }
    
    fn map<G, R>(self, f: G) -> Map<Self, G>
    where
        G: Fn(Self::Item) -> R + Sync + Send,
        R: Send,
    {
        Map {
            base: self,
            map_fn: f,
        }
    }
    
    fn sum<S>(self) -> S
    where
        S: Send + std::iter::Sum<Self::Item> + std::iter::Sum<S>,
        Self::Item: std::ops::Add<Output = Self::Item> + Clone,
    {
        use parking_lot::Mutex;
        use std::sync::Arc;
        
        let partial_sums = Arc::new(Mutex::new(Vec::new()));
        let ps_clone = partial_sums.clone();
        
        self.for_each(move |item| {
            ps_clone.lock().push(item);
        });
        
        let sums = match Arc::try_unwrap(partial_sums) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => {
                let guard = arc.lock();
                (*guard).clone()
            },
        };
        
        sums.into_iter().sum()
    }
    
    fn collect<C>(self) -> C
    where
        C: std::iter::FromIterator<Self::Item>,
        Self::Item: Clone,
    {
        use parking_lot::Mutex;
        use std::sync::Arc;
        
        let results = Arc::new(Mutex::new(Vec::new()));
        let res_clone = results.clone();
        
        self.for_each(move |item| {
            res_clone.lock().push(item);
        });
        
        let items = match Arc::try_unwrap(results) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => {
                let guard = arc.lock();
                (*guard).clone()
            },
        };
        
        items.into_iter().collect()
    }
}

// Implement IntoParallelIterator for common integer range types
impl IntoParallelIterator for std::ops::Range<i32> {
    type Item = i32;
    type Iter = RangeIter<i32>;
    
    fn into_par_iter(self) -> Self::Iter {
        RangeIter { range: self }
    }
}

impl IntoParallelIterator for std::ops::Range<i64> {
    type Item = i64;
    type Iter = RangeIter<i64>;
    
    fn into_par_iter(self) -> Self::Iter {
        RangeIter { range: self }
    }
}

impl IntoParallelIterator for std::ops::Range<u32> {
    type Item = u32;
    type Iter = RangeIter<u32>;
    
    fn into_par_iter(self) -> Self::Iter {
        RangeIter { range: self }
    }
}

impl IntoParallelIterator for std::ops::Range<u64> {
    type Item = u64;
    type Iter = RangeIter<u64>;
    
    fn into_par_iter(self) -> Self::Iter {
        RangeIter { range: self }
    }
}

impl IntoParallelIterator for std::ops::Range<usize> {
    type Item = usize;
    type Iter = RangeIter<usize>;
    
    fn into_par_iter(self) -> Self::Iter {
        RangeIter { range: self }
    }
}
