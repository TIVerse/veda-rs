//! GPU compute support for heterogeneous execution.
//!
//! Provides automatic CPU/GPU task distribution using wgpu for cross-platform
//! GPU access.

pub mod runtime;
pub mod kernel;
pub mod buffer;
pub mod scheduler;

pub use runtime::GpuRuntime;
pub use kernel::{GpuKernel, CompiledKernel};
pub use buffer::{GpuBuffer, BufferPool};
pub use scheduler::GpuScheduler;

use crate::error::Result;

/// Execute a GPU kernel
pub async fn execute<K: GpuKernel>(kernel: K) -> Result<Vec<u8>> {
    let runtime = GpuRuntime::get_or_init().await?;
    runtime.execute_kernel(kernel).await
}
