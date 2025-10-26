//! Chunked parallel iterators

use super::par_iter::{Enumerate, Filter, Fold, Map, Skip, Take};
use super::{IntoParallelIterator, ParallelIterator};
use crate::runtime::with_current_runtime;
use parking_lot::Mutex;
use std::sync::Arc;

/// Parallel iterator over chunks of items
pub struct Chunks<I> {
    pub(crate) base: I,
    pub(crate) chunk_size: usize,
}

/// Parallel iterator over overlapping windows of items
pub struct Windows<I> {
    pub(crate) base: I,
    pub(crate) window_size: usize,
}

impl<I> ParallelIterator for Chunks<I>
where
    I: ParallelIterator,
    I::Item: Clone + Send + Sync + 'static,
{
    type Item = Vec<I::Item>;

    fn for_each<F>(self, consumer: F)
    where
        F: Fn(Self::Item) + Sync + Send + 'static,
    {
        // Collect all items first
        let items: Vec<I::Item> = self.base.collect();

        if items.is_empty() {
            return;
        }

        let items = Arc::new(items);
        let consumer = Arc::new(consumer);
        let chunk_size = self.chunk_size;

        with_current_runtime(|rt| {
            let num_chunks = (items.len() + chunk_size - 1) / chunk_size;
            let mut handles = Vec::new();

            for chunk_idx in 0..num_chunks {
                let start = chunk_idx * chunk_size;
                if start >= items.len() {
                    break;
                }
                let end = (start + chunk_size).min(items.len());

                let consumer_clone = consumer.clone();
                let items_clone = items.clone();
                let (tx, rx) = crossbeam_channel::bounded(0);

                rt.pool.execute(move || {
                    let chunk: Vec<I::Item> = items_clone[start..end].to_vec();
                    consumer_clone(chunk);
                    let _ = tx.send(());
                });

                handles.push(rx);
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

    fn filter<G>(self, f: G) -> Filter<Self, G>
    where
        G: Fn(&Self::Item) -> bool + Sync + Send,
    {
        Filter {
            base: self,
            filter_fn: f,
        }
    }

    fn fold<T, ID, G>(self, identity: ID, fold_op: G) -> Fold<Self, ID, G>
    where
        T: Send,
        ID: Fn() -> T + Sync + Send,
        G: Fn(T, Self::Item) -> T + Sync + Send,
    {
        Fold {
            base: self,
            identity,
            fold_op,
        }
    }

    fn reduce<OP, ID>(self, identity: ID, op: OP) -> Self::Item
    where
        OP: Fn(Self::Item, Self::Item) -> Self::Item + Sync + Send,
        ID: Fn() -> Self::Item + Sync + Send,
        Self::Item: Clone,
    {
        let results = Arc::new(Mutex::new(Vec::new()));
        let res_clone = results.clone();

        self.for_each(move |chunk| {
            res_clone.lock().push(chunk);
        });

        let chunks = match Arc::try_unwrap(results) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => arc.lock().clone(),
        };

        chunks
            .into_iter()
            .reduce(|a, b| op(a, b))
            .unwrap_or_else(|| identity())
    }

    fn sum<S>(self) -> S
    where
        S: Send + std::iter::Sum<Self::Item> + std::iter::Sum<S>,
        Self::Item: std::ops::Add<Output = Self::Item> + Clone,
    {
        let partial_sums = Arc::new(Mutex::new(Vec::new()));
        let ps_clone = partial_sums.clone();

        self.for_each(move |chunk| {
            ps_clone.lock().push(chunk);
        });

        let sums = match Arc::try_unwrap(partial_sums) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => arc.lock().clone(),
        };

        sums.into_iter().sum()
    }

    fn collect<C>(self) -> C
    where
        C: std::iter::FromIterator<Self::Item>,
        Self::Item: Clone,
    {
        let results = Arc::new(Mutex::new(Vec::new()));
        let res_clone = results.clone();

        self.for_each(move |chunk| {
            res_clone.lock().push(chunk);
        });

        let items = match Arc::try_unwrap(results) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => arc.lock().clone(),
        };

        items.into_iter().collect()
    }

    fn enumerate(self) -> Enumerate<Self> {
        Enumerate { base: self }
    }

    fn take(self, n: usize) -> Take<Self> {
        Take { base: self, n }
    }

    fn skip(self, n: usize) -> Skip<Self> {
        Skip { base: self, n }
    }

    fn any<P>(self, predicate: P) -> bool
    where
        P: Fn(&Self::Item) -> bool + Sync + Send + 'static,
        Self::Item: Clone,
    {
        use std::sync::atomic::{AtomicBool, Ordering};

        let found = Arc::new(AtomicBool::new(false));
        let found_clone = found.clone();
        let predicate = Arc::new(predicate);

        self.for_each(move |chunk| {
            if !found_clone.load(Ordering::Relaxed) && predicate(&chunk) {
                found_clone.store(true, Ordering::Relaxed);
            }
        });

        found.load(Ordering::Relaxed)
    }

    fn all<P>(self, predicate: P) -> bool
    where
        P: Fn(&Self::Item) -> bool + Sync + Send + 'static,
        Self::Item: Clone,
    {
        use std::sync::atomic::{AtomicBool, Ordering};

        let all_match = Arc::new(AtomicBool::new(true));
        let all_clone = all_match.clone();
        let predicate = Arc::new(predicate);

        self.for_each(move |chunk| {
            if all_clone.load(Ordering::Relaxed) && !predicate(&chunk) {
                all_clone.store(false, Ordering::Relaxed);
            }
        });

        all_match.load(Ordering::Relaxed)
    }

    fn find_any<P>(self, predicate: P) -> Option<Self::Item>
    where
        P: Fn(&Self::Item) -> bool + Sync + Send + 'static,
        Self::Item: Clone,
    {
        let result = Arc::new(Mutex::new(None));
        let result_clone = result.clone();
        let predicate = Arc::new(predicate);

        self.for_each(move |chunk| {
            if result_clone.lock().is_none() && predicate(&chunk) {
                *result_clone.lock() = Some(chunk);
            }
        });

        match Arc::try_unwrap(result) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => arc.lock().clone(),
        }
    }

    fn flat_map<G, PI>(self, f: G) -> super::advanced_combinators::FlatMap<Self, G>
    where
        G: Fn(Self::Item) -> PI + Sync + Send,
        PI: IntoParallelIterator,
        PI::Item: Send,
    {
        super::advanced_combinators::FlatMap {
            base: self,
            map_fn: f,
        }
    }

    fn zip<Z>(self, other: Z) -> super::advanced_combinators::Zip<Self, Z::Iter>
    where
        Z: IntoParallelIterator,
        Z::Item: Send,
    {
        super::advanced_combinators::Zip {
            left: self,
            right: other.into_par_iter(),
        }
    }

    fn position_any<P>(self, predicate: P) -> Option<usize>
    where
        P: Fn(&Self::Item) -> bool + Sync + Send + 'static,
        Self::Item: Clone,
    {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let position = Arc::new(Mutex::new(None));
        let current_idx = Arc::new(AtomicUsize::new(0));
        let pos_clone = position.clone();
        let predicate = Arc::new(predicate);

        self.for_each(move |chunk| {
            let idx = current_idx.fetch_add(1, Ordering::Relaxed);
            if pos_clone.lock().is_none() && predicate(&chunk) {
                *pos_clone.lock() = Some(idx);
            }
        });

        match Arc::try_unwrap(position) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => *arc.lock(),
        }
    }

    fn partition<P>(self, predicate: P) -> (Vec<Self::Item>, Vec<Self::Item>)
    where
        P: Fn(&Self::Item) -> bool + Sync + Send + 'static,
        Self::Item: Clone,
    {
        let true_items = Arc::new(Mutex::new(Vec::new()));
        let false_items = Arc::new(Mutex::new(Vec::new()));
        let true_clone = true_items.clone();
        let false_clone = false_items.clone();
        let predicate = Arc::new(predicate);

        self.for_each(move |chunk| {
            if predicate(&chunk) {
                true_clone.lock().push(chunk);
            } else {
                false_clone.lock().push(chunk);
            }
        });

        let trues = match Arc::try_unwrap(true_items) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => arc.lock().clone(),
        };

        let falses = match Arc::try_unwrap(false_items) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => arc.lock().clone(),
        };

        (trues, falses)
    }
}

