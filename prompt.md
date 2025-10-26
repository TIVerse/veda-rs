# VEDA Runtime Implementation Prompt

## Context

You are implementing **VEDA (Versatile Execution and Dynamic Adaptation)**, a next-generation parallel runtime library for Rust. This is a production-grade system that will replace Rayon while adding adaptive scheduling, GPU support, async integration, and comprehensive observability.

**Critical Source Document**: All implementation details, architecture decisions, and specifications are defined in `docs/brief.md`. Read this document thoroughly before starting any implementation.

## Code Quality Requirements

### Human-Like Code Standards

Your code MUST NOT be detectable as AI-generated. Follow these principles:

1. **Natural Evolution Patterns**
   - Start with simple implementations, then refactor
   - Leave thoughtful TODO comments for future improvements
   - Include occasional "thinking process" comments
   - Don't over-engineer on first pass
   - Real developers iterate - your code should show this

2. **Variable Naming Philosophy**
   - Mix naming styles naturally: `worker_id`, `wid` (in tight loops), `w` (for iterators)
   - Use domain abbreviations: `sched` for scheduler in internal code, full `scheduler` in public API
   - Include occasional "legacy" names that hint at refactoring: `old_queue`, `queue_v2`
   - Don't be perfectly consistent - humans aren't

3. **Code Organization Quirks**
   - Occasionally group related imports, sometimes alphabetize
   - Mix explicit types with type inference naturally
   - Use both match expressions and if-let where appropriate
   - Include helper functions that aren't perfectly DRY - sometimes duplication happens

4. **Comment Style**
   - Mix `//` and `///` appropriately (doc vs regular)
   - Include rationale comments: "// Using RwLock here instead of Mutex because..."
   - Add occasional "// FIXME: optimize this path" or "// NOTE: benchmarked on 2024-10-15"
   - Reference real papers or algorithms: "// Based on the Chase-Lev work-stealing deque"
   - Some functions have detailed docs, others brief - like real code

5. **Implementation Patterns**
   - Mix functional and imperative styles
   - Don't always use the "most clever" Rust idiom
   - Include defensive programming: `debug_assert!`, bounds checks
   - Some error handling is `unwrap()` in internal code, `?` in public API
   - Occasional `.clone()` where zero-copy would be possible - optimization comes later

6. **Avoid AI Tells**
   - ❌ No perfectly consistent formatting within files
   - ❌ No overly verbose documentation on every single item
   - ❌ No "perfect" error handling everywhere
   - ❌ No textbook-perfect naming (real code has `tmp`, `buf`, `ctx`)
   - ❌ No identical code structure across all modules
   - ✅ Some functions are 5 lines, others are 50 lines
   - ✅ Mix of detailed and sparse comments
   - ✅ Occasional compiler warnings that get fixed later
   - ✅ Some `unsafe` blocks with extensive safety docs, others terser

## Project Structure

Follow the exact structure from `docs/brief.md` Section 5.1. Create all directories and files as specified.

## Implementation Phases

### Phase 0: Foundation (Week 1)

**Goal**: Compilable skeleton with basic functionality

