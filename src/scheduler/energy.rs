use super::WorkerState;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use parking_lot::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    CpuNormal,
    CpuThrottled,
    #[cfg(feature = "gpu")]
    Gpu,
}

#[derive(Debug, Clone)]
pub struct EnergyConfig {
    pub max_watts: f64,
    pub max_temp_celsius: f64,
    pub enable_dvfs: bool,
}

impl Default for EnergyConfig {
    fn default() -> Self {
        Self {
            max_watts: 100.0,
            max_temp_celsius: 85.0,
            enable_dvfs: true,
        }
    }
}

#[derive(Debug)]
pub struct ThermalState {
    current_temp: AtomicU64,
    last_update: RwLock<Instant>,
}

impl ThermalState {
    pub fn new() -> Self {
        Self {
            current_temp: AtomicU64::new(25.0f64.to_bits()),
            last_update: RwLock::new(Instant::now()),
        }
    }
    
    pub fn temperature(&self) -> f64 {
        f64::from_bits(self.current_temp.load(Ordering::Relaxed))
    }
    
    /// Update temperature reading
    pub fn update_temperature(&self, temp: f64) {
        self.current_temp.store(temp.to_bits(), Ordering::Relaxed);
        *self.last_update.write() = Instant::now();
    }
    
    /// Check if system is overheating
    pub fn is_overheating(&self, threshold: f64) -> bool {
        self.temperature() > threshold
    }
}

impl Default for ThermalState {
    fn default() -> Self {
        Self::new()
    }
}

/// Power monitoring for energy-aware scheduling
#[derive(Debug)]
pub struct PowerMonitor {
    current_watts: AtomicU64, // Stored as f64 bits
    accumulated_joules: AtomicU64, // Stored as f64 bits
    last_update: RwLock<Instant>,
}

impl PowerMonitor {
    pub fn new() -> Self {
        Self {
            current_watts: AtomicU64::new(0.0f64.to_bits()),
            accumulated_joules: AtomicU64::new(0.0f64.to_bits()),
            last_update: RwLock::new(Instant::now()),
        }
    }
    
    /// Get current power consumption in watts
    pub fn current_watts(&self) -> f64 {
        f64::from_bits(self.current_watts.load(Ordering::Relaxed))
    }
    
    /// Update power reading and accumulate energy
    pub fn update_power(&self, watts: f64) {
        let now = Instant::now();
        let elapsed = {
            let last = *self.last_update.read();
            now.duration_since(last).as_secs_f64()
        };
        
        // Accumulate energy (joules = watts * seconds)
        let old_joules = f64::from_bits(self.accumulated_joules.load(Ordering::Relaxed));
        let new_joules = old_joules + (watts * elapsed);
        self.accumulated_joules.store(new_joules.to_bits(), Ordering::Relaxed);
        
        // Update current power
        self.current_watts.store(watts.to_bits(), Ordering::Relaxed);
        *self.last_update.write() = now;
    }
    
    /// Get total energy consumed in joules
    pub fn total_joules(&self) -> f64 {
        f64::from_bits(self.accumulated_joules.load(Ordering::Relaxed))
    }
    
    /// Reset energy counter
    pub fn reset_energy(&self) {
        self.accumulated_joules.store(0.0f64.to_bits(), Ordering::Relaxed);
    }
}

impl Default for PowerMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Energy-aware scheduler
pub struct EnergyAwareScheduler {
    power_monitor: PowerMonitor,
    thermal_state: ThermalState,
    config: EnergyConfig,
}

impl EnergyAwareScheduler {
    pub fn new(config: EnergyConfig) -> Self {
        Self {
            power_monitor: PowerMonitor::new(),
            thermal_state: ThermalState::new(),
            config,
        }
    }
    
    /// Check if throttling is needed
    pub fn should_throttle(&self) -> bool {
        self.power_monitor.current_watts() > self.config.max_watts
            || self.thermal_state.is_overheating(self.config.max_temp_celsius)
    }
    
    /// Select execution mode based on energy constraints
    pub fn select_execution_mode(&self) -> ExecutionMode {
        if self.should_throttle() {
            ExecutionMode::CpuThrottled
        } else {
            ExecutionMode::CpuNormal
        }
    }
    
    /// Estimate energy cost of a task
    pub fn estimate_task_energy(&self, duration_ns: u64) -> f64 {
        // Rough estimate: assume 15W per active core
        let base_power_watts = 15.0;
        let duration_seconds = duration_ns as f64 / 1_000_000_000.0;
        base_power_watts * duration_seconds
    }
    
    /// Get power monitor
    pub fn power_monitor(&self) -> &PowerMonitor {
        &self.power_monitor
    }
    
    /// Get thermal state
    pub fn thermal_state(&self) -> &ThermalState {
        &self.thermal_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_power_monitor() {
        let monitor = PowerMonitor::new();
        monitor.update_power(50.0);
        assert_eq!(monitor.current_watts(), 50.0);
    }
    
    #[test]
    fn test_thermal_state() {
        let thermal = ThermalState::new();
        thermal.update_temperature(75.0);
        assert_eq!(thermal.temperature(), 75.0);
        assert!(!thermal.is_overheating(85.0));
        assert!(thermal.is_overheating(70.0));
    }
    
    #[test]
    fn test_energy_aware_scheduler() {
        let config = EnergyConfig {
            max_watts: 100.0,
            max_temp_celsius: 85.0,
            enable_dvfs: true,
        };
        
        let scheduler = EnergyAwareScheduler::new(config);
        assert!(!scheduler.should_throttle());
        
        scheduler.power_monitor().update_power(150.0);
        assert!(scheduler.should_throttle());
    }
}
