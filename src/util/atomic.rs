//! Atomic operations and utilities.

use std::sync::atomic::{AtomicU64, Ordering};

/// Atomic f64 wrapper using bitwise representation
#[derive(Debug)]
pub struct AtomicF64 {
    bits: AtomicU64,
}

impl AtomicF64 {
    /// Create a new atomic f64 with the given initial value
    pub fn new(value: f64) -> Self {
        Self {
            bits: AtomicU64::new(value.to_bits()),
        }
    }
    
    /// Load the value
    pub fn load(&self, ordering: Ordering) -> f64 {
        f64::from_bits(self.bits.load(ordering))
    }
    
    /// Store a value
    pub fn store(&self, value: f64, ordering: Ordering) {
        self.bits.store(value.to_bits(), ordering);
    }
    
    /// Swap values
    pub fn swap(&self, value: f64, ordering: Ordering) -> f64 {
        f64::from_bits(self.bits.swap(value.to_bits(), ordering))
    }
    
    /// Compare and swap
    pub fn compare_exchange(
        &self,
        current: f64,
        new: f64,
        success: Ordering,
        failure: Ordering,
    ) -> Result<f64, f64> {
        match self.bits.compare_exchange(
            current.to_bits(),
            new.to_bits(),
            success,
            failure,
        ) {
            Ok(bits) => Ok(f64::from_bits(bits)),
            Err(bits) => Err(f64::from_bits(bits)),
        }
    }
    
    /// Fetch and add (using compare-exchange loop)
    pub fn fetch_add(&self, value: f64, ordering: Ordering) -> f64 {
        let mut current = self.load(Ordering::Relaxed);
        loop {
            let new = current + value;
            match self.compare_exchange(
                current,
                new,
                ordering,
                Ordering::Relaxed,
            ) {
                Ok(_) => return current,
                Err(actual) => current = actual,
            }
        }
    }
}

impl Default for AtomicF64 {
    fn default() -> Self {
        Self::new(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_atomic_f64_basic() {
        let atomic = AtomicF64::new(3.14);
        assert_eq!(atomic.load(Ordering::Relaxed), 3.14);
        
        atomic.store(2.71, Ordering::Relaxed);
        assert_eq!(atomic.load(Ordering::Relaxed), 2.71);
    }
    
    #[test]
    fn test_atomic_f64_swap() {
        let atomic = AtomicF64::new(1.0);
        let old = atomic.swap(2.0, Ordering::Relaxed);
        assert_eq!(old, 1.0);
        assert_eq!(atomic.load(Ordering::Relaxed), 2.0);
    }
    
    #[test]
    fn test_atomic_f64_fetch_add() {
        let atomic = AtomicF64::new(10.0);
        let old = atomic.fetch_add(5.0, Ordering::Relaxed);
        assert_eq!(old, 10.0);
        assert_eq!(atomic.load(Ordering::Relaxed), 15.0);
    }
}
