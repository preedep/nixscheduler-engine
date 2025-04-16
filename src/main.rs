use crate::engine::engine::JobEngine;
use crate::job::store::{JobStore, SqliteJobStore};
use crate::shard::{DistributedShardManager, LocalShardManager, ShardManager};
use crate::task::registry::TaskRegistry;
use actix_web::main;
use log::{debug, info};
use std::sync::Arc;

mod config;
mod engine;
mod job;
mod scheduler;
mod shard;
mod task;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();
    let app_conf = config::AppConfig::from_env();

    let store = Arc::new(SqliteJobStore::new(&app_conf.database_url).await) as Arc<dyn JobStore>;

    let shard: Arc<dyn ShardManager> = match &app_conf.shard_mode {
        config::ShardMode::Distributed {
            shard_id,
            total_shards,
        } => Arc::new(DistributedShardManager::new(
            *shard_id,
            *total_shards,
            app_conf.clone().into(),
        )),
        config::ShardMode::Local => Arc::new(LocalShardManager::new(app_conf.clone().into())),
    };

    let mut registry = TaskRegistry::new();
    let task_registry = Arc::new(registry);

    let engine = JobEngine::new(
        app_conf.clone().into(),
        store.clone(),
        shard.clone(),
        task_registry.clone(),
    );

    info!("Starting job engine...");
    engine.run().await;

    Ok(())
}
