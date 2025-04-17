use crate::config::AppConfig;

use crate::domain::model::{Job, JobRaw};
use crate::shard::ShardManager;
use crate::utils::hash_job_id;
use std::sync::Arc;

pub struct LocalShardManager {
    pub shard_count: usize,
    _config: Arc<AppConfig>,
}
impl LocalShardManager {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self {
            shard_count: 10,
            _config: config,
        }
    }
}
#[async_trait::async_trait]
impl ShardManager for LocalShardManager {
    async fn assign_shard(&self, job_id: &str) -> usize {
        hash_job_id(job_id) % self.shard_count
    }

    async fn get_local_jobs(&self, all_jobs: Vec<JobRaw>) -> Vec<Job> {
        all_jobs.iter().map(|c| c.to_job().unwrap()).collect()
    }
}
