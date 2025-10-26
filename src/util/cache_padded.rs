//! Cache line padding to prevent false sharing.

use std::fmt;
use std::ops::{Deref, DerefMut};

/// Size of a cache line on most modern CPUs
const CACHE_LINE_SIZE: usize = 64;

/// A value padded to the size of a cache line to prevent false sharing.
#[repr(align(64))]
pub struct CachePadded<T> {
    value: T,
}

impl<T> CachePadded<T> {
    /// Create a new cache-padded value
    pub const fn new(value: T) -> Self {
        Self { value }
    }
    
    /// Get a reference to the inner value
    pub fn get(&self) -> &T {
        &self.value
    }
    
    /// Get a mutable reference to the inner value
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }
    
    /// Unwrap the cache-padded value
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T> Deref for CachePadded<T> {
    type Target = T;
    
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> DerefMut for CachePadded<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T: fmt::Debug> fmt::Debug for CachePadded<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("CachePadded")
            .field(&self.value)
            .finish()
    }
}

impl<T: Clone> Clone for CachePadded<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
        }
    }
}

impl<T: Default> Default for CachePadded<T> {
    fn default() -> Self {
        Self {
            value: T::default(),
        }
    }
}

impl<T: PartialEq> PartialEq for CachePadded<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Eq> Eq for CachePadded<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, size_of};
    
    #[test]
    fn test_cache_padded_alignment() {
        assert_eq!(align_of::<CachePadded<u64>>(), 64);
        assert!(size_of::<CachePadded<u64>>() >= 64);
    }
    
    #[test]
    fn test_cache_padded_value() {
        let padded = CachePadded::new(42);
        assert_eq!(*padded, 42);
        assert_eq!(*padded.get(), 42);
    }
    
    #[test]
    fn test_cache_padded_mut() {
        let mut padded = CachePadded::new(10);
        *padded.get_mut() = 20;
        assert_eq!(*padded, 20);
    }
}
