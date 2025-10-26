use crate::runtime::with_current_runtime;
use std::sync::Arc;

pub trait IntoParallelIterator {
    type Item: Send;
    type Iter: ParallelIterator<Item = Self::Item>;
    
    fn into_par_iter(self) -> Self::Iter;
}

pub trait ParallelIterator: Send + Sized {
    type Item: Send;
    
    fn for_each<F>(self, f: F)
    where
        F: Fn(Self::Item) + Sync + Send + 'static;
    
    fn map<F, R>(self, f: F) -> Map<Self, F>
    where
        F: Fn(Self::Item) -> R + Sync + Send,
        R: Send;
    
    fn filter<F>(self, f: F) -> Filter<Self, F>
    where
        F: Fn(&Self::Item) -> bool + Sync + Send;
    
    fn fold<T, ID, F>(self, identity: ID, fold_op: F) -> Fold<Self, ID, F>
    where
        T: Send,
        ID: Fn() -> T + Sync + Send,
        F: Fn(T, Self::Item) -> T + Sync + Send;
    
    fn sum<S>(self) -> S
    where
        S: Send + std::iter::Sum<Self::Item> + std::iter::Sum<S>,
        Self::Item: std::ops::Add<Output = Self::Item> + Clone;
    
    fn collect<C>(self) -> C
    where
        C: std::iter::FromIterator<Self::Item>,
        Self::Item: Clone;
}

pub struct Map<I, F> {
    base: I,
    map_fn: F,
}

pub struct Filter<I, F> {
    base: I,
    filter_fn: F,
}

pub struct Fold<I, ID, F> {
    base: I,
    identity: ID,
    fold_op: F,
}

impl<I, F, R> ParallelIterator for Map<I, F>
where
    I: ParallelIterator,
    I::Item: Clone,
    F: Fn(I::Item) -> R + Sync + Send + 'static,
    R: Send + 'static,
{
    type Item = R;
    
    fn for_each<G>(self, consumer: G)
    where
        G: Fn(Self::Item) + Sync + Send + 'static,
    {
        let map_fn = Arc::new(self.map_fn);
        let consumer = Arc::new(consumer);
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
        // Collect base items first, then map sequentially to preserve order
        // TODO: optimize with indexed parallel collection
        let base_items: Vec<I::Item> = self.base.collect();
        base_items.into_iter().map(|item| (self.map_fn)(item)).collect()
    }
}

