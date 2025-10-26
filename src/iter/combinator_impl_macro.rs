//! Macro to reduce boilerplate in ParallelIterator implementations

/// Implements the standard new combinator methods for a ParallelIterator type
#[macro_export]
macro_rules! impl_standard_combinators {
    ($self_type:ty) => {
        fn flat_map<G, PI>(self, f: G) -> $crate::iter::advanced_combinators::FlatMap<Self, G>
        where
            G: Fn(Self::Item) -> PI + Sync + Send,
            PI: $crate::iter::IntoParallelIterator,
            PI::Item: Send,
        {
            $crate::iter::advanced_combinators::FlatMap {
                base: self,
                map_fn: f,
            }
        }
        
        fn zip<Z>(self, other: Z) -> $crate::iter::advanced_combinators::Zip<Self, Z::Iter>
        where
            Z: $crate::iter::IntoParallelIterator,
            Z::Item: Send,
        {
            $crate::iter::advanced_combinators::Zip {
                left: self,
                right: other.into_par_iter(),
            }
        }
        
        fn position_any<P>(self, predicate: P) -> Option<usize>
        where
            P: Fn(&Self::Item) -> bool + Sync + Send + 'static,
            Self::Item: Clone,
        {
            use std::sync::Arc;
            use parking_lot::Mutex;
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
            use std::sync::Arc;
            use parking_lot::Mutex;
            
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
    };
}
