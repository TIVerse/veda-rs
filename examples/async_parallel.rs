//! Async parallel execution example

#[cfg(feature = "async")]
use veda_rs::prelude::*;

#[cfg(feature = "async")]
use std::time::Duration;

#[cfg(feature = "async")]
async fn async_task(id: usize, duration_ms: u64) -> usize {
    tokio::time::sleep(Duration::from_millis(duration_ms)).await;
    println!("  Task {} completed after {}ms", id, duration_ms);
    id * 2
}

#[cfg(feature = "async")]
async fn demo_spawn_async() {
    println!("=== spawn_async Demo ===\n");

    init().unwrap();

    // Spawn multiple async tasks
    let handles: Vec<_> = (0..5)
        .map(|i| veda_rs::spawn_async(async move { async_task(i, 50 + i as u64 * 10).await }))
        .collect();

    // Wait for all to complete
    let mut results = Vec::new();
    for handle in handles {
        let result = veda_rs::block_on(handle.join()).unwrap();
        results.push(result);
    }

    println!("\nResults: {:?}", results);
    println!("✓ All async tasks completed\n");

    shutdown();
}

#[cfg(feature = "async")]
async fn demo_par_stream() {
    use futures::stream;
    use veda_rs::ParStreamExt;

    println!("=== Parallel Async Streams Demo ===\n");

    init().unwrap();

    // Create a stream of items
    let items = stream::iter(0..10);

    println!("Processing stream items in parallel...");

    // Process stream items in parallel
    let results = items
        .par_map(|x| async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            x * 2
        })
        .collect()
        .await;

    println!("Results: {:?}", results);
    println!("✓ Parallel stream processing complete\n");

    shutdown();
}

#[cfg(feature = "async")]
async fn demo_mixed_workload() {
    println!("=== Mixed CPU/Async Workload Demo ===\n");

    init().unwrap();

    // CPU-bound parallel work
    println!("Running CPU-bound parallel work...");
    let cpu_result: i64 = (0..1000).into_par_iter().map(|x| x * x).sum();
    println!("  CPU result: {}", cpu_result);

    // Async I/O-bound work
    println!("\nRunning async I/O-bound work...");
    let async_handles: Vec<_> = (0..5)
        .map(|i| {
            veda_rs::spawn_async(async move {
                tokio::time::sleep(Duration::from_millis(20)).await;
                format!("async-result-{}", i)
            })
        })
        .collect();

    let async_results: Vec<String> = futures::future::join_all(
        async_handles
            .into_iter()
            .map(|h| veda_rs::block_on(h.join())),
    )
    .await
    .into_iter()
    .filter_map(|r| r.ok())
    .collect();

    println!("  Async results: {:?}", async_results);

    // More CPU work
    println!("\nRunning more CPU work...");
    let cpu_result2: Vec<i32> = vec![1, 2, 3, 4, 5]
        .into_par_iter()
        .map(|x| x * 10)
        .collect();
    println!("  CPU result: {:?}", cpu_result2);

    println!("\n✓ Mixed workload complete\n");

    shutdown();
}

#[cfg(feature = "async")]
async fn demo_par_for_each() {
    use futures::stream;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use veda_rs::ParStreamExt;

    println!("=== Parallel for_each Demo ===\n");

    init().unwrap();

    let counter = Arc::new(AtomicUsize::new(0));
    let items = stream::iter(0..20);

    println!("Processing 20 items with par_for_each...");

    let counter_clone = counter.clone();
    items
        .par_for_each(move |_item| {
            let c = counter_clone.clone();
            async move {
                tokio::time::sleep(Duration::from_millis(5)).await;
                c.fetch_add(1, Ordering::Relaxed);
            }
        })
        .execute()
        .await;

    println!("Processed {} items", counter.load(Ordering::Relaxed));
    println!("✓ par_for_each complete\n");

    shutdown();
}

#[cfg(feature = "async")]
async fn demo_par_filter() {
    use futures::stream;
    use veda_rs::ParStreamExt;

    println!("=== Parallel Filter Demo ===\n");

    init().unwrap();

    let items = stream::iter(0..20);

    println!("Filtering even numbers in parallel...");

    let filtered = items
        .par_filter(|x| async move {
            tokio::time::sleep(Duration::from_millis(1)).await;
            *x % 2 == 0
        })
        .collect()
        .await;

    println!("Filtered results: {:?}", filtered);
    println!("✓ Parallel filter complete\n");

    shutdown();
}

#[cfg(not(feature = "async"))]
fn main() {
    println!("This example requires the 'async' feature.");
    println!("Run with: cargo run --example async_parallel --features async");
}

#[cfg(feature = "async")]
#[tokio::main]
async fn main() {
    println!("╔════════════════════════════════════════════╗");
    println!("║  VEDA Async Parallel Execution Examples   ║");
    println!("╚════════════════════════════════════════════╝\n");

    demo_spawn_async().await;
    demo_par_stream().await;
    demo_par_for_each().await;
    demo_par_filter().await;
    demo_mixed_workload().await;

    println!("╔════════════════════════════════════════════╗");
    println!("║         All Examples Completed!            ║");
    println!("╚════════════════════════════════════════════╝");
}