impl<I, F> ParallelIterator for Filter<I, F>
where
    I: ParallelIterator,
    I::Item: Clone + 'static,
    F: Fn(&I::Item) -> bool + Sync + Send + 'static,
{
    type Item = I::Item;
    
    fn for_each<G>(self, consumer: G)
    where
        G: Fn(Self::Item) + Sync + Send + 'static,
    {
        let filter_fn = Arc::new(self.filter_fn);
        self.base.for_each(move |item| {
            if filter_fn(&item) {
                consumer(item);
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

impl<I, ID, F, T> ParallelIterator for Fold<I, ID, F>
where
    I: ParallelIterator,
    I::Item: 'static + Clone + Sync,
    ID: Fn() -> T + Sync + Send + 'static,
    F: Fn(T, I::Item) -> T + Sync + Send + 'static,
    T: Send + Clone + 'static,
{
    type Item = T;
    
    fn for_each<G>(self, consumer: G)
    where
        G: Fn(Self::Item) + Sync + Send + 'static,
    {
        // TODO: Implement proper parallel fold
        // For now, execute sequentially to match rayon's fold().sum() behavior  
        let items: Vec<I::Item> = self.base.collect();
        
        // Execute as single chunk to produce one accumulator
        let mut acc = (self.identity)();
        for item in items {
            acc = (self.fold_op)(acc, item);
        }
        consumer(acc);
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
    
    fn fold<T2, ID2, G>(self, identity: ID2, fold_op: G) -> Fold<Self, ID2, G>
    where
        T2: Send,
        ID2: Fn() -> T2 + Sync + Send,
        G: Fn(T2, Self::Item) -> T2 + Sync + Send,
    {
        Fold {
            base: self,
            identity,
            fold_op,
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

pub struct RangeIter<T> {
    range: std::ops::Range<T>,
}

impl<T> RangeIter<T> 
where
    T: Send + Copy + std::cmp::PartialOrd + std::cmp::Ord + std::ops::Add<Output = T> + From<u8> + 'static,
    std::ops::Range<T>: RangeLenCalc,
{
    fn with_indexed_for_each<F>(&self, f: F)
    where
        F: Fn(usize, T) + Sync + Send + 'static,
    {
        let range = self.range.clone();
        let f = Arc::new(f);
        
        with_current_runtime(|rt| {
            let num_threads = rt.pool.num_threads();
            let start = range.start;
            let end = range.end;
            
            let len = range.range_len();
            let chunk_size = (len / num_threads).max(1);
            
            let mut current = start;
            let mut index = 0;
            let mut handles = Vec::new();
            
            while current < end {
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
                let start_index = index;
                
                rt.pool.execute(move || {
                    let mut i = current;
                    let mut idx = start_index;
                    while i < chunk_end {
                        f_clone(idx, i);
                        i = i + T::from(1);
                        idx += 1;
                    }
                    let _ = tx.send(());
                });
                
                handles.push(rx);
                let count = {
                    let mut tmp = current;
                    let mut cnt = 0;
                    while tmp < chunk_end {
                        tmp = tmp + T::from(1);
                        cnt += 1;
                    }
                    cnt
                };
                index += count;
                current = chunk_end;
            }
            
            for handle in handles {
                let _ = handle.recv();
            }
        });
    }
}

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
        
        with_current_runtime(|rt| {
            let num_threads = rt.pool.num_threads();
            let start = range.start;
            let end = range.end;
            
            let len = range.range_len();
            let chunk_size = (len / num_threads).max(1);
            
            let mut current = start;
            let mut handles = Vec::new();
            
            while current < end {
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
    
    fn filter<G>(self, f: G) -> Filter<Self, G>
    where
        G: Fn(&Self::Item) -> bool + Sync + Send,
    {
        Filter {
            base: self,
            filter_fn: f,
        }
    }
    
    fn fold<T2, ID, G>(self, identity: ID, fold_op: G) -> Fold<Self, ID, G>
    where
        T2: Send,
        ID: Fn() -> T2 + Sync + Send,
        G: Fn(T2, Self::Item) -> T2 + Sync + Send,
    {
        Fold {
            base: self,
            identity,
            fold_op,
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
        let range = self.range;
        let start = range.start;
        let end = range.end;
        
        let mut result = Vec::new();
        let mut current = start;
        while current < end {
            result.push(current);
            current = current + T::from(1);
        }
        
        result.into_iter().collect()
    }
}

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

// RangeInclusive support
impl IntoParallelIterator for std::ops::RangeInclusive<i32> {
    type Item = i32;
    type Iter = RangeIter<i32>;
    
    fn into_par_iter(self) -> Self::Iter {
        let start = *self.start();
        let end = *self.end() + 1;
        RangeIter { range: start..end }
    }
}

impl IntoParallelIterator for std::ops::RangeInclusive<i64> {
    type Item = i64;
    type Iter = RangeIter<i64>;
    
    fn into_par_iter(self) -> Self::Iter {
        let start = *self.start();
        let end = *self.end() + 1;
        RangeIter { range: start..end }
    }
}

impl IntoParallelIterator for std::ops::RangeInclusive<u32> {
    type Item = u32;
    type Iter = RangeIter<u32>;
    
    fn into_par_iter(self) -> Self::Iter {
        let start = *self.start();
        let end = *self.end() + 1;
        RangeIter { range: start..end }
    }
}

impl IntoParallelIterator for std::ops::RangeInclusive<u64> {
    type Item = u64;
    type Iter = RangeIter<u64>;
    
    fn into_par_iter(self) -> Self::Iter {
        let start = *self.start();
        let end = *self.end() + 1;
        RangeIter { range: start..end }
    }
}

impl IntoParallelIterator for std::ops::RangeInclusive<usize> {
    type Item = usize;
    type Iter = RangeIter<usize>;
    
    fn into_par_iter(self) -> Self::Iter {
        let start = *self.start();
        let end = *self.end() + 1;
        RangeIter { range: start..end }
    }
}
