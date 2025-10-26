//! GPU compute example demonstrating heterogeneous execution

#[cfg(feature = "gpu")]
use veda_rs::prelude::*;

#[cfg(feature = "gpu")]
use veda_rs::gpu::{GpuRuntime, GpuKernel, GpuBuffer};

#[cfg(feature = "gpu")]
struct VectorAddKernel {
    size: usize,
}

#[cfg(feature = "gpu")]
impl VectorAddKernel {
    fn new(size: usize) -> Self {
        Self { size }
    }
}

#[cfg(feature = "gpu")]
impl GpuKernel for VectorAddKernel {
    fn input_size(&self) -> usize {
        self.size * std::mem::size_of::<f32>() * 2
    }
    
    fn output_size(&self) -> usize {
        self.size * std::mem::size_of::<f32>()
    }
    
    fn compile(&self, device: &wgpu::Device) -> veda_rs::error::Result<veda_rs::gpu::CompiledKernel> {
        // WGSL shader for vector addition
        let shader_source = r#"
            @group(0) @binding(0) var<storage, read> input_a: array<f32>;
            @group(0) @binding(1) var<storage, read> input_b: array<f32>;
            @group(0) @binding(2) var<storage, read_write> output: array<f32>;
            
            @compute @workgroup_size(256)
            fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
                let idx = global_id.x;
                if (idx < arrayLength(&output)) {
                    output[idx] = input_a[idx] + input_b[idx];
                }
            }
        "#;
        
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("vector-add-shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });
        
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("vector-add-pipeline"),
            layout: None,
            module: &shader,
            entry_point: "main",
        });
        
        Ok(veda_rs::gpu::CompiledKernel {
            pipeline,
            workgroup_size: (256, 1, 1),
        })
    }
}

#[cfg(feature = "gpu")]
async fn run_gpu_demo() -> veda_rs::error::Result<()> {
    println!("=== GPU Compute Demo ===\n");
    
    // Initialize GPU runtime
    println!("Initializing GPU runtime...");
    let gpu = GpuRuntime::get_or_init().await?;
    
    println!("✓ GPU device initialized");
    println!("  Device: {}", gpu.device_name());
    println!("  Backend: {}", gpu.backend_name());
    
    // Create test data
    let size = 1024;
    let data_a: Vec<f32> = (0..size).map(|i| i as f32).collect();
    let data_b: Vec<f32> = (0..size).map(|i| (i * 2) as f32).collect();
    
    println!("\nVector addition: {} elements", size);
    println!("  A[0..5] = {:?}", &data_a[0..5]);
    println!("  B[0..5] = {:?}", &data_b[0..5]);
    
    // Create GPU kernel
    let kernel = VectorAddKernel::new(size);
    
    // Execute on GPU
    println!("\nExecuting on GPU...");
    let result = gpu.execute_kernel(kernel).await?;
    
    // Convert result bytes back to f32
    let result_f32: Vec<f32> = result
        .chunks_exact(4)
        .map(|bytes| f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        .collect();
    
    println!("✓ GPU execution complete");
    println!("  Result[0..5] = {:?}", &result_f32[0..5]);
    
    // Verify results
    let mut correct = 0;
    for i in 0..size.min(result_f32.len()) {
        if (result_f32[i] - (data_a[i] + data_b[i])).abs() < 0.001 {
            correct += 1;
        }
    }
    
    println!("\nVerification: {}/{} correct", correct, size);
    
    if correct == size {
        println!("✓ All results correct!");
    } else {
        println!("⚠ Some results incorrect");
    }
    
    Ok(())
}

#[cfg(feature = "gpu")]
fn run_hybrid_demo() {
    println!("\n=== Hybrid CPU/GPU Demo ===\n");
    
    // Initialize VEDA runtime
    let config = Config::builder()
        .num_threads(4)
        .enable_gpu(true)
        .build()
        .unwrap();
    
    init_with_config(config).unwrap();
    
    // CPU parallel computation
    println!("Computing on CPU...");
    let cpu_result: i64 = (0..1_000_000)
        .into_par_iter()
        .map(|x| x * 2)
        .sum();
    
    println!("✓ CPU result: {}", cpu_result);
    
    // Note: GPU execution would happen here in a real scenario
    // For this example, we've demonstrated GPU capability separately
    
    println!("\n✓ Hybrid execution complete");
    
    shutdown();
}

#[cfg(not(feature = "gpu"))]
fn main() {
    println!("This example requires the 'gpu' feature.");
    println!("Run with: cargo run --example gpu_compute --features gpu");
}

#[cfg(feature = "gpu")]
#[tokio::main]
async fn main() -> veda_rs::error::Result<()> {
    // Run GPU-only demo
    if let Err(e) = run_gpu_demo().await {
        eprintln!("GPU demo failed: {}", e);
        eprintln!("This may be expected if no GPU is available.");
    }
    
    // Run hybrid demo
    run_hybrid_demo();
    
    println!("\n=== Demo Complete ===");
    Ok(())
}
