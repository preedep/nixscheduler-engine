use actix_service::{forward_ready, Service, Transform};
use actix_web::{dev::{ServiceRequest, ServiceResponse}, http::header::AUTHORIZATION, Error, HttpMessage, HttpResponse};
use actix_web::body::BoxBody;
use futures_util::future::{ok, LocalBoxFuture, Ready};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, TokenData, Validation};
use once_cell::sync::OnceCell;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, rc::Rc, sync::Mutex};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use log::debug;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub roles: Option<Vec<String>>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Jwk {
    kid: String,
    n: String,
    e: String,
    kty: String,
    alg: String,
    #[serde(rename = "use")]
    use_: String,
}

#[derive(Debug, Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

struct CachedJwks {
    keys: HashMap<String, DecodingKey>,
    fetched_at: Instant,
}

static JWKS_CACHE: OnceCell<Mutex<Option<CachedJwks>>> = OnceCell::new();

async fn fetch_jwks(jwks_url: &str) -> Result<HashMap<String, DecodingKey>, reqwest::Error> {
    let res = Client::new().get(jwks_url).send().await?.json::<Jwks>().await?;
    let mut map = HashMap::new();
    for key in res.keys {
        if key.kty == "RSA" {
            if let Ok(decoding_key) = DecodingKey::from_rsa_components(&key.n, &key.e) {
                map.insert(key.kid.clone(), decoding_key);
            }
        }
    }
    Ok(map)
}

fn validate_token(token: &str, jwks_url: &str) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    let header = decode_header(token)?;
    let kid = header.kid.ok_or(jsonwebtoken::errors::ErrorKind::InvalidToken)?;
    let cache = JWKS_CACHE.get_or_init(|| Mutex::new(None));
    let keys: HashMap<String, DecodingKey>;

    {
        let mut guard = cache.lock().unwrap();

        let reload = guard
            .as_ref()
            .map(|jwks| jwks.fetched_at.elapsed() >= Duration::from_secs(3600))
            .unwrap_or(true);

        if reload {
            let new_keys = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(fetch_jwks(jwks_url))
                .unwrap_or_default();

            *guard = Some(CachedJwks {
                keys: new_keys,
                fetched_at: Instant::now(),
            });
        }

        // ✅ Clone keys before guard is dropped
        keys = guard.as_ref().unwrap().keys.clone();
    }

    let key = keys.get(&kid).ok_or(jsonwebtoken::errors::ErrorKind::InvalidKeyFormat)?;
    decode::<Claims>(token, key, &Validation::new(Algorithm::RS256))
}

#[derive(Debug,Clone)]
pub struct AuthMiddleware {
    pub jwks_url: String,
}

impl AuthMiddleware {
    pub fn new(jwks_url: String) -> Self {
        Self { jwks_url }
    }
}
impl<S> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareMiddleware {
            service: Rc::new(service),
            jwks_url: self.jwks_url.clone(),
        })
    }
}

pub struct AuthMiddlewareMiddleware<S> {
    service: Rc<S>,
    jwks_url: String,
}


impl<S> Service<ServiceRequest> for AuthMiddlewareMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        // 👇 Read & clone header out before entering async block
        let header_value = req
            .headers()
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        let jwks_url = self.jwks_url.clone();
        let srv = self.service.clone();

        Box::pin(async move {
            if let Some(header_value) = header_value {
                if let Some(token) = header_value.strip_prefix("Bearer ") {
                    let token = token.trim().to_string();
                    debug!("Token: {:#?}", token);
                    if let Ok(token_data) = validate_token(&token, &jwks_url) {
                        req.extensions_mut().insert(token_data.claims);
                        return srv.call(req).await;
                    }
                }
            }

            Ok(req.into_response(
                HttpResponse::Unauthorized()
                    .body("Unauthorized")
                    .map_into_boxed_body(),
            ))
        })
    }
}