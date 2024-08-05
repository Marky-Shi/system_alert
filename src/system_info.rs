use crate::{cli::get_powermetrics_output, types::CPUMetrics};
use std::{io, str, thread, time::Duration};
use sysinfo::{Components, Networks, System};
use termion::{color as tcolor, raw::IntoRawMode};
use tui::text::Text;
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, Paragraph},
    Terminal,
};

use regex::Regex;
use tokio::sync::mpsc as tokio_mpsc;

pub async fn get_system_info(
    mut receiver: tokio_mpsc::Receiver<bool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut sys = System::new_all();

    loop {
        // Update all information of our `System` struct.
        sys.refresh_all();

        // Calculate memory usage percentage
        let total_memory = sys.total_memory();
        let used_memory = sys.used_memory();
        let memory_percentage = (used_memory as f64 / total_memory as f64 * 100.0) as u16;

        // Get CPU usage
        let cpu_usages = sys
            .cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage())
            .collect::<Vec<_>>();

        let disk_info:Vec<String> = sys.processes().iter().map(|(pid, processor)| {
            format!("pid:{} cpu usage:{}% read_bytes:{} read_bytes_total: {} write_bytes:{} write_bytes_total:{}\n",
            pid,
            processor.cpu_usage(),
            processor.disk_usage().read_bytes,
            processor.disk_usage().total_read_bytes,
            processor.disk_usage().written_bytes,
            processor.disk_usage().total_written_bytes,
           )
        }).collect();

        let cpu_power = get_powermetrics_output().await;
        let s = parse_cpu_metrics(cpu_power, sys.cpus()[0].brand())
            .await
            .expect("parse cpu metrics failed");
        let cpu_power = format!(
            "{}e_cluster_active: {}\np_cluster_active: {}\ne_cluster_freq_mhz: {}\np_cluster_freq_mhz: {}\ncpu_w: {}\ngpu_w: {}\nane_w: {}\npackage_w: {}\n",
            tcolor::Fg(tcolor::Yellow),
            s.e_cluster_active,
            s.p_cluster_active,
            s.e_cluster_freq_mhz,
            s.p_cluster_freq_mhz,
            s.cpu_w,
            s.gpu_w,
            s.ane_w,
            s.package_w,
        );

        // Get network information
        let networks = Networks::new_with_refreshed_list();
        let network_info: Vec<String> = networks
            .iter()
            .map(|(interface_name, data)| {
                format!(
                    "{}{}: {} B (down) / {} B (up)",
                    tcolor::Fg(tcolor::Cyan),
                    interface_name,
                    data.total_received(),
                    data.total_transmitted()
                )
            })
            .collect();

        // Get component temperatures
        let components = Components::new_with_refreshed_list();
        let component_temps: Vec<String> = components
            .iter()
            .map(|component| {
                format!(
                    "{}{}: {:.2}°C",
                    tcolor::Fg(tcolor::LightGreen),
                    component.label(),
                    component.temperature()
                )
            })
            .collect();

        terminal.draw(|f| {
            // Create a layout with two vertical chunks: one for the top half and one for the bottom half
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(60), // Top half
                        Constraint::Percentage(40), // Bottom half
                    ]
                    .as_ref(),
                )
                .split(f.size());

            // Create a layout for the top chunk with three horizontal chunks
            let top_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(20), // Left third
                        Constraint::Percentage(20), // middle right 
                        Constraint::Percentage(40),
                        Constraint::Percentage(20) // Right third
                    ]
                    .as_ref(),
                )
                .split(chunks[0]);

            // Create a layout for the bottom chunk with two horizontal chunks
            let bottom_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(80),// LEFT
                        Constraint::Percentage(20), // RIGHT
                         // Right half
                    ]
                    .as_ref(),
                )
                .split(chunks[1]);
            let os_info  = format!(
                "System name:             {:?}\nSystem kernel version:   {:?}\nSystem OS version:       {:?}\nSystem host name:        {:?}\nSystem CPU architecture: {:?}\nSystem CPU Name:        {:?}",
                System::name().unwrap(),
                System::kernel_version().unwrap(),
                System::os_version().unwrap(),
                System::host_name().unwrap(),
                System::cpu_arch().unwrap(),
                sys.cpus()[0].brand(),
            );
            let cpu_info = cpu_usages.iter().enumerate().map(|(i, usage)| {
                format!("{}CPU {}: {:.2}%",tcolor::Fg(tcolor::Red), i, usage)
            }).collect::<Vec<_>>().join("\n");
            // Create a block for OSINFO
            let os_info_block = Block::default().title("OSINFO").borders(Borders::ALL);
            let os_info = Paragraph::new(Text::raw(
                os_info + cpu_info.as_str()
            ))
                .block(os_info_block);
            f.render_widget(os_info, top_chunks[0]);

            // Create a block for Component Temperatures
            let component_info_block = Block::default().title("Component Temperatures").borders(Borders::ALL);
            let component_info_paragraph = Paragraph::new(Text::raw(component_temps.join("\n")))
                .block(component_info_block);
            f.render_widget(component_info_paragraph, top_chunks[1]);

            // Create a block for DiskINFO
            let disk_info_block = Block::default().title("DiskINFO && process INFO").borders(Borders::ALL);
            let disk_info_paragraph = Paragraph::new(Text::raw(disk_info.join("\n")))
                .block(disk_info_block);
            f.render_widget(disk_info_paragraph, top_chunks[2]);

            // Create a block for NetworkINFO
            let network_info_block = Block::default().title("NetworkINFO").borders(Borders::ALL);
            let network_info_paragraph = Paragraph::new(Text::raw(network_info.join("\n")))
                .block(network_info_block);
            f.render_widget(network_info_paragraph, top_chunks[3]);

            // Create a block for MemoryINFO
            let memory_info_block = Block::default().title("MemoryINFO").borders(Borders::ALL);
            let mem_info = format!("Total memory: {} GB\nUsed memory: {} GB\nTotal swap: {} bytes\nUsed swap: {} bytes\nMemory usage: {:.2}%",
                total_memory / (1024 * 1024 * 1024),
                used_memory / (1024 * 1024 * 1024),
                sys.total_swap(),
                sys.used_swap(),
                memory_percentage
            );
            let memory_info = Paragraph::new(Text::raw(mem_info))
                .block(memory_info_block);
            f.render_widget(memory_info, bottom_chunks[0]);

            // Create a block for Memory usage gauge
            let title = format!("Total Memory {} GB  Used Memory {} GB  Memory usage: {:.2}%",total_memory/(1024*1024*1024),used_memory/(1024*1024*1024), memory_percentage);
            let memory_gauge_block = Block::default().title(title).borders(Borders::ALL);
            let memory_gauge = Gauge::default()
                .block(memory_gauge_block)
                .gauge_style(Style::default().fg(Color::Yellow).bg(Color::Black))
                .percent(memory_percentage)
                .label(format!("{}%", memory_percentage));
            f.render_widget(memory_gauge, bottom_chunks[0]); // Adjust to be within the MemoryINFO block


            // Create a block for Process info
            let process_info_block = Block::default().title("CPU Power").borders(Borders::ALL);
            let process_info_paragraph = Paragraph::new(Text::raw(cpu_power))
                .block(process_info_block);
            f.render_widget(process_info_paragraph, bottom_chunks[1]);
        })?;

        // Sleep for a short period before refreshing
        thread::sleep(Duration::from_secs(1));

        if receiver.try_recv().is_ok() {
            break;
        }
    }

    terminal.show_cursor()?;
    Ok(())
}