**Tasks**:
1. **Cargo.toml Setup**
   - Add all dependencies from `docs/brief.md` Section 19
   - Configure feature flags from Section 19.2
   - Add package metadata (license, repository, description)
   - Set edition = "2021" (not 2024 as currently specified - that doesn't exist)

2. **Core Infrastructure**
   - `src/lib.rs`: Module declarations, feature-gated exports
   - `src/config.rs`: Config struct with builder pattern
   - `src/error.rs`: Error types using `thiserror`
   - `src/prelude.rs`: Re-exports for ergonomic imports

3. **Basic Execution**
   - `src/executor/task.rs`: Task representation
   - `src/executor/cpu_pool.rs`: Simple thread pool using crossbeam
   - `src/executor/worker.rs`: Worker thread loop
   - Implement basic `spawn()` function

4. **Minimal Iterator Support**
   - `src/iter/par_iter.rs`: Basic `ParallelIterator` trait
   - Implement `for_each()` and `map()` for ranges
   - Use simple chunking strategy initially

**Deliverable**: `cargo test` passes, can run basic parallel operations

### Phase 1: Adaptive Scheduler (Week 2-3)

**Goal**: Dynamic thread pool with work stealing

**Tasks**:
1. **Work Stealing Implementation**
   - `src/scheduler/work_stealing.rs`: Implement using crossbeam-deque
   - Per-worker queues + global injector
   - Randomized stealing with backoff

2. **Adaptive Logic**
   - `src/scheduler/adaptive.rs`: Core adaptation algorithm
   - Load estimation and rebalancing
   - Worker scaling based on utilization
   - Reference Algorithm in Section 7.1 of `docs/brief.md`

3. **Scheduler Integration**
   - `src/scheduler/mod.rs`: SchedulingPolicy enum
   - Connect to executor
   - Metrics collection for feedback

**Deliverable**: Thread pool scales dynamically, work stealing functional

### Phase 2: Telemetry System (Week 4)

**Goal**: Production-grade observability

**Tasks**:
1. **Metrics Collection**
   - `src/telemetry/metrics.rs`: Using hdrhistogram for latencies
   - Lock-free counters with atomic operations
   - Per-worker statistics

2. **Tracing Infrastructure**
   - `src/telemetry/tracing.rs`: Span-based execution tracking
   - Thread-local span management
   - Minimal overhead design

3. **Export System**
   - `src/telemetry/export.rs`: JSON and Prometheus exporters
   - Snapshot mechanism
   - `src/telemetry/feedback.rs`: Adaptive feedback loop

**Deliverable**: Rich metrics available, exportable, low overhead (<5%)

### Phase 3: Scope and Safety (Week 5)

**Goal**: Safe scoped parallelism

**Tasks**:
1. **Scope Implementation**
   - `src/scope/scope.rs`: Lifetime-bound parallel execution
   - Ensure all tasks complete before scope exits
   - Panic propagation and isolation
   - `src/executor/panic_handler.rs`: Task-level fault tolerance

2. **Enhanced Iterator Adapters**
   - `src/iter/`: Complete adapter set (filter, fold, collect, etc.)
   - Efficient collection strategies
   - Parallel bridge from sequential iterators

**Deliverable**: Safe scoped spawning, panic handling, full iterator API

### Phase 4: Deterministic Mode (Week 6)

**Goal**: Reproducible execution for testing

**Tasks**:
1. **Deterministic Scheduler**
   - `src/deterministic/scheduler.rs`: Seed-based scheduling
   - Logical clock for ordering
   - `src/deterministic/rng.rs`: Thread-local seeded RNG

2. **Execution Tracing**
   - `src/deterministic/replay.rs`: Record and replay traces
   - JSON serialization of execution events
   - Replay verification

**Deliverable**: Same seed produces identical results

### Phase 5: Async Bridge (Week 7-8)

**Goal**: Seamless async/await integration

**Tasks**:
1. **Async Executor Bridge**
   - `src/async_bridge/spawn.rs`: Spawn async tasks
   - `src/async_bridge/waker.rs`: Custom waker implementation
   - `src/async_bridge/executor_bridge.rs`: Tokio compatibility

2. **Parallel Async Streams**
   - `src/async_bridge/par_stream.rs`: ParStreamExt trait
   - Concurrent stream processing
   - Backpressure handling

**Deliverable**: Mix sync and async tasks, process streams in parallel

### Phase 6: Memory Management (Week 9)

**Goal**: High-performance allocators

**Tasks**:
1. **Allocator Infrastructure**
   - `src/memory/allocator.rs`: VedaAllocator trait
   - `src/memory/thread_local.rs`: Per-thread arena allocators
   - `src/memory/arena.rs`: Arena implementation

2. **NUMA Support (Linux)**
   - `src/memory/numa.rs`: NUMA node detection
   - NUMA-aware allocation
   - Worker pinning to NUMA nodes

**Deliverable**: Custom allocators working, NUMA support on Linux

### Phase 7: GPU Runtime (Week 10-12)

**Goal**: Automatic CPU/GPU task distribution

**Tasks**:
1. **GPU Infrastructure**
   - `src/gpu/runtime.rs`: wgpu device management
   - `src/gpu/buffer.rs`: Buffer pool for GPU memory
   - `src/gpu/kernel.rs`: Kernel compilation and execution

2. **Scheduler Integration**
   - `src/gpu/scheduler.rs`: CPU vs GPU decision logic
   - Cost estimation for offloading
   - Automatic data transfer

3. **Kernel Trait**
   - Define GpuKernel trait
   - WGSL shader integration
   - Workgroup size computation

**Deliverable**: Automatic GPU offload for compatible tasks

### Phase 8: Advanced Features (Week 13-14)

**Goal**: Production polish

**Tasks**:
1. **Priority Scheduling**
   - `src/scheduler/priority.rs`: Priority queues
   - Deadline scheduling
   - Real-time task support

2. **Energy Awareness**
   - `src/scheduler/energy.rs`: Power monitoring
   - Thermal throttling
   - Energy-efficient scheduling

3. **Utility Components**
   - `src/util/atomic.rs`: Atomic utilities
   - `src/util/cache_padded.rs`: Cache line padding
   - `src/util/backoff.rs`: Exponential backoff

**Deliverable**: All advanced features functional

### Phase 9: Testing & Benchmarking (Week 15-16)

**Goal**: Production-ready quality

**Tasks**:
1. **Comprehensive Test Suite**
   - Unit tests for all modules
   - `tests/integration/`: Rayon compatibility, async, GPU
   - `tests/stress/`: Deadlock, memory leak, scalability tests

2. **Benchmark Suite**
   - `benches/rayon_comparison.rs`: Head-to-head with Rayon
   - `benches/gpu_offload.rs`: CPU vs GPU performance
   - `benches/async_integration.rs`: Async overhead measurement

3. **Example Applications**
   - All examples from Section 15 of `docs/brief.md`
   - Real-world use cases
   - Performance comparisons

**Deliverable**: >80% code coverage, benchmarks show competitive performance

### Phase 10: Documentation & Polish (Week 17-18)

**Goal**: Publishable crate

**Tasks**:
1. **Documentation**
   - Complete API documentation
   - Architecture guide
   - Migration guide from Rayon
   - Performance tuning guide

2. **Examples and Tutorials**
   - Getting started guide
   - Advanced usage patterns
   - Troubleshooting guide

3. **Release Preparation**
   - README.md with badges and examples
   - CHANGELOG.md
   - LICENSE files
   - CONTRIBUTING.md
   - CI/CD setup (GitHub Actions from Section 20.1)

**Deliverable**: Ready for crates.io publication

## Critical Implementation Details

### Work Stealing Algorithm

Use the **Chase-Lev deque** algorithm via `crossbeam-deque`. Key points:

```rust
// Good: Natural implementation with explanatory comments
pub fn steal_task(&self) -> Option<Task> {
    // Try local queue first - better cache locality
    if let Some(task) = self.local_queue.pop() {
        return Some(task);
    }
    
    // Fall back to stealing from random victim
    // Using randomization to avoid contention
    let victim = self.pick_random_victim();
    match self.stealers[victim].steal() {
        Steal::Success(task) => {
            self.metrics.stolen.fetch_add(1, Ordering::Relaxed);
            Some(task)
        }
        Steal::Empty | Steal::Retry => None,
    }
}
```

### Adaptive Scaling

Implement using **exponential moving average** for load estimation:

```rust
// Implementation should feel "discovered" not "designed"
fn update_load_estimate(&mut self) {
    let current_load = self.measure_current_load();
    
    // Alpha = 0.3 chosen after experimentation
    // TODO: make this configurable?
    let alpha = 0.3;
    self.load_estimate = alpha * current_load + (1.0 - alpha) * self.load_estimate;
    
    // Scale workers if estimate is stable and crosses thresholds
    if self.load_stable() {
        self.maybe_scale_workers();
    }
}
```

### Panic Handling

Tasks must be isolated - one panic shouldn't crash the pool:

```rust
std::panic::catch_unwind(AssertUnwindSafe(|| {
    task.execute();
})).unwrap_or_else(|panic_info| {
    // Log panic but keep worker alive
    eprintln!("Task panicked: {:?}", panic_info);
    self.metrics.panics.fetch_add(1, Ordering::Relaxed);
});
```

### GPU Offload Decision

Cost model for CPU vs GPU:

```rust
fn should_use_gpu(&self, task: &Task) -> bool {
    // Need enough work to overcome transfer overhead
    let input_size = task.input_size();
    if input_size < 10_000 {
        return false; // Too small, stay on CPU
    }
    
    // Rough estimate: GPU is 10x faster but has 1ms overhead
    let cpu_time_ms = input_size as f64 * 0.001; // 1μs per element
    let gpu_time_ms = input_size as f64 * 0.0001 + 1.0; // 0.1μs + transfer
    
    gpu_time_ms < cpu_time_ms
}
```

## Anti-Patterns to Avoid

1. **Don't over-abstract initially**
   ```rust
   // Bad: Over-engineered from the start
   trait GenericSchedulingStrategy<T, C, M> { ... }
   
   // Good: Start concrete, abstract later
   struct AdaptiveScheduler {
       workers: Vec<Worker>,
       // ... concrete implementation
   }
   ```

2. **Don't make everything configurable**
   ```rust
   // Bad: Configuration explosion
   pub struct Config {
       pub steal_strategy: StealStrategy,
       pub steal_attempts: usize,
       pub steal_backoff_ns: u64,
       pub steal_batch_size: usize,
       // ... 50 more options
   }
   
   // Good: Sensible defaults, expose only key knobs
   pub struct Config {
       pub num_threads: Option<usize>,
       pub policy: SchedulingPolicy,
       pub enable_gpu: bool,
   }
   ```

3. **Don't prematurely optimize**
   ```rust
   // Bad: Micro-optimization too early
   #[inline(always)]
   #[cold]
   pub fn spawn<F>(...) { ... }
   
   // Good: Get it working, then profile and optimize
   pub fn spawn<F>(...) { ... }
   ```

## Safety and Correctness

### Unsafe Code Guidelines

When you must use `unsafe`:

```rust
// SAFETY: This is safe because:
// 1. The pointer comes from Box::into_raw, so it's valid
// 2. We're the exclusive owner (no aliases)
// 3. The data is initialized (allocated in line 47)
// 4. Memory is properly aligned (Box guarantees this)
unsafe {
    let data = Box::from_raw(ptr);
    // ... use data
}
```

### Concurrency Patterns

Use proven patterns:

- **Atomics**: For simple shared counters
- **RwLock**: For read-heavy data (metrics snapshots)
- **Mutex**: For write-heavy or complex updates
- **Channels**: For producer-consumer
- **Crossbeam**: For lock-free queues

## Performance Requirements

Your implementation must meet these targets (from Section 18.2):

- Task spawning overhead: **< 100ns**
- Idle worker wakeup: **< 1μs**
- Work stealing attempt: **< 500ns**
- Memory per worker: **< 1MB**
- Telemetry overhead: **< 5%**

**How to achieve**:
- Avoid allocations in hot paths
- Use lock-free structures where possible
- Cache-pad hot variables (64-byte alignment)
- Measure with `criterion` benchmarks

## Testing Strategy

### Unit Tests

Test each component in isolation:

```rust
#[test]
fn test_worker_can_steal_from_others() {
    let pool = CpuPool::new(4);
    
    // Fill one worker's queue
    for _ in 0..100 {
        pool.workers[0].push(Task::new(|| {}));
    }
    
    // Other worker should be able to steal
    let stolen = pool.workers[1].try_steal();
    assert!(stolen.is_some());
}
```

### Integration Tests

Test feature combinations:

```rust
#[test]
fn test_adaptive_scheduling_with_telemetry() {
    let config = Config::builder()
        .scheduling_policy(SchedulingPolicy::Adaptive)
        .enable_telemetry(true)
        .build();
    
    veda::init_with_config(config).unwrap();
    
    // Run workload
    (0..10000).into_par_iter().for_each(|_| {
        std::thread::sleep(Duration::from_micros(10));
    });
    
    // Verify adaptive behavior occurred
    let metrics = veda::metrics().snapshot();
    assert!(metrics.worker_scaling_events > 0);
}
```

### Stress Tests

Find race conditions and deadlocks:

```rust
#[test]
#[ignore] // Run with --ignored
fn stress_test_nested_parallelism() {
    for iteration in 0..1000 {
        veda::scope(|s| {
            for i in 0..50 {
                s.spawn(move || {
                    // Nested scope
                    veda::scope(|s2| {
                        for j in 0..10 {
                            s2.spawn(move || {
                                // Deep nesting
                                let _ = (0..100).into_par_iter().sum::<i32>();
                            });
                        }
                    });
                });
            }
        });
        
        if iteration % 100 == 0 {
            println!("Completed {} iterations", iteration);
        }
    }
}
```

## Documentation Standards

### API Documentation

Use this template for public functions:

```rust
/// Spawns a task onto the VEDA runtime.
///
/// The task will be executed on one of the worker threads. This function
/// returns immediately with a handle that can be used to wait for the result.
///
/// # Examples
///
/// ```
/// use veda::spawn;
///
/// let handle = spawn(|| {
///     println!("Running in parallel!");
///     42
/// });
///
/// let result = handle.join().unwrap();
/// assert_eq!(result, 42);
/// ```
///
/// # Panics
///
/// This function will panic if the runtime has not been initialized.
/// Call [`init()`] before spawning tasks.
///
/// # Performance
///
/// Spawning a task has approximately 50-100ns overhead. For very fine-grained
/// tasks (< 1μs), consider batching or using iterators instead.
pub fn spawn<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    // Implementation...
}
```

### Module Documentation

Document architecture in module headers:

```rust
//! Work-stealing scheduler for task distribution.
//!
//! This module implements a randomized work-stealing algorithm based on
//! the Chase-Lev deque. Each worker maintains a local LIFO queue and can
//! steal from other workers' FIFO ends when idle.
//!
//! # Architecture
//!
//! - Local queues: Per-worker deques for tasks
//! - Global injector: Fallback queue for external task submission
//! - Stealers: Read-only handles to steal from other workers
//!
//! # Performance Characteristics
//!
//! - Push: O(1) amortized
//! - Pop: O(1)
//! - Steal: O(1) but may retry on contention
```

## Coding Style Rules

### Formatting

Use `rustfmt` with defaults, but don't obsess:

```rust
// Sometimes break lines naturally
fn long_function_name_with_many_parameters(
    first: Type1,
    second: Type2,
    third: Type3,
) -> Result<Output> {
    // Implementation
}

