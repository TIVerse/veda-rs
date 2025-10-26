//! Benchmarks comparing VEDA to Rayon

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn veda_par_iter_sum(c: &mut Criterion) {
    use veda::prelude::*;
    
    veda::init().unwrap();
    
    let mut group = c.benchmark_group("par_iter_sum");
    
    for size in [1_000, 10_000, 100_000, 1_000_000].iter() {
        group.bench_with_input(BenchmarkId::new("veda", size), size, |b, &size| {
            b.iter(|| {
                (0..size)
                    .into_par_iter()
                    .map(|x| black_box(x * 2))
                    .sum::<i64>()
            });
        });
    }
    
    group.finish();
    veda::shutdown();
}

fn rayon_par_iter_sum(c: &mut Criterion) {
    use rayon::prelude::*;
    
    let mut group = c.benchmark_group("par_iter_sum");
    
    for size in [1_000, 10_000, 100_000, 1_000_000].iter() {
        group.bench_with_input(BenchmarkId::new("rayon", size), size, |b, &size| {
            b.iter(|| {
                (0..size)
                    .into_par_iter()
                    .map(|x| black_box(x * 2))
                    .sum::<i64>()
            });
        });
    }
    
    group.finish();
}

fn veda_par_iter_collect(c: &mut Criterion) {
    use veda::prelude::*;
    
    veda::init().unwrap();
    
    c.bench_function("veda_collect_100k", |b| {
        b.iter(|| {
            let v: Vec<i32> = (0..100_000)
                .into_par_iter()
                .map(|x| black_box(x * 2))
                .collect();
            black_box(v)
        });
    });
    
    veda::shutdown();
}

fn rayon_par_iter_collect(c: &mut Criterion) {
    use rayon::prelude::*;
    
    c.bench_function("rayon_collect_100k", |b| {
        b.iter(|| {
            let v: Vec<i32> = (0..100_000)
                .into_par_iter()
                .map(|x| black_box(x * 2))
                .collect();
            black_box(v)
        });
    });
}

criterion_group!(
    benches,
    veda_par_iter_sum,
    rayon_par_iter_sum,
    veda_par_iter_collect,
    rayon_par_iter_collect
);
criterion_main!(benches);
