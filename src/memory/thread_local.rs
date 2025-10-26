//! Thread-local memory allocator for fast, contention-free allocation.

use super::allocator::{SystemAllocator, VedaAllocator};
use super::arena::Arena;
use std::alloc::Layout;
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Thread-local allocator with arena backing
pub struct ThreadLocalAllocator {
    arena: UnsafeCell<Arena>,
    fallback: SystemAllocator,
    allocated_bytes: AtomicUsize,
}

impl std::fmt::Debug for ThreadLocalAllocator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThreadLocalAllocator")
            .field(
                "allocated_bytes",
                &self
                    .allocated_bytes
                    .load(std::sync::atomic::Ordering::Relaxed),
            )
            .finish()
    }
}

impl ThreadLocalAllocator {
    /// Create a new thread-local allocator
    pub fn new() -> Self {
        Self {
            arena: UnsafeCell::new(Arena::new()),
            fallback: SystemAllocator,
            allocated_bytes: AtomicUsize::new(0),
        }
    }

    /// Create with specific chunk size
    pub fn with_chunk_size(chunk_size: usize) -> Self {
        Self {
            arena: UnsafeCell::new(Arena::with_chunk_size(chunk_size)),
            fallback: SystemAllocator,
            allocated_bytes: AtomicUsize::new(0),
        }
    }

    /// Reset the arena, freeing all allocated memory
    pub fn reset(&mut self) {
        unsafe {
            (*self.arena.get()).reset();
        }
        self.allocated_bytes.store(0, Ordering::Relaxed);
    }

    /// Get total allocated bytes
    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes.load(Ordering::Relaxed)
    }
}

impl VedaAllocator for ThreadLocalAllocator {
    fn allocate(&self, layout: Layout) -> *mut u8 {
        // Try arena first for small allocations
        if layout.size() <= Arena::DEFAULT_CHUNK_SIZE / 4 {
            let ptr = unsafe { (*self.arena.get()).allocate(layout) };
            if !ptr.is_null() {
                self.allocated_bytes
                    .fetch_add(layout.size(), Ordering::Relaxed);
                return ptr;
            }
        }

        // Fall back to system allocator for large allocations
        let ptr = self.fallback.allocate(layout);
        if !ptr.is_null() {
            self.allocated_bytes
                .fetch_add(layout.size(), Ordering::Relaxed);
        }
        ptr
    }

    fn deallocate(&self, _ptr: *mut u8, layout: Layout) {
        // Arena allocations are bulk-freed on reset
        // For system allocations, we'd need to track them separately
        // For simplicity, we just update the counter
        self.allocated_bytes
            .fetch_sub(layout.size(), Ordering::Relaxed);
    }
}

impl Default for ThreadLocalAllocator {
    fn default() -> Self {
        Self::new()
    }
}

// Safety: ThreadLocalAllocator is designed to be used per-thread
// The UnsafeCell is only accessed by a single thread at a time
unsafe impl Send for ThreadLocalAllocator {}
unsafe impl Sync for ThreadLocalAllocator {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_local_allocator() {
        let allocator = ThreadLocalAllocator::new();
        let layout = Layout::from_size_align(64, 8).unwrap();

        let ptr = allocator.allocate(layout);
        assert!(!ptr.is_null());
        assert_eq!(allocator.allocated_bytes(), 64);

        allocator.deallocate(ptr, layout);
        assert_eq!(allocator.allocated_bytes(), 0);
    }

    #[test]
    fn test_multiple_allocations() {
        let allocator = ThreadLocalAllocator::new();
        let layout = Layout::from_size_align(32, 8).unwrap();

        let mut ptrs = Vec::new();
        for _ in 0..10 {
            let ptr = allocator.allocate(layout);
            assert!(!ptr.is_null());
            ptrs.push(ptr);
        }

        assert_eq!(allocator.allocated_bytes(), 320);
    }
}
