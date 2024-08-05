use system_alert::{
    cli::{check_exit, check_root},
    system_info::get_system_info,
};

fn main() {
    check_root();

    let receiver = check_exit();
    if let Err(err) = get_system_info(receiver) {
        eprintln!("Error: {}", err);
    }
}
