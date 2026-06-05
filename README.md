# PC Performance Benchmark

[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

English | 中文

A cross-platform PC performance benchmark tool written in Rust, supporting CPU, memory, storage and GPU performance evaluation. Results can be exported as readable reports, and scoring rules and weights can be customized via a configuration file.

跨平台的 PC 性能基准测试工具，使用 Rust 编写，支持 CPU、内存、存储和 GPU 性能评估。结果可导出为可读报告，支持通过配置文件自定义评分规则和权重。

Features / 功能特性

 CPU Test / CPU 测试: Integer operations, floating point operations, prime counting, matrix multiplication, hash throughput (adaptive test duration)
整数运算、浮点运算、质数计数、矩阵乘法、哈希吞吐（自适应测试时长）

 Memory Test  /  内存测试: Sequential read/write, random read/write, memory bandwidth, latency test
顺序读写、随机读写、内存带宽、延迟测试

 Storage Test  /  存储测试: Sequential/random read/write (MB/s and IOPS)
顺序/随机读写 (MB/s 和 IOPS)

 GPU Test  /  GPU 测试: Vectorized floating point operations using wgpu (automatically skips software renderers; forcing the test is possible but not recommended)
使用 wgpu 进行向量化浮点运算（自动跳过软件渲染器，可强制测试但不推荐）

 Cross-platform / 跨平台: Supports Windows, Linux, macOS (ARM64/x86_64)
支持 Windows、Linux、macOS（ARM64/x86_64）

 Configurable  /  可配置: Adjust per-item weights, reference scores and rating tiers via benchmark_config.toml
通过 benchmark_config.toml 调整每项权重、参考分数和评级等

Quick Start / 快速开始

Prerequisites / 前置要求

· Rust 1.70 or higher    /    Rust 1.70 或更高版本
· Linux users need to install system dependencies (required by wgpu)    /    Linux 用户需安装系统依赖（用于 wgpu）：
  ```bash
  sudo apt install libxcb-shape0-dev libxcb-xfixes0-dev
  ```

Notes / 注意事项

 GPU test requires a graphics card that supports Vulkan/Metal/DirectX 12. If a software renderer is detected (e.g. llvmpipe), the test will be automatically skipped (by default).
    GPU 测试需要支持 Vulkan/Metal/DirectX 12 的显卡。若检测到软件渲染器（如 llvmpipe），测试将自动跳过（默认情况下）。
    
 Read performance in the storage test is affected by the system page cache; results may appear higher than actual hardware performance, which is normal.
    存储测试的读取性能受系统页面缓存影响，结果可能比真实硬件性能偏高，属正常现象。
    
 In Android/Termux environments, the GPU test cannot access the hardware GPU, and CPU information may not be read correctly.
    在 Android/Termux 环境中，GPU 测试无法访问硬件 GPU，且 CPU 信息可能无法正确读取。
    
 The test briefly consumes high CPU/memory/storage resources. Do not run it on battery power or in performance-sensitive production environments.
    测试过程中会短暂占用较高的 CPU/内存/存储资源，请勿在电池模式或对性能敏感的生产环境中运行。
