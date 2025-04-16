use std::sync::Arc;
use chrono::Utc;
use log::{error, info};
use tokio::time::{sleep, Duration};


use crate::job::store::JobStore;
use crate::shard::ShardManager;

use crate::config::AppConfig;
use crate::task::registry::TaskRegistry;

pub struct JobEngine {
    config: Arc<AppConfig>,
    store: Arc<dyn JobStore>,
    shard: Arc<dyn ShardManager>,
    task_registry: Arc<TaskRegistry>,
}

impl JobEngine {
    pub fn new(
        config: Arc<AppConfig>,
        store: Arc<dyn JobStore>,
        shard: Arc<dyn ShardManager>,
        task_registry: Arc<TaskRegistry>,
    ) -> Self {
        Self {
            config,
            store,
            shard,
            task_registry,
        }
    }

    pub async fn run(&self) {
        let my_jobs = self.shard.get_local_jobs(self.store.load_jobs().await).await;
        info!("Running scheduler with {} local jobs", my_jobs.len());

        for job in my_jobs {
            let task_registry = self.task_registry.clone();
            let store = self.store.clone();

            tokio::spawn(async move {
                while let Some(next_time) = job.next_run() {
                    let dur = (next_time - Utc::now()).to_std().unwrap_or(Duration::from_secs(1));
                    sleep(dur).await;

                    if let Some(handler) = task_registry.get(&job.task_type) {
                        info!("[{}] Executing task", job.name);
                        if let Err(e) = handler.handle(&job.payload).await {
                            error!("[{}] Task error: {}", job.name, e);
                        }
                    } else {
                        error!("[{}] No handler for task type '{}'", job.name, job.task_type);
                    }

                    store.update_last_run(&job.id, Utc::now()).await;
                }
                info!("[{}] Invalid cron expression", job.name);
            });
        }

        loop {
            sleep(Duration::from_secs(3600)).await;
        }
    }
}