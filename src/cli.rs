use clap::{Arg, Command};
use libc::getuid;
use termion::{
    event::{Event, Key},
    input::TermRead,
};
use tokio::{process::Command as tokio_comm, sync::mpsc as tokio_mpsc, time::{timeout, Duration}};

#[derive(Debug)]
pub struct CliArgs {
    pub refresh_rate: Option<u64>,
    pub minimal_mode: bool,
    pub config_file: Option<String>,
}

pub fn parse_args() -> CliArgs {
    let matches = Command::new("system-alert")
        .bin_name("sysalert")
        .about("Advanced macOS System Monitor with Apple Silicon optimizations")
        .author("Marky-Shi")
        .version("0.1.0")
        .arg(
            Arg::new("refresh")
                .short('r')
                .long("refresh")
                .value_name("SECONDS")
                .help("Set refresh rate in seconds (default: 1)")
                .value_parser(clap::value_parser!(u64)),
        )
        .arg(
            Arg::new("minimal")
                .short('m')
                .long("minimal")
                .help("Use minimal display mode")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Specify a custom configuration file"),
        )
        .get_matches();

    CliArgs {
        refresh_rate: matches.get_one::<u64>("refresh").copied(),
        minimal_mode: matches.get_flag("minimal"),
        config_file: matches.get_one::<String>("config").cloned(),
    }
}

pub async fn check_root() -> Result<(), Box<dyn std::error::Error>> {
    if unsafe { getuid() != 0 } {
        return Err("This program requires root privileges to access system metrics.".into());
    }
    Ok(())
}

pub async fn get_powermetrics_output() -> Result<String, Box<dyn std::error::Error>> {
    let powermetrics_future = tokio_comm::new("powermetrics")
        .arg("-n")
        .arg("1")
        .arg("--samplers")
        .arg("cpu_power,gpu_power")
        .output();
    
    // Add 5 second timeout to prevent infinite waiting - this fixes potential deadlock
    match timeout(Duration::from_secs(5), powermetrics_future).await {
        Ok(Ok(output)) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to run powermetrics: {}", stderr).into());
            }
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        }
        Ok(Err(e)) => Err(format!("Failed to execute powermetrics: {}", e).into()),
        Err(_) => Err("Powermetrics command timed out after 5 seconds - this may indicate a system issue".into()),
    }
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    Quit,
    NextTab,
    PreviousTab,
    ToggleNotifications,
    Refresh,
}

pub async fn handle_input() -> tokio_mpsc::Receiver<InputEvent> {
    let (tx, rx) = tokio_mpsc::channel(32);

    tokio::spawn(async move {
        let stdin = std::io::stdin();

        for event in stdin.events() {
            if let Ok(event) = event {
                let input_event = match event {
                    Event::Key(Key::Char('q')) | Event::Key(Key::Ctrl('c')) => Some(InputEvent::Quit),
                    Event::Key(Key::Right) | Event::Key(Key::Char('\t')) => Some(InputEvent::NextTab),
                    Event::Key(Key::Left) | Event::Key(Key::BackTab) => Some(InputEvent::PreviousTab),
                    Event::Key(Key::Char('n')) => Some(InputEvent::ToggleNotifications),
                    Event::Key(Key::Char('r')) => Some(InputEvent::Refresh),
                    _ => None,
                };

                if let Some(event) = input_event {
                    let should_quit = matches!(event, InputEvent::Quit);
                    if tx.send(event).await.is_err() {
                        break;
                    }
                    
                    // Exit on quit
                    if should_quit {
                        break;
                    }
                }
            }
        }
    });

    rx
}