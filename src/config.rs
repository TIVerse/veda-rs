use crate::error::{Error, Result};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulingPolicy {
    Fixed,
    Adaptive,
    
    #[cfg(feature = "deterministic")]
    Deterministic { seed: u64 },
    
    #[cfg(feature = "energy-aware")]
    EnergyEfficient,
    
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

#[derive(Debug, Clone)]
pub struct Config {
    pub num_threads: Option<usize>,
    pub scheduling_policy: SchedulingPolicy,
    
    #[cfg(feature = "gpu")]
    pub enable_gpu: bool,
    
    #[cfg(feature = "telemetry")]
    pub enable_telemetry: bool,
    
    #[cfg(feature = "numa")]
    pub enable_numa: bool,
    
    pub pin_workers: bool,
    pub stack_size: Option<usize>,
    pub thread_name_prefix: String,
    
    #[cfg(feature = "energy-aware")]
    pub max_power_watts: Option<f64>,
    
    pub adaptive_interval: Duration,
    pub imbalance_threshold: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            num_threads: None,
            scheduling_policy: SchedulingPolicy::default(),
            
            #[cfg(feature = "gpu")]
            enable_gpu: false,
            
            #[cfg(feature = "telemetry")]
            enable_telemetry: true,
            
            #[cfg(feature = "numa")]
            enable_numa: false,
            
            pin_workers: false,
            stack_size: Some(2 * 1024 * 1024),
            thread_name_prefix: "veda-worker".to_string(),
            
            #[cfg(feature = "energy-aware")]
            max_power_watts: None,
            
            adaptive_interval: Duration::from_millis(100),
            imbalance_threshold: 0.2,
        }
    }
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }
    
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
    
    pub fn worker_threads(&self) -> usize {
        self.num_threads.unwrap_or_else(num_cpus::get)
    }
}

#[derive(Debug, Default)]
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }
    
    pub fn num_threads(mut self, n: usize) -> Self {
        self.config.num_threads = Some(n);
        self
    }
    
    pub fn scheduling_policy(mut self, policy: SchedulingPolicy) -> Self {
        self.config.scheduling_policy = policy;
        self
    }
    
    #[cfg(feature = "gpu")]
    pub fn enable_gpu(mut self, enable: bool) -> Self {
        self.config.enable_gpu = enable;
        self
    }
    
    #[cfg(feature = "telemetry")]
    pub fn enable_telemetry(mut self, enable: bool) -> Self {
        self.config.enable_telemetry = enable;
        self
    }
    
    #[cfg(feature = "numa")]
    pub fn enable_numa(mut self, enable: bool) -> Self {
        self.config.enable_numa = enable;
        self
    }
    
    pub fn pin_workers(mut self, pin: bool) -> Self {
        self.config.pin_workers = pin;
        self
    }
    
    pub fn stack_size(mut self, size: usize) -> Self {
        self.config.stack_size = Some(size);
        self
    }
    
    pub fn thread_name_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.config.thread_name_prefix = prefix.into();
        self
    }
    
    #[cfg(feature = "energy-aware")]
    pub fn max_power_watts(mut self, watts: f64) -> Self {
        self.config.max_power_watts = Some(watts);
        self
    }
    
    pub fn build(self) -> Result<Config> {
        self.config.validate()?;
        Ok(self.config)
    }
}
