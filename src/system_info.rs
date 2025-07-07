// Legacy system_info module - kept for compatibility
// Main functionality has been moved to data_collector and ui modules

use crate::types::CPUMetrics;
use regex::Regex;

lazy_static::lazy_static! {
    static ref ACTIVE_RESIDENCY_REGEX: Regex = Regex::new(r"CPU (\d+) active residency:\s+(\d+\.\d+)%").unwrap();
    static ref FREQUENCY_REGEX: Regex = Regex::new(r"^CPU\s+(\d+)\s+frequency:\s+(\d+)\s+MHz$").unwrap();
}

// Legacy function - deprecated in favor of new architecture
// This function is kept for backward compatibility but is no longer used
#[deprecated(note = "Use the new DataCollector and UI modules instead")]
pub async fn get_system_info(
    _receiver: tokio::sync::mpsc::Receiver<bool>,
) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Warning: get_system_info is deprecated. Please use the new main application entry point.");
    Ok(())
}

#[allow(dead_code)]
async fn parse_cpu_metrics(
    powermetrics_output: String,
) -> Result<CPUMetrics, Box<dyn std::error::Error>> {
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
            let core_id: usize = caps[1].parse()?;
            let active_residency: f64 = caps[2].parse()?;
            if core_id <= 3 {
                e_cluster_active_sum += active_residency;
                e_cluster_count += 1;
            } else {
                p_cluster_active_sum += active_residency;
                p_cluster_count += 1;
            }
        }

        if let Some(caps) = FREQUENCY_REGEX.captures(line) {
            let core_id: usize = caps[1].parse()?;
            let active_freq: f64 = caps[2].parse()?;
            if core_id <= 3 {
                e_cluster_freq_sum += active_freq;
            } else {
                p_cluster_freq_sum += active_freq;
            }
        }

        if line.contains("ANE Power") {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if let Some(power) = fields.get(2) {
                cpu_metrics.ane_w = power.trim_end_matches("mW").parse::<f64>()? / 1000.0;
            }
        } else if line.contains("CPU Power") {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if let Some(power) = fields.get(2) {
                cpu_metrics.cpu_w = power.trim_end_matches("mW").parse::<f64>()? / 1000.0;
            }
        } else if line.contains("GPU Power") {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if let Some(power) = fields.get(2) {
                cpu_metrics.gpu_w = power.trim_end_matches("mW").parse::<f64>()? / 1000.0;
            }
        } else if line.contains("Combined Power (CPU + GPU + ANE)") {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if let Some(power) = fields.get(7) {
                cpu_metrics.package_w = power.trim_end_matches("mW").parse::<f64>()? / 1000.0;
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
