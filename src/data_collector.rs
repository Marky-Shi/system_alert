use crate::{cli::get_powermetrics_output, battery_collector::FastBatteryCollector, types::*};
use sysinfo::{Components, Networks, System};
use std::time::{Duration, Instant};

pub struct DataCollector {
    system: System,
    networks: Networks,
    components: Components,
    last_powermetrics: Option<Instant>,
    cached_cpu_metrics: Option<CPUMetrics>,
    powermetrics_cache_duration: Duration,
    battery_collector: FastBatteryCollector,
}

impl DataCollector {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
            networks: Networks::new_with_refreshed_list(),
            components: Components::new_with_refreshed_list(),
            last_powermetrics: None,
            cached_cpu_metrics: None,
            powermetrics_cache_duration: Duration::from_secs(2),
            battery_collector: FastBatteryCollector::new(),
        }
    }

    // 快速启动版本 - 延迟初始化
    pub fn new_fast() -> Self {
        Self {
            system: System::new(),
            networks: Networks::new(),
            components: Components::new(),
            last_powermetrics: None,
            cached_cpu_metrics: None,
            powermetrics_cache_duration: Duration::from_secs(1),
            battery_collector: FastBatteryCollector::new(),
        }
    }

    pub async fn collect_all_data(&mut self) -> Result<SystemData, Box<dyn std::error::Error>> {
        // 智能刷新 - 只刷新必要的数据
        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.system.refresh_processes_specifics(sysinfo::ProcessesToUpdate::All, true, sysinfo::ProcessRefreshKind::everything());
        self.networks.refresh(true);
        self.components.refresh(true);

        let system_info = self.collect_system_info();
        let cpu_info = self.collect_cpu_info().await?;
        let memory_info = self.collect_memory_info();
        let network_info = self.collect_network_info();
        let temperature_info = self.collect_temperature_info();
        let process_info = self.collect_process_info();
        let total_power = cpu_info.power_metrics.package_w;

        let battery_info = self.battery_collector.get_battery_info().await;
        let thermal_info = self.collect_thermal_info().await;
        let performance_metrics = self.collect_performance_metrics(&cpu_info, total_power).await;
        let system_health = self.collect_system_health().await;

        Ok(SystemData {
            system_info,
            cpu_info,
            memory_info,
            network_info,
            temperature_info,
            process_info,
            battery_info,
            thermal_info,
            performance_metrics,
            system_health,
            timestamp: Instant::now(),
        })
    }

    fn collect_system_info(&self) -> SystemInfo {
        SystemInfo {
            name: System::name().unwrap_or_else(|| "Unknown".to_string()),
            kernel_version: System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
            os_version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
            host_name: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
            cpu_arch: System::cpu_arch(),
            cpu_brand: self.system.cpus().first()
                .map(|cpu| cpu.brand().to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
        }
    }

    async fn collect_cpu_info(&mut self) -> Result<CpuInfo, Box<dyn std::error::Error>> {
        let cpu_usages: Vec<f32> = self.system
            .cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage())
            .collect();

        let average_usage = if !cpu_usages.is_empty() {
            cpu_usages.iter().sum::<f32>() / cpu_usages.len() as f32
        } else {
            0.0
        };

        let power_metrics = if let Some(last_time) = self.last_powermetrics {
            if last_time.elapsed() < self.powermetrics_cache_duration {
                self.cached_cpu_metrics.clone().unwrap_or_default()
            } else {
                self.fetch_fresh_powermetrics().await?
            }
        } else {
            self.fetch_fresh_powermetrics().await?
        };

        Ok(CpuInfo {
            core_usages: cpu_usages,
            average_usage,
            power_metrics,
        })
    }

    async fn fetch_fresh_powermetrics(&mut self) -> Result<CPUMetrics, Box<dyn std::error::Error>> {
        match get_powermetrics_output().await {
            Ok(output) => {
                let metrics = parse_cpu_metrics(output).await?;
                self.cached_cpu_metrics = Some(metrics.clone());
                self.last_powermetrics = Some(Instant::now());
                Ok(metrics)
            }
            Err(e) => {
                // If powermetrics fails, use fallback metrics
                log::warn!("Powermetrics failed, using fallback: {}", e);
                let fallback_metrics = self.get_fallback_cpu_metrics();
                self.cached_cpu_metrics = Some(fallback_metrics.clone());
                self.last_powermetrics = Some(Instant::now());
                Ok(fallback_metrics)
            }
        }
    }

    fn get_fallback_cpu_metrics(&self) -> CPUMetrics {
        // Use real CPU usage as basis for fallback metrics
        let avg_usage = self.system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / self.system.cpus().len() as f32;
        
        // Estimate based on actual CPU usage
        let estimated_power = (avg_usage / 100.0) * 15.0; // Scale with usage
        
        CPUMetrics {
            e_cluster_active: (avg_usage * 0.6) as i32, // E-cores typically more active
            p_cluster_active: (avg_usage * 0.4) as i32, // P-cores less active
            e_cluster_freq_mhz: if avg_usage > 50.0 { 2400 } else { 1800 },
            p_cluster_freq_mhz: if avg_usage > 70.0 { 3200 } else { 2400 },
            ane_w: (estimated_power * 0.05) as f64,
            cpu_w: (estimated_power * 0.6) as f64,
            gpu_w: (estimated_power * 0.2) as f64,
            package_w: estimated_power as f64,
        }
    }

    fn collect_memory_info(&self) -> MemoryInfo {
        let total_memory = self.system.total_memory();
        let used_memory = self.system.used_memory();
        let available_memory = self.system.available_memory();
        let total_swap = self.system.total_swap();
        let used_swap = self.system.used_swap();

        let usage_percentage = if total_memory > 0 {
            (used_memory as f64 / total_memory as f64 * 100.0) as u16
        } else {
            0
        };

        MemoryInfo {
            total_memory,
            used_memory,
            available_memory,
            total_swap,
            used_swap,
            usage_percentage,
        }
    }

    fn collect_network_info(&self) -> Vec<NetworkInterface> {
        self.networks
            .iter()
            .map(|(name, data)| NetworkInterface {
                name: name.to_string(),
                bytes_received: data.total_received(),
                bytes_transmitted: data.total_transmitted(),
                packets_received: data.packets_received(),
                packets_transmitted: data.packets_transmitted(),
            })
            .collect()
    }

    fn collect_temperature_info(&self) -> Vec<TemperatureInfo> {
        self.components
            .iter()
            .filter_map(|component| {
                component.temperature().map(|temp| TemperatureInfo {
                    label: component.label().to_string(),
                    temperature: temp,
                    critical_temperature: component.critical().unwrap_or(100.0),
                })
            })
            .collect()
    }

    fn collect_process_info(&self) -> Vec<ProcessInfo> {
        self.system
            .processes()
            .iter()
            .map(|(pid, process)| ProcessInfo {
                pid: *pid,
                name: process.name().to_string_lossy().to_string(),
                cpu_usage: process.cpu_usage(),
                memory_usage: process.memory(),
                disk_read_bytes: process.disk_usage().read_bytes,
                disk_write_bytes: process.disk_usage().written_bytes,
            })
            .collect()
    }



    async fn collect_thermal_info(&self) -> ThermalInfo {
        let mut thermal_info = ThermalInfo::default();
        
        // Get real thermal pressure from system
        if let Ok(output) = tokio::process::Command::new("sysctl")
            .arg("-n")
            .arg("machdep.xcpm.cpu_thermal_level")
            .output()
            .await
        {
            if let Ok(thermal_str) = String::from_utf8(output.stdout) {
                if let Ok(thermal_level) = thermal_str.trim().parse::<u8>() {
                    thermal_info.thermal_pressure = thermal_level * 10; // Convert to percentage
                }
            }
        }
        
        // Check for thermal throttling via CPU frequency scaling
        thermal_info.thermal_throttling = thermal_info.thermal_pressure > 50;
        
        // Get fan speeds from powermetrics if available
        if let Ok(output) = tokio::process::Command::new("powermetrics")
            .arg("--samplers")
            .arg("smc")
            .arg("-n")
            .arg("1")
            .arg("--show-initial-usage")
            .output()
            .await
        {
            if let Ok(power_str) = String::from_utf8(output.stdout) {
                let mut fan_speeds = Vec::new();
                for line in power_str.lines() {
                    if line.contains("Fan") && line.contains("RPM") {
                        if let Some(rpm_str) = line.split_whitespace().find(|s| s.ends_with("RPM")) {
                            if let Ok(rpm) = rpm_str.trim_end_matches("RPM").parse::<u32>() {
                                fan_speeds.push(rpm);
                            }
                        }
                    }
                }
                if !fan_speeds.is_empty() {
                    thermal_info.fan_speeds = fan_speeds;
                }
            }
        }
        
        // Estimate heat dissipation based on thermal pressure
        thermal_info.heat_dissipation_rate = match thermal_info.thermal_pressure {
            0..=20 => 5.0,
            21..=40 => 10.0,
            41..=60 => 15.0,
            61..=80 => 20.0,
            _ => 25.0,
        };
        
        thermal_info
    }

    async fn collect_performance_metrics(&self, cpu_info: &CpuInfo, total_power: f64) -> PerformanceMetrics {
        let mut metrics = PerformanceMetrics::default();

        // Calculate instructions per watt (estimated)
        if total_power > 0.0 {
            let estimated_instructions = cpu_info.average_usage as f64 * 1000000.0; // Simplified
            metrics.instructions_per_watt = estimated_instructions / total_power;
        }

        // Calculate performance per watt
        if total_power > 0.0 {
            metrics.performance_per_watt = cpu_info.average_usage as f64 / total_power;
        }

        // Calculate frequency efficiency
        let avg_freq = (cpu_info.power_metrics.e_cluster_freq_mhz + cpu_info.power_metrics.p_cluster_freq_mhz) as f64 / 2.0;
        if avg_freq > 0.0 {
            metrics.frequency_efficiency = cpu_info.average_usage as f64 / avg_freq * 1000.0;
        }

        // Determine workload type
        metrics.workload_type = if cpu_info.average_usage < 10.0 {
            "idle".to_string()
        } else if cpu_info.power_metrics.gpu_w > cpu_info.power_metrics.cpu_w {
            "graphics".to_string()
        } else if cpu_info.average_usage > 70.0 {
            "compute".to_string()
        } else {
            "mixed".to_string()
        };

        metrics
    }

    async fn collect_system_health(&self) -> SystemHealthInfo {
        // Get real system health data
        let mut health_info = SystemHealthInfo::default();
        
        // Get real uptime
        if let Ok(output) = tokio::process::Command::new("sysctl")
            .arg("-n")
            .arg("kern.boottime")
            .output()
            .await
        {
            if let Ok(boottime_str) = String::from_utf8(output.stdout) {
                // Parse boottime and calculate uptime
                if let Some(timestamp_str) = boottime_str.split_whitespace().nth(3) {
                    if let Ok(boot_timestamp) = timestamp_str.trim_end_matches(',').parse::<i64>() {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64;
                        health_info.uptime_seconds = (now - boot_timestamp) as u64;
                    }
                }
            }
        }
        
        // Get real load averages
        if let Ok(output) = tokio::process::Command::new("sysctl")
            .arg("-n")
            .arg("vm.loadavg")
            .output()
            .await
        {
            if let Ok(loadavg_str) = String::from_utf8(output.stdout) {
                // Parse "{ 2.66 2.75 2.97 }" format
                let loads: Vec<f64> = loadavg_str
                    .trim()
                    .trim_start_matches('{')
                    .trim_end_matches('}')
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();
                
                if loads.len() >= 3 {
                    health_info.system_load_1min = loads[0];
                    health_info.system_load_5min = loads[1];
                    health_info.system_load_15min = loads[2];
                }
            }
        }
        
        // Calculate power quality score based on load and other factors
        let avg_load = (health_info.system_load_1min + health_info.system_load_5min + health_info.system_load_15min) / 3.0;
        health_info.power_quality_score = if avg_load < 1.0 {
            95
        } else if avg_load < 2.0 {
            85
        } else if avg_load < 3.0 {
            75
        } else {
            65
        };
        
        // Estimate sleep/wake efficiency (simplified)
        health_info.sleep_wake_efficiency = if avg_load < 1.5 { 95.0 } else { 85.0 };
        
        health_info
    }
}

