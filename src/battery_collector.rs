// Fast battery data collector - optimized version
use crate::types::BatteryInfo;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use regex::Regex;

#[derive(Debug, Clone)]
struct BatteryCapacityInfo {
    current_capacity: u32,
    design_capacity: u32,
    cycle_count: u32,
}

#[derive(Debug, Clone)]
struct BatteryDetailedInfo {
    health_percentage: f32,
    cycle_count: u32,
    power_adapter_wattage: f32,
}

pub struct FastBatteryCollector {
    last_update: Option<Instant>,
    cached_data: Option<BatteryInfo>,
    cache_duration: Duration,
}

impl FastBatteryCollector {
    pub fn new() -> Self {
        Self {
            last_update: None,
            cached_data: None,
            cache_duration: Duration::from_secs(5), // 5 second cache
        }
    }

    pub async fn get_battery_info(&mut self) -> BatteryInfo {
        // Check if cache is valid
        if let (Some(last_update), Some(cached_data)) = (self.last_update, &self.cached_data) {
            if last_update.elapsed() < self.cache_duration {
                return cached_data.clone();
            }
        }

        // Try to quickly get real data
        match self.collect_real_battery_data().await {
            Ok(battery_info) => {
                self.cached_data = Some(battery_info.clone());
                self.last_update = Some(Instant::now());
                battery_info
            }
            Err(_) => {
                // If failed, return cached data or default data
                self.cached_data.clone().unwrap_or_else(|| self.get_fallback_data())
            }
        }
    }

    async fn collect_real_battery_data(&self) -> Result<BatteryInfo, Box<dyn std::error::Error>> {
        let mut battery_info = BatteryInfo::default();

        // Method 1: Use pmset to get basic status (fastest and most reliable)
        if let Ok(basic_info) = self.get_battery_basic_from_pmset().await {
            battery_info.percentage = basic_info.percentage;
            battery_info.is_charging = basic_info.is_charging;
            battery_info.is_plugged = basic_info.is_plugged;
            battery_info.time_remaining = basic_info.time_remaining;
        }

        // Method 2: Use system_profiler to get accurate health information (highest priority)
        if let Ok(detailed_info) = self.get_battery_detailed_from_profiler().await {
            battery_info.health_percentage = detailed_info.health_percentage;
            battery_info.cycle_count = detailed_info.cycle_count;
            battery_info.power_adapter_wattage = detailed_info.power_adapter_wattage;
        }

        // Method 3: Use ioreg to get capacity information (supplementary data)
        if let Ok(capacity_info) = self.get_battery_capacity_from_ioreg().await {
            battery_info.current_capacity = capacity_info.current_capacity;
            battery_info.design_capacity = capacity_info.design_capacity;
            
            // If system_profiler didn't get cycle count, use ioreg data
            if battery_info.cycle_count == 0 {
                battery_info.cycle_count = capacity_info.cycle_count;
            }
            
            // Only use ioreg to calculate health when system_profiler completely fails
            if battery_info.health_percentage == 0.0 && capacity_info.design_capacity > 0 {
                battery_info.health_percentage = 
                    (capacity_info.current_capacity as f32 / capacity_info.design_capacity as f32) * 100.0;
            }
        }

        Ok(battery_info)
    }

