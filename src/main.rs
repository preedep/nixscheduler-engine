use crate::api::job_routes;
use crate::engine::engine::JobEngine;
use crate::job::store::{JobStore, SqliteJobStore};
use crate::shard::{DistributedShardManager, LocalShardManager, ShardManager};
use crate::task::registry::TaskRegistry;
use actix_files::Files;
use actix_web::{App, HttpServer, main, web};
use log::{debug, info};
use std::sync::Arc;
use crate::auth::auth_routes;

mod api;
mod azure;
mod config;
mod domain;
mod engine;
mod job;
mod scheduler;
mod shard;
mod task;
mod utils;
mod auth;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();
    let app_conf = config::AppConfig::from_env();

    //let store = Arc::new(SqliteJobStore::new(&app_conf.database_url).await) as Arc<dyn JobStore>;
    let store = Arc::new(SqliteJobStore::new(&app_conf.database_url).await);

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
    registry.register(crate::task::print::PrintTask);
    registry.register(crate::task::adf::AdfTask);
    let task_registry = Arc::new(registry);

    let engine = Arc::new(JobEngine::new(
        app_conf.clone().into(),
        store.clone(),
        shard.clone(),
        task_registry.clone(),
    ));

    let engine_clone = engine.clone();
    tokio::task::spawn(async move {
        info!("Starting job engine...");
        engine_clone.run().await;
    });

    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .app_data(web::Data::new(store.clone()))
            .app_data(web::Data::new(engine.clone()))
            .service(web::scope("/api").service(job_routes()))
            .service(auth_routes())
            .service(Files::new("/", "./statics").index_file("index.html"))
    })
    .bind(("0.0.0.0", 8888))?
    .run()
    .await
}
