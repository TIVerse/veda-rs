use veda_rs::prelude::*;

#[test]
fn test_basic_initialization() {
    veda_rs::shutdown();
    assert!(veda_rs::init().is_ok());
    veda_rs::shutdown();
}

#[test]
fn test_parallel_sum() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();
    
    let sum: i32 = (0i32..1000i32).into_par_iter().sum();
    assert_eq!(sum, 499500);
    
    veda_rs::shutdown();
}

#[test]
fn test_parallel_map_collect() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();
    
    let doubled: Vec<i32> = (0i32..100i32)
        .into_par_iter()
        .map(|x| x * 2)
        .collect();
    
    assert_eq!(doubled.len(), 100);
    assert_eq!(doubled[0], 0);
    assert_eq!(doubled[99], 198);
    
    veda_rs::shutdown();
}

#[test]
fn test_parallel_filter() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();
    
    let evens: Vec<i32> = (0i32..100i32)
        .into_par_iter()
        .filter(|x| x % 2 == 0)
        .collect();
    
    assert_eq!(evens.len(), 50);
    assert!(evens.iter().all(|x| x % 2 == 0));
    
    veda_rs::shutdown();
}

#[test]
fn test_parallel_fold() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();
    
    let product: i64 = (1i32..=5i32)
        .into_par_iter()
        .map(|x| x as i64)
        .fold(|| 1i64, |acc, x| acc * x)
        .sum();
    
    assert_eq!(product, 120);
    
    veda_rs::shutdown();
}

#[test]
fn test_custom_config() {
    veda_rs::shutdown();
    
    let config = veda_rs::Config::builder()
        .num_threads(2)
        .scheduling_policy(veda_rs::SchedulingPolicy::Fixed)
        .build()
        .unwrap();
    
    assert!(veda_rs::init_with_config(config).is_ok());
    
    let sum: i32 = (0i32..100i32).into_par_iter().sum();
    assert_eq!(sum, 4950);
    
    veda_rs::shutdown();
}

#[test]
fn test_scoped_spawn() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();
    
    use std::sync::Arc;
    use parking_lot::Mutex;
    
    let counter = Arc::new(Mutex::new(0));
    
    veda_rs::scope::scope(|s| {
        for _ in 0..10 {
            let counter = counter.clone();
            s.spawn(move || {
                *counter.lock() += 1;
            });
        }
    });
    
    assert_eq!(*counter.lock(), 10);
    
    veda_rs::shutdown();
}

#[test]
fn test_nested_parallelism() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();
    
    let result: Vec<Vec<i32>> = (0i32..10i32)
        .into_par_iter()
        .map(|i| {
            (0i32..10i32)
                .into_par_iter()
                .map(move |j| i * 10 + j)
                .collect()
        })
        .collect();
    
    assert_eq!(result.len(), 10);
    assert_eq!(result[0].len(), 10);
    assert_eq!(result[5][7], 57);
    
    veda_rs::shutdown();
}

#[test]
fn test_large_workload() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();
    
    let sum: i64 = (0i32..1_000_000i32)
        .into_par_iter()
        .map(|x| x as i64)
        .sum();
    
    assert_eq!(sum, 499999500000);
    
    veda_rs::shutdown();
}

#[test]
fn test_empty_iterator() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();
    
    let sum: i32 = (0i32..0i32).into_par_iter().sum();
    assert_eq!(sum, 0);
    
    let empty: Vec<i32> = (0i32..0i32).into_par_iter().collect();
    assert!(empty.is_empty());
    
    veda_rs::shutdown();
}
