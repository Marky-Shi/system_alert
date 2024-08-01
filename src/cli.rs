use clap::Command;
use libc::getuid;
use std::process::exit;

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
