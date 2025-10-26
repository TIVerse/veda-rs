//! GPU runtime management using wgpu.

use super::kernel::{GpuKernel, CompiledKernel};
use super::buffer::BufferPool;
use crate::error::{Error, Result};
use std::sync::Arc;
use parking_lot::RwLock;
use wgpu;

/// GPU runtime for managing device and queue
pub struct GpuRuntime {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    adapter_info: wgpu::AdapterInfo,
    buffer_pool: BufferPool,
}

impl GpuRuntime {
    /// Initialize the GPU runtime
    pub async fn new() -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| Error::gpu("No GPU adapter found"))?;
        
        let adapter_info = adapter.get_info();
        
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("veda-gpu-device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .map_err(|e| Error::gpu(format!("Failed to request device: {}", e)))?;
        
        let device = Arc::new(device);
        let queue = Arc::new(queue);
        let buffer_pool = BufferPool::new(Arc::clone(&device));
        
        Ok(Self {
            device,
            queue,
            adapter_info,
            buffer_pool,
        })
    }
    
    /// Get or initialize the global GPU runtime
    pub async fn get_or_init() -> Result<Arc<Self>> {
        static RUNTIME: RwLock<Option<Arc<GpuRuntime>>> = RwLock::new(None);
        
        {
            let runtime = RUNTIME.read();
            if let Some(rt) = runtime.as_ref() {
                return Ok(Arc::clone(rt));
            }
        }
        
        let mut runtime = RUNTIME.write();
        if let Some(rt) = runtime.as_ref() {
            return Ok(Arc::clone(rt));
        }
        
        let rt = Arc::new(Self::new().await?);
        *runtime = Some(Arc::clone(&rt));
        Ok(rt)
    }
    
    /// Execute a GPU kernel
    pub async fn execute_kernel<K: GpuKernel>(&self, kernel: K) -> Result<Vec<u8>> {
        let compiled = kernel.compile(&self.device)?;
        
        // Create input/output buffers
        let input_size = kernel.input_size();
        let output_size = kernel.output_size();
        
        let input_buffer = self.buffer_pool.acquire(input_size);
        let output_buffer = self.buffer_pool.acquire(output_size);
        
        // Execute kernel
        compiled.execute(&self.queue, &input_buffer, &output_buffer).await?;
        
        // Read back results
        let result = output_buffer.read_data().await?;
        
        Ok(result)
    }
    
    /// Get device reference
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
    
    /// Get queue reference
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
    
    /// Get adapter info
    pub fn adapter_info(&self) -> &wgpu::AdapterInfo {
        &self.adapter_info
    }
    
    /// Get buffer pool
    pub fn buffer_pool(&self) -> &BufferPool {
        &self.buffer_pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_gpu_runtime_init() {
        // This test requires a GPU, so it may fail in CI
        if let Ok(runtime) = GpuRuntime::new().await {
            println!("GPU: {:?}", runtime.adapter_info().name);
            assert!(runtime.device().limits().max_compute_workgroup_size_x > 0);
        }
    }
}
