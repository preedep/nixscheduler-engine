mod distributed;
mod local;



pub use distributed::DistributedShardManager;
pub use local::LocalShardManager;
use crate::domain::model::{Job, JobRaw};

#[async_trait::async_trait]
pub trait ShardManager: Send + Sync {
    async fn assign_shard(&self, job_id: &str) -> usize;
    async fn get_local_jobs(&self, all_jobs: Vec<JobRaw>) -> Vec<Job>; // only jobs this node owns
}
