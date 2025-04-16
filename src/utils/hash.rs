use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use twox_hash::XxHash64;
pub fn hash_job_id(job_id: &str) -> usize {
    let mut hasher = DefaultHasher::new();
    job_id.hash(&mut hasher);
    hasher.finish() as usize
}
pub fn stable_hash_job_id(job_id: &str) -> usize {
    use std::hash::{Hash, Hasher};
    let mut hasher = XxHash64::default();
    job_id.hash(&mut hasher);
    hasher.finish() as usize
}