    async fn get_battery_basic_from_pmset(&self) -> Result<BatteryInfo, Box<dyn std::error::Error>> {
        // pmset -g batt - quickly get basic battery status
        let pmset_future = tokio::process::Command::new("pmset")
            .arg("-g")
            .arg("batt")
            .output();

        let output = timeout(Duration::from_secs(1), pmset_future).await??;
        
        if !output.status.success() {
            return Err("pmset command failed".into());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        self.parse_pmset_basic(&output_str)
    }

    async fn get_battery_capacity_from_ioreg(&self) -> Result<BatteryCapacityInfo, Box<dyn std::error::Error>> {
        // ioreg -l | grep -i "Capacity" - get capacity information
        let ioreg_future = tokio::process::Command::new("sh")
            .arg("-c")
            .arg("ioreg -l | grep -i 'Capacity\\|CycleCount'")
            .output();

        let output = timeout(Duration::from_secs(2), ioreg_future).await??;
        
        if !output.status.success() {
            return Err("ioreg capacity command failed".into());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        self.parse_ioreg_capacity(&output_str)
    }

    async fn get_battery_detailed_from_profiler(&self) -> Result<BatteryDetailedInfo, Box<dyn std::error::Error>> {
        // system_profiler SPPowerDataType - get detailed information
        let profiler_future = tokio::process::Command::new("system_profiler")
            .arg("SPPowerDataType")
            .output();

        let output = timeout(Duration::from_secs(3), profiler_future).await??;
        
        if !output.status.success() {
            return Err("system_profiler command failed".into());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        self.parse_profiler_detailed(&output_str)
    }

    fn parse_pmset_basic(&self, output: &str) -> Result<BatteryInfo, Box<dyn std::error::Error>> {
        let mut battery_info = BatteryInfo::default();
        
        // Optimized regex based on actual output
        // Actual output: "-InternalBattery-0 (id=20775011)	98%; charging; 0:13 remaining present: true"
        let percentage_regex = Regex::new(r"(\d+)%").unwrap();
        let time_regex = Regex::new(r"(\d+):(\d+) remaining").unwrap();
        
        for line in output.lines() {
            // Check if this is a battery status line
            if line.contains("InternalBattery") {
                // Extract battery percentage
                if let Some(caps) = percentage_regex.captures(line) {
                    if let Ok(percentage) = caps[1].parse::<f32>() {
                        battery_info.percentage = percentage;
                    }
                }
                
                // Check charging status - more precise judgment
                battery_info.is_charging = line.contains("charging");
                battery_info.is_plugged = !line.contains("Battery Power");
                
                // Extract remaining time
                if let Some(caps) = time_regex.captures(line) {
                    if let (Ok(hours), Ok(minutes)) = (caps[1].parse::<u32>(), caps[2].parse::<u32>()) {
                        battery_info.time_remaining = Some(hours * 3600 + minutes * 60);
                    }
                }
                
                // Check special cases
                if line.contains("no estimate") || line.contains("(no estimate)") {
                    battery_info.time_remaining = None;
                }
            }
            
            // Check power status line
            if line.contains("Now drawing from") {
                battery_info.is_plugged = line.contains("AC Power");
            }
        }
        
        Ok(battery_info)
    }

    fn parse_ioreg_capacity(&self, output: &str) -> Result<BatteryCapacityInfo, Box<dyn std::error::Error>> {
        let mut capacity_info = BatteryCapacityInfo {
            current_capacity: 0,
            design_capacity: 0,
            cycle_count: 0,
        };
        
        // Regex based on actual ioreg output
        let current_capacity_regex = Regex::new(r#""AppleRawCurrentCapacity"\s*=\s*(\d+)"#).unwrap();
        let design_capacity_regex = Regex::new(r#""DesignCapacity"\s*=\s*(\d+)"#).unwrap();
        let cycle_count_regex = Regex::new(r#""CycleCount"\s*=\s*(\d+)"#).unwrap();
        
        // Alternative regex for different formats
        let alt_current_regex = Regex::new(r#""CurrentCapacity"\s*=\s*(\d+)"#).unwrap();
        let alt_design_regex = Regex::new(r#""MaxCapacity"\s*=\s*(\d+)"#).unwrap();
        
        for line in output.lines() {
            let line = line.trim();
            
            // Extract current capacity
            if let Some(caps) = current_capacity_regex.captures(line) {
                if let Ok(capacity) = caps[1].parse::<u32>() {
                    capacity_info.current_capacity = capacity;
                }
            } else if let Some(caps) = alt_current_regex.captures(line) {
                if let Ok(capacity) = caps[1].parse::<u32>() {
                    capacity_info.current_capacity = capacity;
                }
            }
            
            // Extract design capacity
            if let Some(caps) = design_capacity_regex.captures(line) {
                if let Ok(capacity) = caps[1].parse::<u32>() {
                    capacity_info.design_capacity = capacity;
                }
            } else if let Some(caps) = alt_design_regex.captures(line) {
                if let Ok(capacity) = caps[1].parse::<u32>() {
                    capacity_info.design_capacity = capacity;
                }
            }
            
            // Extract cycle count
            if let Some(caps) = cycle_count_regex.captures(line) {
                if let Ok(cycles) = caps[1].parse::<u32>() {
                    capacity_info.cycle_count = cycles;
                }
            }
        }
        
        Ok(capacity_info)
    }

    fn parse_profiler_detailed(&self, output: &str) -> Result<BatteryDetailedInfo, Box<dyn std::error::Error>> {
        let mut detailed_info = BatteryDetailedInfo {
            health_percentage: 0.0,
            cycle_count: 0,
            power_adapter_wattage: 0.0,
        };
        
        // Optimized regex based on actual system_profiler SPPowerDataType output
        // Key information extraction: battery level, health, cycle count
        let _state_of_charge_regex = Regex::new(r"State of Charge \(%\):\s*(\d+)").unwrap();
        let maximum_capacity_regex = Regex::new(r"Maximum Capacity:\s*(\d+)%").unwrap();
        let cycle_count_regex = Regex::new(r"Cycle Count:\s*(\d+)").unwrap();
        let condition_regex = Regex::new(r"Condition:\s*(\w+(?:\s+\w+)*)").unwrap();
        let wattage_regex = Regex::new(r"Wattage \(W\):\s*(\d+)").unwrap();
        
        for line in output.lines() {
            let line = line.trim();
            
            // Extract battery health - most important metric
            if let Some(caps) = maximum_capacity_regex.captures(line) {
                if let Ok(health) = caps[1].parse::<f32>() {
                    detailed_info.health_percentage = health;
                }
            }
            
            // Extract cycle count - 电池寿命的关键指标
            else if let Some(caps) = cycle_count_regex.captures(line) {
                if let Ok(cycles) = caps[1].parse::<u32>() {
                    detailed_info.cycle_count = cycles;
                }
            }
            
            // Extract battery condition, estimate health if no specific percentage
            else if let Some(caps) = condition_regex.captures(line) {
                let condition = &caps[1];
                
                // Only use condition estimation when no specific percentage available
                if detailed_info.health_percentage == 0.0 {
                    let estimated_health = match condition {
                        "Normal" => 95.0,
                        "Replace Soon" => 75.0,
                        "Replace Now" => 50.0,
                        "Service Battery" => 30.0,
                        _ => 85.0,
                    };
                    detailed_info.health_percentage = estimated_health;
                }
            }
            
            // Extract charger power
            else if let Some(caps) = wattage_regex.captures(line) {
                if let Ok(wattage) = caps[1].parse::<f32>() {
                    detailed_info.power_adapter_wattage = wattage;
                }
            }
        }
        
        Ok(detailed_info)
    }

    #[allow(dead_code)]
    fn parse_ioreg_output(&self, output: &str) -> Result<BatteryInfo, Box<dyn std::error::Error>> {
        let mut battery_info = BatteryInfo::default();

        for line in output.lines() {
            let line = line.trim();
            
            // Parse battery percentage
            if line.contains("\"StateOfCharge\"") {
                if let Some(value) = self.extract_number_from_line(line) {
                    battery_info.percentage = value as f32;
                }
            }
            
            // Parse cycle count
            else if line.contains("\"CycleCount\"") {
                if let Some(value) = self.extract_number_from_line(line) {
                    battery_info.cycle_count = value as u32;
                }
            }
            
            // Parse current capacity
            else if line.contains("\"CurrentCapacity\"") {
                if let Some(value) = self.extract_number_from_line(line) {
                    battery_info.current_capacity = value as u32;
                }
            }
            
            // Parse design capacity
            else if line.contains("\"DesignCapacity\"") {
                if let Some(value) = self.extract_number_from_line(line) {
                    battery_info.design_capacity = value as u32;
                }
            }
            
            // Parse voltage (mV)
            else if line.contains("\"Voltage\"") {
                if let Some(value) = self.extract_number_from_line(line) {
                    battery_info.voltage = value as f32 / 1000.0; // Convert to V
                }
            }
            
            // Parse current (mA)
            else if line.contains("\"InstantAmperage\"") {
                if let Some(value) = self.extract_number_from_line(line) {
                    battery_info.amperage = value as f32 / 1000.0; // Convert to A
                    battery_info.is_charging = value > 0.0;
                }
            }
            
            // Parse external power
            else if line.contains("\"ExternalConnected\"") {
                battery_info.is_plugged = line.contains("Yes") || line.contains("true");
            }
            
            // Parse temperature
            else if line.contains("\"Temperature\"") {
                if let Some(value) = self.extract_number_from_line(line) {
                    battery_info.temperature = value as f32 / 100.0; // Convert to Celsius
                }
            }
        }

        // Calculate health
        if battery_info.design_capacity > 0 && battery_info.current_capacity > 0 {
            battery_info.health_percentage = 
                (battery_info.current_capacity as f32 / battery_info.design_capacity as f32) * 100.0;
        }

        // Estimate remaining time
        if battery_info.amperage < 0.0 && battery_info.current_capacity > 0 {
            let hours = battery_info.current_capacity as f32 / (-battery_info.amperage * 1000.0);
            battery_info.time_remaining = Some((hours * 3600.0) as u32);
        }

        Ok(battery_info)
    }

    #[allow(dead_code)]
    fn extract_number_from_line(&self, line: &str) -> Option<f64> {
        // Extract number from format
        if let Some(equals_pos) = line.find('=') {
            let value_part = &line[equals_pos + 1..].trim();
            // Remove possible semicolons and spaces
            let clean_value = value_part.trim_end_matches(';').trim();
            clean_value.parse::<f64>().ok()
        } else {
            None
        }
    }



    #[allow(dead_code)]
    fn parse_time_string(&self, time_str: &str) -> Option<u32> {
        // Parse time format
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() == 2 {
            if let (Ok(hours), Ok(minutes)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                return Some(hours * 3600 + minutes * 60);
            }
        }
        None
    }

    #[allow(dead_code)]
    async fn get_battery_from_sysfs(&self) -> Result<BatteryInfo, Box<dyn std::error::Error>> {
        // macOS 没有 /sys/class/power_supply，但可以尝试其他系统文件
        // 这里返回一个基本的电池信息
        Err("sysfs not available on macOS".into())
    }

    fn get_fallback_data(&self) -> BatteryInfo {
        // Only return fallback when all real data collection fails
        // This should rarely be used - prefer real data collection
        BatteryInfo::default()
    }
}