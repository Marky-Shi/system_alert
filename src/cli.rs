use clap::Command;
use libc::getuid;
use std::process::exit;
use std::process::Command as stdcomm;
use std::sync::mpsc;
use std::thread;
use termion::{
    event::{Event, Key},
    input::TermRead,
};

pub fn cli() -> Command {
    Command::new("system-alert")
        .bin_name("sysalert")
        .about("MacOS System Alert")
        .author("Scc")
        .version("0.1.0")
}

// this commonod must run as root
pub fn check_root() {
    if unsafe { getuid() != 0 } {
        eprintln!("The program must be run as root");
        exit(0x100);
    }
}

pub fn get_disk_info() -> String {
    let output = stdcomm::new("diskutil")
        .arg("list")
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8_lossy(&output.stdout);

    format!("\n{}\n", stdout)
}

pub fn get_powermetrics_output() -> String {
    let output = stdcomm::new("powermetrics")
        .arg("-n")
        .arg("1")
        .output()
        .expect("Failed to execute powermetrics");

    if !output.status.success() {
        panic!("Failed to run powermetrics");
    }

    String::from_utf8_lossy(&output.stdout).into_owned()
}

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
