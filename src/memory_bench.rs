use std::time::Instant;
use std::alloc::{alloc, dealloc, Layout};
use rand::Rng;
use indicatif::{ProgressBar, ProgressStyle};
use colored::*;
use crate::config::MemoryConfig;

pub fn run_memory_benchmark(total_ram: u64, config: &MemoryConfig) -> f64 {
    let ram_gb = total_ram as f64 / 1024.0 / 1024.0 / 1024.0;
    println!("  Total RAM: {:.2} GB", ram_gb);
    
    let pb = ProgressBar::new(4);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
        .unwrap()
        .progress_chars("#>-"));
    
    let mut scores = Vec::new();
    
    pb.set_message("Sequential read/write...");
    let score1 = test_sequential_access(config);
    scores.push(score1);
    pb.inc(1);
    
    pb.set_message("Random read/write...");
    let score2 = test_random_access(config);
    scores.push(score2);
    pb.inc(1);
    
    pb.set_message("Memory bandwidth...");
    let score3 = test_memory_bandwidth(config);
    scores.push(score3);
    pb.inc(1);
    
    pb.set_message("Latency test...");
    let score4 = test_latency(config);
    scores.push(score4);
    pb.inc(1);
    
    pb.finish_with_message("Memory benchmark complete!");
    
    println!("Sequential read/write : {:.2}", score1);
    println!("Random read/write : {:.2}", score2);
    println!("Memory bandwidth : {:.2}", score3);
    println!("Latency test : {:.2}", score4);
    
    let avg_score = scores.iter().sum::<f64>() / scores.len() as f64;
    println!("  {}: {:.2}", "Memory Score".bright_green(), avg_score);
    avg_score
}

fn test_sequential_access(config: &MemoryConfig) -> f64 {
    let size = config.sequential_access_test_size;
    let layout = Layout::array::<u64>(size).unwrap();
    let ptr = unsafe { alloc(layout) as *mut u64 };
    
    let start = Instant::now();
    unsafe {
        for i in 0..size {
            *ptr.add(i) = i as u64;
        }
    }
    let mut sum: u64 = 0;
    unsafe {
        for i in 0..size {
            sum = sum.wrapping_add(*ptr.add(i));
        }
    }
    std::hint::black_box(sum);
    let elapsed = start.elapsed().as_secs_f64();
    let bandwidth = (size * std::mem::size_of::<u64>() * 2) as f64 / elapsed / 1_000_000_000.0;
    unsafe { dealloc(ptr as *mut u8, layout); }
    bandwidth / config.sequential_access_ref
}

fn test_random_access(config: &MemoryConfig) -> f64 {
    let size = config.random_access_test_size;
    let ops = config.random_access_operations;
    
    let layout = Layout::array::<u64>(size).unwrap();
    let ptr = unsafe { alloc(layout) as *mut u64 };
    unsafe {
        for i in 0..size {
            *ptr.add(i) = i as u64;
        }
    }
    
    let mut rng = rand::thread_rng();
    let indices: Vec<usize> = (0..ops)
        .map(|_| rng.gen_range(0..size))
        .collect();
    
    let start = Instant::now();
    let mut sum: u64 = 0;
    for &idx in &indices {
        unsafe {
            sum = sum.wrapping_add(*ptr.add(idx));
            *ptr.add(idx) = sum;
        }
    }
    std::hint::black_box(sum);
    let elapsed = start.elapsed().as_secs_f64();
    let ops_per_sec = indices.len() as f64 / elapsed;
    unsafe { dealloc(ptr as *mut u8, layout); }
    ops_per_sec / 1_000_000.0 / config.random_access_ref
}

fn test_memory_bandwidth(config: &MemoryConfig) -> f64 {
    let size = config.memory_bandwidth_test_size;
    let iterations = config.memory_bandwidth_iterations;
    
    let mut src = vec![0u64; size];
    let mut dst = vec![0u64; size];
    for i in 0..size {
        src[i] = i as u64;
    }
    
    let start = Instant::now();
    for _ in 0..iterations {
        dst.copy_from_slice(&src);
    }
    let elapsed = start.elapsed().as_secs_f64();
    let total_bytes = (size * std::mem::size_of::<u64>() * iterations) as f64;
    let bandwidth = total_bytes / elapsed / 1_000_000_000.0;
    std::hint::black_box(&dst);
    bandwidth / config.memory_bandwidth_ref
}

fn test_latency(config: &MemoryConfig) -> f64 {
    let size = config.latency_test_size;
    let operations = config.latency_operations;
    
    let mut data: Vec<(u64, usize)> = vec![(0, 0); size];
    let mut order: Vec<usize> = (0..size).collect();
    
    // Fisher-Yates shuffle
    for i in (1..size).rev() {
        let j = rand::thread_rng().gen_range(0..=i);
        order.swap(i, j);
    }
    
    // 创建随机链表
    for i in 0..size {
        let next = if i < size - 1 { order[i + 1] } else { order[0] };
        data[order[i]] = (0, next);
    }
    
    let start = Instant::now();
    let mut current = order[0];
    let mut sum: u64 = 0;
    for _ in 0..operations {
        sum = sum.wrapping_add(data[current].0);
        current = data[current].1;
    }
    std::hint::black_box(sum);
    let elapsed = start.elapsed().as_secs_f64();
    let latency_ns = elapsed * 1_000_000_000.0 / operations as f64;
    (1000.0 / latency_ns) / config.latency_ref
}