use crate::job::Job;

#[async_trait::async_trait]
pub trait ShardManager: Send + Sync {
    async fn assign_shard(&self, job_id: &str) -> usize;
    async fn get_local_jobs(&self, all_jobs: Vec<Job>) -> Vec<Job>; // only jobs this node owns
}