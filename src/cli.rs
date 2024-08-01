use clap::Command;
use libc::getuid;
use std::process::exit;
use std::process::Command as stdcomm;

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
