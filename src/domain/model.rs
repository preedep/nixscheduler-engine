use crate::domain::task_payload::TaskPayload;
use chrono::{DateTime, Utc};
use cron::Schedule;
use log::debug;
use std::fmt::Display;
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
        }
        .to_string();
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
pub struct Job {
    pub id: String,
    pub name: String,
    pub cron: String,
    pub task_type: TaskPayload,
    pub payload: String,
    pub last_run: Option<DateTime<Utc>>,
    pub status: JobStatus,
    pub message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct JobRaw {
    pub id: String,
    pub name: String,
    pub cron: String,
    pub task_type: String,
    pub payload: String,
    pub last_run: Option<DateTime<Utc>>,
    pub status: JobStatus,
    pub message: Option<String>,
}

impl Job {
    pub fn next_run(&self) -> Option<DateTime<Utc>> {
        let schedule = Schedule::from_str(&self.cron).ok()?;
        schedule.upcoming(Utc).next()
    }
}

impl JobRaw {
    pub fn to_job(&self) -> Result<Job, Box<dyn std::error::Error>> {
        let task_json = format!(
            r#"{{ "task_type": "{}", "payload": {} }}"#,
            self.task_type, self.payload
        );
        debug!("Task JSON: {}", task_json);

        let task: TaskPayload = serde_json::from_str(&task_json)?;

        Ok(Job {
            id: self.id.clone(),
            name: self.name.clone(),
            cron: self.cron.clone(),
            task_type: task,
            last_run: self.last_run,
            status: self.status.clone(),
            payload: "".to_string(),
            message: self.message.clone(),
        })
    }
}
