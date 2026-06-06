use std::time::Instant;
use colored::*;
use wgpu::util::DeviceExt;
use crate::config::GpuConfig;

pub async fn run_gpu_benchmark(config: &GpuConfig) -> f64 {
    println!("  Enumerating GPUs...");

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapters = instance.enumerate_adapters(wgpu::Backends::all());
    if adapters.is_empty() {
        println!("  {} No GPU adapter found", "⚠".bright_yellow());
        return 0.0;
    }

    println!("  Found {} GPU(s)", adapters.len());

    let mut total_score = 0.0;
    let mut gpu_count = 0;

    for (idx, adapter) in adapters.iter().enumerate() {
        println!("\n  --- GPU #{} ---", idx + 1);
        if let Some(score) = run_benchmark_on_adapter(adapter, config).await {
            total_score += score;
            gpu_count += 1;
        }
    }

    if gpu_count == 0 {
        println!("  {} No usable GPU found", "⚠".bright_yellow());
        return 0.0;
    }

    println!("\n  {} (sum of {}) = {:.2}", "Multi-GPU Total Score".bright_blue(), gpu_count, total_score);
    total_score
}

/// 在单个适配器上运行完整的 GPU 基准测试，返回该 GPU 的分数（若失败则返回 None）
async fn run_benchmark_on_adapter(adapter: &wgpu::Adapter, config: &GpuConfig) -> Option<f64> {
    let info = adapter.get_info();
    let is_software = info.name.to_lowercase().contains("llvmpipe")
        || info.name.to_lowercase().contains("software")
        || info.name.to_lowercase().contains("basic render");

    if is_software {
        if config.force_test_vgpu {
            println!(
                "  {} Software renderer detected ({}). \n  Forcing running GPU benchmark\n    (this will be extremely slow and may cause some faults!!!).",
                "⚠".bright_yellow(),
                info.name
            );
        } else {
            println!(
                "  {} Software renderer detected ({}). Skipping this GPU.",
                "⚠".bright_yellow(),
                info.name
            );
            return None;
        }
    }

    println!("  GPU: {}", info.name.bright_cyan());
    println!("  Backend: {:?}", info.backend);

    let (device, queue) = match adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Benchmark Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            },
            None,
        )
        .await
    {
        Ok((device, queue)) => (device, queue),
        Err(_) => {
            println!("  {} Failed to create GPU device", "⚠".bright_yellow());
            return None;
        }
    };

    // 1. 显存带宽测试
    let bandwidth_gbps = measure_vram_bandwidth(&device, &queue, config).await;
    let bandwidth_score = (bandwidth_gbps / config.vram_bandwidth_ref) * config.reference_score;
    println!("  VRAM Bandwidth: {:.2} GB/s", bandwidth_gbps);
    println!("  VRAM Bandwidth Score: {:.2}", bandwidth_score);

    // 2. 计算性能测试
    let compute_score = run_compute_benchmark(&device, &queue, config).await;

    // 综合 GPU 总分（带宽权重 30%，计算权重 70%）
    let vram_bw_weight = config.vram_capacity_weight; // 复用原容量权重作为带宽权重
    let compute_weight = 1.0 - vram_bw_weight;
    let gpu_total_score = compute_score * compute_weight + bandwidth_score * vram_bw_weight;

    println!("  GPU Compute Score: {:.2}", compute_score);
    println!("  {}: {:.2}", "GPU Score".bright_blue(), gpu_total_score);

    Some(gpu_total_score)
}

