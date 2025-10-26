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

    let doubled: Vec<i32> = (0i32..100i32).into_par_iter().map(|x| x * 2).collect();

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
fn test_parallel_fold_reduce() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();

    // Test proper parallel fold with reduce using the special method
    let product: i64 = (1i32..=5i32)
        .into_par_iter()
        .map(|x| x as i64)
        .fold(|| 1i64, |acc, x| acc * x)
        .reduce(|a, b| a * b);

    assert_eq!(product, 120);

    // Test reduce on range directly
    let sum: i32 = (1i32..=10i32).into_par_iter().reduce(|| 0, |a, b| a + b);

    assert_eq!(sum, 55);

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

    use parking_lot::Mutex;
    use std::sync::Arc;

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

    let sum: i64 = (0i32..1_000_000i32).into_par_iter().map(|x| x as i64).sum();

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

#[test]
fn test_single_element() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();

    let sum: i32 = (5i32..6i32).into_par_iter().sum();
    assert_eq!(sum, 5);

    let result: Vec<i32> = (5i32..6i32).into_par_iter().map(|x| x * 2).collect();
    assert_eq!(result, vec![10]);

    veda_rs::shutdown();
}

#[test]
fn test_chained_operations() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();

    let result: Vec<i32> = (1i32..100i32)
        .into_par_iter()
        .filter(|x| x % 2 == 0)
        .map(|x| x * x)
        .filter(|x| *x < 1000)
        .collect();

    assert!(result.len() > 0);
    assert!(result.iter().all(|&x| x < 1000));

    veda_rs::shutdown();
}

#[test]
fn test_different_numeric_types() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();

    let sum_u32: u32 = (0u32..100u32).into_par_iter().sum();
    assert_eq!(sum_u32, 4950);

    let sum_i64: i64 = (0i64..100i64).into_par_iter().sum();
    assert_eq!(sum_i64, 4950);

    let sum_usize: usize = (0usize..100usize).into_par_iter().sum();
    assert_eq!(sum_usize, 4950);

    veda_rs::shutdown();
}

#[test]
fn test_reduce_operations() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();

    let max: i32 = (1i32..=100i32)
        .into_par_iter()
        .reduce(|| i32::MIN, |a, b| a.max(b));
    assert_eq!(max, 100);

    let min: i32 = (1i32..=100i32)
        .into_par_iter()
        .reduce(|| i32::MAX, |a, b| a.min(b));
    assert_eq!(min, 1);

    veda_rs::shutdown();
}

#[test]
fn test_range_inclusive() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();

    let sum: i32 = (1i32..=10i32).into_par_iter().sum();
    assert_eq!(sum, 55);

    let collected: Vec<i32> = (1i32..=5i32).into_par_iter().collect();
    assert_eq!(collected.len(), 5);
    assert!(collected.contains(&1));
    assert!(collected.contains(&5));

    veda_rs::shutdown();
}

#[test]
fn test_vec_parallel_iter() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();

    let vec = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let sum: i32 = vec.into_par_iter().sum();
    assert_eq!(sum, 55);

    veda_rs::shutdown();
}

#[test]
fn test_vec_map_collect() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();

    let vec = vec![1, 2, 3, 4, 5];
    let doubled: Vec<i32> = vec.into_par_iter().map(|x| x * 2).collect();
    assert_eq!(doubled.len(), 5);
    assert!(doubled.contains(&2));
    assert!(doubled.contains(&10));

    veda_rs::shutdown();
}

#[test]
fn test_slice_parallel_iter() {
    use veda_rs::prelude::ParallelSlice;

    veda_rs::shutdown();
    veda_rs::init().unwrap();

    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let sum: i32 = data.par_iter().sum();
    assert_eq!(sum, 55);

    veda_rs::shutdown();
}

#[test]
fn test_any_predicate() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();

    assert!((0i32..100).into_par_iter().any(|x| *x > 50));
    assert!(!(0i32..10).into_par_iter().any(|x| *x > 100));

    let vec = vec![1, 2, 3, 4, 5];
    assert!(vec.into_par_iter().any(|x| *x == 3));

    veda_rs::shutdown();
}

#[test]
fn test_all_predicate() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();

    assert!((1i32..100).into_par_iter().all(|x| *x > 0));
    assert!(!(0i32..100).into_par_iter().all(|x| *x > 50));

    let vec = vec![2, 4, 6, 8, 10];
    assert!(vec.into_par_iter().all(|x| *x % 2 == 0));

    veda_rs::shutdown();
}

#[test]
fn test_find_any_method() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();

    let found = (0i32..100).into_par_iter().find_any(|x| *x == 42);
    assert_eq!(found, Some(42));

    let not_found = (0i32..10).into_par_iter().find_any(|x| *x > 100);
    assert_eq!(not_found, None);

    veda_rs::shutdown();
}

#[test]
fn test_enumerate_combinator() {
    veda_rs::shutdown();
    veda_rs::init().unwrap();

    let results: Vec<(usize, i32)> = (10i32..15i32).into_par_iter().enumerate().collect();

    assert_eq!(results.len(), 5);

    veda_rs::shutdown();
}
