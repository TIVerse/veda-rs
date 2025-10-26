pub mod buffer;
pub mod kernel;
pub mod runtime;
pub mod scheduler;

pub use buffer::{BufferPool, GpuBuffer};
pub use kernel::{CompiledKernel, GpuKernel};
pub use runtime::GpuRuntime;
pub use scheduler::GpuScheduler;

use crate::error::Result;

pub async fn execute<K: GpuKernel>(kernel: K) -> Result<Vec<u8>> {
    let runtime = GpuRuntime::get_or_init().await?;
    runtime.execute_kernel(kernel).await
}