/// 显存带宽测试 (GB/s)
async fn measure_vram_bandwidth(device: &wgpu::Device, queue: &wgpu::Queue, config: &GpuConfig) -> f64 {
    let size_mb = config.vram_bandwidth_test_size_mb.max(1).min(1024);
    let buffer_size = (size_mb * 1024 * 1024) as u64;

    let src_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("VRAM Bandwidth Source"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let dst_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("VRAM Bandwidth Dest"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // 预热
    for _ in 0..3 {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        encoder.copy_buffer_to_buffer(&src_buffer, 0, &dst_buffer, 0, buffer_size);
        queue.submit(Some(encoder.finish()));
        device.poll(wgpu::Maintain::Wait);
    }

    const ITERATIONS: u32 = 100;
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        encoder.copy_buffer_to_buffer(&src_buffer, 0, &dst_buffer, 0, buffer_size);
        queue.submit(Some(encoder.finish()));
    }
    device.poll(wgpu::Maintain::Wait);
    let elapsed = start.elapsed().as_secs_f64();

    let total_bytes_copied = buffer_size as f64 * ITERATIONS as f64;
    total_bytes_copied / elapsed / 1_000_000_000.0
}

/// 计算性能测试 (基于 GFLOPS)
async fn run_compute_benchmark(device: &wgpu::Device, queue: &wgpu::Queue, config: &GpuConfig) -> f64 {
    let shader_source = r#"
        @group(0) @binding(0)
        var<storage, read_write> data: array<vec4<f32>>;

        @compute @workgroup_size(256)
        fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
            let idx = gid.x;
            if idx >= arrayLength(&data) { return; }
            var v = data[idx];
            for (var i = 0u; i < 2500u; i++) {
                v = v * 1.0001 + 0.9999;
                v = v * v - vec4(0.0001);
                v = max(v, vec4(0.0));
            }
            data[idx] = v;
        }
    "#;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Benchmark Shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    const NUM_VEC4: usize = 5_000_000;
    let buffer_size = (NUM_VEC4 * std::mem::size_of::<[f32; 4]>()) as u64;

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Benchmark Buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let mut initial_data = vec![[0.0f32; 4]; NUM_VEC4];
    for (i, vec4) in initial_data.iter_mut().enumerate() {
        for comp in 0..4 {
            vec4[comp] = ((i * 4 + comp) as f32).sin();
        }
    }

    let staging_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Staging Buffer"),
        contents: bytemuck::cast_slice(&initial_data),
        usage: wgpu::BufferUsages::COPY_SRC,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Bind Group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Compute Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: "main",
    });

    // 预热
    for _ in 0..3 {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Warmup Encoder"),
        });
        encoder.copy_buffer_to_buffer(&staging_buffer, 0, &buffer, 0, buffer_size);
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Warmup Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&compute_pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            let workgroup_size = 256u32;
            let workgroups = ((NUM_VEC4 as u32) + workgroup_size - 1) / workgroup_size;
            compute_pass.dispatch_workgroups(workgroups, 1, 1);
        }
        queue.submit(Some(encoder.finish()));
        device.poll(wgpu::Maintain::Wait);
    }

    let iterations = config.iteration.min(999u32).max(1u32);
    let start = Instant::now();

    for _ in 0..iterations {
        let mut compute_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Compute Encoder"),
        });
        {
            let mut compute_pass = compute_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&compute_pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            let workgroup_size = 256u32;
            let workgroups = ((NUM_VEC4 as u32) + workgroup_size - 1) / workgroup_size;
            compute_pass.dispatch_workgroups(workgroups, 1, 1);
        }
        queue.submit(Some(compute_encoder.finish()));
    }
    device.poll(wgpu::Maintain::Wait);

    let elapsed = start.elapsed().as_secs_f64();
    let avg_time_per_iter = elapsed / iterations as f64;

    let ops_per_vec4_per_loop = 20.0;
    let total_loops = 2500.0;
    let ops_per_vec4 = ops_per_vec4_per_loop * total_loops;
    let total_ops = NUM_VEC4 as f64 * ops_per_vec4;
    let flops_per_second = total_ops / avg_time_per_iter;
    let gflops = flops_per_second / 1_000_000_000.0;

    let compute_score = (gflops / config.vec_ref) * config.reference_score;
    println!("  GPU Performance: {:.2} GFLOPS", gflops);
    println!("  Data size: {} vectors ({:.2} MB)", NUM_VEC4, buffer_size as f64 / 1024.0 / 1024.0);
    println!("  Operations: {:.2} billion per iteration", total_ops / 1_000_000_000.0);

    compute_score
}