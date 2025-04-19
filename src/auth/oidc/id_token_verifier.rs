use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm};
use serde::{Deserialize};
use serde_json::Value;
use crate::auth::oidc::IdTokenClaims;

async fn fetch_jwks(jwks_uri: &str) -> Result<serde_json::Value, reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client.get(jwks_uri).send().await?;
    Ok(res.json::<serde_json::Value>().await?)
}
pub async fn verify_id_token(id_token: &str, client_id: &str, expected_nonce: &str, jwks_uri: &str, expected_issuer: &str)
                         -> Result<IdTokenClaims, String>
{
    let jwks = fetch_jwks(jwks_uri).await.map_err(|e| e.to_string())?;

    let header = decode_header(id_token).map_err(|e| format!("Invalid header: {}", e))?;
    let kid = header.kid.ok_or("Missing kid in header")?;

    let keys = jwks["keys"].as_array().ok_or("Invalid JWKS format")?;
    let key = keys.iter().find(|k| k["kid"] == kid).ok_or("Key ID not found in JWKS")?;

    let n = key["n"].as_str().ok_or("Missing 'n' in key")?;
    let e = key["e"].as_str().ok_or("Missing 'e' in key")?;

    let decoding_key = DecodingKey::from_rsa_components(n, e).map_err(|e| e.to_string())?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[client_id]);
    validation.set_issuer(&[expected_issuer]);
    validation.validate_exp = true;
    
    let token_data = decode::<IdTokenClaims>(id_token, &decoding_key, &validation)
        .map_err(|e| format!("Token validation failed: {}", e))?;

    // ตรวจสอบ nonce เพิ่มเติม
    if !token_data.claims.is_nonce_valid(expected_nonce){
        return Err("Nonce mismatch".to_string());
    }
    
    Ok(token_data.claims)
}