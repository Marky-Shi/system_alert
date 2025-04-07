use system_alert::{
    cli::{check_exit, check_root},
    system_info::get_system_info,
};

use log::{info, error};
use env_logger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    info!("System monitor starting");
    check_root().await?;
    let receiver = check_exit().await;
    
    match get_system_info(receiver).await {
        Ok(_) => info!("System monitor exited normally"),
        Err(e) => error!("System monitor error: {}", e),
    }
    
    Ok(())
}
