use chrono::{DateTime, Utc};
use cron::Schedule;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub cron: String,
    pub task_type: String,
    pub payload: String,
    pub last_run: Option<DateTime<Utc>>,
}

impl Job {
    pub fn next_run(&self) -> Option<DateTime<Utc>> {
        let schedule = Schedule::from_str(&self.cron).ok()?;
        schedule.upcoming(Utc).next()
    }
}
