use actix_web::{get, post, delete, put, web, HttpResponse, Responder, Scope};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::job::store::SqliteJobStore;

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

#[post("")]
async fn create_job(
    data: web::Json<JobRequest>,
    store: web::Data<Arc<SqliteJobStore>>,
) -> impl Responder {
    HttpResponse::Ok().finish()
}

#[get("")]
async fn list_jobs(store: web::Data<Arc<SqliteJobStore>>) -> impl Responder {
    HttpResponse::Ok().finish()
}

#[get("/{id}")]
async fn get_job_by_id(
    path: web::Path<String>,
    store: web::Data<Arc<SqliteJobStore>>,
) -> impl Responder {
    let id = path.into_inner();

    HttpResponse::Ok().finish()
}

#[put("/{id}")]
async fn update_job(
    path: web::Path<String>,
    data: web::Json<JobRequest>,
    store: web::Data<Arc<SqliteJobStore>>,
) -> impl Responder {
    let id = path.into_inner();

    HttpResponse::Ok().finish()
}

#[delete("/{id}")]
async fn delete_job(
    path: web::Path<String>,
    store: web::Data<Arc<SqliteJobStore>>,
) -> impl Responder {
    let id = path.into_inner();
    HttpResponse::Ok().finish()
}

pub fn job_routes() -> Scope {
    web::scope("/jobs")
        .service(create_job)
        .service(list_jobs)
        .service(get_job_by_id)
        .service(update_job)
        .service(delete_job)
}
