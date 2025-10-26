//! Scalability stress tests

use veda_rs::prelude::*;
use std::time::Instant;

#[test]
fn test_large_range() {
    shutdown();
    init().unwrap();
    
    let start = Instant::now();
    let sum: i64 = (0..10_000_000).into_par_iter().sum();
    let duration = start.elapsed();
    
    assert_eq!(sum, 49999995000000);
    println!("Large range (10M items): {:?}", duration);
    
    shutdown();
}

#[test]
fn test_many_small_tasks() {
    shutdown();
    init().unwrap();
    
    let start = Instant::now();
    let results: Vec<i32> = (0..100_000)
        .into_par_iter()
        .map(|x| x * 2)
        .collect();
    let duration = start.elapsed();
    
    assert_eq!(results.len(), 100_000);
    println!("Many small tasks (100K): {:?}", duration);
    
    shutdown();
}

#[test]
fn test_nested_parallelism_scalability() {
    shutdown();
    init().unwrap();
    
    let start = Instant::now();
    let result: i64 = (0..100)
        .into_par_iter()
        .map(|x| {
            (0..1000).into_par_iter().map(|y| (x + y) as i64).sum::<i64>()
        })
        .sum();
    let duration = start.elapsed();
    
    assert!(result > 0);
    println!("Nested parallelism: {:?}", duration);
    
    shutdown();
}

#[test]
fn test_thread_scaling() {
    for num_threads in [1, 2, 4, 8] {
        shutdown();
        
        let config = Config::builder()
            .num_threads(num_threads)
            .build()
            .unwrap();
        
        init_with_config(config).unwrap();
        
        let start = Instant::now();
        let sum: i64 = (0..1_000_000).into_par_iter().sum();
        let duration = start.elapsed();
        
        assert_eq!(sum, 499999500000);
        println!("Threads: {}, Time: {:?}", num_threads, duration);
        
        shutdown();
    }
}

#[test]
fn test_memory_intensive() {
    shutdown();
    init().unwrap();
    
    let start = Instant::now();
    let data: Vec<Vec<i32>> = (0..1000)
        .into_par_iter()
        .map(|i| (0..1000).map(|j| i + j).collect())
        .collect();
    let duration = start.elapsed();
    
    assert_eq!(data.len(), 1000);
    assert_eq!(data[0].len(), 1000);
    println!("Memory intensive (1M i32s): {:?}", duration);
    
    shutdown();
}

#[test]
fn test_chunked_processing() {
    shutdown();
    init().unwrap();
    
    let data: Vec<i32> = (0..100_000).collect();
    
    let start = Instant::now();
    let sums: Vec<i32> = data.into_par_iter()
        .par_chunks(1000)
        .map(|chunk| chunk.iter().sum())
        .collect();
    let duration = start.elapsed();
    
    assert_eq!(sums.len(), 100);
    println!("Chunked processing (100K in 1K chunks): {:?}", duration);
    
    shutdown();
}

#[test]
fn test_high_contention() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    shutdown();
    init().unwrap();
    
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();
    
    let start = Instant::now();
    (0..100_000).into_par_iter().for_each(move |_| {
        counter_clone.fetch_add(1, Ordering::Relaxed);
    });
    let duration = start.elapsed();
    
    assert_eq!(counter.load(Ordering::Relaxed), 100_000);
    println!("High contention (100K atomic ops): {:?}", duration);
    
    shutdown();
}

#[test]
fn test_complex_pipeline() {
    shutdown();
    init().unwrap();
    
    let start = Instant::now();
    let result: i64 = (0..10_000)
        .into_par_iter()
        .filter(|x| x % 2 == 0)
        .map(|x| x * x)
        .filter(|x| x % 3 == 0)
        .map(|x| x as i64)
        .sum();
    let duration = start.elapsed();
    
    assert!(result > 0);
    println!("Complex pipeline: {:?}", duration);
    
    shutdown();
}
