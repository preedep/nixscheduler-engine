use log::{debug, info};

mod engine;
mod shard;
mod job;
mod utils;
mod config;

fn main() {
    pretty_env_logger::init();
    let app_conf = config::AppConfig::from_env();
    debug!(
        "Hello, world! This is a test log message from the NiX Scheduler-Engine with config: {:?}",
        app_conf);

}
