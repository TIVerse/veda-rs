use std::hint::spin_loop;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub struct Backoff {
    step: AtomicUsize,
}

impl Backoff {
    const SPIN_LIMIT: usize = 6;
    const YIELD_LIMIT: usize = 10;
    
    pub fn new() -> Self {
        Self {
            step: AtomicUsize::new(0),
        }
    }
    
    pub fn reset(&self) {
        self.step.store(0, Ordering::Relaxed);
    }
    
    pub fn spin(&self) {
        let step = self.step.fetch_add(1, Ordering::Relaxed);
        
        if step <= Self::SPIN_LIMIT {
            for _ in 0..(1 << step.min(Self::SPIN_LIMIT)) {
                spin_loop();
            }
        } else if step <= Self::YIELD_LIMIT {
            thread::yield_now();
        } else {
            thread::sleep(Duration::from_micros(1));
        }
    }
    
    pub fn is_completed(&self) -> bool {
        self.step.load(Ordering::Relaxed) > Self::YIELD_LIMIT
    }
    
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

#[derive(Debug)]
pub struct SimpleBackoff {
    step: usize,
}

impl SimpleBackoff {
    const MAX_SPINS: usize = 10;
    
    pub fn new() -> Self {
        Self { step: 0 }
    }
    
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
        
        assert!(!backoff.is_completed());
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
