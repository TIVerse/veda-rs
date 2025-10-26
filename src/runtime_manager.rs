//! Runtime manager that coordinates scheduler, telemetry, and feedback loop.

use crate::config::Config;
use crate::error::Result;
use crate::runtime::Runtime;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "telemetry")]
use crate::telemetry::{Metrics, FeedbackController};
#[cfg(feature = "telemetry")]
use crate::telemetry::feedback::FeedbackConfig;

/// Runtime manager that coordinates adaptive scheduling
pub struct RuntimeManager {
    runtime: Arc<Runtime>,
    #[cfg(feature = "telemetry")]
    feedback_handle: Option<thread::JoinHandle<()>>,
    #[cfg(feature = "telemetry")]
    feedback_shutdown: Arc<AtomicBool>,
}

impl RuntimeManager {
    pub fn new(config: Config) -> Result<Self> {
        let runtime = Arc::new(Runtime::new(config)?);
        
        #[cfg(feature = "telemetry")]
        let (feedback_handle, feedback_shutdown) = {
            let shutdown = Arc::new(AtomicBool::new(false));
            let handle = if runtime.config().enable_telemetry {
                let metrics = runtime.pool.metrics.clone();
                let shutdown_clone = shutdown.clone();
                
                let feedback_config = FeedbackConfig {
                    min_task_rate: 10.0,
                    max_latency_ns: 100_000_000, // 100ms
                    max_power_watts: None,
                    update_interval: Duration::from_millis(100),
                    history_size: 100,
                };
                
                let controller = FeedbackController::new(metrics, feedback_config);
                
                Some(thread::Builder::new()
                    .name("veda-feedback".to_string())
                    .spawn(move || {
                        feedback_loop(controller, shutdown_clone);
                    })
                    .expect("Failed to spawn feedback thread"))
            } else {
                None
            };
            (handle, shutdown)
        };
        
        Ok(Self {
            runtime,
            #[cfg(feature = "telemetry")]
            feedback_handle,
            #[cfg(feature = "telemetry")]
            feedback_shutdown,
        })
    }
    
    pub fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
    }
}

impl Drop for RuntimeManager {
    fn drop(&mut self) {
        #[cfg(feature = "telemetry")]
        {
            self.feedback_shutdown.store(true, Ordering::Release);
            if let Some(handle) = self.feedback_handle.take() {
                let _ = handle.join();
            }
        }
    }
}

#[cfg(feature = "telemetry")]
fn feedback_loop(controller: FeedbackController, shutdown: Arc<AtomicBool>) {
    use crate::util::{BackpressureController, BackpressureConfig};
    
    // Create backpressure controller for adaptive rate limiting
    let backpressure = BackpressureController::new(BackpressureConfig {
        max_queue_size: 10_000,
        target_latency_ms: 100,
        rate_limit_per_sec: None,
        backoff_factor: 0.5,
    });
    
    while !shutdown.load(Ordering::Acquire) {
        let action = controller.update();
        
        match action {
            crate::telemetry::feedback::FeedbackAction::IncreaseParallelism { reason } => {
                // Increase max queue size to allow more parallel work
                let current_max = backpressure.queue_size() * 2;
                backpressure.set_max_queue_size(current_max.min(50_000));
                
                if cfg!(debug_assertions) {
                    eprintln!("[VEDA Feedback] Increasing parallelism: {} (new max queue: {})", 
                        reason, current_max);
                }
            }
            crate::telemetry::feedback::FeedbackAction::ReduceLoad { reason } => {
                // Reduce max queue size to throttle admission
                let current_max = (backpressure.queue_size() * 3 / 4).max(1000);
                backpressure.set_max_queue_size(current_max);
                
                if cfg!(debug_assertions) {
                    eprintln!("[VEDA Feedback] Reducing load: {} (new max queue: {})", 
                        reason, current_max);
                }
            }
            crate::telemetry::feedback::FeedbackAction::OptimizeResources { reason } => {
                // Get metrics delta to determine optimization direction
                if let Some(delta) = controller.compute_delta() {
                    if delta.utilization_change < -0.1 {
                        // Utilization dropping, tighten queue
                        let current_max = (backpressure.queue_size() * 9 / 10).max(1000);
                        backpressure.set_max_queue_size(current_max);
                    }
                }
                
                if cfg!(debug_assertions) {
                    eprintln!("[VEDA Feedback] Optimizing resources: {}", reason);
                }
            }
            crate::telemetry::feedback::FeedbackAction::None => {}
        }
        
        thread::sleep(Duration::from_millis(100));
    }
}