async fn parse_cpu_metrics(
    powermetrics_output: String,
    model_name: &str,
) -> Result<CPUMetrics, Box<dyn std::error::Error>> {
    let lines: Vec<&str> = powermetrics_output.lines().collect();
    let mut cpu_metrics = CPUMetrics::default();

    let mut e_cluster_active_sum = 0.0;
    let mut p_cluster_active_sum = 0.0;
    let mut e_cluster_freq_sum = 0.0;
    let mut p_cluster_freq_sum = 0.0;
    let mut e_cluster_count = 0;
    let mut p_cluster_count = 0;

    let max_cores = if model_name == "Apple M3 Max" { 15 } else { 11 };

    for i in 0..=max_cores {
        let active_re = Regex::new(&format!(r"CPU {} active residency:\s+(\d+\.\d+)%", i)).unwrap();
        let freq_re = Regex::new(&format!(r"^CPU\s+{}\s+frequency:\s+(\d+)\s+MHz$", i)).unwrap();

        for line in &lines {
            if let Some(caps) = active_re.captures(line) {
                let active_residency: f64 = caps[1].parse().unwrap();
                if i <= 3 {
                    e_cluster_active_sum += active_residency;
                    e_cluster_count += 1;
                } else {
                    p_cluster_active_sum += active_residency;
                    p_cluster_count += 1;
                }
            }

            if let Some(caps) = freq_re.captures(line) {
                let active_freq: f64 = caps[1].parse().unwrap();
                if i <= 3 {
                    e_cluster_freq_sum += active_freq;
                    e_cluster_count += 1;
                } else {
                    p_cluster_freq_sum += active_freq;
                    p_cluster_count += 1;
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

    for line in lines {
        if line.contains("ANE Power") {
            let fields: Vec<&str> = line.split_whitespace().collect();
            cpu_metrics.ane_w = fields[2].trim_end_matches("mW").parse::<f64>().unwrap() / 1000.0;
        } else if line.contains("CPU Power") {
            let fields: Vec<&str> = line.split_whitespace().collect();
            cpu_metrics.cpu_w = fields[2].trim_end_matches("mW").parse::<f64>().unwrap() / 1000.0;
        } else if line.contains("GPU Power") {
            let fields: Vec<&str> = line.split_whitespace().collect();
            cpu_metrics.gpu_w = fields[2].trim_end_matches("mW").parse::<f64>().unwrap() / 1000.0;
        } else if line.contains("Combined Power (CPU + GPU + ANE)") {
            let fields: Vec<&str> = line.split_whitespace().collect();
            cpu_metrics.package_w =
                fields[7].trim_end_matches("mW").parse::<f64>().unwrap() / 1000.0;
        }
    }

    Ok(cpu_metrics)
}
