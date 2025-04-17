use chrono::Utc;
use log::{debug, error, info};
use std::sync::Arc;
use tokio::time::{Duration, sleep};

use crate::config::AppConfig;
use crate::domain::model::{Job, JobStatus};
use crate::job::store::JobStore;
use crate::shard::ShardManager;
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

    pub async fn reload_job_by_id(&self, job_id: &str) {
        debug!("Reloading job by ID: {}", job_id);

        if let Some(job) = self
            .store
            .load_jobs()
            .await
            .into_iter()
            .find(|job| job.id == job_id)
        {
            debug!("Found job with ID: {}", job_id);
            self.schedule(job.to_job().unwrap()).await;
        }
    }

    pub async fn schedule(&self, job: Job) {
        let task_registry = self.task_registry.clone();
        task_registry.print_all_handlers();
        let store = self.store.clone();
        store.update_status(&job.id, JobStatus::Scheduled).await;
        tokio::spawn(async move {
            while let Some(next_time) = job.next_run() {
                let dur = (next_time - Utc::now())
                    .to_std()
                    .unwrap_or(Duration::from_secs(1));
                sleep(dur).await;
                let task_type = job.task_type.clone();
                
                match task_registry.get(task_type.task_type_name()) {
                    Some(handler) => {
                        store.update_status(&job.id, JobStatus::Start).await;
                        info!("[{}] Executing task", job.name);
                        store.update_status(&job.id, JobStatus::Running).await;
                        if let Err(e) = handler.handle(&task_type).await {
                            error!("[{}] Task error: {}", job.name, e);
                            store.update_status(&job.id, JobStatus::Failed).await;
                        }
                    }
                    None => error!("[{}] No handler for task type '{}'", job.name, task_type),
                }
                store.update_status(&job.id, JobStatus::Success).await;
                store.update_last_run(&job.id, Utc::now()).await;
            }
            info!("[{}] Invalid cron expression", job.name);
        });
    }

    pub async fn run(&self) {
        let my_jobs = self
            .shard
            .get_local_jobs(self.store.load_jobs().await)
            .await;
        info!("Running scheduler with {} local jobs", my_jobs.len());

        for job in my_jobs {
            self.schedule(job).await;
        }

        loop {
            sleep(Duration::from_secs(3600)).await;
        }
    }
}
