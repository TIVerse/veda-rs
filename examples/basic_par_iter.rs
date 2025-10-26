use veda_rs::prelude::*;

fn main() {
    veda_rs::init().expect("Failed to init");
    
    println!("=== Basic Parallel Iterator Example ===\n");
    let sum: i32 = (0i32..1000i32)
        .into_par_iter()
        .map(|x| x * 2)
        .sum();
    
    println!("Parallel sum: {}", sum);
    let doubled: Vec<i32> = (0i32..100i32)
        .into_par_iter()
        .map(|x| x * 2)
        .collect();
    
    println!("Doubled first 10: {:?}", &doubled[..10]);
    let evens: Vec<i32> = (0i32..1000i32)
        .into_par_iter()
        .filter(|x| x % 2 == 0)
        .map(|x| x * x)
        .collect();
    
    println!("Even squares (first 10): {:?}", &evens[..10]);
    let product: i64 = (1i32..=10i32)
        .into_par_iter()
        .map(|x| x as i64)
        .fold(|| 1i64, |acc, x| acc * x)
        .sum();
    
    println!("Factorial of 10: {}", product);
    
    veda_rs::shutdown();
    
    println!("\n=== Example Complete ===");
}
