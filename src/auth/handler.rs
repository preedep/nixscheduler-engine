use std::env;
use std::fs::metadata;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder, Scope};
use actix_web::cookie::Cookie;
use log::debug;
use serde::Deserialize;
use urlencoding::encode;
use uuid::Uuid;
use crate::auth::oidc::{fetch_metadata, verify_id_token};

static COOKIE_OIDC_NONCE: &str = "oidc_nonce";
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

    // Set nonce in secure cookie (expires in ~5 mins)
    let cookie = Cookie::build(COOKIE_OIDC_NONCE, nonce.clone())
        .path("/")
        .secure(true)
        .http_only(true)
        .max_age(time::Duration::minutes(5))
        .finish();
    

    // redirect ไปยัง IDP (เช่น Entra ID)
    HttpResponse::Found()
        .append_header(("Location", redirect_url))
        .finish()
}

#[post("/callback")]
pub async fn callback(form: web::Form<OidcCallbackForm>,
                      req: HttpRequest) -> impl Responder {
    // 🧱 Step 1: ตรวจสอบว่ามี error กลับมาจาก IDP หรือไม่
    debug!("OIDC Callback Form: {:#?}", form);
    if let Some(error) = &form.error {
        let description = form.error_description.clone().unwrap_or_default();
        return HttpResponse::BadRequest().body(format!(
            "OIDC Error: {} - {}",
            error, description
        ));
    }
    let tenant_id = env::var("AZURE_TENANT_ID").expect("Missing AZURE_TENANT_ID");
    let client_id = env::var("OIDC_CLIENT_ID").expect("Missing OIDC_CLIENT_ID");
    let metadata = fetch_metadata(&tenant_id).await;
    let metadata = match metadata {
        Ok(metadata) => metadata,
        Err(e) => {
            eprintln!("Error fetching OIDC metadata: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    if let Some(id_token) = &form.id_token {
        // Extract nonce from secure cookie
        let nonce_cookie = req.cookie(COOKIE_OIDC_NONCE);
        let expected_nonce = match nonce_cookie {
            Some(cookie) => cookie.value().to_string(),
            None => return HttpResponse::BadRequest().body("Missing nonce cookie"),
        };
        
        debug!("Expected nonce: {}", expected_nonce);
        let id = verify_id_token(id_token,
                                 &client_id, expected_nonce.as_str(),
                                 &metadata.jwks_uri,
                                 &metadata.issuer).await.map_err(|e| {
            eprintln!("Error verifying ID token: {}", e);
            HttpResponse::Unauthorized().body("Invalid ID token")
        }).unwrap();
        debug!("ID Token Claims: {:#?}", id);
        // finish login process 
    }else {
        return HttpResponse::BadRequest().body("Missing id_token");
    }
    HttpResponse::Ok().finish()
}

pub fn auth_routes() -> Scope {
    web::scope("/auth")
        .service(login)
        .service(callback)
}