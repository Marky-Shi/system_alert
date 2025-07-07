use std::time::Instant;
use sysinfo::Pid;

#[derive(Default, Debug, Clone)]
pub struct CPUMetrics {
    pub e_cluster_active: i32,
    pub p_cluster_active: i32,
    pub e_cluster_freq_mhz: i32,
    pub p_cluster_freq_mhz: i32,
    pub cpu_w: f64,
    pub gpu_w: f64,
    pub ane_w: f64,
    pub package_w: f64,
}

impl std::fmt::Display for CPUMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "E-Cluster Active: {}%\nP-Cluster Active: {}%\nE-Cluster Freq: {} MHz\nP-Cluster Freq: {} MHz\nCPU Power: {:.2}W\nGPU Power: {:.2}W\nANE Power: {:.2}W\nPackage Power: {:.2}W",
        self.e_cluster_active,
        self.p_cluster_active,
        self.e_cluster_freq_mhz,
        self.p_cluster_freq_mhz,
        self.cpu_w,
        self.gpu_w,
        self.ane_w,
        self.package_w,
        )
    }
}

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub name: String,
    pub kernel_version: String,
    pub os_version: String,
    pub host_name: String,
    pub cpu_arch: String,
    pub cpu_brand: String,
}

#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub core_usages: Vec<f32>,
    pub average_usage: f32,
    pub power_metrics: CPUMetrics,
}

#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub total_memory: u64,
    pub used_memory: u64,
    pub available_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub usage_percentage: u16,
}

#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub bytes_received: u64,
    pub bytes_transmitted: u64,
    pub packets_received: u64,
    pub packets_transmitted: u64,
}

#[derive(Debug, Clone)]
pub struct TemperatureInfo {
    pub label: String,
    pub temperature: f32,
    pub critical_temperature: f32,
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: Pid,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct BatteryInfo {
    pub percentage: f32,
    pub is_charging: bool,
    pub is_plugged: bool,
    pub health_percentage: f32,
    pub cycle_count: u32,
    pub time_remaining: Option<u32>, // minutes
    pub power_adapter_wattage: f32,
    pub current_capacity: u32, // mAh
    pub design_capacity: u32, // mAh
    pub voltage: f32, // V
    pub amperage: f32, // A
    pub temperature: f32, // °C
}

impl Default for BatteryInfo {
    fn default() -> Self {
        Self {
            percentage: 0.0,
            is_charging: false,
            is_plugged: false,
            health_percentage: 0.0,
            cycle_count: 0,
            time_remaining: None,
            power_adapter_wattage: 0.0,
            current_capacity: 0,
            design_capacity: 0,
            voltage: 0.0,
            amperage: 0.0,
            temperature: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThermalInfo {
    pub fan_speeds: Vec<u32>, // RPM
    pub thermal_throttling: bool,
    pub heat_dissipation_rate: f32, // W
    pub thermal_pressure: u8, // 0-100
}

impl Default for ThermalInfo {
    fn default() -> Self {
        Self {
            fan_speeds: Vec::new(),
            thermal_throttling: false,
            heat_dissipation_rate: 0.0,
            thermal_pressure: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub instructions_per_watt: f64,
    pub performance_per_watt: f64,
    pub frequency_efficiency: f64,
    pub workload_type: String, // "compute", "graphics", "mixed", "idle"
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            instructions_per_watt: 0.0,
            performance_per_watt: 0.0,
            frequency_efficiency: 0.0,
            workload_type: "idle".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemHealthInfo {
    pub uptime_seconds: u64,
    pub sleep_wake_efficiency: f32, // %
    pub power_quality_score: u8, // 0-100
    pub system_load_1min: f64,
    pub system_load_5min: f64,
    pub system_load_15min: f64,
}

impl Default for SystemHealthInfo {
    fn default() -> Self {
        Self {
            uptime_seconds: 0,
            sleep_wake_efficiency: 0.0,
            power_quality_score: 0,
            system_load_1min: 0.0,
            system_load_5min: 0.0,
            system_load_15min: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemData {
    pub system_info: SystemInfo,
    pub cpu_info: CpuInfo,
    pub memory_info: MemoryInfo,
    pub network_info: Vec<NetworkInterface>,
    pub temperature_info: Vec<TemperatureInfo>,
    pub process_info: Vec<ProcessInfo>,
    pub battery_info: BatteryInfo,
    pub thermal_info: ThermalInfo,
    pub performance_metrics: PerformanceMetrics,
    pub system_health: SystemHealthInfo,
    pub timestamp: Instant,
}