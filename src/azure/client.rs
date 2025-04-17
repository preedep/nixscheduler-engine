use azure_core::credentials::TokenCredential;
use std::sync::Arc;

use azure_identity::DefaultAzureCredential;
use log::debug;
use reqwest::Client;
use serde_json::json;

pub struct AdfClient {
    pub subscription_id: String,
    pub resource_group: String,
    pub factory_name: String,
    pub credential: Arc<DefaultAzureCredential>,
    pub client: Client,
}

impl AdfClient {
    pub fn new(
        subscription_id: String,
        resource_group: String,
        factory_name: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let credential = DefaultAzureCredential::new()?;
        Ok(Self {
            subscription_id,
            resource_group,
            factory_name,
            credential,
            client: Client::new(),
        })
    }
    pub async fn trigger_pipeline_run(
        &self,
        pipeline_name: &str,
        parameters: Option<serde_json::Value>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        debug!("trigger pipeline run for {} ", pipeline_name,);
        let token_response = self
            .credential
            .get_token(&["https://management.azure.com/.default"])
            .await?;

        let access_token = token_response.token.secret();
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.DataFactory/factories/{}/pipelines/{}/createRun?api-version=2018-06-01",
            self.subscription_id, self.resource_group, self.factory_name, pipeline_name
        );

        let body = if let Some(params) = parameters {
            json!({ "parameters": params })
        } else {
            json!({})
        };

        let res = self
            .client
            .post(&url)
            .bearer_auth(access_token)
            .json(&body)
            .send()
            .await?;

        if res.status().is_success() {
            let resp_json: serde_json::Value = res.json().await?;
            let run_id = resp_json["runId"].as_str().unwrap_or_default().to_string();

            debug!("run_id: {}", run_id);
        } else {
            let err_text = res.text().await?;
            let err_msg = format!("Failed to trigger pipeline run: {}", err_text);
            return Err(err_msg.into());
        }

        Ok("".to_string())
    }
    pub async fn get_pipeline_status(
        &self,
        pipeline_name: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        debug!("get pipeline status for {}", pipeline_name);
        Ok("".to_string())
    }
}
