//! GPU task scheduling and CPU/GPU distribution.

use super::kernel::GpuKernel;
use std::time::Duration;

/// GPU scheduler for deciding CPU vs GPU execution
pub struct GpuScheduler {
    gpu_available: bool,
    gpu_utilization: f64,
}

impl GpuScheduler {
    /// Create a new GPU scheduler
    pub fn new() -> Self {
        Self {
            gpu_available: true,
            gpu_utilization: 0.0,
        }
    }
    
    /// Check if GPU is available
    pub fn gpu_available(&self) -> bool {
        self.gpu_available
    }
    
    /// Set GPU availability
    pub fn set_gpu_available(&mut self, available: bool) {
        self.gpu_available = available;
    }
    
    /// Get current GPU utilization (0.0 to 1.0)
    pub fn gpu_utilization(&self) -> f64 {
        self.gpu_utilization
    }
    
    /// Update GPU utilization
    pub fn update_utilization(&mut self, utilization: f64) {
        self.gpu_utilization = utilization.clamp(0.0, 1.0);
    }
    
    /// Decide whether to offload a task to GPU
    pub fn should_offload<K: GpuKernel>(&self, kernel: &K, input_size: usize) -> bool {
        // Check if GPU is available
        if !self.gpu_available {
            return false;
        }
        
        // Check if kernel thinks GPU is worthwhile
        if !kernel.gpu_worthwhile(input_size) {
            return false;
        }
        
        // Check GPU utilization - don't offload if GPU is saturated
        if self.gpu_utilization > 0.8 {
            return false;
        }
        
        // Estimate speedup
        let cpu_time = self.estimate_cpu_time(input_size);
        let gpu_time = kernel.estimate_duration(input_size);
        let transfer_overhead = self.estimate_transfer_time(input_size);
        
        let total_gpu_time = gpu_time + transfer_overhead;
        
        // Offload if GPU is at least 1.5x faster
        total_gpu_time < cpu_time * 2 / 3
    }
    
    /// Estimate CPU execution time
    fn estimate_cpu_time(&self, input_size: usize) -> Duration {
        // Rough heuristic: 10 nanoseconds per element on CPU
        Duration::from_nanos((input_size as u64) * 10)
    }
    
    /// Estimate transfer overhead
    fn estimate_transfer_time(&self, input_size: usize) -> Duration {
        // Rough heuristic: PCIe transfer at ~10 GB/s
        let bytes = input_size * std::mem::size_of::<f32>();
        let seconds = bytes as f64 / 10_000_000_000.0;
        Duration::from_secs_f64(seconds)
    }
}

impl Default for GpuScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu::kernel::VectorAddKernel;
    
    #[test]
    fn test_gpu_scheduler() {
        let mut scheduler = GpuScheduler::new();
        assert!(scheduler.gpu_available());
        
        scheduler.set_gpu_available(false);
        assert!(!scheduler.gpu_available());
    }
    
    #[test]
    fn test_should_offload() {
        let scheduler = GpuScheduler::new();
        let kernel = VectorAddKernel::new(100_000);
        
        // Large workload should be offloaded
        assert!(scheduler.should_offload(&kernel, 100_000));
        
        // Small workload should not
        assert!(!scheduler.should_offload(&kernel, 100));
    }
    
    #[test]
    fn test_utilization() {
        let mut scheduler = GpuScheduler::new();
        
        scheduler.update_utilization(0.5);
        assert_eq!(scheduler.gpu_utilization(), 0.5);
        
        scheduler.update_utilization(1.5); // Should be clamped
        assert_eq!(scheduler.gpu_utilization(), 1.0);
    }
}
