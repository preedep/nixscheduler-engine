use actix_web::{get, post, delete, put, web, HttpResponse, Responder, Scope};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::Utc;
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

#[post("")]
async fn create_job(
    data: web::Json<JobRequest>,
    store: web::Data<Arc<SqliteJobStore>>,
) -> impl Responder {
    let job = Job {
        id: data.id.clone(),
        name: data.name.clone(),
        cron: data.cron.clone(),
        task_type: data.task_type.clone(),
        payload: data.payload.clone(),
        last_run: None,
    };

    match store.insert_job(&job).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => HttpResponse::InternalServerError().body(format!("Insert failed: {}", e)),
    }
}

#[get("")]
async fn list_jobs(store: web::Data<Arc<SqliteJobStore>>) -> impl Responder {
    let result = store.load_jobs().await;
    let response: Vec<JobResponse> = result.into_iter().map(JobResponse::from).collect();
    HttpResponse::Ok().json(response)
}

#[get("/{id}")]
async fn get_job_by_id(
    path: web::Path<String>,
    store: web::Data<Arc<SqliteJobStore>>,
) -> impl Responder {
    let id = path.into_inner();

    match store.get_job_by_id(&id).await {
        Ok(Some(job)) => HttpResponse::Ok().json(JobResponse::from(job)),
        Ok(None) => HttpResponse::NotFound().body("Job not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[put("/{id}")]
async fn update_job(
    path: web::Path<String>,
    data: web::Json<JobRequest>,
    store: web::Data<Arc<SqliteJobStore>>,
) -> impl Responder {
    let id = path.into_inner();

    let job = Job {
        id,
        name: data.name.clone(),
        cron: data.cron.clone(),
        task_type: data.task_type.clone(),
        payload: data.payload.clone(),
        last_run: Some(Utc::now()), // optional: หรือใช้ None
    };

    match store.update_job(&job).await {
        Ok(_) => HttpResponse::Ok().body("Job updated"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Update failed: {}", e)),
    }
}

#[delete("/{id}")]
async fn delete_job(
    path: web::Path<String>,
    store: web::Data<Arc<SqliteJobStore>>,
) -> impl Responder {
    let id = path.into_inner();

    match store.delete_job(&id).await {
        Ok(_) => HttpResponse::Ok().body("Job deleted"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Delete failed: {}", e)),
    }
}

pub fn job_routes() -> Scope {
    web::scope("/jobs")
        .service(create_job)
        .service(list_jobs)
        .service(get_job_by_id)
        .service(update_job)
        .service(delete_job)
}
