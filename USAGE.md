# VEDA-RS Usage Guide

## Table of Contents
- [Installation](#installation)
- [Basic Usage](#basic-usage)
- [Parallel Iterators](#parallel-iterators)
- [Configuration](#configuration)
- [Scoped Parallelism](#scoped-parallelism)
- [Async Integration](#async-integration)
- [GPU Support](#gpu-support)
- [Error Handling](#error-handling)
- [Best Practices](#best-practices)
- [Examples](#examples)

## Installation

Add VEDA-RS to your `Cargo.toml`:

```toml
[dependencies]
veda-rs = "1.0"
```

### Feature Flags

```toml
[dependencies]
veda-rs = { version = "1.0", features = ["async", "gpu", "telemetry"] }
```

Available features:
- `adaptive` - Adaptive scheduling (enabled by default)
- `telemetry` - Metrics and observability (enabled by default)
- `deterministic` - Deterministic execution mode
- `async` - Async/await integration
- `gpu` - GPU compute support
- `numa` - NUMA-aware allocation
- `energy-aware` - Energy-efficient scheduling
- `custom-allocators` - Custom memory allocators

## Basic Usage

```rust
use veda_rs::prelude::*;

fn main() -> Result<(), veda_rs::Error> {
    // Initialize the runtime
    veda_rs::init()?;
    
    // Use parallel iterators
    let sum: i32 = (0..1000).into_par_iter().sum();
    println!("Sum: {}", sum);
    
    // Clean up
    veda_rs::shutdown();
    Ok(())
}
```

## Parallel Iterators

VEDA-RS provides parallel versions of many standard iterator operations:

### Basic Operations

```rust
// Map
let squares: Vec<_> = (0..10).into_par_iter().map(|x| x * x).collect();

// Filter
let evens: Vec<_> = (0..10).into_par_iter().filter(|&x| x % 2 == 0).collect();

// Fold/Reduce
let sum = (1..=100).into_par_iter().fold(|| 0, |a, b| a + b);
let max = (1..=100).into_par_iter().reduce(|| 0, |a, b| a.max(b));

// Any/All
let has_even = (0..10).into_par_iter().any(|x| x % 2 == 0);
let all_even = (0..10).into_par_iter().all(|x| x % 2 == 0);

// Find
let first_even = (0..10).into_par_iter().find_any(|&x| x % 2 == 0);
```

### Range Support

```rust
// Standard ranges
(0..100).into_par_iter().for_each(|i| {
    println!("Processing {}", i);
});

// Inclusive ranges
(0..=100).into_par_iter().for_each(|i| {
    println!("Processing {}", i);
});
```

### Vector and Slice Support

```rust
let vec = vec![1, 2, 3, 4, 5];

// Consuming iterator
let sum: i32 = vec.into_par_iter().sum();

// Non-consuming iterator
let slice = &[1, 2, 3, 4, 5];
let sum: i32 = slice.par_iter().sum();
```

## Configuration

Customize the runtime behavior:

```rust
use veda_rs::config::{Config, SchedulingPolicy};

let config = Config::default()
    .with_num_threads(4)
    .with_stack_size(8 * 1024 * 1024) // 8MB stack
    .with_scheduling_policy(SchedulingPolicy::WorkStealing);

// Initialize with custom config
veda_rs::init_with_config(config)?;
```

## Scoped Parallelism

```rust
use veda_rs::scope;

let mut data = vec![1, 2, 3, 4, 5];

scope(|s| {
    for x in &mut data {
        s.spawn(move |_| {
            *x *= 2;
        });
    }
});

println!("{:?}", data); // [2, 4, 6, 8, 10]
```

## Async Integration

Enable the `async` feature to use with async/await:

```rust
use veda_rs::async_bridge as veda_async;

#[tokio::main]
async fn main() {
    let result = veda_async::spawn(async {
        // Your async code here
        42
    }).await;
    
    println!("Result: {}", result);
}
```

## GPU Support

Enable the `gpu` feature for GPU-accelerated operations:

```rust
#[cfg(feature = "gpu")]
{
    use veda_rs::gpu::{Device, Buffer};
    
    let device = Device::default()?;
    let buffer = Buffer::from_slice(&[1.0f32, 2.0, 3.0, 4.0], &device);
    
    // Execute GPU operations...
}
```

## Error Handling

VEDA-RS uses the `Result` type for error handling:

```rust
match veda_rs::init() {
    Ok(_) => {
        // Runtime initialized successfully
    }
    Err(e) => {
        eprintln!("Failed to initialize VEDA-RS: {}", e);
        return;
    }
}
```

## Best Practices

1. **Initialize Once**: Call `veda_rs::init()` once at the start of your program.
2. **Shutdown Gracefully**: Call `veda_rs::shutdown()` when done to clean up resources.
3. **Chunk Large Workloads**: For very large datasets, consider processing in chunks.
4. **Avoid Fine-grained Tasks**: Ensure each parallel task does enough work to offset scheduling overhead.
5. **Use Scoped Parallelism** for tasks that need to borrow data from the stack.

## Examples

### Word Count

```rust
use std::collections::HashMap;
use veda_rs::prelude::*;

fn word_count(text: &str) -> HashMap<String, usize> {
    text.par_split_whitespace()
        .map(|word| (word.to_lowercase(), 1))
        .fold(
            || HashMap::new(),
            |mut acc, (word, count)| {
                *acc.entry(word).or_insert(0) += count;
                acc
            },
        )
        .reduce(
            || HashMap::new(),
            |mut a, b| {
                for (word, count) in b {
                    *a.entry(word).or_insert(0) += count;
                }
                a
            },
        )
}
```

### Parallel Image Processing

```rust
use image::{RgbImage, Rgb};
use veda_rs::prelude::*;

fn apply_grayscale(image: &mut RgbImage) {
    let (width, height) = image.dimensions();
    
    // Process rows in parallel
    (0..height).into_par_iter().for_each(|y| {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            let luma = (pixel[0] as f32 * 0.299 
                      + pixel[1] as f32 * 0.587 
                      + pixel[2] as f32 * 0.114) as u8;
            image.put_pixel(x, y, Rgb([luma, luma, luma]));
        }
    });
}
```

For more examples, check the `examples/` directory in the repository.
