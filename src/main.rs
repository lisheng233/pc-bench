mod cpu_bench;
mod gpu_bench;
mod memory_bench;
mod storage_bench;
mod system_info;
mod config;

use colored::*;
use std::time::Instant;
use std::path::Path;
use std::fs;
use config::Config;

#[tokio::main]
async fn main() {
    let start = Instant::now();
    println!("NOTICE: this program is only used for simple test");
    println!("{}", "PC Performance Benchmark");
    println!("{}\n", "Testing your system performance...");
    println!("\tPlease run the test during your PC's 'free time'.");
    println!("\tPlease make your PC's foucs on this program to get accurate results.\n");

    // 加载配置
    let config_path = Path::new("benchmark_config.toml");
    let config = Config::from_file(config_path).unwrap_or_else(|e| {
        eprintln!("Warning: Failed to load config: {}, using defaults.", e);
        let _ = fs::remove_file(&config_path);
        Config::default()
    });
    let exists = match fs::exists(&config_path) {
        Ok(file) => file,
        Err(error) =>panic!("Problem : {error:?}"),
    };
    if !exists {
        let _ = fs::write(config_path, r#"[scoring]
cpu_weight = 0.5
memory_weight = 0.2
storage_weight = 0.2
gpu_weight = 0.1

[cpu]
calc_average_cpu_score = true
num_threads_ref = 4
integer_ref = 240.0
float_ref = 60.0
prime_ref = 6000.0
matrix_ref = 4.0
hash_ref = 24.0
reference_score = 10000.0

[memory]
sequential_access_ref = 0.004
sequential_access_test_size = 100000000
random_access_ref = 0.01
random_access_test_size = 10000000
memory_bandwidth_ref = 0.005
memory_bandwidth_test_size = 50000000
latency_ref = 0.002
latency_test_size = 10000000

[storage]
sequential_write_ref = 0.2
sequential_read_ref = 0.8
random_write_ref = 10.0
random_read_ref = 100.0

[gpu]
force_test_vgpu = false
reference_score = 10000.0
iteration = 16
vec_num=2000000
vec_ref = 3000.0
vram_bandwidth_ref = 64.0
vram_bandwidth_test_size_mb = 128
vram_capacity_weight = 0.3

[rating]
excellent = 10000.0
very_good = 7000.0
good = 5000.0
average = 3000.0

[rating.labels]
excellent = "Excellent"
very_good = "Very Good"
good = "Good"
average = "Average"
below_average = "Below Average""#);
    }

    let sys_info = system_info::get_system_info();
    system_info::print_system_info(&sys_info);

    let mut total_score = 0.0;

    println!("\n{}", "[CPU Benchmark]");
    let cpu_score = cpu_bench::run_cpu_benchmark(&config.cpu);
    total_score += cpu_score * config.scoring.cpu_weight;

    println!("\n{}", "[Memory Benchmark]".bright_green().bold());
    let memory_score = memory_bench::run_memory_benchmark(sys_info.total_ram, &config.memory);
    total_score += memory_score * config.scoring.memory_weight;

    println!("\n{}", "[Storage Benchmark]".bright_magenta().bold());
    let storage_score = storage_bench::run_storage_benchmark(&config.storage).await;
    total_score += storage_score * config.scoring.storage_weight;

    println!("\n{}", "[GPU Benchmark]".bright_blue().bold());
    let gpu_score = gpu_bench::run_gpu_benchmark(&config.gpu).await;
    total_score += gpu_score * config.scoring.gpu_weight;

    println!("\n{}", "=".repeat(50).bright_white());
    println!("{}", "=== FINAL RESULTS ===".bright_cyan().bold());
    println!("{}", "=".repeat(50).bright_white());

    // 在 main 函数中，计算 total_score 之后
let rating = {
    let cfg = &config.rating;
    if total_score >= cfg.excellent {
        cfg.labels.excellent.bright_green()
    } else if total_score >= cfg.very_good {
        cfg.labels.very_good.bright_green()
    } else if total_score >= cfg.good {
        cfg.labels.good.bright_yellow()
    } else if total_score >= cfg.average {
        cfg.labels.average.bright_yellow()
    } else {
        cfg.labels.below_average.bright_red()
    }
};

    println!("{}: {:.2}", "Total Score".bright_white().bold(), total_score);
    println!("{}: {}", "Rating".bright_white().bold(), rating);

    println!("\n{}", "Category Scores:".bright_white());
    println!("  CPU Score:      {:.2}", cpu_score);
    println!("  Memory Score:   {:.2}", memory_score);
    println!("  Storage Score:  {:.2}", storage_score);
    println!("  GPU Score:      {:.2}", gpu_score);
    println!("\nTOTAL TIME : {:.3}s", start.elapsed().as_secs_f64());

    println!("\nPress Enter to exit...");
    let mut _a = String::new();
    let _ = std::io::stdin().read_line(&mut _a);
}