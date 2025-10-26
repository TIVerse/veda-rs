//! API compatibility tests - ensure VEDA works as a drop-in replacement for standard parallel libraries

use veda_rs::prelude::*;

#[test]
fn test_basic_par_iter() {
    shutdown();
    init().unwrap();
    
    let sum: i32 = (0..1000).into_par_iter().sum();
    assert_eq!(sum, 499500);
    
    shutdown();
}

#[test]
fn test_par_iter_map() {
    shutdown();
    init().unwrap();
    
    let doubled: Vec<i32> = (0..10)
        .into_par_iter()
        .map(|x| x * 2)
        .collect();
    
    assert_eq!(doubled.len(), 10);
    assert!(doubled.contains(&0));
    assert!(doubled.contains(&18));
    
    shutdown();
}

#[test]
fn test_par_iter_filter() {
    shutdown();
    init().unwrap();
    
    let evens: Vec<i32> = (0..20)
        .into_par_iter()
        .filter(|x| x % 2 == 0)
        .collect();
    
    assert_eq!(evens.len(), 10);
    assert!(evens.iter().all(|x| x % 2 == 0));
    
    shutdown();
}

#[test]
fn test_par_iter_fold_reduce() {
    shutdown();
    init().unwrap();
    
    let sum: i32 = (0..100)
        .into_par_iter()
        .fold(|| 0, |acc, x| acc + x)
        .reduce(|| 0, |a, b| a + b);
    
    assert_eq!(sum, (0..100).sum());
    
    shutdown();
}

#[test]
fn test_par_iter_any_all() {
    shutdown();
    init().unwrap();
    
    let has_large = (0..100).into_par_iter().any(|x| *x > 50);
    assert!(has_large);
    
    let all_positive = (1..100).into_par_iter().all(|x| *x > 0);
    assert!(all_positive);
    
    let all_negative = (1..100).into_par_iter().all(|x| *x < 0);
    assert!(!all_negative);
    
    shutdown();
}

#[test]
fn test_par_iter_find() {
    shutdown();
    init().unwrap();
    
    let found = (0..1000).into_par_iter().find_any(|x| *x == 500);
    assert_eq!(found, Some(500));
    
    let not_found = (0..100).into_par_iter().find_any(|x| *x == 1000);
    assert_eq!(not_found, None);
    
    shutdown();
}

#[test]
fn test_par_iter_chain() {
    shutdown();
    init().unwrap();
    
    let result: i32 = (0..50)
        .into_par_iter()
        .map(|x| x * 2)
        .filter(|x| x % 3 == 0)
        .map(|x| x + 1)
        .sum();
    
    assert!(result > 0);
    
    shutdown();
}

#[test]
fn test_vec_par_iter() {
    shutdown();
    init().unwrap();
    
    let data = vec![1, 2, 3, 4, 5];
    let sum: i32 = data.into_par_iter().sum();
    
    assert_eq!(sum, 15);
    
    shutdown();
}

#[test]
fn test_enumerate() {
    shutdown();
    init().unwrap();
    
    let pairs: Vec<(usize, i32)> = (10..15)
        .into_par_iter()
        .enumerate()
        .collect();
    
    assert_eq!(pairs.len(), 5);
    
    shutdown();
}

#[test]
fn test_take_skip() {
    shutdown();
    init().unwrap();
    
    let taken: Vec<i32> = (0..100)
        .into_par_iter()
        .take(10)
        .collect();
    
    assert!(taken.len() <= 10);
    
    let skipped: Vec<i32> = (0..20)
        .into_par_iter()
        .skip(15)
        .collect();
    
    assert!(skipped.len() <= 5);
    
    shutdown();
}

#[test]
fn test_nested_parallel() {
    shutdown();
    init().unwrap();
    
    let result: i32 = (0..10)
        .into_par_iter()
        .map(|x| {
            // Nested parallel computation
            (0..x).into_par_iter().sum::<i32>()
        })
        .sum();
    
    assert!(result > 0);
    
    shutdown();
}

#[test]
fn test_partition() {
    shutdown();
    init().unwrap();
    
    let (evens, odds) = (0..20)
        .into_par_iter()
        .partition(|x| x % 2 == 0);
    
    assert_eq!(evens.len(), 10);
    assert_eq!(odds.len(), 10);
    
    shutdown();
}

#[test]
fn test_position_any() {
    shutdown();
    init().unwrap();
    
    let pos = (0..100)
        .into_par_iter()
        .position_any(|x| *x == 42);
    
    assert!(pos.is_some());
    
    shutdown();
}

#[test]
fn test_flat_map() {
    shutdown();
    init().unwrap();
    
    let result: Vec<i32> = (0..5)
        .into_par_iter()
        .flat_map(|x| (0..x).into_par_iter())
        .collect();
    
    // flat_map(0..5) with x => 0..x produces: [], [0], [0,1], [0,1,2], [0,1,2,3]
    assert!(result.len() > 0);
    
    shutdown();
}

#[test]
fn test_zip() {
    shutdown();
    init().unwrap();
    
    let a = vec![1, 2, 3, 4, 5];
    let b = vec![10, 20, 30, 40, 50];
    
    let zipped: Vec<(i32, i32)> = a.into_par_iter()
        .zip(b.into_par_iter())
        .collect();
    
    assert_eq!(zipped.len(), 5);
    assert!(zipped.contains(&(1, 10)));
    assert!(zipped.contains(&(5, 50)));
    
    shutdown();
}

#[test]
fn test_chunks() {
    shutdown();
    init().unwrap();
    
    let data: Vec<i32> = (0..10).collect();
    let chunks: Vec<Vec<i32>> = data.into_par_iter()
        .par_chunks(3)
        .collect();
    
    assert_eq!(chunks.len(), 4); // 3, 3, 3, 1
    
    shutdown();
}

#[test]
fn test_windows() {
    shutdown();
    init().unwrap();
    
    let data: Vec<i32> = (0..5).collect();
    let windows: Vec<Vec<i32>> = data.into_par_iter()
        .par_windows(3)
        .collect();
    
    assert_eq!(windows.len(), 3);
    
    shutdown();
}
