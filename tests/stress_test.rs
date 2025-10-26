//! Stress tests for VEDA runtime

use veda_rs::prelude::*;
use std::sync::Arc;
use parking_lot::Mutex;

#[test]
#[ignore] // Run with --ignored flag
fn stress_test_many_small_tasks() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();
    
    for _ in 0..100 {
        let sum: i32 = (0i32..1000i32).into_par_iter().sum();
        assert_eq!(sum, 499_500);
    }
    
    veda_rs::shutdown();
}

#[test]
#[ignore]
fn stress_test_nested_scopes() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();
    
    let counter = Arc::new(Mutex::new(0));
    
    for _ in 0..10 {
        veda_rs::scope::scope(|s| {
            for _ in 0..100 {
                let counter = counter.clone();
                s.spawn(move || {
                    veda_rs::scope::scope(|s2| {
                        for _ in 0..10 {
                            let counter = counter.clone();
                            s2.spawn(move || {
                                *counter.lock() += 1;
                            });
                        }
                    });
                });
            }
        });
    }
    
    assert_eq!(*counter.lock(), 100_000);
    
    veda_rs::shutdown();
}

#[test]
#[ignore]
fn stress_test_concurrent_init_shutdown() {
    use std::thread;
    
    // Test that multiple init/shutdown cycles work correctly
    for i in 0..10 {
        veda_rs::shutdown();
        veda_rs::init().unwrap();
        
        let sum: i32 = (0i32..100i32).into_par_iter().sum();
        assert_eq!(sum, 4950, "Iteration {}", i);
        
        veda_rs::shutdown();
        
        // Brief pause to ensure cleanup
        thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[test]
#[ignore]
fn stress_test_high_contention() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();
    
    let data = Arc::new(Mutex::new(vec![0i32; 100]));
    
    veda_rs::scope::scope(|s| {
        for _ in 0..1000 {
            let data = data.clone();
            s.spawn(move || {
                let mut guard = data.lock();
                for item in guard.iter_mut() {
                    *item += 1;
                }
            });
        }
    });
    
    let guard = data.lock();
    assert!(guard.iter().all(|&x| x == 1000));
    
    veda_rs::shutdown();
}

#[test]
#[ignore]
fn stress_test_panic_recovery() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();
    
    // Spawn tasks that panic - should not crash the runtime
    for _ in 0..10 {
        veda_rs::scope::scope(|s| {
            // Mix of panicking and non-panicking tasks
            for i in 0..100 {
                s.spawn(move || {
                    if i % 10 == 0 {
                        panic!("Intentional panic");
                    }
                });
            }
        });
    }
    
    // Runtime should still work after panics
    let sum: i32 = (0i32..100i32).into_par_iter().sum();
    assert_eq!(sum, 4950);
    
    veda_rs::shutdown();
}

#[test]
#[ignore]
fn stress_test_large_parallel_workload() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();
    
    // Process 1 million items
    let sum: i64 = (0i32..1_000_000i32)
        .into_par_iter()
        .map(|x| x as i64)
        .filter(|x| x % 2 == 0)
        .map(|x| x * x)
        .sum();
    
    // Verify computation is correct
    let expected: i64 = (0..1_000_000i64)
        .filter(|x| x % 2 == 0)
        .map(|x| x * x)
        .sum();
    
    assert_eq!(sum, expected);
    
    veda_rs::shutdown();
}

#[test]
#[ignore]
fn stress_test_repeated_operations() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();
    
    // Repeat the same operation many times to test stability
    for iteration in 0..1000 {
        let sum: i32 = (0i32..100i32).into_par_iter().sum();
        assert_eq!(sum, 4950, "Failed at iteration {}", iteration);
    }
    
    veda_rs::shutdown();
}

#[test]
#[ignore]
fn stress_test_memory_pressure() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();
    
    // Create large allocations to test memory handling
    let _results: Vec<Vec<i32>> = (0i32..100i32)
        .into_par_iter()
        .map(|i| {
            // Each task allocates a large vector
            (0..10000).map(|j| i * 10000 + j).collect()
        })
        .collect();
    
    veda_rs::shutdown();
}

#[test]
#[ignore]
fn stress_test_work_stealing() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();
    
    // Variable workload to trigger work stealing
    let results: Vec<i32> = (0i32..1000i32)
        .into_par_iter()
        .map(|x| {
            // Simulate variable work - some tasks are slower
            if x % 100 == 0 {
                std::thread::sleep(std::time::Duration::from_micros(100));
            }
            x * 2
        })
        .collect();
    
    assert_eq!(results.len(), 1000);
    
    veda_rs::shutdown();
}
