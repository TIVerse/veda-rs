//! Adaptive workload example - demonstrates dynamic load balancing

use veda::prelude::*;
use std::thread;
use std::time::Duration;

fn variable_workload(i: usize) -> usize {
    // Simulate variable-length computation
    if i % 100 == 0 {
        // Occasional heavy task
        thread::sleep(Duration::from_millis(10));
    } else if i % 10 == 0 {
        // Medium task
        thread::sleep(Duration::from_millis(1));
    }
    // Light task - just compute
    (0..100).map(|j| i + j).sum()
}

fn main() {
    println!("=== Adaptive Workload Example ===\n");
    
    // Configure with adaptive scheduling
    let config = veda::Config::builder()
        .scheduling_policy(veda::SchedulingPolicy::Adaptive)
        .num_threads(4)
        .build()
        .expect("Failed to build config");
    
    veda::init_with_config(config).expect("Failed to initialize");
    
    println!("Processing variable workload with adaptive scheduling...");
    let start = std::time::Instant::now();
    
    let results: Vec<usize> = (0..1000)
        .into_par_iter()
        .map(variable_workload)
        .collect();
    
    let elapsed = start.elapsed();
    
    println!("Processed {} tasks in {:?}", results.len(), elapsed);
    println!("Average: {:.2}ms per task", elapsed.as_secs_f64() * 1000.0 / results.len() as f64);
    
    // Get runtime metrics if telemetry is enabled
    #[cfg(feature = "telemetry")]
    {
        use veda::telemetry::export::{ConsoleExporter, MetricsExporter};
        
        println!("\n--- Runtime Metrics ---");
        let metrics = veda::telemetry::metrics::Metrics::default();
        let snapshot = metrics.snapshot();
        let exporter = ConsoleExporter::new(true);
        let _ = exporter.export(&snapshot);
    }
    
    veda::shutdown();
    println!("\n=== Example Complete ===");
}
