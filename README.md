# VEDA: Versatile Execution and Dynamic Adaptation

[![Crates.io](https://img.shields.io/crates/v/veda-rs.svg)](https://crates.io/crates/veda-rs)
[![Documentation](https://docs.rs/veda-rs/badge.svg)](https://docs.rs/veda-rs)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![CI](https://github.com/veda-rs/veda/workflows/CI/badge.svg)](https://github.com/veda-rs/veda/actions)

A parallel runtime library for Rust with adaptive scheduling and work-stealing. Designed to be mostly compatible with Rayon but with better handling of variable workloads.

## Features

- Adaptive thread pools with dynamic worker scaling
- Work stealing scheduler
- Rayon-compatible API for parallel iterators
- Scoped parallelism support
- Optional telemetry and metrics
- Optional deterministic execution mode
- GPU support (experimental)
- Async/await integration

## Installation

Add VEDA to your `Cargo.toml`:

```toml
[dependencies]
veda-rs = "1.0"
```

### Feature Flags

```toml
[dependencies]
veda-rs = { version = "1.0", features = ["telemetry", "async", "gpu"] }
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

## Quick Start

### Basic Parallel Iterator

```rust
use veda::prelude::*;

fn main() {
    veda::init().unwrap();
    
    let sum: i32 = (0..1000)
        .into_par_iter()
        .map(|x| x * 2)
        .sum();
    
    println!("Sum: {}", sum);
    veda::shutdown();
}
```

### Custom Configuration

```rust
use veda::{Config, SchedulingPolicy};

fn main() {
    let config = Config::builder()
        .num_threads(4)
        .scheduling_policy(SchedulingPolicy::Adaptive)
        .build()
        .unwrap();
    
    veda::init_with_config(config).unwrap();
    veda::shutdown();
}
```

### Scoped Parallelism

```rust
use veda::scope;

fn main() {
    veda::init().unwrap();
    
    let mut data = vec![0; 100];
    
    scope::scope(|s| {
        for chunk in data.chunks_mut(10) {
            s.spawn(move || {
                for item in chunk {
                    *item += 1;
                }
            });
        }
    });
    
    veda::shutdown();
}
```

## Examples

See the [`examples/`](examples/) directory for more comprehensive examples:

- [`basic_par_iter.rs`](examples/basic_par_iter.rs) - Basic parallel iterator usage
- [`adaptive_workload.rs`](examples/adaptive_workload.rs) - Adaptive scheduling with variable workloads
- [`scoped_parallelism.rs`](examples/scoped_parallelism.rs) - Safe scoped task spawning
- [`custom_priorities.rs`](examples/custom_priorities.rs) - Task prioritization
- [`deterministic_debug.rs`](examples/deterministic_debug.rs) - Reproducible execution

Run an example:

```bash
cargo run --example basic_par_iter
```

## Advanced Usage

### Adaptive Scheduling

```rust
let config = Config::builder()
    .scheduling_policy(SchedulingPolicy::Adaptive)
    .build()
    .unwrap();

veda::init_with_config(config).unwrap();

let results: Vec<_> = (0..10000)
    .into_par_iter()
    .map(|i| expensive_computation(i))
    .collect();
```

### Deterministic Execution

```rust
#[cfg(feature = "deterministic")]
{
    let config = Config::builder()
        .scheduling_policy(SchedulingPolicy::Deterministic { seed: 42 })
        .build()
        .unwrap();
    
    veda::init_with_config(config).unwrap();
    let result = (0..1000).into_par_iter().map(|x| x * x).sum();
}
```

### GPU Offloading

```rust
#[cfg(feature = "gpu")]
{
    use veda::gpu::{GpuKernel, VectorAddKernel};
    
    let kernel = VectorAddKernel::new(1_000_000);
    let result = veda::gpu::execute(kernel).await?;
}
```

### Telemetry

```rust
#[cfg(feature = "telemetry")]
{
    use veda::telemetry::export::{ConsoleExporter, MetricsExporter};
    
    let metrics = veda::telemetry::metrics::Metrics::default();
    let snapshot = metrics.snapshot();
    let exporter = ConsoleExporter::new(true);
    exporter.export(&snapshot)?;
}
```

## Architecture

VEDA is built with a modular architecture:

```
┌─────────────────────────────────────────────────────────────┐
│                         VEDA Runtime                         │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────────┐        ┌──────────────────┐            │
│  │  User Interface │───────▶│   Telemetry      │            │
│  │  - par_iter()   │        │   Subsystem      │            │
│  │  - spawn()      │        │  - Metrics       │            │
│  │  - scope()      │        │  - Tracing       │            │
│  └────────┬────────┘        └────────┬─────────┘            │
│           │                          │                       │
│           ▼                          ▼                       │
│  ┌─────────────────────────────────────────┐                │
│  │         Adaptive Scheduler               │                │
│  │  - Load Balancing                        │                │
│  │  - Task Prioritization                   │                │
│  │  - Energy Awareness                      │                │
│  └────────┬─────────────────────────────────┘                │
│           │                                                   │
│     ┌─────┴──────┬──────────────┬──────────────┐            │
│     ▼            ▼              ▼              ▼            │
│  ┌──────┐   ┌──────┐      ┌─────────┐    ┌─────────┐       │
│  │ CPU  │   │ CPU  │      │  GPU    │    │  Async  │       │
│  │ Pool │   │ Pool │ ...  │ Runtime │    │  Bridge │       │
│  └──────┘   └──────┘      └─────────┘    └─────────┘       │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

## Performance

Benchmarks show similar performance to Rayon on uniform workloads, with better performance on variable workloads:

| Workload Type | VEDA vs Rayon | Notes |
|---------------|---------------|-------|
| CPU-bound, uniform | ~1.0x | Similar to Rayon |
| CPU-bound, variable | 1.2-1.8x | Adaptive scheduling wins |
| Small tasks (<1ms) | ~0.95x | Slight overhead from telemetry |
| Large tasks (>100ms) | 1.1-1.5x | Better load balancing |

Run benchmarks:

```bash
cargo bench
```

## Testing

VEDA includes comprehensive test coverage:

```bash
cargo test
cargo test --test integration_test

# Run stress tests (long-running)
cargo test --test stress_test -- --ignored

# Run all tests with all features
cargo test --all-features
```

## Migration from Rayon

VEDA is designed as a drop-in replacement for Rayon:

**Before (Rayon):**
```rust
use rayon::prelude::*;

fn main() {
    let sum: i32 = (0..1000).into_par_iter().sum();
}
```

**After (VEDA):**
```rust
use veda::prelude::*;

fn main() {
    veda::init().unwrap();
    let sum: i32 = (0..1000).into_par_iter().sum();
    veda::shutdown();
}
```

The main differences:
1. Explicit `init()` and `shutdown()` calls for runtime management
2. Additional configuration options available
3. Optional features for advanced capabilities

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

**Areas where we'd love help:**
- Performance optimizations
- Additional GPU kernel implementations
- Platform-specific NUMA support
- Documentation improvements
- More examples and use cases

## License

VEDA is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

## Acknowledgments

VEDA builds upon the foundational work of:
- **Rayon** - pioneering work in Rust data parallelism
- **Tokio** - async runtime architecture
- **Crossbeam** - lock-free data structures
- **wgpu** - modern GPU abstraction

## Contact

- **Documentation**: https://docs.rs/veda-rs
- **Repository**: https://github.com/veda-rs/veda
- **Issues**: https://github.com/veda-rs/veda/issues
- **Discussions**: https://github.com/veda-rs/veda/discussions

## Roadmap

Current:
- Adaptive thread pool
- Rayon-compatible API
- Basic telemetry
- Panic isolation
- Deterministic mode

Planned:
- GPU compute support
- Energy-aware scheduling
- NUMA support
- Better telemetry
