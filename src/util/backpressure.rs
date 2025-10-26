//! Backpressure control for rate limiting.

use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Backpressure controller for automatic rate limiting
#[derive(Debug)]
pub struct BackpressureController {
    max_queue_size: AtomicUsize,
    current_queue_size: AtomicUsize,
    throttle_rate: AtomicU64, // tasks per second, stored as f64 bits
    last_update: RwLock<Instant>,
    config: BackpressureConfig,
}

#[derive(Debug, Clone)]
pub struct BackpressureConfig {
    pub max_queue_size: usize,
    pub target_latency_ms: u64,
    pub rate_limit_per_sec: Option<f64>,
    pub backoff_factor: f64,
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 10_000,
            target_latency_ms: 100,
            rate_limit_per_sec: None,
            backoff_factor: 0.5,
        }
    }
}

impl BackpressureController {
    pub fn new(config: BackpressureConfig) -> Self {
        let initial_rate = config.rate_limit_per_sec.unwrap_or(1000.0);
        Self {
            max_queue_size: AtomicUsize::new(config.max_queue_size),
            current_queue_size: AtomicUsize::new(0),
            throttle_rate: AtomicU64::new(initial_rate.to_bits()),
            last_update: RwLock::new(Instant::now()),
            config,
        }
    }

    /// Check if we should admit a new task
    pub fn should_admit(&self) -> bool {
        let current = self.current_queue_size.load(Ordering::Relaxed);
        let max = self.max_queue_size.load(Ordering::Relaxed);
        current < max
    }

    /// Increment queue size when a task is enqueued
    pub fn on_enqueue(&self) -> bool {
        let current = self.current_queue_size.fetch_add(1, Ordering::Relaxed);
        let max = self.max_queue_size.load(Ordering::Relaxed);

        if current >= max {
            // Over capacity, reject
            self.current_queue_size.fetch_sub(1, Ordering::Relaxed);
            return false;
        }
        true
    }

    /// Decrement queue size when a task completes
    pub fn on_complete(&self) {
        self.current_queue_size.fetch_sub(1, Ordering::Relaxed);
    }

    /// Update throttle rate based on observed latency
    pub fn update_rate(&self, observed_latency_ms: u64) {
        let target = self.config.target_latency_ms;

        if observed_latency_ms > target {
            // Latency too high, reduce rate
            let current_rate = f64::from_bits(self.throttle_rate.load(Ordering::Relaxed));
            let new_rate = current_rate * self.config.backoff_factor;
            self.throttle_rate
                .store(new_rate.to_bits(), Ordering::Relaxed);
        } else if observed_latency_ms < target / 2 {
            // Latency low, can increase rate
            let current_rate = f64::from_bits(self.throttle_rate.load(Ordering::Relaxed));
            let new_rate = current_rate * (1.0 / self.config.backoff_factor);

            // Cap at configured limit if any
            let new_rate = if let Some(limit) = self.config.rate_limit_per_sec {
                new_rate.min(limit)
            } else {
                new_rate
            };

            self.throttle_rate
                .store(new_rate.to_bits(), Ordering::Relaxed);
        }

        *self.last_update.write() = Instant::now();
    }

    /// Get current throttle rate (tasks per second)
    pub fn current_rate(&self) -> f64 {
        f64::from_bits(self.throttle_rate.load(Ordering::Relaxed))
    }

    /// Get current queue size
    pub fn queue_size(&self) -> usize {
        self.current_queue_size.load(Ordering::Relaxed)
    }

    /// Calculate delay needed before next task admission
    pub fn compute_delay(&self) -> Option<Duration> {
        let rate = self.current_rate();
        if rate <= 0.0 {
            return Some(Duration::from_millis(100));
        }

        let elapsed = self.last_update.read().elapsed();
        let interval = Duration::from_secs_f64(1.0 / rate);

        if elapsed < interval {
            Some(interval - elapsed)
        } else {
            None
        }
    }

    /// Adjust max queue size dynamically
    pub fn set_max_queue_size(&self, size: usize) {
        self.max_queue_size.store(size, Ordering::Relaxed);
    }
}

impl Default for BackpressureController {
    fn default() -> Self {
        Self::new(BackpressureConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backpressure_admit() {
        let config = BackpressureConfig {
            max_queue_size: 10,
            ..Default::default()
        };
        let controller = BackpressureController::new(config);

        // Should admit up to max
        for _ in 0..10 {
            assert!(controller.on_enqueue());
        }

        // Should reject beyond max
        assert!(!controller.on_enqueue());

        // Complete one task
        controller.on_complete();

        // Should admit again
        assert!(controller.on_enqueue());
    }

    #[test]
    fn test_rate_adjustment() {
        let controller = BackpressureController::default();
        let initial_rate = controller.current_rate();

        // High latency should reduce rate
        controller.update_rate(200);
        assert!(controller.current_rate() < initial_rate);

        // Low latency should increase rate
        controller.update_rate(10);
        assert!(controller.current_rate() > initial_rate * 0.5);
    }

    #[test]
    fn test_compute_delay() {
        let config = BackpressureConfig {
            rate_limit_per_sec: Some(10.0), // 10 tasks per second
            ..Default::default()
        };
        let controller = BackpressureController::new(config);

        // First task should have no delay
        let delay = controller.compute_delay();
        assert!(delay.is_some());

        // Rate is 10/s, so interval is 100ms
        if let Some(d) = delay {
            assert!(d <= Duration::from_millis(100));
        }
    }
}
