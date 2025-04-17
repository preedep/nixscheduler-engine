use std::sync::Arc;

use azure_identity::DefaultAzureCredential;
use log::debug;
use reqwest::Client;
pub struct AdfClient {
    pub subscription_id: String,
    pub resource_group: String,
    pub factory_name: String,
    pub credential: Arc<DefaultAzureCredential>,
    pub client: Client,
}

impl AdfClient {
    pub fn new(subscription_id: String, resource_group: String, factory_name: String) -> Result<Self,Box<dyn std::error::Error>> {
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
        debug!("trigger pipeline run for {} with parameters {}", pipeline_name, parameters.unwrap_or(serde_json::Value::Null));
        Ok("".to_string())
    }
    pub async fn get_pipeline_status(&self, pipeline_name: &str) -> Result<String, Box<dyn std::error::Error>> {
        debug!("get pipeline status for {}", pipeline_name);
        Ok("".to_string())
    }
}