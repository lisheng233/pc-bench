use rayon::prelude::*;
use std::time::Instant;
use rand::Rng;
use indicatif::{ProgressBar, ProgressStyle};
use colored::*;
use crate::config::CpuConfig;

pub fn run_cpu_benchmark(config: &CpuConfig) -> f64 {
    let num_threads = num_cpus::get();
    println!("  CPU Cores: {}", num_threads);

    let pb = ProgressBar::new(5);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
        .unwrap()
        .progress_chars("#>-"));

    let mut scores = Vec::new();
    let mut raw_performance = Vec::new();

    pb.set_message("Integer operations...");
    let (score1, mops) = test_integer_operations(num_threads, config);
    scores.push(score1);
    raw_performance.push(format!("{:.0} MOPS", mops));
    pb.inc(1);

    pb.set_message("Floating point operations...");
    let (score2, mflops) = test_floating_point_operations(num_threads, config);
    scores.push(score2);
    raw_performance.push(format!("{:.0} MFLOPS", mflops));
    pb.inc(1);

    pb.set_message("Prime counting...");
    let (score3, kprimes) = test_prime_counting(num_threads, config);
    scores.push(score3);
    raw_performance.push(format!("{:.2} kPrimes/s", kprimes));
    pb.inc(1);

    pb.set_message("Matrix multiplication...");
    let (score4, gflops) = test_matrix_multiplication(num_threads, config);
    scores.push(score4);
    raw_performance.push(format!("{:.2} GFLOPS", gflops));
    pb.inc(1);

    pb.set_message("Hash throughput...");
    let (score5, mhash) = test_hash_throughput(num_threads, config);
    scores.push(score5);
    raw_performance.push(format!("{:.2} MHash/s", mhash));
    pb.inc(1);

    pb.finish_with_message("CPU benchmark complete!");

    println!("  Integer operations    : {}  →  {:.2}", raw_performance[0].bright_white(), scores[0]);
    println!("  Floating point        : {}  →  {:.2}", raw_performance[1].bright_white(), scores[1]);
    println!("  Prime counting        : {}  →  {:.2}", raw_performance[2].bright_white(), scores[2]);
    println!("  Matrix multiplication : {}  →  {:.2}", raw_performance[3].bright_white(), scores[3]);
    println!("  Hash throughput       : {}  →  {:.2}", raw_performance[4].bright_white(), scores[4]);

    let avg_score = scores.iter().sum::<f64>() / scores.len() as f64;
    println!("  {}: {:.2}", "CPU Score".bright_green(), avg_score);
    avg_score
}

/// 自适应执行，确保每个测试运行约 1.5 秒
fn run_with_adaptive_time<F>(mut f: F, target_secs: f64) -> f64
where
    F: FnMut(u64) -> f64,
{
    let warmup = 2;
    for _ in 0..warmup {
        f(10_000);
    }

    let mut iter = 10_000;
    let mut elapsed = 0.0;
    let mut result = 0.0;
    for _ in 0..5 {
        let start = Instant::now();
        result = f(iter);
        elapsed = start.elapsed().as_secs_f64();
        if elapsed >= target_secs * 0.5 {
            break;
        }
        iter = (iter as f64 * (target_secs / elapsed).clamp(1.2, 10.0)) as u64;
    }
    let target = target_secs;
    if elapsed > target * 0.8 && elapsed < target * 1.2 {
        return result / elapsed;
    }
    let adjusted = (iter as f64 * (target / elapsed)) as u64;
    let start = Instant::now();
    let result = f(adjusted);
    let elapsed = start.elapsed().as_secs_f64();
    result / elapsed
}

