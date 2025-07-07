use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub refresh_rate: u64,
    pub minimal_mode: bool,
    pub thresholds: ThresholdConfig,
    pub display: DisplayConfig,
    pub notifications: NotificationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    pub cpu_warning: f32,
    pub cpu_critical: f32,
    pub memory_warning: u16,
    pub memory_critical: u16,
    pub temperature_warning: f32,
    pub temperature_critical: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub show_temperatures: bool,
    pub show_network: bool,
    pub show_processes: bool,
    pub show_history: bool,
    pub history_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub enabled: bool,
    pub cooldown_seconds: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            refresh_rate: 1,
            minimal_mode: false,
            thresholds: ThresholdConfig {
                cpu_warning: 75.0,
                cpu_critical: 90.0,
                memory_warning: 75,
                memory_critical: 90,
                temperature_warning: 70.0,
                temperature_critical: 85.0,
            },
            display: DisplayConfig {
                show_temperatures: true,
                show_network: true,
                show_processes: true,
                show_history: true,
                history_size: 60,
            },
            notifications: NotificationConfig {
                enabled: true,
                cooldown_seconds: 30,
            },
        }
    }
}

impl Config {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn merge_with_cli(&mut self, refresh_rate: Option<u64>, minimal_mode: bool) {
        if let Some(rate) = refresh_rate {
            self.refresh_rate = rate;
        }
        if minimal_mode {
            self.minimal_mode = true;
        }
    }
}