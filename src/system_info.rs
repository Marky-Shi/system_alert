use std::{io, sync::mpsc, thread, time::Duration};
use sysinfo::{Components, Disks, Networks, System};
use termion::{
    color as tcolor,
    event::{Event, Key},
    input::TermRead,
    raw::IntoRawMode,
};
use tui::text::Text;
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, Paragraph},
    Terminal,
};

pub fn check_exit() -> mpsc::Receiver<bool> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let stdin = std::io::stdin();

        for event in stdin.events() {
            let c = event.expect("get stdin event failed");
            match c {
                Event::Key(Key::Char('q')) => break,
                _ => continue,
            };
        }

        tx.send(true).unwrap();
    });

    rx
}

pub fn get_system_info(receiver: mpsc::Receiver<bool>) -> io::Result<()> {
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

        // Get disk information
        let disks = Disks::new_with_refreshed_list();
        let disk_info: Vec<String> = disks
            .iter()
            .map(|disk| {
                format!(
                    "{}{}{}: {}/{} B",
                    tcolor::Fg(tcolor::Green),
                    tcolor::Bg(tcolor::LightMagenta),
                    disk.name().to_string_lossy(),
                    disk.available_space(),
                    disk.total_space()
                )
            })
            .collect();

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
                    "{}{}: {}°C",
                    tcolor::Fg(tcolor::LightGreen),
                    component.label(),
                    component.temperature()
                )
            })
            .collect();

        // Get processes information
        let process_info: Vec<String> = sys
            .processes()
            .iter()
            .map(|(pid, process)| {
                format!(
                    "{}[{}] {:?} {:?}",
                    tcolor::Fg(tcolor::Yellow),
                    pid,
                    process.name(),
                    process.disk_usage()
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
                        Constraint::Percentage(33), // Left third
                        Constraint::Percentage(33), // Middle third
                        Constraint::Percentage(34), // Right third
                    ]
                    .as_ref(),
                )
                .split(chunks[0]);

            // Create a layout for the bottom chunk with two horizontal chunks
            let bottom_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(50), // Left half
                        Constraint::Percentage(50), // Right half
                    ]
                    .as_ref(),
                )
                .split(chunks[1]);

            // Create a block for CPUINFO
            let cpu_info_block = Block::default().title("CPUINFO").borders(Borders::ALL);
            let cpu_info = Paragraph::new(Text::raw(cpu_usages.iter().enumerate().map(|(i, usage)| {
                format!("{}CPU {}: {:.2}%",tcolor::Fg(tcolor::Red), i, usage)
            }).collect::<Vec<_>>().join("\n")))
                .block(cpu_info_block);
            f.render_widget(cpu_info, top_chunks[0]);

            // Create a block for Component Temperatures
            let component_info_block = Block::default().title("Component Temperatures").borders(Borders::ALL);
            let component_info_paragraph = Paragraph::new(Text::raw(component_temps.join("\n")))
                .block(component_info_block);
            f.render_widget(component_info_paragraph, top_chunks[1]);

            // Create a block for DiskINFO
            let disk_info_block = Block::default().title("DiskINFO").borders(Borders::ALL);
            let disk_info_paragraph = Paragraph::new(Text::raw(disk_info.join("\n")))
                .block(disk_info_block);
            f.render_widget(disk_info_paragraph, top_chunks[2]);

            // Create a block for NetworkINFO
            let network_info_block = Block::default().title("NetworkINFO").borders(Borders::ALL);
            let network_info_paragraph = Paragraph::new(Text::raw(network_info.join("\n")))
                .block(network_info_block);
            f.render_widget(network_info_paragraph, top_chunks[2]);

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

            // Create a block for OSINFO
            let os_info_block = Block::default().title("OSINFO").borders(Borders::ALL);
            let os_info = Paragraph::new(Text::raw(format!(
                "System name:             {:?}\nSystem kernel version:   {:?}\nSystem OS version:       {:?}\nSystem host name:        {:?}\nSystem CPU architecture: {:?}",
                System::name().unwrap(),
                System::kernel_version().unwrap(),
                System::os_version().unwrap(),
                System::host_name().unwrap(),
                System::cpu_arch().unwrap()
            )))
                .block(os_info_block);
            f.render_widget(os_info, bottom_chunks[0]);

            // Create a block for Process info
            let process_info_block = Block::default().title("Process Info").borders(Borders::ALL);
            let process_info_paragraph = Paragraph::new(Text::raw(process_info.join("\n")))
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
