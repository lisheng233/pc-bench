# PC Performance Benchmark

[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![Build Status](https://github.com/lisheng23/pc-bench/actions/workflows/build.yml/badge.svg)](https://github.com/lisheng23/pc-bench/actions)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

跨平台的 PC 性能基准测试工具，使用 Rust 编写，支持 CPU、内存、存储和 GPU 性能评估。结果可导出为可读报告，支持通过配置文件自定义评分规则和权重。

## 功能特性

- **CPU 测试**：整数运算、浮点运算、质数计数、矩阵乘法、哈希吞吐（自适应测试时长）
- **内存测试**：顺序读写、随机读写、内存带宽、延迟测试
- **存储测试**：顺序/随机读写 (MB/s 和 IOPS)
- **GPU 测试**：使用 `wgpu` 进行向量化浮点运算（自动跳过软件渲染器）
- **跨平台**：支持 Windows、Linux、macOS（ARM64/x86_64）
- **可配置**：通过 `benchmark_config.toml` 调整每项权重、参考分数和评级

## 快速开始

### 前置要求

- Rust 1.70 或更高版本
- Linux 用户需安装系统依赖（用于 `wgpu`）：
  ```bash
  sudo apt install libxcb-shape0-dev libxcb-xfixes0-devi

注意事项

· GPU 测试需要支持 Vulkan/Metal/DirectX 12 的显卡。若检测到软件渲染器（如 llvmpipe），测试将自动跳过。
· 存储测试的读取性能受系统页面缓存影响，结果可能比真实硬件性能偏高，属正常现象。
· 在 Android/Termux 环境中，GPU 测试无法访问硬件 GPU，且 CPU 信息可能无法正确读取。
· 测试过程中会短暂占用较高的 CPU/内存/存储资源，请勿在电池模式或对性能敏感的生产环境中运行。
