use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct OidcMetadata {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub jwks_uri: String,
    pub userinfo_endpoint: Option<String>,
    pub id_token_signing_alg_values_supported: Vec<String>,
    pub response_modes_supported: Vec<String>,
    pub scopes_supported: Option<Vec<String>>,
    pub end_session_endpoint: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IdTokenClaims {
    sub: String,
    email: Option<String>,
    name: Option<String>,
    nonce: Option<String>,
    aud: String,
    iss: String,
    exp: usize,
    iat: usize,
}

impl IdTokenClaims {
    pub fn is_nonce_valid(&self, expected_nonce: &str) -> bool {
        if let Some(nonce) = &self.nonce {
            nonce == expected_nonce
        } else {
            false
        }
    }
}