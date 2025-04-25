use std::env;
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

static COOKIE_TIMEOUT: i64 = 30; // นาที
static COOKIE_NONCE_TIMEOUT: i64 = 5; // นาที

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

/// Helper: เช็คว่าเป็น production หรือไม่
fn is_production() -> bool {
    env::var("APP_ENV")
        .map(|env| env.to_lowercase() == "prod" || env.to_lowercase() == "production")
        .unwrap_or(false)
}

/// Helper: สร้าง Cookie access_token ตาม mode (Prod / Dev)
fn build_access_token_cookie(access_token: &str) -> Cookie<'static> {
    let mut cookie = Cookie::build(COOKIE_ACCESS_TOKEN, access_token.to_owned())
        .path("/")
        .max_age(time::Duration::minutes(COOKIE_TIMEOUT));

    if is_production() {
        cookie = cookie.secure(true).http_only(true); // Production: secure, httpOnly
    } else {
        cookie = cookie.secure(false).http_only(false); // Dev: allow JS access
    }

    cookie.finish()
}

/// Helper: สร้าง Cookie logged_in สำหรับ JS
fn build_logged_in_cookie() -> Cookie<'static> {
    Cookie::build(COOKIE_LOGGED_STATE, "true")
        .path("/")
        .secure(is_production()) // Prod = true, Dev = false
        .http_only(false)         // ต้องเป็น false เสมอ เพราะ JS ต้องอ่านได้
        .max_age(time::Duration::minutes(COOKIE_TIMEOUT))
        .finish()
}

/// Helper: สร้าง Cookie oidc_nonce สำหรับ verify
fn build_oidc_nonce_cookie(nonce: &str) -> Cookie<'static> {
    Cookie::build(COOKIE_OIDC_NONCE, nonce.to_owned())
        .path("/")
        .secure(true)    // ควร secure เสมอ
        .http_only(true) // ต้อง httpOnly เพื่อความปลอดภัย
        .max_age(time::Duration::minutes(COOKIE_NONCE_TIMEOUT))
        .finish()
}

/// GET /auth/login
#[get("/login")]
pub async fn login() -> impl Responder {
    debug!("Login");

    let tenant_id = env::var("AZURE_TENANT_ID").expect("Missing AZURE_TENANT_ID");
    let metadata = match fetch_metadata(&tenant_id).await {
        Ok(metadata) => metadata,
        Err(e) => {
            error!("Error fetching OIDC metadata: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let client_id = env::var("OIDC_CLIENT_ID").expect("Missing OIDC_CLIENT_ID");
    let redirect_uri = env::var("OIDC_REDIRECT_URI").expect("Missing OIDC_REDIRECT_URI");
    let scope = env::var("OIDC_SCOPES").unwrap_or_else(|_| "openid profile email".to_string());

    let state = Uuid::new_v4().to_string();
    let nonce = Uuid::new_v4().to_string();

    let redirect_url = format!(
        "{}?response_type=id_token%20token&client_id={}&redirect_uri={}&scope={}&state={}&response_mode=form_post&nonce={}",
        metadata.authorization_endpoint,
        encode(&client_id),
        encode(&redirect_uri),
        encode(&scope),
        encode(&state),
        encode(&nonce)
    );

    HttpResponse::Found()
        .append_header(("Location", redirect_url))
        .cookie(build_oidc_nonce_cookie(&nonce))
        .cookie(build_logged_in_cookie())
        .finish()
}

/// POST /auth/callback
#[post("/callback")]
pub async fn callback(form: web::Form<OidcCallbackForm>, req: HttpRequest) -> impl Responder {
    debug!("OIDC Callback Form: {:#?}", form);

    if let Some(error) = &form.error {
        return HttpResponse::BadRequest().body(format!(
            "OIDC Error: {} - {}",
            error,
            form.error_description.clone().unwrap_or_default()
        ));
    }

    let tenant_id = env::var("AZURE_TENANT_ID").expect("Missing AZURE_TENANT_ID");
    let client_id = env::var("OIDC_CLIENT_ID").expect("Missing OIDC_CLIENT_ID");
    let metadata = match fetch_metadata(&tenant_id).await {
        Ok(metadata) => metadata,
        Err(e) => {
            error!("Error fetching OIDC metadata: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    if let Some(id_token) = &form.id_token {
        let expected_nonce = match req.cookie(COOKIE_OIDC_NONCE) {
            Some(cookie) => cookie.value().to_string(),
            None => return HttpResponse::BadRequest().body("Missing nonce cookie"),
        };

        let id = match verify_id_token(
            id_token,
            &client_id,
            &expected_nonce,
            &metadata.jwks_uri,
            &metadata.issuer,
        )
            .await
        {
            Ok(id) => id,
            Err(e) => {
                error!("Error verifying ID token: {}", e);
                return HttpResponse::Unauthorized().body("Invalid ID token");
            }
        };

        debug!("ID Token Claims: {:#?}", id);

        if let Some(access_token) = &form.access_token {
            return HttpResponse::Found()
                .append_header(("Location", "/index.html"))
                .cookie(build_access_token_cookie(access_token))
                .finish();
        }

        return HttpResponse::BadRequest().body("Missing access_token");
    }

    HttpResponse::BadRequest().body("Missing id_token")
}

/// POST /auth/logout
#[post("/logout")]
pub async fn logout() -> impl Responder {
    debug!("Logout");

    let tenant_id = env::var("AZURE_TENANT_ID").expect("Missing AZURE_TENANT_ID");

    let metadata = match fetch_metadata(&tenant_id).await {
        Ok(metadata) => metadata,
        Err(e) => {
            error!("Error fetching OIDC metadata: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let cookie_clear = Cookie::build(COOKIE_OIDC_NONCE, "")
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
            encode(&app_login_url)
        );

        HttpResponse::Found()
            .append_header(("Location", redirect))
            .cookie(cookie_clear)
            .finish()
    } else {
        HttpResponse::BadRequest().body("Missing end_session_endpoint")
    }
}

/// Mount route /auth/*
pub fn auth_routes() -> Scope {
    web::scope("/auth")
        .service(login)
        .service(callback)
        .service(logout)
}