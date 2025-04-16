use log::info;

mod engine;
mod shard;
mod job;
mod utils;

fn main() {
    pretty_env_logger::init();
    info!(
        "Hello, world! This is a test log message from the NiX Scheduler-Engine")
}
