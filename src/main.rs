use log::info;

mod engine;
mod shard;
mod job;
mod utils;
mod config;

fn main() {
    pretty_env_logger::init();
    let app_conf = config::AppConfig::from_env();

    info!(
        "Hello, world! This is a test log message from the NiX Scheduler-Engine")
}
