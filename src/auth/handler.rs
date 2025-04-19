use std::env;
use actix_web::{get, web, HttpResponse, Responder, Scope};
use urlencoding::encode;
use uuid::Uuid;
use crate::api::{create_job, delete_job, get_job_by_id, list_jobs, update_job};

/// GET /auth/login
#[get("/login")]
pub async fn login() -> impl Responder {
    // โหลดค่าจาก .env หรือ config system
    let client_id = env::var("OIDC_CLIENT_ID").expect("Missing OIDC_CLIENT_ID");
    let redirect_uri = env::var("OIDC_REDIRECT_URI").expect("Missing OIDC_REDIRECT_URI");
    let authorization_endpoint = env::var("OIDC_AUTH_URL").expect("Missing OIDC_AUTH_URL");
    let scope = env::var("OIDC_SCOPES").unwrap_or_else(|_| "openid profile email".to_string());

    // สร้าง state เพื่อป้องกัน CSRF
    let state = Uuid::new_v4().to_string();

    // สร้าง URL เพื่อ redirect ไปยัง Authorization Endpoint (Entra ID หรืออื่น ๆ)
    let redirect_url = format!(
        "{}?response_type=id_token&client_id={}&redirect_uri={}&scope={}&state={}&response_mode=form_post",
        authorization_endpoint,
        encode(&client_id),
        encode(&redirect_uri),
        encode(&scope),
        encode(&state),
    );

    // redirect ไปยัง IDP (เช่น Entra ID)
    HttpResponse::Found()
        .append_header(("Location", redirect_url))
        .finish()
}

pub fn auth_routes() -> Scope {
    web::scope("/auth")
        .service(login)
}