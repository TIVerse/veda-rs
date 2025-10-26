//! Benchmarks comparing parallel vs sequential execution

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use veda_rs::prelude::*;

fn sequential_sum(n: i32) -> i32 {
    (0..n).sum()
}

fn parallel_sum(n: i32) -> i32 {
    (0i32..n).into_par_iter().sum()
}

fn sequential_map_filter(n: i32) -> Vec<i32> {
    (0..n).filter(|x| x % 2 == 0).map(|x| x * x).collect()
}

fn parallel_map_filter(n: i32) -> Vec<i32> {
    (0i32..n)
        .into_par_iter()
        .filter(|x| x % 2 == 0)
        .map(|x| x * x)
        .collect()
}

fn sequential_fold(n: i32) -> i64 {
    (1..=n)
        .map(|x| x as i64)
        .fold(1, |acc, x| acc * (x % 100 + 1))
}

fn parallel_fold(n: i32) -> i64 {
    (1i32..=n)
        .into_par_iter()
        .map(|x| x as i64)
        .fold(|| 1i64, |acc, x| acc * (x % 100 + 1))
        .reduce(|a, b| a * b)
}

fn bench_sum(c: &mut Criterion) {
    veda_rs::init().expect("Failed to initialize");

    let mut group = c.benchmark_group("sum");

    for size in [100, 1_000, 10_000, 100_000].iter() {
        group.bench_with_input(BenchmarkId::new("sequential", size), size, |b, &size| {
            b.iter(|| sequential_sum(black_box(size)))
        });

        group.bench_with_input(BenchmarkId::new("parallel", size), size, |b, &size| {
            b.iter(|| parallel_sum(black_box(size)))
        });
    }

    group.finish();
    veda_rs::shutdown();
}

fn bench_map_filter(c: &mut Criterion) {
    veda_rs::init().expect("Failed to initialize");

    let mut group = c.benchmark_group("map_filter");

    for size in [100, 1_000, 10_000, 100_000].iter() {
        group.bench_with_input(BenchmarkId::new("sequential", size), size, |b, &size| {
            b.iter(|| sequential_map_filter(black_box(size)))
        });

        group.bench_with_input(BenchmarkId::new("parallel", size), size, |b, &size| {
            b.iter(|| parallel_map_filter(black_box(size)))
        });
    }

    group.finish();
    veda_rs::shutdown();
}

fn bench_fold(c: &mut Criterion) {
    veda_rs::init().expect("Failed to initialize");

    let mut group = c.benchmark_group("fold");

    for size in [100, 1_000, 10_000].iter() {
        group.bench_with_input(BenchmarkId::new("sequential", size), size, |b, &size| {
            b.iter(|| sequential_fold(black_box(size)))
        });

        group.bench_with_input(BenchmarkId::new("parallel", size), size, |b, &size| {
            b.iter(|| parallel_fold(black_box(size)))
        });
    }

    group.finish();
    veda_rs::shutdown();
}

criterion_group!(benches, bench_sum, bench_map_filter, bench_fold);
criterion_main!(benches);
