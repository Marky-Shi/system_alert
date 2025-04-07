use std::process::Command;

pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

pub struct Notification {
    title: String,
    message: String,
    level: AlertLevel,
}

impl Notification {
    pub fn new(title: &str, message: &str, level: AlertLevel) -> Self {
        Self {
            title: title.to_string(),
            message: message.to_string(),
            level,
        }
    }
    
    pub fn send(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 根据警告级别设置不同的通知样式
        let subtitle = match self.level {
            AlertLevel::Info => "Information",
            AlertLevel::Warning => "Warning",
            AlertLevel::Critical => "Critical Alert",
        };
        
        // 使用MacOS的osascript发送通知
        let script = format!(
            r#"display notification "{}" with title "{}" subtitle "{}""#,
            self.message, self.title, subtitle
        );
        
        Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()?;
            
        Ok(())
    }
}

pub fn check_thresholds(cpu_usage: f32, memory_percentage: u16) -> Option<Notification> {
    if cpu_usage > 90.0 {
        return Some(Notification::new(
            "CPU Alert",
            &format!("CPU usage is critically high: {:.2}%", cpu_usage),
            AlertLevel::Critical,
        ));
    } else if cpu_usage > 75.0 {
        return Some(Notification::new(
            "CPU Alert",
            &format!("CPU usage is high: {:.2}%", cpu_usage),
            AlertLevel::Warning,
        ));
    }
    
    if memory_percentage > 90 {
        return Some(Notification::new(
            "Memory Alert",
            &format!("Memory usage is critically high: {}%", memory_percentage),
            AlertLevel::Critical,
        ));
    } else if memory_percentage > 75 {
        return Some(Notification::new(
            "Memory Alert",
            &format!("Memory usage is high: {}%", memory_percentage),
            AlertLevel::Warning,
        ));
    }
    
    None
}