impl<I> ParallelIterator for Windows<I>
where
    I: ParallelIterator,
    I::Item: Clone + Send + Sync + 'static,
{
    type Item = Vec<I::Item>;

    fn for_each<F>(self, consumer: F)
    where
        F: Fn(Self::Item) + Sync + Send + 'static,
    {
        // Collect all items first
        let items: Vec<I::Item> = self.base.collect();

        if items.len() < self.window_size {
            return;
        }

        let items = Arc::new(items);
        let consumer = Arc::new(consumer);
        let window_size = self.window_size;

        with_current_runtime(|rt| {
            let num_windows = items.len() - window_size + 1;
            let mut handles = Vec::new();

            for window_idx in 0..num_windows {
                let start = window_idx;
                let end = start + window_size;

                let consumer_clone = consumer.clone();
                let items_clone = items.clone();
                let (tx, rx) = crossbeam_channel::bounded(0);

                rt.pool.execute(move || {
                    let window: Vec<I::Item> = items_clone[start..end].to_vec();
                    consumer_clone(window);
                    let _ = tx.send(());
                });

                handles.push(rx);
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

    fn filter<G>(self, f: G) -> Filter<Self, G>
    where
        G: Fn(&Self::Item) -> bool + Sync + Send,
    {
        Filter {
            base: self,
            filter_fn: f,
        }
    }

    fn fold<T, ID, G>(self, identity: ID, fold_op: G) -> Fold<Self, ID, G>
    where
        T: Send,
        ID: Fn() -> T + Sync + Send,
        G: Fn(T, Self::Item) -> T + Sync + Send,
    {
        Fold {
            base: self,
            identity,
            fold_op,
        }
    }

    fn reduce<OP, ID>(self, identity: ID, op: OP) -> Self::Item
    where
        OP: Fn(Self::Item, Self::Item) -> Self::Item + Sync + Send,
        ID: Fn() -> Self::Item + Sync + Send,
        Self::Item: Clone,
    {
        let results = Arc::new(Mutex::new(Vec::new()));
        let res_clone = results.clone();

        self.for_each(move |window| {
            res_clone.lock().push(window);
        });

        let windows = match Arc::try_unwrap(results) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => arc.lock().clone(),
        };

        windows
            .into_iter()
            .reduce(|a, b| op(a, b))
            .unwrap_or_else(|| identity())
    }

    fn sum<S>(self) -> S
    where
        S: Send + std::iter::Sum<Self::Item> + std::iter::Sum<S>,
        Self::Item: std::ops::Add<Output = Self::Item> + Clone,
    {
        let partial_sums = Arc::new(Mutex::new(Vec::new()));
        let ps_clone = partial_sums.clone();

        self.for_each(move |window| {
            ps_clone.lock().push(window);
        });

        let sums = match Arc::try_unwrap(partial_sums) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => arc.lock().clone(),
        };

        sums.into_iter().sum()
    }

    fn collect<C>(self) -> C
    where
        C: std::iter::FromIterator<Self::Item>,
        Self::Item: Clone,
    {
        let results = Arc::new(Mutex::new(Vec::new()));
        let res_clone = results.clone();

        self.for_each(move |window| {
            res_clone.lock().push(window);
        });

        let items = match Arc::try_unwrap(results) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => arc.lock().clone(),
        };

        items.into_iter().collect()
    }

    fn enumerate(self) -> Enumerate<Self> {
        Enumerate { base: self }
    }

    fn take(self, n: usize) -> Take<Self> {
        Take { base: self, n }
    }

    fn skip(self, n: usize) -> Skip<Self> {
        Skip { base: self, n }
    }

    fn any<P>(self, predicate: P) -> bool
    where
        P: Fn(&Self::Item) -> bool + Sync + Send + 'static,
        Self::Item: Clone,
    {
        use std::sync::atomic::{AtomicBool, Ordering};

        let found = Arc::new(AtomicBool::new(false));
        let found_clone = found.clone();
        let predicate = Arc::new(predicate);

        self.for_each(move |window| {
            if !found_clone.load(Ordering::Relaxed) && predicate(&window) {
                found_clone.store(true, Ordering::Relaxed);
            }
        });

        found.load(Ordering::Relaxed)
    }

    fn all<P>(self, predicate: P) -> bool
    where
        P: Fn(&Self::Item) -> bool + Sync + Send + 'static,
        Self::Item: Clone,
    {
        use std::sync::atomic::{AtomicBool, Ordering};

        let all_match = Arc::new(AtomicBool::new(true));
        let all_clone = all_match.clone();
        let predicate = Arc::new(predicate);

        self.for_each(move |window| {
            if all_clone.load(Ordering::Relaxed) && !predicate(&window) {
                all_clone.store(false, Ordering::Relaxed);
            }
        });

        all_match.load(Ordering::Relaxed)
    }

    fn find_any<P>(self, predicate: P) -> Option<Self::Item>
    where
        P: Fn(&Self::Item) -> bool + Sync + Send + 'static,
        Self::Item: Clone,
    {
        let result = Arc::new(Mutex::new(None));
        let result_clone = result.clone();
        let predicate = Arc::new(predicate);

        self.for_each(move |window| {
            if result_clone.lock().is_none() && predicate(&window) {
                *result_clone.lock() = Some(window);
            }
        });

        match Arc::try_unwrap(result) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => arc.lock().clone(),
        }
    }

    fn flat_map<G, PI>(self, f: G) -> super::advanced_combinators::FlatMap<Self, G>
    where
        G: Fn(Self::Item) -> PI + Sync + Send,
        PI: IntoParallelIterator,
        PI::Item: Send,
    {
        super::advanced_combinators::FlatMap {
            base: self,
            map_fn: f,
        }
    }

    fn zip<Z>(self, other: Z) -> super::advanced_combinators::Zip<Self, Z::Iter>
    where
        Z: IntoParallelIterator,
        Z::Item: Send,
    {
        super::advanced_combinators::Zip {
            left: self,
            right: other.into_par_iter(),
        }
    }

    fn position_any<P>(self, predicate: P) -> Option<usize>
    where
        P: Fn(&Self::Item) -> bool + Sync + Send + 'static,
        Self::Item: Clone,
    {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let position = Arc::new(Mutex::new(None));
        let current_idx = Arc::new(AtomicUsize::new(0));
        let pos_clone = position.clone();
        let predicate = Arc::new(predicate);

        self.for_each(move |window| {
            let idx = current_idx.fetch_add(1, Ordering::Relaxed);
            if pos_clone.lock().is_none() && predicate(&window) {
                *pos_clone.lock() = Some(idx);
            }
        });

        match Arc::try_unwrap(position) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => *arc.lock(),
        }
    }

    fn partition<P>(self, predicate: P) -> (Vec<Self::Item>, Vec<Self::Item>)
    where
        P: Fn(&Self::Item) -> bool + Sync + Send + 'static,
        Self::Item: Clone,
    {
        let true_items = Arc::new(Mutex::new(Vec::new()));
        let false_items = Arc::new(Mutex::new(Vec::new()));
        let true_clone = true_items.clone();
        let false_clone = false_items.clone();
        let predicate = Arc::new(predicate);

        self.for_each(move |window| {
            if predicate(&window) {
                true_clone.lock().push(window);
            } else {
                false_clone.lock().push(window);
            }
        });

        let trues = match Arc::try_unwrap(true_items) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => arc.lock().clone(),
        };

        let falses = match Arc::try_unwrap(false_items) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => arc.lock().clone(),
        };

        (trues, falses)
    }
}

