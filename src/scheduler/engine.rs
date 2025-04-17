use std::sync::Arc;

use crate::config::AppConfig;
use crate::domain::model::Job;
use crate::job::store::JobStore;
use crate::scheduler::wheel::Scheduler;
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

    pub async fn run(&self) {
        // 1. โหลด jobs ทั้งหมด
        let all_jobs = self.store.load_jobs().await;

        // 2. Filter เฉพาะ job ที่ shard นี้รับผิดชอบ
        let local_jobs: Vec<Job> = self.shard.get_local_jobs(all_jobs).await;

        println!(
            "[Engine] Starting on {:?} mode with {} job(s)",
            self.config.shard_mode,
            local_jobs.len()
        );

        // 3. สร้าง Scheduler
        let scheduler = Scheduler::new(self.task_registry.clone(), self.store.clone());

        // 4. Add job เข้า priority queue
        for job in local_jobs {
            scheduler.add_job(job).await;
        }

        // 5. เริ่ม run loop
        scheduler.run().await;
    }
}
