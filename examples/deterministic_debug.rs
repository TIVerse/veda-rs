//! Deterministic execution for debugging and testing

#[cfg(feature = "deterministic")]
use veda::prelude::*;

#[cfg(feature = "deterministic")]
fn complex_computation(x: i32) -> i32 {
    let mut result = x;
    for i in 1..10 {
        result = result.wrapping_mul(i).wrapping_add(x);
    }
    result
}

#[cfg(feature = "deterministic")]
fn main() {
    println!("=== Deterministic Execution Example ===\n");
    
    let seed = 42;
    
    // Configure deterministic mode
    let config = veda::Config::builder()
        .scheduling_policy(veda::SchedulingPolicy::Deterministic { seed })
        .num_threads(4)
        .build()
        .expect("Failed to build config");
    
    veda::init_with_config(config).expect("Failed to initialize");
    
    println!("Running with deterministic seed: {}", seed);
    
    // Run 1
    let result1: Vec<i32> = (0..100)
        .into_par_iter()
        .map(complex_computation)
        .collect();
    
    println!("First run complete: {} results", result1.len());
    
    veda::shutdown();
    
    // Run 2 with same seed
    let config2 = veda::Config::builder()
        .scheduling_policy(veda::SchedulingPolicy::Deterministic { seed })
        .num_threads(4)
        .build()
        .expect("Failed to build config");
    
    veda::init_with_config(config2).expect("Failed to initialize");
    
    let result2: Vec<i32> = (0..100)
        .into_par_iter()
        .map(complex_computation)
        .collect();
    
    println!("Second run complete: {} results", result2.len());
    
    // Verify determinism
    let identical = result1 == result2;
    println!("\nResults identical: {}", identical);
    
    if identical {
        println!("✓ Deterministic execution verified!");
    } else {
        println!("✗ Results differ (this shouldn't happen)");
    }
    
    veda::shutdown();
    println!("\n=== Example Complete ===");
}

#[cfg(not(feature = "deterministic"))]
fn main() {
    eprintln!("This example requires the 'deterministic' feature.");
    eprintln!("Run with: cargo run --example deterministic_debug --features deterministic");
}
