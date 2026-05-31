mod cpu_bench;
mod gpu_bench;
mod memory_bench;
mod storage_bench;
mod system_info;
mod utils;
mod config;

use colored::*;
use std::time::Instant;
use std::path::Path;
use config::Config;

#[tokio::main]
async fn main() {
    let start = Instant::now();
    println!("NOTICE: this program is only used for simple test");
    println!("{}", "PC Performance Benchmark");
    println!("{}\n", "Testing your system performance...");

    // 加载配置
    let config_path = Path::new("benchmark_config.toml");
    let config = Config::from_file(config_path).unwrap_or_else(|e| {
        eprintln!("Warning: Failed to load config: {}, using defaults.", e);
        Config::default()
    });

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