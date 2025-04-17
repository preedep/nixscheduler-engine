use chrono::{DateTime, Utc};
use log::{debug, error, info};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep_until, Duration, Instant};
use crate::domain::model::Job;
use crate::job::store::JobStore;
use crate::task::registry::TaskRegistry;

#[derive(Clone)]
pub struct ScheduledJob {
    pub job: Job,
    pub run_at: Instant,
}

impl PartialEq for ScheduledJob {
    fn eq(&self, other: &Self) -> bool {
        self.run_at.eq(&other.run_at)
    }
}
impl Eq for ScheduledJob {}

impl PartialOrd for ScheduledJob {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(other.run_at.cmp(&self.run_at)) // min-heap
    }
}
impl Ord for ScheduledJob {
    fn cmp(&self, other: &Self) -> Ordering {
        other.run_at.cmp(&self.run_at)
    }
}

pub struct Scheduler {
    queue: Arc<Mutex<BinaryHeap<ScheduledJob>>>,
    task_registry: Arc<TaskRegistry>,
    store: Arc<dyn JobStore>,
}

impl Scheduler {
    pub fn new(task_registry: Arc<TaskRegistry>, store: Arc<dyn JobStore>) -> Self {
        Scheduler {
            queue: Arc::new(Mutex::new(BinaryHeap::new())),
            task_registry,
            store,
        }
    }

    pub async fn add_job(&self, job: Job) {
        if let Some(next_time) = job.next_run() {
            let now = Utc::now();
            let dur = (next_time - now).to_std().unwrap_or(Duration::from_secs(1));
            let run_at = Instant::now() + dur;

            let mut queue = self.queue.lock().await;
            queue.push(ScheduledJob { job, run_at });
        }
    }

    pub async fn run(&self) {
        loop {
            let opt_job = {
                let mut queue = self.queue.lock().await;
                queue.pop()
            };

            if let Some(scheduled) = opt_job {
                let now = Instant::now();
                if scheduled.run_at > now {
                    sleep_until(scheduled.run_at).await;
                }
                
                let task_type = scheduled.job.task_type.clone().task_type_name();
                let job = scheduled.job.clone();
                // run task
                let handler_opt = self.task_registry.get(task_type);
                if let Some(handler) = handler_opt {
                    let payload = scheduled.job.payload.clone();
                    let job_name = scheduled.job.name.clone();
                    let job_id = scheduled.job.id.clone();
                    let store = self.store.clone();
                    

                    tokio::spawn(async move {
                        info!("[{}] Execution with Payload: {:?}", job_name, payload);
                        
                        if let Err(err) = handler.handle(&job.task_type).await {
                            error!("[{}] Error: {}", job_name, err);
                        }
                        store.update_last_run(&job_id, Utc::now()).await;
                    });
                }

                // reschedule recurring job
                self.add_job(scheduled.job).await;
            } else {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

