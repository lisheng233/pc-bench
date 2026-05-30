// gpu_bench.rs
use std::time::Instant;
use colored::*;
use wgpu::util::DeviceExt;

pub async fn run_gpu_benchmark() -> f64 {
    println!("  Checking GPU availability...");

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapter = match instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
    {
        Some(adapter) => adapter,
        None => {
            println!("  {} No GPU adapter found", "⚠".bright_yellow());
            return 0.0;
        }
    };

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
            return 0.0;
        }
    };

    let info = adapter.get_info();
    println!("  GPU: {}", info.name.bright_cyan());
    println!("  Backend: {:?}", info.backend);
    let is_software_renderer = info.name.to_lowercase().contains("llvmpipe")
    || info.name.to_lowercase().contains("software")
    || info.name.to_lowercase().contains("basic render");
if is_software_renderer {
    println!("  {} Software renderer detected ({}). GPU benchmark would be extremely slow, skipping.",
        "⚠".bright_yellow(), info.name);
    return 0.0;
}
    // ----- 使用 vec4<f32> 向量运算的 Shader -----
    let shader_source = r#"
        @group(0) @binding(0)
        var<storage, read_write> data: array<vec4<f32>>;

        @compute @workgroup_size(256)
        fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
            let idx = gid.x;
            if idx >= arrayLength(&data) { return; }

            var v = data[idx];
            // 循环 1000 次向量 FMA (乘加) 运算
            // 每条 v = v * a + b 对每个分量执行 2 次浮点操作
            // 总浮点操作数 = 4 (分量) * 2 * loop_count
            for (var i = 0u; i < 1000u; i++) {
                v = v * 1.0001 + 0.9999;
            }
            data[idx] = v;
        }
    "#;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Benchmark Shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    // 使用 vec4<f32> 元素个数：保持内存占用约 8 MB (2 百万个 vec4)
    const NUM_VEC4: usize = 2_000_000; // 2M * 16 bytes = 32 MB
    let buffer_size = (NUM_VEC4 * std::mem::size_of::<[f32; 4]>()) as u64;

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Benchmark Buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    // 初始化数据：每个 vec4 的分量填入 (index * 4 + comp) 的正弦值
    let mut initial_data = vec![[0.0f32; 4]; NUM_VEC4];
    for (i, vec4) in initial_data.iter_mut().enumerate() {
        for comp in 0..4 {
            vec4[comp] = ((i * 4 + comp) as f32).sin();
        }
    }

    // 上传初始数据到 staging buffer，再复制到 GPU
    let staging_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Staging Buffer"),
        contents: bytemuck::cast_slice(&initial_data),
        usage: wgpu::BufferUsages::COPY_SRC,
    });

    // 绑定组布局
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

    // 预热：复制数据到 GPU 并执行一次，确保资源分配完成
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

    // 正式测试：只测量 GPU 计算时间（数据已在 GPU 上）
    // 执行 50 次 dispatch 并求平均，消除单次波动
    const ITERATIONS: u32 = 50;
    let start = Instant::now();

    for _ in 0..ITERATIONS {
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
    // 等待所有 GPU 工作完成
    device.poll(wgpu::Maintain::Wait);

    let elapsed = start.elapsed().as_secs_f64();
    let avg_time_per_iter = elapsed / ITERATIONS as f64;

    // 计算浮点运算量
    // 每次 dispatch 处理 NUM_VEC4 个 vec4，每个 vec4 循环 1000 次，每次循环执行 2 次浮点运算 per 分量
    // 每个分量 2 次运算，共 4 个分量 → 8 次浮点运算 / 迭代
    // 总 FLOPs = NUM_VEC4 * 8 * 1000
    let flops_per_dispatch = NUM_VEC4 as f64 * 8.0 * 1000.0;
    let flops_per_second = flops_per_dispatch / avg_time_per_iter;
    let gflops = flops_per_second / 1_000_000_000.0;

    // 归一化得分：以 RTX 3060 在此测试下约 1200 GFLOPS 作为 10000 分基准
    let reference_gflops = 1200.0;
    let gpu_score = (gflops / reference_gflops) * 10000.0;

    println!("  GPU Performance: {:.2} GFLOPS", gflops);
    println!("  {}: {:.2}", "GPU Score".bright_blue(), gpu_score);

    gpu_score
}