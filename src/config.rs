use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// 全局配置
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub scoring: ScoringWeights,
    pub cpu: CpuConfig,
    pub memory: MemoryConfig,
    pub storage: StorageConfig,
    pub gpu: GpuConfig,
    pub rating: RatingConfig,
}

/// 总分权重配置
#[derive(Debug, Deserialize, Serialize)]
pub struct ScoringWeights {
    pub cpu_weight: f64,
    pub memory_weight: f64,
    pub storage_weight: f64,
    pub gpu_weight: f64,
}

/// CPU 基准配置
#[derive(Debug, Deserialize, Serialize)]
pub struct CpuConfig {
    pub calc_average_cpu_score: bool,
    pub num_threads_ref:usize,
    pub integer_ref: f64,
    pub float_ref: f64,
    pub prime_ref: f64,
    pub matrix_ref: f64,
    pub hash_ref: f64,
    pub reference_score: f64,
}

/// 内存基准配置
#[derive(Debug, Deserialize, Serialize)]
pub struct MemoryConfig {
    pub sequential_access_ref:f64,
    pub sequential_access_test_size:usize,
    pub random_access_ref:f64,
    pub random_access_test_size:usize,
    pub memory_bandwidth_ref:f64,
    pub memory_bandwidth_test_size:usize,
    pub latency_ref:f64,
    pub latency_test_size:usize,
}

/// 存储基准配置
#[derive(Debug, Deserialize, Serialize)]
pub struct StorageConfig {
    pub sequential_write_ref:f64,
    pub sequential_read_ref : f64,
    pub random_write_ref:f64,
    pub random_read_ref:f64,
}

/// GPU 基准配置
#[derive(Debug, Deserialize, Serialize)]
pub struct GpuConfig {
    pub force_test_vgpu: bool,
    pub reference_score: f64,
    pub iteration: u32,
    pub vec_ref: f64,
    pub vram_bandwidth_ref: f64,
    pub vram_bandwidth_test_size_mb: usize,
    pub vram_capacity_weight: f64,
}

/// 评级配置
#[derive(Debug, Deserialize, Serialize)]
pub struct RatingConfig {
    pub excellent: f64,
    pub very_good: f64,
    pub good: f64,
    pub average: f64,
    pub labels: RatingLabels,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RatingLabels {
    pub excellent: String,
    pub very_good: String,
    pub good: String,
    pub average: String,
    pub below_average: String,
}

impl Default for RatingLabels {
    fn default() -> Self {
        Self {
            excellent: "Excellent".to_string(),
            very_good: "Very Good".to_string(),
            good: "Good".to_string(),
            average: "Average".to_string(),
            below_average: "Below Average".to_string(),
        }
    }
}

impl Default for RatingConfig {
    fn default() -> Self {
        Self {
            excellent: 10000.0,
            very_good: 7000.0,
            good: 5000.0,
            average: 3000.0,
            labels: RatingLabels::default(),
        }
    }
}

impl Default for CpuConfig {
    fn default() -> Self {
        Self {
            calc_average_cpu_score: true,
            num_threads_ref: 4,
            integer_ref: 240.0,
            float_ref: 60.0,
            prime_ref: 6000.0,
            matrix_ref: 4.0,
            hash_ref: 24.0,
            reference_score: 10000.0,
        }
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
        sequential_access_ref:0.004,
        sequential_access_test_size:100_000_000,
        random_access_ref:0.01,
        random_access_test_size:10_000_000,
        memory_bandwidth_ref:0.005,
        memory_bandwidth_test_size:50_000_000,
        latency_ref:0.002,
        latency_test_size:10_000_000,
    }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self { sequential_write_ref:0.2,
            sequential_read_ref:0.8,
            random_write_ref:10.0,
            random_read_ref:100.0
        }
    }
}

impl Default for GpuConfig {
    fn default() -> Self {
        Self { 
            force_test_vgpu: false,
            reference_score: 10000.0 ,
            iteration: 64,
            vec_ref:5000.0,
            vram_bandwidth_ref: 200.0,
            vram_bandwidth_test_size_mb: 128,
            vram_capacity_weight: 0.3,
            }
    }
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            cpu_weight: 0.5,
            memory_weight: 0.2,
            storage_weight: 0.2,
            gpu_weight: 0.1,
        }
    }
}

impl Config {
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scoring: ScoringWeights::default(),
            cpu: CpuConfig::default(),
            memory: MemoryConfig::default(),
            storage: StorageConfig::default(),
            gpu: GpuConfig::default(),
            rating: RatingConfig::default(),
        }
    }
}