fn test_integer_operations(num_threads: usize, config: &CpuConfig) -> (f64, f64) {
    let target_secs = 1.5;
    let mops_per_sec = run_with_adaptive_time(|iter| {
        let total_ops = iter as u64 * 3;
        (0..num_threads * 2).into_par_iter().for_each(|_| {
            let mut sum: i64 = 0;
            for i in 0..iter {
                sum = sum.wrapping_add(i as i64);
                sum = sum.wrapping_mul((i % 100 + 1) as i64);
                sum = sum.wrapping_div((i % 99 + 1) as i64);
            }
            std::hint::black_box(sum);
        });
        total_ops as f64
    }, target_secs);
    let mops = mops_per_sec / 1_000_000.0;
    let score = (mops / config.integer_ref) * config.reference_score;
    (score, mops)
}

fn test_floating_point_operations(num_threads: usize, config: &CpuConfig) -> (f64, f64) {
    let target_secs = 1.5;
    let rng_buf: Vec<f64> = (0..10_000_000).map(|_| rand::thread_rng().gen()).collect();
    let mflops_per_sec = run_with_adaptive_time(|iter| {
        let total_flops = iter as f64 * 6.0;
        (0..num_threads * 2).into_par_iter().for_each(|_| {
            let mut result = 0.0;
            for i in 0..iter {
                let a = rng_buf[(i % rng_buf.len() as u64) as usize];
                result += a.sin() * a.cos();
                result += a.sqrt().abs();
                result = result.ln().exp();
            }
            std::hint::black_box(result);
        });
        total_flops
    }, target_secs);
    let mflops = mflops_per_sec / 1_000_000.0;
    let score = (mflops / config.float_ref) * config.reference_score;
    (score, mflops)
}

fn test_prime_counting(_num_threads: usize, config: &CpuConfig) -> (f64, f64) {
    let target_secs = 1.5;
    let primes_per_sec = run_with_adaptive_time(|limit| {
        let limit = limit as usize;
        let mut is_prime = vec![true; limit + 1];
        let sqrt_limit = (limit as f64).sqrt() as usize;
        for i in 2..=sqrt_limit {
            if is_prime[i] {
                for j in (i*i..=limit).step_by(i) {
                    is_prime[j] = false;
                }
            }
        }
        (2..=limit).filter(|&x| is_prime[x]).count() as f64
    }, target_secs);
    let kprimes = primes_per_sec / 1000.0;
    let score = (kprimes / config.prime_ref) * config.reference_score;
    (score, kprimes)
}

fn test_matrix_multiplication(num_threads: usize, config: &CpuConfig) -> (f64, f64) {
    let target_secs = 1.5;
    let size = 512;
    let total_flops = 2.0 * size as f64 * size as f64 * size as f64;

    let gflops_per_sec = run_with_adaptive_time(|_| {
        let matrix_a = vec![1.0f64; size * size];
        let matrix_b = vec![2.0f64; size * size];
        let result = vec![0.0f64; size * size];
        (0..num_threads).into_par_iter().for_each(|_| {
            let mut res = result.clone();
            for i in 0..size {
                for k in 0..size {
                    let aik = matrix_a[i * size + k];
                    let row_b = &matrix_b[k * size..(k+1)*size];
                    let row_res = &mut res[i * size..(i+1)*size];
                    for j in 0..size {
                        row_res[j] += aik * row_b[j];
                    }
                }
            }
            std::hint::black_box(res);
        });
        total_flops
    }, target_secs);
    let gflops = gflops_per_sec / 1_000_000_000.0;
    let score = (gflops / config.matrix_ref) * config.reference_score;
    (score, gflops)
}

fn test_hash_throughput(num_threads: usize, config: &CpuConfig) -> (f64, f64) {
    use std::hash::Hasher;
    use twox_hash::XxHash64;

    let target_secs = 1.5;
    let hashes_per_sec = run_with_adaptive_time(|iter| {
        let total_hashes = iter as f64;
        (0..num_threads * 2).into_par_iter().for_each(|_| {
            let mut hasher = XxHash64::default();
            for i in 0..iter {
                hasher.write_u64(i);
                hasher.write(b"benchmark string");
            }
            std::hint::black_box(hasher.finish());
        });
        total_hashes
    }, target_secs);
    let mhash = hashes_per_sec / 1_000_000.0;
    let score = (mhash / config.hash_ref) * config.reference_score;
    (score, mhash)
}