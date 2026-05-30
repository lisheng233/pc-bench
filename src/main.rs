// main.rs
mod cpu_bench;
mod gpu_bench;
mod memory_bench;
mod storage_bench;
mod system_info;
mod utils;
use std::io;
use colored::*;
use std::time::Instant;

#[tokio::main]
async fn main() {
    let start =Instant::now();
    println!("{}", "PC Performance Benchmark");
    println!("NOTICE: this program is only used for simlple test");
    println!("{}\n", "Testing your system performance...");
    
    // 获取系统信息
    let sys_info = system_info::get_system_info();
    system_info::print_system_info(&sys_info);
    
    let mut total_score = 0.0;
    
    // CPU 基准测试
    println!("\n{}", "[CPU Benchmark]");
    let cpu_score = cpu_bench::run_cpu_benchmark();
    total_score += cpu_score * 0.4; // CPU权重40%
    
    // 内存基准测试
    println!("\n{}", "[Memory Benchmark]".bright_green().bold());
    let memory_score = memory_bench::run_memory_benchmark(sys_info.total_ram);
    total_score += memory_score * 0.3; // 内存权重30%
    
    // 存储基准测试
    println!("\n{}", "[Storage Benchmark]".bright_magenta().bold());
    let storage_score = storage_bench::run_storage_benchmark().await;
    total_score += storage_score * 0.2; // 存储权重20%
    
    // GPU 基准测试（如果可用）
    println!("\n{}", "[GPU Benchmark]".bright_blue().bold());
    let gpu_score = gpu_bench::run_gpu_benchmark().await;
    total_score += gpu_score * 0.1; // GPU权重10%
    
    // 总分
    println!("\n{}", "=".repeat(50).bright_white());
    println!("{}", "=== FINAL RESULTS ===".bright_cyan().bold().to_string());
    println!("{}", "=".repeat(50).bright_white());
    
    let rating = match total_score {
        s if s >= 8000.0*1.25 => "Excellent".bright_green(),
        s if s >= 5000.0*1.25 => "Very Good".bright_green(),
        s if s >= 3000.0*1.25 => "Good".bright_yellow(),
        s if s >= 1500.0*1.25 => "Average".bright_yellow(),
        _ => "Below Average".bright_red(),
    };
    
    println!("{}: {:.2}", "Total Score".bright_white().bold(), total_score);
    println!("{}: {}", "Rating".bright_white().bold(), rating);
    
    // 分类得分
    println!("\n{}", "Category Scores:".bright_white());
    println!("  CPU Score:      {:.2}", cpu_score);
    println!("  Memory Score:   {:.2}", memory_score);
    println!("  Storage Score:  {:.2}", storage_score);
    println!("  GPU Score:      {:.2}", gpu_score);
    println!("\nTOTAL TIME : {:.3}s",start.elapsed().as_secs_f64());
    let mut a=String::new();
    let _=io::stdin().read_line(&mut a);
}