// Other times keep it compact
fn short(a: u32, b: u32) -> u32 { a + b }
```

### Naming Conventions

- **Modules**: `snake_case`
- **Types**: `PascalCase`
- **Functions**: `snake_case`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Lifetimes**: `'a`, `'b`, or descriptive like `'scope`

```rust
const MAX_WORKERS: usize = 128;

struct WorkerPool { /* ... */ }

fn spawn_worker<'scope>(...) { /* ... */ }
```

### Error Handling

Mix styles appropriately:

```rust
// Public API: Explicit Result types
pub fn init_with_config(config: Config) -> Result<Runtime> {
    config.validate()?;
    Runtime::new(config)
}

// Internal: Sometimes unwrap with context
fn worker_thread(id: usize) {
    let local_queue = WORKER_QUEUES.get(id)
        .expect("worker queue not initialized");
    // ...
}

// Hot path: Assume invariants hold
fn steal_task_unchecked(&self, victim: usize) -> Task {
    debug_assert!(victim < self.workers.len());
    unsafe { self.stealers.get_unchecked(victim).steal() }
}
```

## Feature Flag Organization

```rust
// src/lib.rs

// Core - always available
pub mod config;
pub mod error;
pub mod executor;
pub mod iter;
pub mod scope;

// Telemetry - default feature
#[cfg(feature = "telemetry")]
pub mod telemetry;

// Async bridge - opt-in
#[cfg(feature = "async")]
pub mod async_bridge;

// GPU runtime - opt-in
#[cfg(feature = "gpu")]
pub mod gpu;

// Deterministic mode - opt-in
#[cfg(feature = "deterministic")]
pub mod deterministic;

// NUMA support - opt-in
#[cfg(feature = "numa")]
pub mod memory;
```

