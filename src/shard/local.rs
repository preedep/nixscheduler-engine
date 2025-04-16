use crate::job::Job;
use crate::shard::ShardManager;
use crate::utils::hash_job_id;

pub struct LocalShardManager {
    pub shard_count: usize,
}

#[async_trait::async_trait]
impl ShardManager for LocalShardManager {
    async fn assign_shard(&self, job_id: &str) -> usize {
        hash_job_id(job_id) % self.shard_count
    }

    async fn get_local_jobs(&self, all_jobs: Vec<Job>) -> Vec<Job> {
        all_jobs // รับหมดทุก job
    }
}