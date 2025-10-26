//! Basic parallel iterator example - drop-in replacement for Rayon

use veda::prelude::*;

fn main() {
    // Initialize the VEDA runtime
    veda::init().expect("Failed to initialize VEDA");
    
    println!("=== Basic Parallel Iterator Example ===\n");
    
    // Simple parallel sum - identical to Rayon API
    let sum: i32 = (0..1000)
        .into_par_iter()
        .map(|x| x * 2)
        .sum();
    
    println!("Parallel sum: {}", sum);
    
    // Parallel collect
    let doubled: Vec<i32> = (0..100)
        .into_par_iter()
        .map(|x| x * 2)
        .collect();
    
    println!("Doubled first 10: {:?}", &doubled[..10]);
    
    // Parallel filter and map
    let evens: Vec<i32> = (0..1000)
        .into_par_iter()
        .filter(|x| x % 2 == 0)
        .map(|x| x * x)
        .collect();
    
    println!("Even squares (first 10): {:?}", &evens[..10]);
    
    // Parallel fold
    let product: i64 = (1..=10)
        .into_par_iter()
        .map(|x| x as i64)
        .fold(|| 1i64, |acc, x| acc * x)
        .sum();
    
    println!("Factorial of 10: {}", product);
    
    // Shutdown runtime
    veda::shutdown();
    
    println!("\n=== Example Complete ===");
}