## Git Commit Strategy

Write commits like a human developer:

```
feat: implement basic work-stealing scheduler

Added the core work-stealing logic using crossbeam-deque.
Each worker has a local queue and can steal from others
when idle.

Still need to add adaptive scaling and better metrics.

---

fix: worker deadlock under high contention

Workers were spinning indefinitely when stealing failed.
Added exponential backoff with thread parking.

Fixes #42

---

refactor: simplify task spawning path

Removed unnecessary allocation by using a stack-based
approach for small closures. Benchmarks show 15% improvement
in spawn overhead.

---

docs: add examples for GPU offloading

---

test: add stress test for nested parallelism
```

## Project Milestones

### Milestone 1: MVP (End of Week 6)
- ✅ Basic parallel iterators working
- ✅ Work stealing functional
- ✅ Scope-based parallelism safe
- ✅ Deterministic mode implemented
- ✅ Basic telemetry

**Deliverable**: Can replace Rayon in simple use cases

### Milestone 2: Advanced CPU (End of Week 10)
- ✅ Adaptive scheduler tuned
- ✅ Full iterator API
- ✅ Panic isolation
- ✅ Priority scheduling
- ✅ NUMA support

**Deliverable**: Production-ready for CPU workloads

### Milestone 3: Full Platform (End of Week 14)
- ✅ Async integration complete
- ✅ GPU offloading working
- ✅ Energy awareness
- ✅ Custom allocators

