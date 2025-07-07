use crate::{config::Config, history::HistoryData, types::*};
use std::io;
use termion::raw::IntoRawMode;
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Gauge},
    Frame, Terminal,
};

pub struct UI {
    terminal: Terminal<TermionBackend<termion::raw::RawTerminal<std::io::Stderr>>>,
}

impl UI {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let stderr = io::stderr().into_raw_mode()?;
        let backend = TermionBackend::new(stderr);
        let terminal = Terminal::new(backend)?;
        
        Ok(Self {
            terminal,
        })
    }

    pub fn show_loading_screen(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.terminal.draw(|f| {
            let size = f.size();
            
            let loading_text = vec![
                "🚀 System Alert Monitor",
                "",
                "⏳ Initializing system monitoring...",
                "📊 Loading data collectors...",
                "🔧 Setting up components...",
                "",
                "Please wait a moment...",
                "",
                "Press 'q' to quit when ready",
            ];
            
            let loading_content = loading_text.join("\n");
            
            let loading_block = Paragraph::new(loading_content)
                .block(Block::default()
                    .title("🔄 Starting Up")
                    .borders(Borders::ALL))
                .style(Style::default().fg(Color::Cyan))
                .alignment(Alignment::Center);
            
            // Center the loading screen
            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                ].as_ref())
                .split(size);
            
            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(60),
                    Constraint::Percentage(20),
                ].as_ref())
                .split(vertical_chunks[1]);
            
            f.render_widget(loading_block, horizontal_chunks[1]);
        })?;
        
        Ok(())
    }

    pub fn draw(
        &mut self,
        data: &SystemData,
        history: &HistoryData,
        config: &Config,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let minimal_mode = config.minimal_mode;
        let data_clone = data.clone();
        let history_clone = history.clone();
        
        self.terminal.draw(move |f| {
            if minimal_mode {
                Self::draw_minimal_layout_static(f, &data_clone);
            } else {
                Self::draw_full_layout_static(f, &data_clone, &history_clone);
            }
        })?;
        Ok(())
    }

    fn draw_minimal_layout_static(f: &mut Frame<TermionBackend<termion::raw::RawTerminal<std::io::Stderr>>>, data: &SystemData) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(f.size());

        // CPU and Memory Gauges
        let cpu_gauge = Gauge::default()
            .block(Block::default().title("CPU Usage").borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent(data.cpu_info.average_usage as u16)
            .label(format!("{:.1}%", data.cpu_info.average_usage));
        f.render_widget(cpu_gauge, chunks[0]);

        let memory_gauge = Gauge::default()
            .block(Block::default().title("Memory Usage").borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Green))
            .percent(data.memory_info.usage_percentage)
            .label(format!("{}%", data.memory_info.usage_percentage));
        f.render_widget(memory_gauge, chunks[1]);
    }

    fn draw_full_layout_static(
        f: &mut Frame<TermionBackend<termion::raw::RawTerminal<std::io::Stderr>>>,
        data: &SystemData,
        _history: &HistoryData,
    ) {
        // 全新的4象限布局设计
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Percentage(50), // 上半部分
                Constraint::Percentage(50), // 下半部分
            ].as_ref())
            .split(f.size());

        // 上半部分：CPU和功率信息
        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // CPU详情
                Constraint::Percentage(40), // 实时功率统计
            ].as_ref())
            .split(main_chunks[0]);

        // 下半部分：内存/温度 和 进程/网络
        let bottom_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // 内存和温度
                Constraint::Percentage(50), // 进程和网络
            ].as_ref())
            .split(main_chunks[1]);

        Self::draw_cpu_section(f, top_chunks[0], data);
        Self::draw_power_section(f, top_chunks[1], data);
        Self::draw_memory_temp_section(f, bottom_chunks[0], data);
        Self::draw_process_network_section(f, bottom_chunks[1], data);
    }

    // 新的CPU专区 - 蓝色主题
    fn draw_cpu_section(
        f: &mut Frame<TermionBackend<termion::raw::RawTerminal<std::io::Stderr>>>,
        area: Rect,
        data: &SystemData,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6),  // CPU总览
                Constraint::Min(0),     // 核心详情
            ].as_ref())
            .split(area);

        // CPU Overview Information
        let cpu_overview = format!(
            "🔵 CPU: {} ({})\nAverage Usage: {:.1}%\nE-Cluster: {}% @ {} MHz\nP-Cluster: {}% @ {} MHz",
            data.system_info.cpu_brand,
            data.system_info.cpu_arch,
            data.cpu_info.average_usage,
            data.cpu_info.power_metrics.e_cluster_active,
            data.cpu_info.power_metrics.e_cluster_freq_mhz,
            data.cpu_info.power_metrics.p_cluster_active,
            data.cpu_info.power_metrics.p_cluster_freq_mhz
        );

        let cpu_block = Paragraph::new(cpu_overview)
            .block(Block::default().title("🔵 CPU Information").borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(cpu_block, chunks[0]);

        // CPU Core Details
        let core_details: Vec<String> = data.cpu_info.core_usages
            .iter()
            .enumerate()
            .map(|(i, usage)| {
                let bar_length = (*usage / 100.0 * 20.0) as usize;
                let bar = "█".repeat(bar_length) + &"░".repeat(20 - bar_length);
                format!("Core {:2}: {:5.1}% [{}]", i, usage, bar)
            })
            .collect();

        let cores_block = Paragraph::new(core_details.join("\n"))
            .block(Block::default().title("CPU Cores").borders(Borders::ALL))
            .style(Style::default().fg(Color::Blue));
        f.render_widget(cores_block, chunks[1]);
    }

    // 实时功率统计专区 - 红色主题
    fn draw_power_section(
        f: &mut Frame<TermionBackend<termion::raw::RawTerminal<std::io::Stderr>>>,
        area: Rect,
        data: &SystemData,
    ) {
        // Comprehensive Power & System Analytics
        let total_power = data.cpu_info.power_metrics.package_w;
        let cpu_percent = if total_power > 0.0 { data.cpu_info.power_metrics.cpu_w / total_power * 100.0 } else { 0.0 };
        let _gpu_percent = if total_power > 0.0 { data.cpu_info.power_metrics.gpu_w / total_power * 100.0 } else { 0.0 };
        let _ane_percent = if total_power > 0.0 { data.cpu_info.power_metrics.ane_w / total_power * 100.0 } else { 0.0 };

        // Battery status indicators
        let battery_status = if data.battery_info.is_charging { "⚡ CHARGING" }
                           else if data.battery_info.percentage > 80.0 { "🔋 FULL" }
                           else if data.battery_info.percentage > 50.0 { "🔋 GOOD" }
                           else if data.battery_info.percentage > 20.0 { "🔋 LOW" }
                           else { "🔋 CRITICAL" };

        // Health indicators
        let health_status = if data.battery_info.health_percentage > 90.0 { "💚 EXCELLENT" }
                          else if data.battery_info.health_percentage > 80.0 { "💛 GOOD" }
                          else if data.battery_info.health_percentage > 70.0 { "🧡 FAIR" }
                          else { "❤️ POOR" };

        // Fan status
        let fan_status = if data.thermal_info.fan_speeds.is_empty() { "🔇 SILENT" }
                        else if data.thermal_info.fan_speeds.iter().max().unwrap_or(&0) > &3000 { "🌪️ HIGH" }
                        else if data.thermal_info.fan_speeds.iter().max().unwrap_or(&0) > &1500 { "💨 MEDIUM" }
                        else { "🍃 LOW" };

        // Uptime formatting
        let uptime_days = data.system_health.uptime_seconds / 86400;
        let uptime_hours = (data.system_health.uptime_seconds % 86400) / 3600;
        let uptime_mins = (data.system_health.uptime_seconds % 3600) / 60;

        // Create visual bars
        let battery_bar_len = (data.battery_info.percentage / 100.0 * 10.0) as usize;
        let health_bar_len = (data.battery_info.health_percentage / 100.0 * 10.0) as usize;
        let cpu_bar_len = (cpu_percent / 100.0 * 10.0) as usize;
        
        let battery_bar = "█".repeat(battery_bar_len) + &"░".repeat(10 - battery_bar_len);
        let health_bar = "█".repeat(health_bar_len) + &"░".repeat(10 - health_bar_len);
        let cpu_bar = "█".repeat(cpu_bar_len) + &"░".repeat(10 - cpu_bar_len);

        let power_info = format!(
            "🔴 Comprehensive System Analytics\n\
            ┌─────────────────────────────────────────────┐\n\
            │ 🔋 BATTERY & POWER                          │\n\
            │ Battery: {:5.1}% [{:10}] {:11}    │\n\
            │ Health:  {:5.1}% [{:10}] {:11}    │\n\
            │ Cycles: {:4} | Adapter: {:4.1}W            │\n\
            │ Voltage: {:4.2}V | Current: {:5.2}A        │\n\
            ├─────────────────────────────────────────────┤\n\
            │ ⚡ POWER CONSUMPTION                        │\n\
            │ Total: {:5.2}W | CPU [{:10}] {:4.1}%       │\n\
            │ GPU: {:5.2}W | ANE: {:5.2}W | Eff: {:4.1}   │\n\
            ├─────────────────────────────────────────────┤\n\
            │ 🌡️ THERMAL MANAGEMENT                       │\n\
            │ Fans: {:9} | Throttle: {:5}            │\n\
            │ Pressure: {:3}% | Max RPM: {:4}            │\n\
            ├─────────────────────────────────────────────┤\n\
            │ 📊 PERFORMANCE METRICS                      │\n\
            │ Perf/Watt: {:6.1} | Workload: {:8}       │\n\
            │ Freq Eff: {:7.1} | Load: {:4.2}           │\n\
            ├─────────────────────────────────────────────┤\n\
            │ 🏥 SYSTEM HEALTH                            │\n\
            │ Uptime: {:2}d {:2}h {:2}m | Quality: {:3}%    │\n\
            │ Load: {:4.2} {:4.2} {:4.2} | Efficiency: {:3}% │\n\
            └─────────────────────────────────────────────┘",
            // Battery section
            data.battery_info.percentage, battery_bar, battery_status,
            data.battery_info.health_percentage, health_bar, health_status,
            data.battery_info.cycle_count, data.battery_info.power_adapter_wattage,
            data.battery_info.voltage, data.battery_info.amperage,
            // Power section
            total_power, cpu_bar, cpu_percent,
            data.cpu_info.power_metrics.gpu_w, data.cpu_info.power_metrics.ane_w, 
            data.performance_metrics.performance_per_watt,
            // Thermal section
            fan_status, if data.thermal_info.thermal_throttling { "YES" } else { "NO" },
            data.thermal_info.thermal_pressure,
            data.thermal_info.fan_speeds.iter().max().unwrap_or(&0),
            // Performance section
            data.performance_metrics.performance_per_watt,
            data.performance_metrics.workload_type,
            data.performance_metrics.frequency_efficiency,
            data.system_health.system_load_1min,
            // System health section
            uptime_days, uptime_hours, uptime_mins, data.system_health.power_quality_score,
            data.system_health.system_load_1min, data.system_health.system_load_5min, 
            data.system_health.system_load_15min, data.system_health.sleep_wake_efficiency as u8
        );

        let power_block = Paragraph::new(power_info)
            .block(Block::default().title("🔴 Power Analytics").borders(Borders::ALL))
            .style(Style::default().fg(Color::Red));
        f.render_widget(power_block, area);
    }

    // 内存和温度专区 - 绿色和黄色主题
    fn draw_memory_temp_section(
        f: &mut Frame<TermionBackend<termion::raw::RawTerminal<std::io::Stderr>>>,
        area: Rect,
        data: &SystemData,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50), // 内存
                Constraint::Percentage(50), // 温度
            ].as_ref())
            .split(area);

        // 内存信息 - 绿色主题
        let memory_bar_length = (data.memory_info.usage_percentage as f32 / 100.0 * 30.0) as usize;
        let memory_bar = "█".repeat(memory_bar_length) + &"░".repeat(30 - memory_bar_length);
        
        let memory_info = format!(
            "🟢 Memory: {:.1}GB / {:.1}GB ({}%)\n[{}]\nSwap: {:.1}GB / {:.1}GB",
            data.memory_info.used_memory as f64 / (1024.0 * 1024.0 * 1024.0),
            data.memory_info.total_memory as f64 / (1024.0 * 1024.0 * 1024.0),
            data.memory_info.usage_percentage,
            memory_bar,
            data.memory_info.used_swap as f64 / (1024.0 * 1024.0 * 1024.0),
            data.memory_info.total_swap as f64 / (1024.0 * 1024.0 * 1024.0)
        );

        let memory_block = Paragraph::new(memory_info)
            .block(Block::default().title("🟢 Memory").borders(Borders::ALL))
            .style(Style::default().fg(Color::Green));
        f.render_widget(memory_block, chunks[0]);

        // Temperature Information - Yellow Theme
        let temp_details: Vec<String> = data.temperature_info.iter().map(|temp| {
            let status = if temp.temperature > 80.0 { "🔥 HOT" } 
                        else if temp.temperature > 60.0 { "⚠️ WARM" } 
                        else { "✅ COOL" };
            format!("{}: {:.1}°C {}", temp.label, temp.temperature, status)
        }).collect();

        let temp_info = format!("🟡 Temperature Monitor\n{}", temp_details.join("\n"));

        let temp_block = Paragraph::new(temp_info)
            .block(Block::default().title("🟡 Temperature").borders(Borders::ALL))
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(temp_block, chunks[1]);
    }

    // 进程和网络专区 - 紫色和青色主题
    fn draw_process_network_section(
        f: &mut Frame<TermionBackend<termion::raw::RawTerminal<std::io::Stderr>>>,
        area: Rect,
        data: &SystemData,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(70), // 进程
                Constraint::Percentage(30), // 网络
            ].as_ref())
            .split(area);

        // 进程信息 - 紫色主题
        let mut top_processes = data.process_info.clone();
        top_processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
        top_processes.truncate(8);

        let process_details: Vec<String> = top_processes.iter().map(|process| {
            format!("{:>8} {:>5.1}% {:>6.0}M {}",
                process.pid,
                process.cpu_usage,
                process.memory_usage as f64 / (1024.0 * 1024.0),
                process.name.chars().take(20).collect::<String>()
            )
        }).collect();

        let process_info = format!("🟣 Top Processes\n     PID   CPU%   MEM   NAME\n{}", process_details.join("\n"));

        let process_block = Paragraph::new(process_info)
            .block(Block::default().title("🟣 Processes").borders(Borders::ALL))
            .style(Style::default().fg(Color::Magenta));
        f.render_widget(process_block, chunks[0]);

        // Network Information - Cyan Theme
        let total_rx = data.network_info.iter().map(|n| n.bytes_received).sum::<u64>();
        let total_tx = data.network_info.iter().map(|n| n.bytes_transmitted).sum::<u64>();

        let network_info = format!(
            "🔵 Network Total\n↓ {:.1}GB  ↑ {:.1}GB",
            total_rx as f64 / (1024.0 * 1024.0 * 1024.0),
            total_tx as f64 / (1024.0 * 1024.0 * 1024.0)
        );

        let network_block = Paragraph::new(network_info)
            .block(Block::default().title("🔵 Network").borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(network_block, chunks[1]);
    }

    pub fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.terminal.clear()?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    pub fn next_tab(&mut self) {
        // 新界面不需要tab切换
    }

    pub fn previous_tab(&mut self) {
        // 新界面不需要tab切换
    }
}