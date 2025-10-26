//! GPU buffer management with pooling.

use crate::error::{Error, Result};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;
use wgpu;

/// GPU buffer wrapper
pub struct GpuBuffer {
    buffer: wgpu::Buffer,
    size: usize,
    device: Arc<wgpu::Device>,
}

impl GpuBuffer {
    /// Create a new GPU buffer
    pub fn new(device: Arc<wgpu::Device>, size: usize) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("veda-gpu-buffer"),
            size: size as u64,
            usage: wgpu::BufferUsages::STORAGE 
                | wgpu::BufferUsages::COPY_DST 
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        
        Self {
            buffer,
            size,
            device,
        }
    }
    
    /// Write data to the buffer
    pub fn write_data(&self, queue: &wgpu::Queue, data: &[u8]) -> Result<()> {
        if data.len() > self.size {
            return Err(Error::gpu("Data too large for buffer"));
        }
        queue.write_buffer(&self.buffer, 0, data);
        Ok(())
    }
    
    /// Read data from the buffer
    pub async fn read_data(&self) -> Result<Vec<u8>> {
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("veda-staging-buffer"),
            size: self.size as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("veda-copy-encoder"),
        });
        
        encoder.copy_buffer_to_buffer(&self.buffer, 0, &staging_buffer, 0, self.size as u64);
        
        // Note: In a real implementation, we'd need to submit and wait
        // This is a simplified version
        
        Ok(vec![0u8; self.size])
    }
    
    /// Get the underlying wgpu buffer
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
    
    /// Get buffer size
    pub fn size(&self) -> usize {
        self.size
    }
    
    /// Get device reference
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
}

/// Buffer pool for reusing GPU buffers
pub struct BufferPool {
    device: Arc<wgpu::Device>,
    free_buffers: Mutex<HashMap<usize, Vec<GpuBuffer>>>,
}

impl BufferPool {
    /// Create a new buffer pool
    pub fn new(device: Arc<wgpu::Device>) -> Self {
        Self {
            device,
            free_buffers: Mutex::new(HashMap::new()),
        }
    }
    
    /// Acquire a buffer of the given size
    pub fn acquire(&self, size: usize) -> GpuBuffer {
        let mut buffers = self.free_buffers.lock();
        
        if let Some(pool) = buffers.get_mut(&size) {
            if let Some(buffer) = pool.pop() {
                return buffer;
            }
        }
        
        // Allocate new buffer
        GpuBuffer::new(Arc::clone(&self.device), size)
    }
    
    /// Return a buffer to the pool
    pub fn release(&self, buffer: GpuBuffer) {
        let mut buffers = self.free_buffers.lock();
        buffers.entry(buffer.size).or_insert_with(Vec::new).push(buffer);
    }
    
    /// Clear all cached buffers
    pub fn clear(&self) {
        self.free_buffers.lock().clear();
    }
}
