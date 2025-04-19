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
}