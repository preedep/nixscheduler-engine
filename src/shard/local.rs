use std::sync::Arc;
use crate::config::AppConfig;
use crate::job::Job;
use crate::shard::ShardManager;
use crate::utils::hash_job_id;

pub struct LocalShardManager {
    pub shard_count: usize,
    _config: Arc<AppConfig>
}
impl LocalShardManager {
    pub fn new(config: Arc<AppConfig>) -> Self {
       Self {
            shard_count: 10,
            _config: config
          }
       }
}
#[async_trait::async_trait]
impl ShardManager for LocalShardManager {
    async fn assign_shard(&self, job_id: &str) -> usize {
        hash_job_id(job_id) % self.shard_count
    }

    async fn get_local_jobs(&self, all_jobs: Vec<Job>) -> Vec<Job> {
        all_jobs
    }
}