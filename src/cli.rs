use clap::Command;
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
        .author("marky")
        .version("0.1.0")
}

// this commonod must run as root
pub async fn check_root() -> Result<(), Box<dyn std::error::Error>> {
    if unsafe { getuid() != 0 } {
        eprintln!("The program must be run as root");
        exit(0x100);
    }
    Ok(())
}

pub async fn get_powermetrics_output() -> String {
    let output = tokio_comm::new("powermetrics")
        .arg("-n")
        .arg("1")
        .output()
        .await
        .expect("Failed to execute powermetrics");

    if !output.status.success() {
        panic!("Failed to run powermetrics");
    }
    let output = String::from_utf8_lossy(&output.stdout).into_owned();
    output
}

pub async fn check_exit() -> tokio_mpsc::Receiver<bool> {
    let (tx, rx) = tokio_mpsc::channel(1);

    tokio::spawn(async move {
        let stdin = std::io::stdin();

        for event in stdin.events() {
            let c = event.expect("get stdin event failed");
            match c {
                Event::Key(Key::Char('q')) => break,
                _ => continue,
            };
        }

        tx.send(true).await.unwrap();
    });

    rx
}
