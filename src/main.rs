use system_alert::{
    cli::{check_exit, check_root},
    system_info::get_system_info,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    check_root().await?;
    let receiver = check_exit().await;
    get_system_info(receiver)
        .await
        .expect("get system info failed");
    Ok(())
}