async fn parse_cpu_metrics(
    powermetrics_output: String,
) -> Result<CPUMetrics, Box<dyn std::error::Error>> {
    use regex::Regex;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref ACTIVE_RESIDENCY_REGEX: Regex = Regex::new(r"CPU (\d+) active residency:\s+(\d+\.\d+)%").unwrap();
        static ref FREQUENCY_REGEX: Regex = Regex::new(r"^CPU\s+(\d+)\s+frequency:\s+(\d+)\s+MHz$").unwrap();
    }

    let lines: Vec<&str> = powermetrics_output.lines().collect();
    let mut cpu_metrics = CPUMetrics::default();

    let mut e_cluster_active_sum = 0.0;
    let mut p_cluster_active_sum = 0.0;
    let mut e_cluster_freq_sum = 0.0;
    let mut p_cluster_freq_sum = 0.0;
    let mut e_cluster_count = 0;
    let mut p_cluster_count = 0;

    for line in &lines {
        if let Some(caps) = ACTIVE_RESIDENCY_REGEX.captures(line) {
            if let (Ok(core_id), Ok(active_residency)) = (caps[1].parse::<usize>(), caps[2].parse::<f64>()) {
                if core_id <= 3 {
                    e_cluster_active_sum += active_residency;
                    e_cluster_count += 1;
                } else {
                    p_cluster_active_sum += active_residency;
                    p_cluster_count += 1;
                }
            }
        }

        if let Some(caps) = FREQUENCY_REGEX.captures(line) {
            if let (Ok(core_id), Ok(active_freq)) = (caps[1].parse::<usize>(), caps[2].parse::<f64>()) {
                if core_id <= 3 {
                    e_cluster_freq_sum += active_freq;
                } else {
                    p_cluster_freq_sum += active_freq;
                }
            }
        }

        if line.contains("ANE Power") {
            if let Some(power_str) = line.split_whitespace().nth(2) {
                if let Ok(power) = power_str.trim_end_matches("mW").parse::<f64>() {
                    cpu_metrics.ane_w = power / 1000.0;
                }
            }
        } else if line.contains("CPU Power") {
            if let Some(power_str) = line.split_whitespace().nth(2) {
                if let Ok(power) = power_str.trim_end_matches("mW").parse::<f64>() {
                    cpu_metrics.cpu_w = power / 1000.0;
                }
            }
        } else if line.contains("GPU Power") {
            if let Some(power_str) = line.split_whitespace().nth(2) {
                if let Ok(power) = power_str.trim_end_matches("mW").parse::<f64>() {
                    cpu_metrics.gpu_w = power / 1000.0;
                }
            }
        } else if line.contains("Combined Power (CPU + GPU + ANE)") {
            if let Some(power_str) = line.split_whitespace().nth(7) {
                if let Ok(power) = power_str.trim_end_matches("mW").parse::<f64>() {
                    cpu_metrics.package_w = power / 1000.0;
                }
            }
        }
    }

    if e_cluster_count > 0 {
        cpu_metrics.e_cluster_active = (e_cluster_active_sum / e_cluster_count as f64) as i32;
        cpu_metrics.e_cluster_freq_mhz = (e_cluster_freq_sum / e_cluster_count as f64) as i32;
    }

    if p_cluster_count > 0 {
        cpu_metrics.p_cluster_active = (p_cluster_active_sum / p_cluster_count as f64) as i32;
        cpu_metrics.p_cluster_freq_mhz = (p_cluster_freq_sum / p_cluster_count as f64) as i32;
    }

    Ok(cpu_metrics)
}