/// Extension trait for creating chunked iterators
pub trait ParallelChunks: ParallelIterator + Sized {
    /// Split into chunks of the given size, processing chunks in parallel
    fn par_chunks(self, chunk_size: usize) -> Chunks<Self> {
        assert!(chunk_size > 0, "chunk size must be greater than 0");
        Chunks {
            base: self,
            chunk_size,
        }
    }

    /// Create overlapping windows of the given size, processing windows in parallel
    fn par_windows(self, window_size: usize) -> Windows<Self> {
        assert!(window_size > 0, "window size must be greater than 0");
        Windows {
            base: self,
            window_size,
        }
    }
}

// Implement for all parallel iterators
impl<T: ParallelIterator> ParallelChunks for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iter::IntoParallelIterator;

    #[test]
    fn test_par_chunks() {
        crate::runtime::shutdown();
        crate::runtime::init().unwrap();

        let data: Vec<i32> = (0..10).collect();
        let chunks: Vec<Vec<i32>> = data.into_par_iter().par_chunks(3).collect();

        assert_eq!(chunks.len(), 4); // 3, 3, 3, 1
        assert_eq!(chunks[0], vec![0, 1, 2]);
        assert_eq!(chunks[1], vec![3, 4, 5]);
        assert_eq!(chunks[2], vec![6, 7, 8]);
        assert_eq!(chunks[3], vec![9]);

        crate::runtime::shutdown();
    }

    #[test]
    fn test_par_windows() {
        crate::runtime::shutdown();
        crate::runtime::init().unwrap();

        let data: Vec<i32> = (0..5).collect();
        let windows: Vec<Vec<i32>> = data.into_par_iter().par_windows(3).collect();

        assert_eq!(windows.len(), 3); // [0,1,2], [1,2,3], [2,3,4]
        assert!(windows.contains(&vec![0, 1, 2]));
        assert!(windows.contains(&vec![1, 2, 3]));
        assert!(windows.contains(&vec![2, 3, 4]));

        crate::runtime::shutdown();
    }

    #[test]
    fn test_chunk_operations() {
        crate::runtime::shutdown();
        crate::runtime::init().unwrap();

        let data: Vec<i32> = (0..12).collect();

        // Map over chunks
        let sums: Vec<i32> = data
            .into_par_iter()
            .par_chunks(4)
            .map(|chunk| chunk.iter().sum())
            .collect();

        assert_eq!(sums.len(), 3);
        assert_eq!(sums[0], 0 + 1 + 2 + 3); // 6
        assert_eq!(sums[1], 4 + 5 + 6 + 7); // 22
        assert_eq!(sums[2], 8 + 9 + 10 + 11); // 38

        crate::runtime::shutdown();
    }
}
