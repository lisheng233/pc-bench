use std::time::Instant;
use std::fs::{self, File};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::PathBuf;
use rand::Rng;
use indicatif::{ProgressBar, ProgressStyle};
use colored::*;
use crate::config::StorageConfig;

pub async fn run_storage_benchmark(config: &StorageConfig) -> f64 {
    let pb = ProgressBar::new(4);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
        .unwrap()
        .progress_chars("#>-"));
    
    let test_file = std::env::temp_dir().join("benchmark_test_file.tmp");
    let mut scores = Vec::new();
    
    pb.set_message("Sequential write...");
    let score1 = test_sequential_write(&test_file,&config);
    scores.push(score1);
    pb.inc(1);
    
    pb.set_message("Sequential read...");
    let score2 = test_sequential_read(&test_file,&config);
    scores.push(score2);
    pb.inc(1);
    
    pb.set_message("Random write...");
    let score3 = test_random_write(&test_file,&config);
    scores.push(score3);
    pb.inc(1);
    
    pb.set_message("Random read...");
    let score4 = test_random_read(&test_file,&config);
    scores.push(score4);
    pb.inc(1);
    
    pb.finish_with_message("Storage benchmark complete!");
    println!("Sequential write : {:.2}", score1);
    println!("Sequential read : {:.2}", score2);
    println!("Random write : {:.2}", score3);
    println!("Random read : {:.2}", score4);
    
    let _ = fs::remove_file(&test_file);
    let avg_score = scores.iter().sum::<f64>() / scores.len() as f64;
    println!("  {}: {:.2}", "Storage Score".bright_green(), avg_score);
    avg_score
}

fn test_sequential_write(file_path: &PathBuf,config:&StorageConfig) -> f64 {
    let size = 500_000_000;
    let buffer = vec![0u8; 1_000_000];
    let mut file = File::create(file_path).unwrap();
    let start = Instant::now();
    for _ in 0..(size / buffer.len()) {
        file.write_all(&buffer).unwrap();
    }
    file.sync_all().unwrap();
    let elapsed = start.elapsed().as_secs_f64();
    let speed = size as f64 / elapsed / 1_000_000.0;
    speed / config.sequential_write_ref
}

fn test_sequential_read(file_path: &PathBuf,config:&StorageConfig) -> f64 {
    let mut file = File::open(file_path).unwrap();
    let mut buffer = vec![0u8; 1_000_000];
    let start = Instant::now();
    loop {
        match file.read(&mut buffer) {
            Ok(0) => break,
            Ok(_) => continue,
            Err(_) => break,
        }
    }
    let elapsed = start.elapsed().as_secs_f64();
    let file_size = file.metadata().unwrap().len() as f64;
    let speed = file_size / elapsed / 1_000_000.0;
    speed /config.sequential_read_ref
}

fn test_random_write(file_path: &PathBuf,config:&StorageConfig) -> f64 {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .open(file_path)
        .unwrap();
    let file_size = file.metadata().unwrap().len();
    let block_size = 4096;
    let num_operations = 10000;
    let mut rng = rand::thread_rng();
    let buffer = vec![0u8; block_size];
    let start = Instant::now();
    for _ in 0..num_operations {
        let offset = rng.gen_range(0..(file_size / block_size as u64)) * block_size as u64;
        file.seek(SeekFrom::Start(offset)).unwrap();
        file.write_all(&buffer).unwrap();
    }
    file.sync_all().unwrap();
    let elapsed = start.elapsed().as_secs_f64();
    let iops = num_operations as f64 / elapsed;
    iops / config.random_write_ref
}

fn test_random_read(file_path: &PathBuf,config:&StorageConfig) -> f64 {
    let mut file = File::open(file_path).unwrap();
    let file_size = file.metadata().unwrap().len();
    let block_size = 4096;
    let num_operations = 10000;
    let mut rng = rand::thread_rng();
    let mut buffer = vec![0u8; block_size];
    let start = Instant::now();
    for _ in 0..num_operations {
        let offset = rng.gen_range(0..(file_size / block_size as u64)) * block_size as u64;
        file.seek(SeekFrom::Start(offset)).unwrap();
        let _ = file.read(&mut buffer).unwrap();
    }
    let elapsed = start.elapsed().as_secs_f64();
    let iops = num_operations as f64 / elapsed;
    iops / config.random_read_ref
}