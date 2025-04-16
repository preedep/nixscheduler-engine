use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

pub fn hash_job_id(job_id: &str) -> usize {
    let mut hasher = DefaultHasher::new();
    job_id.hash(&mut hasher);
    hasher.finish() as usize
}