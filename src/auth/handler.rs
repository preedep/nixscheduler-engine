use std::env;
use std::fs::metadata;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder, Scope};
use actix_web::cookie::Cookie;
use log::{debug, error};
use serde::Deserialize;
use urlencoding::encode;
use uuid::Uuid;
use crate::auth::oidc::{fetch_metadata, verify_id_token};

static COOKIE_OIDC_NONCE: &str = "oidc_nonce";
static COOKIE_ACCESS_TOKEN: &str = "access_token";

static COOKIE_LOGGED_STATE: &str = "logged_in";
#[derive(Debug, Deserialize)]
pub struct OidcCallbackForm {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    id_token: Option<String>,
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<String>,
    token_type: Option<String>,
    scope: Option<String>,
    error_description: Option<String>,
}
/// GET /auth/login
#[get("/login")]
pub async fn login() -> impl Responder {
    debug!("Login");
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
        "{}?response_type=id_token%20token&client_id={}&redirect_uri={}&scope={}&state={}&response_mode=form_post&nonce={}",
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
        .cookie(cookie)
        .cookie(
            Cookie::build(COOKIE_LOGGED_STATE, "true")
                .path("/")
                .secure(true)
                .http_only(false) // ต้อง false ถ้าให้ JS เห็น cookie
                .max_age(time::Duration::minutes(30))
                .finish()
        )
        .finish()
}

#[post("/logout")]
pub async fn logout() -> impl Responder {
    debug!("Logout");
    
    let tenant_id = env::var("AZURE_TENANT_ID").expect("Missing AZURE_TENANT_ID");

    let metadata = match fetch_metadata(&tenant_id).await {
        Ok(metadata) => metadata,
        Err(e) => {
            eprintln!("Error fetching OIDC metadata: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    debug!("OIDC Metadata: {:#?}", metadata);

    // ลบ cookie ที่เกี่ยวข้องกับ OIDC
    let cookie = Cookie::build(COOKIE_OIDC_NONCE, "")
        .path("/")
        .secure(true)
        .http_only(true)
        .max_age(time::Duration::seconds(0))
        .finish();

    
    if let Some(logout_url) = metadata.end_session_endpoint {
        
        let app_login_url = env::var("APP_LOGIN_URL").expect("Missing APP_LOGIN_URL");
        let redirect = format!(
            "{}?post_logout_redirect_uri={}",
            logout_url,
            urlencoding::encode(app_login_url.as_str())  // ✅ ใส่ URL เต็ม
        );

        HttpResponse::Found()
            .append_header(("Location", redirect))  // ✅ redirect จริงไปที่ Azure logout
            .cookie(cookie)
            .finish()
    } else {
        HttpResponse::BadRequest().body("Missing end_session_endpoint")
    }
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
            error!("Error fetching OIDC metadata: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    return if let Some(id_token) = &form.id_token {
        
        debug!("ID Token: {}", id_token);
        // Extract nonce from secure cookie
        let nonce_cookie = req.cookie(COOKIE_OIDC_NONCE);
        debug!("Nonce Cookie: {:?}", nonce_cookie);
        
        let expected_nonce = match nonce_cookie {
            Some(cookie) => cookie.value().to_string(),
            None => return HttpResponse::BadRequest().body("Missing nonce cookie"),
        };
        
        debug!("Expected nonce: {}", expected_nonce);
        let id = verify_id_token(id_token,
                                 &client_id, expected_nonce.as_str(),
                                 &metadata.jwks_uri,
                                 &metadata.issuer).await.map_err(|e| {
            error!("Error verifying ID token: {}", e);
            HttpResponse::Unauthorized().body("Invalid ID token")
        });
        let id = match id {
            Ok(id) => id,
            Err(e) => return e,
        };
        debug!("ID Token Claims: {:#?}", id);

        if let Some(access_token) = &form.access_token {
            debug!("Access Token: {}", access_token);
            let cookie = Cookie::build(COOKIE_ACCESS_TOKEN, access_token.as_str())
                .http_only(true)
                .secure(true)
                .path("/")
                .max_age(time::Duration::minutes(30))
                .finish();

            HttpResponse::Found()
                .append_header(("Location", "/index.html"))
                .cookie(cookie)
                .finish()
        } else {
            HttpResponse::BadRequest().body("Missing access_token")
        }
    }else {
        HttpResponse::BadRequest().body("Missing id_token")
    }
}

pub fn auth_routes() -> Scope {
    web::scope("/auth")
        .service(login)
        .service(logout)
        .service(callback)
}