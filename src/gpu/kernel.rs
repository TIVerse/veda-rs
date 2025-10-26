//! GPU kernel abstraction and compilation.

use super::buffer::GpuBuffer;
use crate::error::Result;
use wgpu;
use std::time::Duration;

/// Trait for GPU kernels
pub trait GpuKernel: Send + Sync {
    /// Compile the kernel to a compute pipeline
    fn compile(&self, device: &wgpu::Device) -> Result<CompiledKernel>;
    
    /// Estimate execution duration for scheduling
    fn estimate_duration(&self, input_size: usize) -> Duration {
        // Default heuristic: 1 nanosecond per element
        Duration::from_nanos(input_size as u64)
    }
    
    /// Check if GPU execution is worthwhile
    fn gpu_worthwhile(&self, input_size: usize) -> bool {
        // Default threshold: 10,000 elements
        input_size > 10_000
    }
    
    /// Get input data size in bytes
    fn input_size(&self) -> usize;
    
    /// Get output data size in bytes
    fn output_size(&self) -> usize;
}

/// Compiled GPU kernel ready for execution
pub struct CompiledKernel {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    workgroup_size: (u32, u32, u32),
}

impl CompiledKernel {
    /// Create a new compiled kernel
    pub fn new(
        pipeline: wgpu::ComputePipeline,
        bind_group_layout: wgpu::BindGroupLayout,
        workgroup_size: (u32, u32, u32),
    ) -> Self {
        Self {
            pipeline,
            bind_group_layout,
            workgroup_size,
        }
    }
    
    /// Execute the kernel
    pub async fn execute(
        &self,
        queue: &wgpu::Queue,
        input: &GpuBuffer,
        output: &GpuBuffer,
    ) -> Result<()> {
        // Create bind group
        let bind_group = input.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("veda-kernel-bind-group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: input.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: output.buffer().as_entire_binding(),
                },
            ],
        });
        
        // Create command encoder
        let mut encoder = input.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("veda-kernel-encoder"),
        });
        
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("veda-compute-pass"),
                timestamp_writes: None,
            });
            
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            
            let (x, y, z) = self.workgroup_size;
            compute_pass.dispatch_workgroups(x, y, z);
        }
        
        queue.submit(Some(encoder.finish()));
        
        Ok(())
    }
    
    /// Get workgroup size
    pub fn workgroup_size(&self) -> (u32, u32, u32) {
        self.workgroup_size
    }
}

/// Simple vector addition kernel example
pub struct VectorAddKernel {
    size: usize,
}

impl VectorAddKernel {
    pub fn new(size: usize) -> Self {
        Self { size }
    }
}

impl GpuKernel for VectorAddKernel {
    fn compile(&self, device: &wgpu::Device) -> Result<CompiledKernel> {
        let shader_source = r#"
            @group(0) @binding(0) var<storage, read> input: array<f32>;
            @group(0) @binding(1) var<storage, read_write> output: array<f32>;
            
            @compute @workgroup_size(256)
            fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
                let idx = global_id.x;
                output[idx] = input[idx] * 2.0;
            }
        "#;
        
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("veda-vector-add-shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });
        
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("veda-vector-add-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("veda-vector-add-pipeline-layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("veda-vector-add-pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });
        
        let workgroup_count = ((self.size as u32 + 255) / 256).max(1);
        
        Ok(CompiledKernel::new(
            pipeline,
            bind_group_layout,
            (workgroup_count, 1, 1),
        ))
    }
    
    fn input_size(&self) -> usize {
        self.size * std::mem::size_of::<f32>()
    }
    
    fn output_size(&self) -> usize {
        self.size * std::mem::size_of::<f32>()
    }
}
