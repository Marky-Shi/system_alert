use crate::{config::ThresholdConfig, types::SystemData};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone)]
pub struct Notification {
    title: String,
    message: String,
    level: AlertLevel,
    #[allow(dead_code)]
    timestamp: Instant,
}

impl Notification {
    pub fn new(title: &str, message: &str, level: AlertLevel) -> Self {
        Self {
            title: title.to_string(),
            message: message.to_string(),
            level,
            timestamp: Instant::now(),
        }
    }
    
    pub async fn send(&self) -> Result<(), Box<dyn std::error::Error>> {
        let subtitle = match self.level {
            AlertLevel::Info => "Information",
            AlertLevel::Warning => "Warning",
            AlertLevel::Critical => "Critical Alert",
        };
        
        let script = format!(
            r#"display notification "{}" with title "{}" subtitle "{}""#,
            self.message, self.title, subtitle
        );
        
        tokio::process::Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .await?;
            
        Ok(())
    }
}

pub struct NotificationManager {
    last_notifications: HashMap<String, Instant>,
    cooldown_duration: Duration,
    enabled: bool,
}

impl NotificationManager {
    pub fn new(enabled: bool, cooldown_seconds: u64) -> Self {
        Self {
            last_notifications: HashMap::new(),
            cooldown_duration: Duration::from_secs(cooldown_seconds),
            enabled,
        }
    }

    pub async fn check_and_send_notifications(
        &mut self,
        data: &SystemData,
        thresholds: &ThresholdConfig,
    ) -> Result<Vec<Notification>, Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(vec![]);
        }

        let mut notifications = Vec::new();

        // Check CPU usage
        if let Some(notification) = self.check_cpu_threshold(data.cpu_info.average_usage, thresholds) {
            if self.should_send_notification("cpu") {
                notification.send().await?;
                self.last_notifications.insert("cpu".to_string(), Instant::now());
                notifications.push(notification);
            }
        }

        // Check memory usage
        if let Some(notification) = self.check_memory_threshold(data.memory_info.usage_percentage, thresholds) {
            if self.should_send_notification("memory") {
                notification.send().await?;
                self.last_notifications.insert("memory".to_string(), Instant::now());
                notifications.push(notification);
            }
        }

        // Check temperature
        if let Some(notification) = self.check_temperature_threshold(&data.temperature_info, thresholds) {
            if self.should_send_notification("temperature") {
                notification.send().await?;
                self.last_notifications.insert("temperature".to_string(), Instant::now());
                notifications.push(notification);
            }
        }

        Ok(notifications)
    }

    fn should_send_notification(&self, key: &str) -> bool {
        if let Some(last_time) = self.last_notifications.get(key) {
            last_time.elapsed() >= self.cooldown_duration
        } else {
            true
        }
    }

    fn check_cpu_threshold(&self, cpu_usage: f32, thresholds: &ThresholdConfig) -> Option<Notification> {
        if cpu_usage > thresholds.cpu_critical {
            Some(Notification::new(
                "CPU Alert",
                &format!("CPU usage is critically high: {:.1}%", cpu_usage),
                AlertLevel::Critical,
            ))
        } else if cpu_usage > thresholds.cpu_warning {
            Some(Notification::new(
                "CPU Alert",
                &format!("CPU usage is high: {:.1}%", cpu_usage),
                AlertLevel::Warning,
            ))
        } else {
            None
        }
    }

    fn check_memory_threshold(&self, memory_percentage: u16, thresholds: &ThresholdConfig) -> Option<Notification> {
        if memory_percentage > thresholds.memory_critical {
            Some(Notification::new(
                "Memory Alert",
                &format!("Memory usage is critically high: {}%", memory_percentage),
                AlertLevel::Critical,
            ))
        } else if memory_percentage > thresholds.memory_warning {
            Some(Notification::new(
                "Memory Alert",
                &format!("Memory usage is high: {}%", memory_percentage),
                AlertLevel::Warning,
            ))
        } else {
            None
        }
    }

    fn check_temperature_threshold(&self, temperatures: &[crate::types::TemperatureInfo], thresholds: &ThresholdConfig) -> Option<Notification> {
        for temp_info in temperatures {
            if temp_info.temperature > thresholds.temperature_critical {
                return Some(Notification::new(
                    "Temperature Alert",
                    &format!("{} temperature is critically high: {:.1}°C", temp_info.label, temp_info.temperature),
                    AlertLevel::Critical,
                ));
            } else if temp_info.temperature > thresholds.temperature_warning {
                return Some(Notification::new(
                    "Temperature Alert",
                    &format!("{} temperature is high: {:.1}°C", temp_info.label, temp_info.temperature),
                    AlertLevel::Warning,
                ));
            }
        }
        None
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_cooldown(&mut self, cooldown_seconds: u64) {
        self.cooldown_duration = Duration::from_secs(cooldown_seconds);
    }
}