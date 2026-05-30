// system_info.rs
use sysinfo::System;

pub struct SystemInfo {
    pub cpu_name: String,
    pub cpu_cores: usize,
    pub total_ram: u64,
    pub os_name: String,
    pub kernel_version: String,
}

pub fn get_system_info() -> SystemInfo {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    let cpu_name = sys.cpus()
        .first()
        .map(|cpu| cpu.brand().to_string())
        .unwrap_or_else(|| "Unknown CPU".to_string());
    
    let cpu_cores = sys.cpus().len();
    let total_ram = sys.total_memory();
    let os_name = System::name().unwrap_or_else(|| "Unknown OS".to_string());
    let kernel_version = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
    
    SystemInfo {
        cpu_name,
        cpu_cores,
        total_ram,
        os_name,
        kernel_version,
    }
}

pub fn print_system_info(info: &SystemInfo) {
    println!("{}", "System Information:");
    println!("  {}: {}", "OS", info.os_name);
    println!("  {}: {}", "Kernel", info.kernel_version);
    println!("  {}: {}", "CPU", info.cpu_name);
    println!("  {}: {}", "CPU Cores", info.cpu_cores);
    println!("  {}: {:.2} GB", "Total RAM", 
             info.total_ram as f64 / 1024.0 / 1024.0 / 1024.0);
}