use crate::auth::oidc::model::OidcMetadata;
use reqwest::Client;

pub async fn fetch_metadata(tenant_id: &str) -> Result<OidcMetadata, reqwest::Error> {
    let url = format!(
        "https://login.microsoftonline.com/{}/v2.0/.well-known/openid-configuration",
        tenant_id
    );

    let client = Client::new();
    let res = client.get(&url).send().await?;
    let metadata = res.json::<OidcMetadata>().await?;
    Ok(metadata)
}
