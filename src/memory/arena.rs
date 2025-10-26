//! Arena allocator for bulk allocations.

use std::alloc::{Layout, alloc, dealloc};
use std::ptr::NonNull;

/// Chunk of memory in the arena
struct Chunk {
    data: NonNull<u8>,
    size: usize,
}

impl Chunk {
    fn new(size: usize) -> Self {
        let layout = Layout::from_size_align(size, 64).expect("Invalid layout");
        let data = unsafe {
            let ptr = alloc(layout);
            NonNull::new(ptr).expect("Allocation failed")
        };
        
        Self { data, size }
    }
}

impl Drop for Chunk {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.size, 64).expect("Invalid layout");
        unsafe {
            dealloc(self.data.as_ptr(), layout);
        }
    }
}

/// Arena allocator that allocates in large chunks
pub struct Arena {
    chunks: Vec<Chunk>,
    current_chunk: usize,
    current_offset: usize,
    chunk_size: usize,
}

impl Arena {
    /// Default chunk size (1 MB)
    pub const DEFAULT_CHUNK_SIZE: usize = 1024 * 1024;
    
    /// Create a new arena with default chunk size
    pub fn new() -> Self {
        Self::with_chunk_size(Self::DEFAULT_CHUNK_SIZE)
    }
    
    /// Create an arena with specified chunk size
    pub fn with_chunk_size(chunk_size: usize) -> Self {
        Self {
            chunks: Vec::new(),
            current_chunk: 0,
            current_offset: 0,
            chunk_size,
        }
    }
    
    /// Allocate memory from the arena
    pub fn allocate(&mut self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();
        
        // Ensure we have at least one chunk
        if self.chunks.is_empty() {
            self.allocate_chunk();
        }
        
        // Align current offset
        let offset = (self.current_offset + align - 1) & !(align - 1);
        
        // Check if allocation fits in current chunk
        if offset + size > self.chunk_size {
            // Need a new chunk
            self.allocate_chunk();
            return self.allocate(layout);
        }
        
        // Allocate from current chunk
        let ptr = unsafe {
            self.chunks[self.current_chunk]
                .data
                .as_ptr()
                .add(offset)
        };
        
        self.current_offset = offset + size;
        ptr
    }
    
    /// Allocate a new chunk
    fn allocate_chunk(&mut self) {
        let chunk = Chunk::new(self.chunk_size);
        self.chunks.push(chunk);
        self.current_chunk = self.chunks.len() - 1;
        self.current_offset = 0;
    }
    
    /// Reset the arena, keeping allocated chunks but resetting pointers
    pub fn reset(&mut self) {
        self.current_chunk = 0;
        self.current_offset = 0;
    }
    
    /// Get total allocated bytes across all chunks
    pub fn capacity(&self) -> usize {
        self.chunks.len() * self.chunk_size
    }
    
    /// Get currently used bytes in the arena
    pub fn used(&self) -> usize {
        if self.chunks.is_empty() {
            0
        } else {
            self.current_chunk * self.chunk_size + self.current_offset
        }
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        // Chunks will be dropped automatically
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_arena_basic() {
        let mut arena = Arena::with_chunk_size(1024);
        
        let layout = Layout::from_size_align(64, 8).unwrap();
        let ptr = arena.allocate(layout);
        assert!(!ptr.is_null());
        
        assert_eq!(arena.used(), 64);
    }
    
    #[test]
    fn test_arena_multiple_allocations() {
        let mut arena = Arena::with_chunk_size(1024);
        
        let layout = Layout::from_size_align(32, 8).unwrap();
        
        for _ in 0..10 {
            let ptr = arena.allocate(layout);
            assert!(!ptr.is_null());
        }
        
        assert!(arena.used() >= 320);
    }
    
    #[test]
    fn test_arena_chunk_overflow() {
        let mut arena = Arena::with_chunk_size(128);
        
        let layout = Layout::from_size_align(64, 8).unwrap();
        
        // Allocate enough to require multiple chunks
        let ptr1 = arena.allocate(layout);
        let ptr2 = arena.allocate(layout);
        let ptr3 = arena.allocate(layout); // Should trigger new chunk
        
        assert!(!ptr1.is_null());
        assert!(!ptr2.is_null());
        assert!(!ptr3.is_null());
        
        assert!(arena.capacity() >= 256); // At least 2 chunks
    }
    
    #[test]
    fn test_arena_reset() {
        let mut arena = Arena::with_chunk_size(1024);
        
        let layout = Layout::from_size_align(64, 8).unwrap();
        arena.allocate(layout);
        
        assert!(arena.used() > 0);
        
        arena.reset();
        assert_eq!(arena.used(), 0);
    }
}
