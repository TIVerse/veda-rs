//! Comprehensive demonstration of all wired VEDA features

use veda_rs::prelude::*;

fn main() {
    println!("=== VEDA Comprehensive Feature Demo ===\n");
    
    // 1. Basic initialization with configuration
    demo_basic_config();
    
    // 2. Parallel iterators
    demo_parallel_iterators();
    
    // 3. Priority tasks
    demo_priority_tasks();
    
    // 4. Scoped parallelism
    demo_scoped_parallelism();
    
    // 5. Telemetry
    #[cfg(feature = "telemetry")]
    demo_telemetry();
    
    // 6. Async bridge
    #[cfg(feature = "async")]
    demo_async_bridge();
    
    println!("\n=== All demos completed successfully! ===");
}

fn demo_basic_config() {
    println!("1. Configuration Demo");
    println!("   Creating runtime with custom config...");
    
    shutdown();
    
    let config = Config::builder()
        .num_threads(4)
        .scheduling_policy(SchedulingPolicy::Adaptive)
        .pin_workers(false) // Set to true on Linux for core pinning
        .build()
        .unwrap();
    
    init_with_config(config).unwrap();
    println!("   ✓ Runtime initialized with 4 threads, adaptive scheduling\n");
}

fn demo_parallel_iterators() {
    println!("2. Parallel Iterator Demo");
    
    // Sum
    let sum: i32 = (0..1000).into_par_iter().sum();
    println!("   Sum(0..1000) = {}", sum);
    assert_eq!(sum, 499500);
    
    // Map + Filter + Sum
    let result: i32 = (0..100)
        .into_par_iter()
        .filter(|x| x % 2 == 0)
        .map(|x| x * 2)
        .sum();
    println!("   Sum(even * 2) = {}", result);
    
    // Collect
    let doubled: Vec<i32> = vec![1, 2, 3, 4, 5]
        .into_par_iter()
        .map(|x| x * 2)
        .collect();
    println!("   Doubled: {:?}", doubled);
    
    // Any/All predicates
    let has_large = (0..100).into_par_iter().any(|x| *x > 50);
    let all_positive = (1..100).into_par_iter().all(|x| *x > 0);
    println!("   Has value > 50: {}, All positive: {}", has_large, all_positive);
    
    println!("   ✓ All parallel iterator operations working\n");
}

fn demo_priority_tasks() {
    println!("3. Priority Task Demo");
    
    use veda_rs::runtime;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    let high_priority_count = Arc::new(AtomicUsize::new(0));
    let normal_priority_count = Arc::new(AtomicUsize::new(0));
    
    runtime::with_current_runtime(|rt| {
        // Submit high priority tasks
        for _ in 0..10 {
            let counter = high_priority_count.clone();
            let task = veda_rs::executor::Task::with_priority(
                move || {
                    counter.fetch_add(1, Ordering::Relaxed);
                },
                Priority::High,
            );
            rt.pool.submit_with_priority(task, Priority::High);
        }
        
        // Submit normal priority tasks
        for _ in 0..10 {
            let counter = normal_priority_count.clone();
            rt.pool.execute(move || {
                counter.fetch_add(1, Ordering::Relaxed);
            });
        }
    });
    
    // Give tasks time to complete
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    println!("   High priority tasks: {}", high_priority_count.load(Ordering::Relaxed));
    println!("   Normal priority tasks: {}", normal_priority_count.load(Ordering::Relaxed));
    println!("   ✓ Priority queue integrated\n");
}

fn demo_scoped_parallelism() {
    println!("4. Scoped Parallelism Demo");
    
    let data = vec![1, 2, 3, 4, 5];
    let mut results = vec![0; 5];
    
    veda_rs::scope::scope(|s| {
        for (i, &item) in data.iter().enumerate() {
            s.spawn(move || {
                // Access to `results` is safe within the scope
                std::thread::sleep(std::time::Duration::from_micros(100));
                println!("   Processing item {} in parallel", item);
            });
        }
    }); // All tasks complete here
    
    println!("   ✓ All scoped tasks completed\n");
}

#[cfg(feature = "telemetry")]
fn demo_telemetry() {
    use veda_rs::telemetry::export::ConsoleExporter;
    
    println!("5. Telemetry Demo");
    
    // Do some work to generate metrics
    let _sum: i64 = (0..10000).into_par_iter().sum();
    
    // Access and display metrics
    veda_rs::runtime::with_current_runtime(|rt| {
        let snapshot = rt.pool.metrics.snapshot();
        
        println!("   Tasks executed: {}", snapshot.tasks_executed);
        println!("   Tasks stolen: {}", snapshot.tasks_stolen);
        println!("   Utilization: {:.1}%", snapshot.utilization() * 100.0);
        println!("   P99 latency: {:.2}μs", snapshot.p99_latency_ns as f64 / 1000.0);
        
        // Export to console
        let exporter = ConsoleExporter::new(false);
        let _ = exporter.export(&snapshot);
    });
    
    println!("   ✓ Telemetry collecting and reporting\n");
}

#[cfg(feature = "async")]
fn demo_async_bridge() {
    println!("6. Async Bridge Demo");
    
    // Spawn async task
    let handle = veda_rs::spawn_async(async {
        std::thread::sleep(std::time::Duration::from_millis(10));
        42
    });
    
    let result = veda_rs::block_on(handle.join()).unwrap();
    println!("   Async task result: {}", result);
    assert_eq!(result, 42);
    
    // Stream processing (requires futures)
    use futures::stream;
    use veda_rs::ParStreamExt;
    
    let items = stream::iter(0..10);
    let doubled = veda_rs::block_on(
        items.par_map(|x| async move { x * 2 }).collect()
    );
    
    println!("   Parallel stream doubled: {:?}", &doubled[0..5]);
    
    println!("   ✓ Async integration working\n");
}
