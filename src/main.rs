use system_alert::{
    cli::{check_root, handle_input, parse_args, InputEvent},
    config::Config,
    data_collector::DataCollector,
    history::HistoryData,
    notification::NotificationManager,
    ui::UI,
    types::*,
};

use log::{error, info, warn};
use std::time::Duration;
use tokio::time::interval;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    info!("Advanced System Monitor starting...");
    
    // Check root privileges (but don't exit if not root, just warn)
    if let Err(e) = check_root().await {
        warn!("Running without root privileges: {}", e);
        warn!("Some features (like powermetrics) may not work properly.");
        warn!("For full functionality, run with: sudo cargo run");
    }
    
    // Parse command line arguments
    let cli_args = parse_args();
    
    // Load configuration
    let mut config = if let Some(config_file) = &cli_args.config_file {
        match Config::load_from_file(config_file) {
            Ok(config) => {
                info!("Loaded configuration from: {}", config_file);
                config
            }
            Err(e) => {
                warn!("Failed to load config file {}: {}. Using defaults.", config_file, e);
                Config::default()
            }
        }
    } else {
        Config::default()
    };
    
    // Apply CLI overrides
    config.merge_with_cli(cli_args.refresh_rate, cli_args.minimal_mode);
    
    // Initialize UI first - immediate startup
    let mut ui = UI::new()?;
    info!("UI initialized - starting data collection in background...");
    
    // Show loading screen immediately
    ui.show_loading_screen()?;
    
    // Initialize other components in background
    let mut data_collector = DataCollector::new_fast();
    let mut history = HistoryData::new(config.display.history_size);
    let mut notification_manager = NotificationManager::new(
        config.notifications.enabled,
        config.notifications.cooldown_seconds,
    );
    
    // Set up input handling
    let mut input_receiver = handle_input().await;
    
    // Set up refresh timer
    let mut refresh_interval = interval(Duration::from_secs(config.refresh_rate));
    
    // Create initial empty data for immediate display
    let mut system_data = create_placeholder_data();
    
    info!("System monitor initialized. Press 'q' to quit, 'n' to toggle notifications, 'r' to force refresh.");
    
    // Main event loop
    loop {
        tokio::select! {
            // Handle input events
            input_event = input_receiver.recv() => {
                match input_event {
                    Some(InputEvent::Quit) => {
                        info!("Quit signal received");
                        break;
                    }
                    Some(InputEvent::NextTab) => {
                        ui.next_tab();
                    }
                    Some(InputEvent::PreviousTab) => {
                        ui.previous_tab();
                    }
                    Some(InputEvent::ToggleNotifications) => {
                        let new_state = !config.notifications.enabled;
                        config.notifications.enabled = new_state;
                        notification_manager.set_enabled(new_state);
                        info!("Notifications {}", if new_state { "enabled" } else { "disabled" });
                    }
                    Some(InputEvent::Refresh) => {
                        // Force immediate refresh by continuing to the refresh logic
                    }
                    None => {
                        warn!("Input channel closed");
                        break;
                    }
                }
            }
            
            // Handle periodic refresh
            _ = refresh_interval.tick() => {
                // Collect system data asynchronously
                match data_collector.collect_all_data().await {
                    Ok(new_data) => {
                        system_data = new_data;
                        // Update history
                        history.update_from_system_data(&system_data);
                        
                        // Check for notifications
                        if let Err(e) = notification_manager
                            .check_and_send_notifications(&system_data, &config.thresholds)
                            .await
                        {
                            error!("Notification error: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Data collection error: {}", e);
                        // Keep using previous data, don't crash
                    }
                }
                
                // Always update UI (even with old data)
                if let Err(e) = ui.draw(&system_data, &history, &config) {
                    error!("UI draw error: {}", e);
                }
            }
        }
    }
    
    // Cleanup
    ui.cleanup()?;
    info!("System monitor exited normally");
    
    Ok(())
}

fn create_placeholder_data() -> SystemData {
    use std::time::Instant;
    
    SystemData {
        system_info: SystemInfo {
            name: "macOS".to_string(),
            kernel_version: "Loading...".to_string(),
            os_version: "Loading...".to_string(),
            host_name: "Loading...".to_string(),
            cpu_arch: "arm64".to_string(),
            cpu_brand: "Apple Silicon".to_string(),
        },
        cpu_info: CpuInfo {
            core_usages: vec![0.0; 8], // 8 cores placeholder
            average_usage: 0.0,
            power_metrics: CPUMetrics::default(),
        },
        memory_info: MemoryInfo {
            total_memory: 0,
            used_memory: 0,
            available_memory: 0,
            total_swap: 0,
            used_swap: 0,
            usage_percentage: 0,
        },
        network_info: vec![],
        temperature_info: vec![],
        process_info: vec![],
        battery_info: BatteryInfo::default(),
        thermal_info: ThermalInfo::default(),
        performance_metrics: PerformanceMetrics::default(),
        system_health: SystemHealthInfo::default(),
        timestamp: Instant::now(),
    }
}
