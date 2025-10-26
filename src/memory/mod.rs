//! Memory management subsystem for NUMA-aware and efficient allocation.

pub mod allocator;
pub mod thread_local;

#[cfg(feature = "numa")]
pub mod numa;

pub mod arena;

pub use allocator::VedaAllocator;
pub use thread_local::ThreadLocalAllocator;
pub use arena::Arena;

#[cfg(feature = "numa")]
pub use numa::{NumaAllocator, NumaNode};
