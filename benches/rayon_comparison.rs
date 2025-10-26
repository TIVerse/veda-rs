//! Benchmarks comparing VEDA to Rayon

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use veda_rs::prelude::*;
use rayon::prelude::*;

mod rayon_bench {
    use rayon::prelude::*;
    
    pub fn par_range(start: i64, end: i64) -> impl ParallelIterator<Item = i64> + 'static {
        (start..end).into_par_iter()
    }
}

mod veda_bench {
    use veda_rs::prelude::*;
    
    pub fn par_range(start: i64, end: i64) -> impl ParallelIterator<Item = i64> + 'static {
        (start..end).into_par_iter()
    }
}

fn veda_par_iter_sum(c: &mut Criterion) {
    veda_rs::init().unwrap();
    
    let mut group = c.benchmark_group("par_iter_sum");
    
    for size in [1_000, 10_000, 100_000, 1_000_000].iter() {
        group.bench_with_input(BenchmarkId::new("veda", size), size, |b, &size| {
            b.iter(|| {
                veda_bench::par_range(0, size as i64)
                    .map(|x| black_box(x * 2))
                    .sum::<i64>()
            })
        });
    }
    
    group.finish();
    veda_rs::shutdown();
}

fn rayon_par_iter_sum(c: &mut Criterion) {
    let mut group = c.benchmark_group("par_iter_sum");
    
    for size in [1_000, 10_000, 100_000, 1_000_000].iter() {
        group.bench_with_input(BenchmarkId::new("rayon", size), size, |b, &size| {
            b.iter(|| {
                use rayon::iter::ParallelIterator;
                rayon_bench::par_range(0, size as i64)
                    .map(|x| black_box(x * 2))
                    .sum::<i64>()
            })
        });
    }
    
    group.finish();
}

fn veda_par_iter_collect(c: &mut Criterion) {
    veda_rs::init().unwrap();
    
    let mut group = c.benchmark_group("par_iter_collect");
    
    for size in [1_000, 10_000, 100_000].iter() {
        group.bench_with_input(BenchmarkId::new("veda", size), size, |b, &size| {
            b.iter(|| {
                let range: std::ops::Range<i64> = 0..size as i64;
                veda_rs::IntoParallelIterator::into_par_iter(range)
                    .map(|x| black_box(x * 2))
                    .collect::<Vec<_>>()
            })
        });
    }
    
    group.finish();
    veda_rs::shutdown();
}

fn rayon_par_iter_collect(c: &mut Criterion) {
    let mut group = c.benchmark_group("par_iter_collect");
    
    for size in [1_000, 10_000, 100_000].iter() {
        group.bench_with_input(BenchmarkId::new("rayon", size), size, |b, &size| {
            b.iter(|| {
                use rayon::iter::ParallelIterator;
                rayon_bench::par_range(0, size as i64)
                    .map(|x| black_box(x * 2))
                    .collect::<Vec<_>>()
            })
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    veda_par_iter_sum,
    rayon_par_iter_sum,
    veda_par_iter_collect,
    rayon_par_iter_collect
);

criterion_main!(benches);
