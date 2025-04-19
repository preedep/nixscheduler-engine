use std::env;
use std::fs::metadata;
use actix_web::{get, post, web, HttpResponse, Responder, Scope};
use log::debug;
use serde::Deserialize;
use urlencoding::encode;
use uuid::Uuid;
use crate::auth::oidc::fetch_metadata;


#[derive(Debug, Deserialize)]
pub struct OidcCallbackForm {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    id_token: Option<String>,
    access_token: Option<String>,
    expires_in: Option<String>,
    token_type: Option<String>,
    scope: Option<String>,
    error_description: Option<String>,
}
/// GET /auth/login
#[get("/login")]
pub async fn login() -> impl Responder {
    // โหลดค่าจาก .env หรือ config system
    let tenant_id = env::var("AZURE_TENANT_ID").expect("Missing AZURE_TENANT_ID");
    let metadata = fetch_metadata(&tenant_id).await;
    let metadata = match metadata {
        Ok(metadata) => metadata,
        Err(e) => {
            eprintln!("Error fetching OIDC metadata: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    debug!("OIDC Metadata: {:?}", metadata);
    
    let client_id = env::var("OIDC_CLIENT_ID").expect("Missing OIDC_CLIENT_ID");
    let redirect_uri = env::var("OIDC_REDIRECT_URI").expect("Missing OIDC_REDIRECT_URI");
    let authorization_endpoint = metadata.authorization_endpoint;
    let scope = env::var("OIDC_SCOPES").unwrap_or_else(|_| "openid profile email".to_string());

    // สร้าง state เพื่อป้องกัน CSRF
    let state = Uuid::new_v4().to_string();
    let nonce = Uuid::new_v4().to_string();
    
    // สร้าง URL เพื่อ redirect ไปยัง Authorization Endpoint (Entra ID หรืออื่น ๆ)
    let redirect_url = format!(
        "{}?response_type=id_token&client_id={}&redirect_uri={}&scope={}&state={}&response_mode=form_post&nonce={}",
        authorization_endpoint,
        encode(&client_id),
        encode(&redirect_uri),
        encode(&scope),
        encode(&state),
        encode(&nonce)
    );

    // redirect ไปยัง IDP (เช่น Entra ID)
    HttpResponse::Found()
        .append_header(("Location", redirect_url))
        .finish()
}

#[post("/callback")]
pub async fn callback(form: web::Form<OidcCallbackForm>) -> impl Responder {
    // 🧱 Step 1: ตรวจสอบว่ามี error กลับมาจาก IDP หรือไม่
    debug!("OIDC Callback Form: {:#?}", form);
    if let Some(error) = &form.error {
        let description = form.error_description.clone().unwrap_or_default();
        return HttpResponse::BadRequest().body(format!(
            "OIDC Error: {} - {}",
            error, description
        ));
    }
    HttpResponse::Ok().finish()
}

pub fn auth_routes() -> Scope {
    web::scope("/auth")
        .service(login)
        .service(callback)
}