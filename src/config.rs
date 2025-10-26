//! Runtime configuration for VEDA.
//!
//! The configuration system uses a builder pattern for ergonomics while
//! providing sensible defaults for most use cases.

use crate::error::{Error, Result};
use std::time::Duration;

/// Scheduling policy for the runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulingPolicy {
    /// Fixed thread pool with work stealing (like Rayon)
    Fixed,
    
    /// Adaptive thread pool that scales based on load
    Adaptive,
    
    /// Deterministic scheduling for reproducible execution
    #[cfg(feature = "deterministic")]
    Deterministic { seed: u64 },
    
    /// Energy-efficient scheduling
    #[cfg(feature = "energy-aware")]
    EnergyEfficient,
    
    /// Low-latency scheduling for real-time workloads
    LowLatency,
}

impl Default for SchedulingPolicy {
    fn default() -> Self {
        #[cfg(feature = "adaptive")]
        return SchedulingPolicy::Adaptive;
        
        #[cfg(not(feature = "adaptive"))]
        SchedulingPolicy::Fixed
    }
}

/// Configuration for the VEDA runtime.
#[derive(Debug, Clone)]
pub struct Config {
    /// Number of worker threads (None = use num_cpus)
    pub num_threads: Option<usize>,
    
    /// Scheduling policy
    pub scheduling_policy: SchedulingPolicy,
    
    /// Enable GPU support
    #[cfg(feature = "gpu")]
    pub enable_gpu: bool,
    
    /// Enable telemetry collection
    #[cfg(feature = "telemetry")]
    pub enable_telemetry: bool,
    
    /// Enable NUMA-aware allocation
    #[cfg(feature = "numa")]
    pub enable_numa: bool,
    
    /// Pin workers to CPU cores
    pub pin_workers: bool,
    
    /// Stack size for worker threads
    pub stack_size: Option<usize>,
    
    /// Thread name prefix
    pub thread_name_prefix: String,
    
    /// Maximum power consumption in watts (for energy-aware mode)
    #[cfg(feature = "energy-aware")]
    pub max_power_watts: Option<f64>,
    
    /// Adaptive scheduler tuning parameters
    pub adaptive_interval: Duration,
    pub imbalance_threshold: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            num_threads: None, // Use num_cpus
            scheduling_policy: SchedulingPolicy::default(),
            
            #[cfg(feature = "gpu")]
            enable_gpu: false,
            
            #[cfg(feature = "telemetry")]
            enable_telemetry: true,
            
            #[cfg(feature = "numa")]
            enable_numa: false,
            
            pin_workers: false,
            stack_size: Some(2 * 1024 * 1024), // 2MB
            thread_name_prefix: "veda-worker".to_string(),
            
            #[cfg(feature = "energy-aware")]
            max_power_watts: None,
            
            // Adaptive scheduler settings - tuned through experimentation
            adaptive_interval: Duration::from_millis(100),
            imbalance_threshold: 0.2, // 20% coefficient of variation
        }
    }
}

impl Config {
    /// Create a new configuration builder.
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }
    
    /// Validate the configuration.
    pub fn validate(&self) -> Result<()> {
        if let Some(n) = self.num_threads {
            if n == 0 {
                return Err(Error::config("num_threads must be > 0"));
            }
            if n > 1024 {
                return Err(Error::config("num_threads too large (max 1024)"));
            }
        }
        
        if self.imbalance_threshold <= 0.0 || self.imbalance_threshold >= 1.0 {
            return Err(Error::config("imbalance_threshold must be in (0, 1)"));
        }
        
        #[cfg(feature = "energy-aware")]
        if let Some(watts) = self.max_power_watts {
            if watts <= 0.0 {
                return Err(Error::config("max_power_watts must be > 0"));
            }
        }
        
        Ok(())
    }
    
    /// Get the actual number of worker threads to use
    pub fn worker_threads(&self) -> usize {
        self.num_threads.unwrap_or_else(num_cpus::get)
    }
}

/// Builder for creating runtime configurations.
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    /// Create a new configuration builder with defaults
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }
    
    /// Set the number of worker threads
    pub fn num_threads(mut self, n: usize) -> Self {
        self.config.num_threads = Some(n);
        self
    }
    
    /// Set the scheduling policy
    pub fn scheduling_policy(mut self, policy: SchedulingPolicy) -> Self {
        self.config.scheduling_policy = policy;
        self
    }
    
    /// Enable or disable GPU support
    #[cfg(feature = "gpu")]
    pub fn enable_gpu(mut self, enable: bool) -> Self {
        self.config.enable_gpu = enable;
        self
    }
    
    /// Enable or disable telemetry
    #[cfg(feature = "telemetry")]
    pub fn enable_telemetry(mut self, enable: bool) -> Self {
        self.config.enable_telemetry = enable;
        self
    }
    
    /// Enable NUMA-aware allocation
    #[cfg(feature = "numa")]
    pub fn enable_numa(mut self, enable: bool) -> Self {
        self.config.enable_numa = enable;
        self
    }
    
    /// Pin workers to CPU cores
    pub fn pin_workers(mut self, pin: bool) -> Self {
        self.config.pin_workers = pin;
        self
    }
    
    /// Set worker thread stack size
    pub fn stack_size(mut self, size: usize) -> Self {
        self.config.stack_size = Some(size);
        self
    }
    
    /// Set thread name prefix
    pub fn thread_name_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.config.thread_name_prefix = prefix.into();
        self
    }
    
    /// Set maximum power consumption (energy-aware mode)
    #[cfg(feature = "energy-aware")]
    pub fn max_power_watts(mut self, watts: f64) -> Self {
        self.config.max_power_watts = Some(watts);
        self
    }
    
    /// Build and validate the configuration
    pub fn build(self) -> Result<Config> {
        self.config.validate()?;
        Ok(self.config)
    }
}
