//! Exponential backoff for busy-wait loops in lock-free algorithms.

use std::hint::spin_loop;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

/// Exponential backoff for spin loops
#[derive(Debug)]
pub struct Backoff {
    step: AtomicUsize,
}

impl Backoff {
    const SPIN_LIMIT: usize = 6;
    const YIELD_LIMIT: usize = 10;
    
    /// Create a new backoff instance
    pub fn new() -> Self {
        Self {
            step: AtomicUsize::new(0),
        }
    }
    
    /// Reset the backoff to its initial state
    pub fn reset(&self) {
        self.step.store(0, Ordering::Relaxed);
    }
    
    /// Perform one step of backoff
    pub fn spin(&self) {
        let step = self.step.fetch_add(1, Ordering::Relaxed);
        
        if step <= Self::SPIN_LIMIT {
            // Spin with exponential backoff
            for _ in 0..(1 << step.min(Self::SPIN_LIMIT)) {
                spin_loop();
            }
        } else if step <= Self::YIELD_LIMIT {
            // Yield to other threads
            thread::yield_now();
        } else {
            // Sleep for a short duration
            thread::sleep(Duration::from_micros(1));
        }
    }
    
    /// Check if we've reached the sleep phase
    pub fn is_completed(&self) -> bool {
        self.step.load(Ordering::Relaxed) > Self::YIELD_LIMIT
    }
    
    /// Snooze for a longer duration (when queue is empty)
    pub fn snooze(&self) {
        if self.step.load(Ordering::Relaxed) <= Self::YIELD_LIMIT {
            thread::yield_now();
        } else {
            thread::sleep(Duration::from_micros(10));
        }
    }
}

impl Default for Backoff {
    fn default() -> Self {
        Self::new()
    }
}

/// A simpler backoff strategy with just spin and yield
#[derive(Debug)]
pub struct SimpleBackoff {
    step: usize,
}

impl SimpleBackoff {
    const MAX_SPINS: usize = 10;
    
    /// Create a new simple backoff
    pub fn new() -> Self {
        Self { step: 0 }
    }
    
    /// Perform backoff
    pub fn spin(&mut self) {
        if self.step < Self::MAX_SPINS {
            for _ in 0..(1 << self.step) {
                spin_loop();
            }
            self.step += 1;
        } else {
            thread::yield_now();
        }
    }
    
    /// Reset backoff
    pub fn reset(&mut self) {
        self.step = 0;
    }
}

impl Default for SimpleBackoff {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_backoff_progression() {
        let backoff = Backoff::new();
        
        // Should start with spinning
        assert!(!backoff.is_completed());
        
        // After many spins, should reach sleep phase
        for _ in 0..20 {
            backoff.spin();
        }
        
        assert!(backoff.is_completed());
    }
    
    #[test]
    fn test_backoff_reset() {
        let backoff = Backoff::new();
        
        for _ in 0..20 {
            backoff.spin();
        }
        assert!(backoff.is_completed());
        
        backoff.reset();
        assert!(!backoff.is_completed());
    }
    
    #[test]
    fn test_simple_backoff() {
        let mut backoff = SimpleBackoff::new();
        
        // Should not panic
        for _ in 0..20 {
            backoff.spin();
        }
        
        backoff.reset();
        // Should work after reset
        backoff.spin();
    }
}
