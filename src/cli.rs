use clap::{Command, Arg};
use libc::getuid;
use std::process::exit;
use termion::{
    event::{Event, Key},
    input::TermRead,
};

use tokio::{process::Command as tokio_comm, sync::mpsc as tokio_mpsc};

pub fn cli() -> Command {
    Command::new("system-alert")
        .bin_name("sysalert")
        .about("MacOS System Alert")
        .author("Marky-Shi")
        .version("0.1.0")
        .arg(
            Arg::new("refresh")
                .short('r')
                .long("refresh")
                .value_name("SECONDS")
                .help("Set refresh rate in seconds")
                .value_parser(clap::value_parser!(u64))
        )
        .arg(
            Arg::new("minimal")
                .short('m')
                .long("minimal")
                .help("Use minimal display mode")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Specify a custom config file")
        )
}

// this commonod must run as root
pub async fn check_root() -> Result<(), Box<dyn std::error::Error>> {
    if unsafe { getuid() != 0 } {
        eprintln!("The program must be run as root");
        exit(0x100);
    }
    Ok(())
}

pub async fn get_powermetrics_output() -> Result<String, Box<dyn std::error::Error>> {
    let output = tokio_comm::new("powermetrics")
        .arg("-n")
        .arg("1")
        .output()
        .await
        .map_err(|e| format!("Failed to execute powermetrics: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to run powermetrics: {}", stderr).into());
    }
    
    let output_str = String::from_utf8_lossy(&output.stdout).into_owned();
    Ok(output_str)
}

pub async fn check_exit() -> tokio_mpsc::Receiver<bool> {
    let (tx, rx) = tokio_mpsc::channel(1);

    tokio::spawn(async move {
        let stdin = std::io::stdin();

        for event in stdin.events() {
            let c = event.expect("Failed to get stdin event");
            match c {
                Event::Key(Key::Char('q')) => break,
                _ => continue,
            };
        }

        if let Err(e) = tx.send(true).await {
            eprintln!("Failed to send exit signal: {}", e);
        }
    });

    rx
}
