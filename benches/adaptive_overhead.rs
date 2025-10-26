//! Benchmarks for adaptive scheduling overhead

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use veda::prelude::*;

fn bench_fixed_scheduling(c: &mut Criterion) {
    let config = veda::Config::builder()
        .scheduling_policy(veda::SchedulingPolicy::Fixed)
        .build()
        .unwrap();
    
    veda::init_with_config(config).unwrap();
    
    c.bench_function("fixed_scheduling", |b| {
        b.iter(|| {
            (0..10_000)
                .into_par_iter()
                .map(|x| black_box(x * x))
                .sum::<i32>()
        });
    });
    
    veda::shutdown();
}

fn bench_adaptive_scheduling(c: &mut Criterion) {
    let config = veda::Config::builder()
        .scheduling_policy(veda::SchedulingPolicy::Adaptive)
        .build()
        .unwrap();
    
    veda::init_with_config(config).unwrap();
    
    c.bench_function("adaptive_scheduling", |b| {
        b.iter(|| {
            (0..10_000)
                .into_par_iter()
                .map(|x| black_box(x * x))
                .sum::<i32>()
        });
    });
    
    veda::shutdown();
}

fn bench_variable_workload(c: &mut Criterion) {
    let config = veda::Config::builder()
        .scheduling_policy(veda::SchedulingPolicy::Adaptive)
        .build()
        .unwrap();
    
    veda::init_with_config(config).unwrap();
    
    c.bench_function("variable_workload", |b| {
        b.iter(|| {
            (0..1000)
                .into_par_iter()
                .map(|x| {
                    // Variable workload
                    let n = if x % 10 == 0 { 1000 } else { 10 };
                    (0..n).sum::<i32>()
                })
                .sum::<i32>()
        });
    });
    
    veda::shutdown();
}

criterion_group!(
    benches,
    bench_fixed_scheduling,
    bench_adaptive_scheduling,
    bench_variable_workload
);
criterion_main!(benches);