**Deliverable**: All features from `docs/brief.md`

### Milestone 4: Release Ready (End of Week 18)
- ✅ Comprehensive tests (>80% coverage)
- ✅ Benchmarks competitive with Rayon
- ✅ Complete documentation
- ✅ Examples and guides

**Deliverable**: v1.0.0 published to crates.io

## Final Checklist

Before considering the project complete:

- [ ] All modules from `docs/brief.md` Section 5.1 exist
- [ ] All features from Section 3 implemented
- [ ] All examples from Section 15 work
- [ ] Performance targets from Section 18.2 met
- [ ] Test coverage > 80%
- [ ] All public APIs documented
- [ ] Benchmarks show competitive performance
- [ ] CI/CD pipeline passing
- [ ] README.md complete with examples
- [ ] CHANGELOG.md tracks all changes
- [ ] License files present (MIT OR Apache-2.0)
- [ ] Code passes `cargo clippy --all-features`
- [ ] Code formatted with `rustfmt`
- [ ] No unsafe code in public API
- [ ] All unsafe blocks have safety comments
- [ ] Platform support: Linux, Windows, macOS
- [ ] Rayon compatibility verified
- [ ] Migration guide written

## Key Principles to Remember

1. **Start simple, iterate**: Don't try to build everything perfectly from the start
2. **Code like a human**: Show your thinking process through comments and code evolution
3. **Test thoroughly**: Every feature needs tests
4. **Document extensively**: API docs, architecture guides, examples
5. **Benchmark constantly**: Performance is a feature
6. **Safety first**: No compromises on memory safety or data race freedom
7. **Follow the spec**: `docs/brief.md` is the source of truth
8. **Write idiomatic Rust**: Use the type system, embrace ownership
9. **Make it maintainable**: Code will be read more than written
10. **Production quality**: This is not a prototype, it's a real library

## Getting Started

1. **Read** `docs/brief.md` completely (2,551 lines)
2. **Plan** your implementation approach for Phase 0
3. **Set up** Cargo.toml with dependencies and features
4. **Implement** basic executor and task system
5. **Add** minimal parallel iterator support
6. **Test** that basic operations work
7. **Iterate** through remaining phases

## Success Criteria

The project is complete when:
- A Rust developer can replace `rayon = "1.8"` with `veda = "1.0"` and their code compiles with minimal changes
- Performance is competitive with or better than Rayon
- GPU offloading provides measurable speedup for compatible workloads
- Async integration works seamlessly with Tokio
- Deterministic mode enables reproducible testing
- Telemetry provides actionable insights
- Code quality passes professional review
- **No AI detector flags the code as generated**

---

Remember: You're building infrastructure that will power parallel computing in Rust for years. Take the time to do it right. Write code that you'd be proud to maintain. Leave the codebase better than you found it.

Good luck! 🚀
