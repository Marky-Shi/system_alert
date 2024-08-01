use system_alert::system_info::{check_exit, get_system_info};

fn main() {
    let receiver = check_exit();
    if let Err(err) = get_system_info(receiver) {
        eprintln!("Error: {}", err);
    }
}
