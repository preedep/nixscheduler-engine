use std::fmt::Display;
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

#[derive(Debug, Clone, PartialEq, Eq,Serialize,Deserialize)]
pub enum AdfPipelineStatus {
    Queued,
    InProgress,
    Succeeded,
    Failed,
    Cancelled,
    Canceling,
    TimedOut,
    Skipped,
    WaitingOnDependency,
    Unknown(String), // เผื่ออนาคตมีสถานะอื่น
}

impl From<&str> for AdfPipelineStatus {
    fn from(s: &str) -> Self {
        match s {
            "Queued" => Self::Queued,
            "InProgress" => Self::InProgress,
            "Succeeded" => Self::Succeeded,
            "Failed" => Self::Failed,
            "Cancelled" => Self::Cancelled,
            "Canceling" => Self::Canceling,
            "TimedOut" => Self::TimedOut,
            "Skipped" => Self::Skipped,
            "WaitingOnDependency" => Self::WaitingOnDependency,
            other => Self::Unknown(other.to_string()),
        }
    }
}
impl Display for AdfPipelineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            AdfPipelineStatus::Queued => "Queued",
            AdfPipelineStatus::InProgress => "InProgress",
            AdfPipelineStatus::Succeeded => "Succeeded",
            AdfPipelineStatus::Failed => "Failed",
            AdfPipelineStatus::Cancelled => "Cancelled",
            AdfPipelineStatus::Canceling => "Canceling",
            AdfPipelineStatus::TimedOut => "TimedOut",
            AdfPipelineStatus::Skipped => "Skipped",
            AdfPipelineStatus::WaitingOnDependency => "WaitingOnDependency",
            AdfPipelineStatus::Unknown(s) => s,
        }
        .to_string();
        write!(f, "{}", str)
    }
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct AdfPipelineRunStatus {
    pub runId: String,
    pub runGroupId: Option<String>,
    pub pipelineName: String,
    pub parameters: Option<HashMap<String, String>>,
    pub invokedBy: Option<InvokedBy>,
    pub runStart: Option<String>, // You can change to chrono::DateTime<Utc> if needed
    pub runEnd: Option<String>,
    pub durationInMs: Option<u64>,
    pub status: AdfPipelineStatus, // Or use enum if needed
    pub message: Option<String>,
    pub lastUpdated: Option<String>,
    pub annotations: Option<Vec<String>>,
    pub runDimensions: Option<HashMap<String, String>>,
    pub isLatest: Option<bool>,
    pub tier: Option<String>,
    pub tags: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvokedBy {
    pub id: Option<String>,
    pub name: Option<String>,
    pub invokedByType: Option<String>,
}

impl AdfClient {
    pub fn new(
        subscription_id: String,
        resource_group: String,
        factory_name: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {

        debug!("Checking environment variables...");
        debug!("AZURE_CLIENT_ID: {:?}", std::env::var("AZURE_CLIENT_ID"));
        debug!("AZURE_TENANT_ID: {:?}", std::env::var("AZURE_TENANT_ID"));
        debug!("AZURE_CLIENT_SECRET: {:?}", std::env::var("AZURE_CLIENT_SECRET").map(|s| "***".to_string()));
        
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
            .await;
        match token_response {
            Ok(token) => {
                debug!("Token acquired successfully");
                let access_token = token.token.secret();
                let url = format!(
                    "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.DataFactory/factories/{}/pipelines/{}/createRun?api-version=2018-06-01",
                    self.subscription_id, self.resource_group, self.factory_name, pipeline_name
                );
                debug!("url: {}", url);
                
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
                    Ok(run_id)
                } else {
                    let err_text = res.text().await?;
                    let err_msg = format!("Failed to trigger pipeline run: {}", err_text);
                    Err(err_msg.into())
                }
            },
            Err(e) => {
                let err_msg = format!("Failed to acquire token: {}", e);
                return Err(err_msg.into());
            }
        }
    }
    pub async fn get_pipeline_status(
        &self,
        run_id: &str,
    ) -> Result<AdfPipelineRunStatus, Box<dyn std::error::Error>> {
        
        let token_response = self
            .credential
            .get_token(&["https://management.azure.com/.default"])
            .await;
        match token_response {
            Ok(token) => {
                debug!("Token acquired successfully");
                let access_token = token.token.secret();
                let url = format!(
                    "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.DataFactory/factories/{}/pipelineruns/{}?api-version=2018-06-01",
                    self.subscription_id, self.resource_group, self.factory_name, run_id
                );
                debug!("url: {}", url);
                let res = self
                    .client
                    .get(&url)
                    .bearer_auth(access_token)
                    .send()
                    .await?;
                if res.status().is_success() {
                    let res = res.json::<AdfPipelineRunStatus>().await?;
                    Ok(res)
                }else{
                    let err_text = res.text().await?;
                    let err_msg = format!("Failed to get pipeline status: {}", err_text);
                    Err(err_msg.into())
                }
            },
            Err(e) => {
                let err_msg = format!("Failed to acquire token: {}", e);
                Err(err_msg.into())
            }
        }
    }
}
