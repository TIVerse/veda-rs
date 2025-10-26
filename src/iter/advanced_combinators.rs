//! Advanced parallel iterator combinators

use super::par_iter::{Enumerate, Filter, Fold, Map, Skip, Take};
use super::{IntoParallelIterator, ParallelIterator};
use crate::runtime::with_current_runtime;
use parking_lot::Mutex;
use std::sync::Arc;

/// FlatMap combinator
pub struct FlatMap<I, F> {
    pub(crate) base: I,
    pub(crate) map_fn: F,
}

impl<I, F, PI> ParallelIterator for FlatMap<I, F>
where
    I: ParallelIterator,
    F: Fn(I::Item) -> PI + Sync + Send + Clone + 'static,
    PI: IntoParallelIterator,
    PI::Item: Send + Clone + 'static,
    I::Item: Clone + 'static,
{
    type Item = PI::Item;

    fn for_each<G>(self, consumer: G)
    where
        G: Fn(Self::Item) + Sync + Send + 'static,
    {
        let consumer = Arc::new(consumer);
        let map_fn = Arc::new(self.map_fn);

        self.base.for_each(move |item| {
            let inner_iter = map_fn(item).into_par_iter();
            let consumer_clone = consumer.clone();
            inner_iter.for_each(move |inner_item| {
                consumer_clone(inner_item);
            });
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

        self.for_each(move |item| {
            res_clone.lock().push(item);
        });

        let items = match Arc::try_unwrap(results) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => arc.lock().clone(),
        };

        items
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

        self.for_each(move |item| {
            ps_clone.lock().push(item);
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

        self.for_each(move |item| {
            res_clone.lock().push(item);
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

        self.for_each(move |item| {
            if !found_clone.load(Ordering::Relaxed) && predicate(&item) {
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

        self.for_each(move |item| {
            if all_clone.load(Ordering::Relaxed) && !predicate(&item) {
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

        self.for_each(move |item| {
            if result_clone.lock().is_none() && predicate(&item) {
                *result_clone.lock() = Some(item);
            }
        });

        match Arc::try_unwrap(result) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => arc.lock().clone(),
        }
    }

    fn flat_map<G, PI2>(self, f: G) -> FlatMap<Self, G>
    where
        G: Fn(Self::Item) -> PI2 + Sync + Send,
        PI2: IntoParallelIterator,
        PI2::Item: Send,
    {
        FlatMap {
            base: self,
            map_fn: f,
        }
    }

    fn zip<Z>(self, other: Z) -> Zip<Self, Z::Iter>
    where
        Z: IntoParallelIterator,
        Z::Item: Send,
    {
        Zip {
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

        self.for_each(move |item| {
            let idx = current_idx.fetch_add(1, Ordering::Relaxed);
            if pos_clone.lock().is_none() && predicate(&item) {
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

        self.for_each(move |item| {
            if predicate(&item) {
                true_clone.lock().push(item);
            } else {
                false_clone.lock().push(item);
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

/// Zip combinator
pub struct Zip<L, R> {
    pub(crate) left: L,
    pub(crate) right: R,
}

impl<L, R> ParallelIterator for Zip<L, R>
where
    L: ParallelIterator,
    R: ParallelIterator,
    L::Item: Clone + Send + Sync + 'static,
    R::Item: Clone + Send + Sync + 'static,
{
    type Item = (L::Item, R::Item);

    fn for_each<F>(self, consumer: F)
    where
        F: Fn(Self::Item) + Sync + Send + 'static,
    {
        // Collect both sides
        let left_items: Vec<L::Item> = self.left.collect();
        let right_items: Vec<R::Item> = self.right.collect();

        // Zip and process in parallel
        let min_len = left_items.len().min(right_items.len());
        let left_arc = Arc::new(left_items);
        let right_arc = Arc::new(right_items);
        let consumer = Arc::new(consumer);

        with_current_runtime(|rt| {
            let num_threads = rt.pool.num_threads();
            let chunk_size = (min_len / num_threads).max(1);
            let mut handles = Vec::new();

            for chunk_idx in 0..num_threads {
                let start = chunk_idx * chunk_size;
                if start >= min_len {
                    break;
                }
                let end = ((chunk_idx + 1) * chunk_size).min(min_len);

                let consumer_clone = consumer.clone();
                let left_clone = left_arc.clone();
                let right_clone = right_arc.clone();
                let (tx, rx) = crossbeam_channel::bounded(0);

                rt.pool.execute(move || {
                    for i in start..end {
                        consumer_clone((left_clone[i].clone(), right_clone[i].clone()));
                    }
                    let _ = tx.send(());
                });

                handles.push(rx);
            }

            for handle in handles {
                let _ = handle.recv();
            }
        });
    }

    fn map<G, T>(self, f: G) -> Map<Self, G>
    where
        G: Fn(Self::Item) -> T + Sync + Send,
        T: Send,
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

        self.for_each(move |item| {
            res_clone.lock().push(item);
        });

        let items = match Arc::try_unwrap(results) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => arc.lock().clone(),
        };

        items
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

        self.for_each(move |item| {
            ps_clone.lock().push(item);
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

        self.for_each(move |item| {
            res_clone.lock().push(item);
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

        self.for_each(move |item| {
            if !found_clone.load(Ordering::Relaxed) && predicate(&item) {
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

        self.for_each(move |item| {
            if all_clone.load(Ordering::Relaxed) && !predicate(&item) {
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

        self.for_each(move |item| {
            if result_clone.lock().is_none() && predicate(&item) {
                *result_clone.lock() = Some(item);
            }
        });

        match Arc::try_unwrap(result) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => arc.lock().clone(),
        }
    }

    fn flat_map<G, PI>(self, f: G) -> FlatMap<Self, G>
    where
        G: Fn(Self::Item) -> PI + Sync + Send,
        PI: IntoParallelIterator,
        PI::Item: Send,
    {
        FlatMap {
            base: self,
            map_fn: f,
        }
    }

    fn zip<Z>(self, other: Z) -> Zip<Self, Z::Iter>
    where
        Z: IntoParallelIterator,
        Z::Item: Send,
    {
        Zip {
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

        self.for_each(move |item| {
            let idx = current_idx.fetch_add(1, Ordering::Relaxed);
            if pos_clone.lock().is_none() && predicate(&item) {
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

        self.for_each(move |item| {
            if predicate(&item) {
                true_clone.lock().push(item);
            } else {
                false_clone.lock().push(item);
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
