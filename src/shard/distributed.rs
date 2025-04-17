use crate::config::AppConfig;

use crate::domain::model::{Job, JobRaw};
use crate::shard::ShardManager;
use crate::utils::hash_job_id;
use std::sync::Arc;

pub struct DistributedShardManager {
    pub shard_id: usize,
    pub total_shards: usize,
    _config: Arc<AppConfig>,
}

impl DistributedShardManager {
    pub fn new(shard_id: usize, total_shards: usize, config: Arc<AppConfig>) -> Self {
        Self {
            shard_id,
            total_shards,
            _config: config,
        }
    }
}
#[async_trait::async_trait]
impl ShardManager for DistributedShardManager {
    async fn assign_shard(&self, job_id: &str) -> usize {
        hash_job_id(job_id) % self.total_shards
    }

    async fn get_local_jobs(&self, all_jobs: Vec<JobRaw>) -> Vec<Job> {
        let mut local_jobs = Vec::new();

        for job in all_jobs {
            // Use an asynchronous call to filter jobs
            if self.assign_shard(&job.id).await == self.shard_id {
                local_jobs.push(job.to_job().unwrap());
            }
        }

        local_jobs
    }
}
