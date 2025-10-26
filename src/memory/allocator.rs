//! Custom allocator traits for VEDA.

use std::alloc::{GlobalAlloc, Layout, System};

/// Trait for custom allocators in VEDA
pub trait VedaAllocator: Send + Sync + std::fmt::Debug {
    /// Allocate memory with the given layout
    fn allocate(&self, layout: Layout) -> *mut u8;

    /// Deallocate memory
    fn deallocate(&self, ptr: *mut u8, layout: Layout);

    /// Allocate zeroed memory
    fn allocate_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = self.allocate(layout);
        if !ptr.is_null() {
            unsafe {
                std::ptr::write_bytes(ptr, 0, layout.size());
            }
        }
        ptr
    }

    /// Reallocate memory (default implementation uses allocate + copy + deallocate)
    fn reallocate(&self, ptr: *mut u8, old_layout: Layout, new_layout: Layout) -> *mut u8 {
        let new_ptr = self.allocate(new_layout);
        if !new_ptr.is_null() && !ptr.is_null() {
            unsafe {
                let copy_size = old_layout.size().min(new_layout.size());
                std::ptr::copy_nonoverlapping(ptr, new_ptr, copy_size);
            }
            self.deallocate(ptr, old_layout);
        }
        new_ptr
    }
}

/// System allocator wrapper that implements VedaAllocator
#[derive(Debug)]
pub struct SystemAllocator;

impl VedaAllocator for SystemAllocator {
    fn allocate(&self, layout: Layout) -> *mut u8 {
        unsafe { System.alloc(layout) }
    }

    fn deallocate(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

impl Default for SystemAllocator {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_allocator() {
        let allocator = SystemAllocator;
        let layout = Layout::from_size_align(1024, 8).unwrap();

        let ptr = allocator.allocate(layout);
        assert!(!ptr.is_null());

        allocator.deallocate(ptr, layout);
    }

    #[test]
    fn test_allocate_zeroed() {
        let allocator = SystemAllocator;
        let layout = Layout::from_size_align(16, 8).unwrap();

        let ptr = allocator.allocate_zeroed(layout);
        assert!(!ptr.is_null());

        // Verify it's zeroed
        unsafe {
            for i in 0..16 {
                assert_eq!(*ptr.add(i), 0);
            }
        }

        allocator.deallocate(ptr, layout);
    }
}
