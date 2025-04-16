use actix_web::main;
use log::{debug, info};

mod engine;
mod shard;
mod job;
mod utils;
mod config;
mod task;

#[actix_web::main]
async fn main()  -> std::io::Result<()> {
    pretty_env_logger::init();
    let app_conf = config::AppConfig::from_env();
    debug!(
        "Hello, world! This is a test log message from the NiX Scheduler-Engine with config: {:?}",
        app_conf);
    Ok(())
}
