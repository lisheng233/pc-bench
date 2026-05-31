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
    pub reference_score: f64,
}

/// 存储基准配置
#[derive(Debug, Deserialize, Serialize)]
pub struct StorageConfig {
    pub reference_score: f64,
}

/// GPU 基准配置
#[derive(Debug, Deserialize, Serialize)]
pub struct GpuConfig {
    pub reference_score: f64,
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
            integer_ref: 246.0,
            float_ref: 62.0,
            prime_ref: 6222.77,
            matrix_ref: 4.79,
            hash_ref: 24.02,
            reference_score: 10000.0,
        }
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self { reference_score: 10000.0 }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self { reference_score: 10000.0 }
    }
}

impl Default for GpuConfig {
    fn default() -> Self {
        Self { reference_score: 10000.0 }
    }
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            cpu_weight: 0.4,
            memory_weight: 0.3,
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