//! NUMA-aware memory allocation (Linux-specific).

use super::allocator::{VedaAllocator, SystemAllocator};
use std::alloc::Layout;
use std::sync::Arc;

/// Represents a NUMA node
#[derive(Debug, Clone)]
pub struct NumaNode {
    pub id: usize,
    allocator: Arc<dyn VedaAllocator>,
}

impl NumaNode {
    /// Create a new NUMA node
    pub fn new(id: usize) -> Self {
        Self {
            id,
            allocator: Arc::new(SystemAllocator),
        }
    }
    
    /// Allocate memory on this NUMA node
    pub fn allocate(&self, layout: Layout) -> *mut u8 {
        self.allocator.allocate(layout)
    }
    
    /// Deallocate memory
    pub fn deallocate(&self, ptr: *mut u8, layout: Layout) {
        self.allocator.deallocate(ptr, layout);
    }
}

/// NUMA-aware allocator that distributes allocations across nodes
pub struct NumaAllocator {
    nodes: Vec<NumaNode>,
    current_node: std::sync::atomic::AtomicUsize,
}

impl NumaAllocator {
    /// Create a new NUMA allocator
    pub fn new() -> crate::error::Result<Self> {
        // On Linux, we could use libnuma to detect actual NUMA nodes
        // For now, we'll create a simple round-robin allocator
        
        #[cfg(target_os = "linux")]
        let num_nodes = Self::detect_numa_nodes();
        
        #[cfg(not(target_os = "linux"))]
        let num_nodes = 1;
        
        let nodes = (0..num_nodes)
            .map(NumaNode::new)
            .collect();
        
        Ok(Self {
            nodes,
            current_node: std::sync::atomic::AtomicUsize::new(0),
        })
    }
    
    #[cfg(target_os = "linux")]
    fn detect_numa_nodes() -> usize {
        // Try to detect NUMA nodes from sysfs
        if let Ok(entries) = std::fs::read_dir("/sys/devices/system/node") {
            let count = entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_name()
                        .to_str()
                        .map(|s| s.starts_with("node"))
                        .unwrap_or(false)
                })
                .count();
            
            if count > 0 {
                return count;
            }
        }
        
        // Fallback to single node
        1
    }
    
    /// Allocate memory on a specific NUMA node
    pub fn allocate_on_node(&self, layout: Layout, node_id: usize) -> *mut u8 {
        if let Some(node) = self.nodes.get(node_id) {
            node.allocate(layout)
        } else {
            std::ptr::null_mut()
        }
    }
    
    /// Allocate memory with round-robin node selection
    pub fn allocate_round_robin(&self, layout: Layout) -> *mut u8 {
        let node_id = self.current_node.fetch_add(1, std::sync::atomic::Ordering::Relaxed) 
            % self.nodes.len();
        self.allocate_on_node(layout, node_id)
    }
    
    /// Get number of NUMA nodes
    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }
    
    /// Get a reference to a NUMA node
    pub fn node(&self, id: usize) -> Option<&NumaNode> {
        self.nodes.get(id)
    }
}

impl Default for NumaAllocator {
    fn default() -> Self {
        Self::new().expect("Failed to create NUMA allocator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_numa_allocator_creation() {
        let allocator = NumaAllocator::new().unwrap();
        assert!(allocator.num_nodes() >= 1);
    }
    
    #[test]
    fn test_numa_node_allocation() {
        let allocator = NumaAllocator::new().unwrap();
        let layout = Layout::from_size_align(1024, 8).unwrap();
        
        let ptr = allocator.allocate_on_node(layout, 0);
        assert!(!ptr.is_null());
        
        if let Some(node) = allocator.node(0) {
            node.deallocate(ptr, layout);
        }
    }
    
    #[test]
    fn test_round_robin_allocation() {
        let allocator = NumaAllocator::new().unwrap();
        let layout = Layout::from_size_align(64, 8).unwrap();
        
        let mut ptrs = Vec::new();
        for _ in 0..10 {
            let ptr = allocator.allocate_round_robin(layout);
            assert!(!ptr.is_null());
            ptrs.push(ptr);
        }
        
        // Clean up
        for (i, ptr) in ptrs.iter().enumerate() {
            let node_id = i % allocator.num_nodes();
            if let Some(node) = allocator.node(node_id) {
                node.deallocate(*ptr, layout);
            }
        }
    }
}
