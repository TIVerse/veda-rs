//! Benchmarks for adaptive scheduling overhead

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use veda_rs::prelude::*;
use veda_rs::SchedulingPolicy;

fn bench_static_scheduling(c: &mut Criterion) {
    let config = veda_rs::Config::builder()
        .scheduling_policy(SchedulingPolicy::Fixed)
        .build()
        .unwrap();
    
    veda_rs::init_with_config(config).unwrap();
    
    c.bench_function("static_scheduling", |b| {
        b.iter(|| {
            (0..10_000 as i32)
                .into_par_iter()
                .map(|x| black_box(x * x))
                .sum::<i32>()
        })
    });
    
    veda_rs::shutdown();
}

fn bench_adaptive_scheduling(c: &mut Criterion) {
    let config = veda_rs::Config::builder()
        .scheduling_policy(SchedulingPolicy::Adaptive)
        .build()
        .unwrap();
    
    veda_rs::init_with_config(config).unwrap();
    
    c.bench_function("adaptive_scheduling", |b| {
        b.iter(|| {
            (0..10_000 as i32)
                .into_par_iter()
                .map(|x| black_box(x * x))
                .sum::<i32>()
        })
    });
    
    veda_rs::shutdown();
}

fn bench_variable_workload(c: &mut Criterion) {
    let config = veda_rs::Config::builder()
        .scheduling_policy(SchedulingPolicy::Adaptive)
        .build()
        .unwrap();
    
    veda_rs::init_with_config(config).unwrap();
    
    c.bench_function("variable_workload", |b| {
        b.iter(|| {
            (0..1_000 as i32)
                .into_par_iter()
                .map(|x| {
                    let n = black_box(x);
                    (0..n).sum::<i32>()
                })
                .sum::<i32>()
        })
    });
    
    veda_rs::shutdown();
}

criterion_group!(
    benches,
    bench_static_scheduling,
    bench_adaptive_scheduling,
    bench_variable_workload
);
criterion_main!(benches);
