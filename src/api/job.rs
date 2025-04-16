use actix_web::{get, post, delete, put, web, HttpResponse, Responder, Scope, ResponseError};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::Utc;
use thiserror::Error;
use crate::job::Job;
use crate::job::store::{JobStore, SqliteJobStore};

#[derive(Debug, Deserialize)]
pub struct JobRequest {
    pub id: String,
    pub name: String,
    pub cron: String,
    pub task_type: String,
    pub payload: String,
}

#[derive(Debug, Serialize)]
pub struct JobResponse {
    pub id: String,
    pub name: String,
    pub cron: String,
    pub task_type: String,
    pub payload: String,
    pub last_run: Option<String>,
}

impl From<Job> for JobResponse {
    fn from(job: Job) -> Self {
        Self {
            id: job.id,
            name: job.name,
            cron: job.cron,
            task_type: job.task_type,
            payload: job.payload,
            last_run: job.last_run.map(|dt| dt.to_rfc3339()),
        }
    }
}
#[derive(Debug, Error)]
pub enum JobApiError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Job not found")]
    NotFound,
}

impl ResponseError for JobApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            JobApiError::DatabaseError(e) => {
                HttpResponse::InternalServerError().body(format!("Database error: {}", e))
            }
            JobApiError::NotFound => HttpResponse::NotFound().body("Job not found"),
        }
    }
}

#[post("")]
async fn create_job(
    data: web::Json<JobRequest>,
    store: web::Data<Arc<SqliteJobStore>>,
) -> Result<HttpResponse, JobApiError>{
    let job = Job {
        id: data.id.clone(),
        name: data.name.clone(),
        cron: data.cron.clone(),
        task_type: data.task_type.clone(),
        payload: data.payload.clone(),
        last_run: None,
    };

    store.insert_job(&job).await?;
    Ok(HttpResponse::Created().finish())
}

#[get("")]
async fn list_jobs(store: web::Data<Arc<SqliteJobStore>>) -> Result<HttpResponse, JobApiError> {
    let result = store.load_jobs().await;
    let response: Vec<JobResponse> = result.into_iter().map(JobResponse::from).collect();
    Ok(HttpResponse::Ok().json(response))
}

#[get("/{id}")]
async fn get_job_by_id(
    path: web::Path<String>,
    store: web::Data<Arc<SqliteJobStore>>,
) -> Result<HttpResponse, JobApiError> {
    let id = path.into_inner();
    match store.get_job_by_id(&id).await? {
        Some(job) => Ok(HttpResponse::Ok().json(JobResponse::from(job))),
        None => Err(JobApiError::NotFound),
    }
}

#[put("/{id}")]
async fn update_job(
    path: web::Path<String>,
    data: web::Json<JobRequest>,
    store: web::Data<Arc<SqliteJobStore>>,
) -> Result<HttpResponse, JobApiError> {
    let id = path.into_inner();
    let job = Job {
        id,
        name: data.name.clone(),
        cron: data.cron.clone(),
        task_type: data.task_type.clone(),
        payload: data.payload.clone(),
        last_run: Some(Utc::now()),
    };

    store.update_job(&job).await?;
    Ok(HttpResponse::Ok().body("Job updated"))
}

#[delete("/{id}")]
async fn delete_job(
    path: web::Path<String>,
    store: web::Data<Arc<SqliteJobStore>>,
) -> Result<HttpResponse, JobApiError> {
    let id = path.into_inner();
    store.delete_job(&id).await?;
    Ok(HttpResponse::Ok().body("Job deleted"))
}

pub fn job_routes() -> Scope {
    web::scope("/jobs")
        .service(create_job)
        .service(list_jobs)
        .service(get_job_by_id)
        .service(update_job)
        .service(delete_job)
}
