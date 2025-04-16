use std::fmt::Display;
use chrono::{DateTime, Utc};
use cron::Schedule;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum JobStatus {
    Start,
    Scheduled,
    Running,
    Success,
    Failed,
    Disabled,
}


impl Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            JobStatus::Start => "start",
            JobStatus::Scheduled => "scheduled",
            JobStatus::Running => "running",
            JobStatus::Success => "success",
            JobStatus::Failed => "failed",
            JobStatus::Disabled => "disabled",
        }.to_string();
        write!(f, "{}", str)
    }
}

impl std::str::FromStr for JobStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "start" => Ok(JobStatus::Start),
            "scheduled" => Ok(JobStatus::Scheduled),
            "running" => Ok(JobStatus::Running),
            "success" => Ok(JobStatus::Success),
            "failed" => Ok(JobStatus::Failed),
            "disabled" => Ok(JobStatus::Disabled),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pipeline {
    pub id: String,
    pub name: String,
    pub jobs: Vec<Job>, // Job List ใน Pipeline
    pub is_sequential: bool, // true = sequential, false = parallel
}

#[derive(Debug, Clone)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub cron: String,
    pub task_type: String,
    pub payload: String,
    pub last_run: Option<DateTime<Utc>>,
    pub status: JobStatus,
}

impl Job {
    pub fn next_run(&self) -> Option<DateTime<Utc>> {
        let schedule = Schedule::from_str(&self.cron).ok()?;
        schedule.upcoming(Utc).next()
    }
}
