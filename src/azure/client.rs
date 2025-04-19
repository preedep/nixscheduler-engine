use azure_core::credentials::TokenCredential;
use azure_identity::DefaultAzureCredential;
use log::debug;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;

pub struct AdfClient {
    pub subscription_id: String,
    pub resource_group: String,
    pub factory_name: String,
    pub credential: Arc<DefaultAzureCredential>,
    pub client: Client,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    Unknown(String),
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
        write!(
            f,
            "{}",
            match self {
                Self::Queued => "Queued",
                Self::InProgress => "InProgress",
                Self::Succeeded => "Succeeded",
                Self::Failed => "Failed",
                Self::Cancelled => "Cancelled",
                Self::Canceling => "Canceling",
                Self::TimedOut => "TimedOut",
                Self::Skipped => "Skipped",
                Self::WaitingOnDependency => "WaitingOnDependency",
                Self::Unknown(s) => s,
            }
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdfPipelineRunStatus {
    pub run_id: String,
    pub run_group_id: Option<String>,
    pub pipeline_name: String,
    pub parameters: Option<HashMap<String, String>>,
    pub invoked_by: Option<InvokedBy>,
    pub run_start: Option<String>,
    pub run_end: Option<String>,
    pub duration_in_ms: Option<u64>,
    pub status: AdfPipelineStatus,
    pub message: Option<String>,
    pub last_updated: Option<String>,
    pub annotations: Option<Vec<String>>,
    pub run_dimensions: Option<HashMap<String, String>>,
    pub is_latest: Option<bool>,
    pub tier: Option<String>,
    pub tags: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvokedBy {
    pub id: Option<String>,
    pub name: Option<String>,
    pub invoked_by_type: Option<String>,
}

impl AdfClient {
    pub fn new(
        subscription_id: String,
        resource_group: String,
        factory_name: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        debug!("Checking environment variables...");
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
        let token = self
            .credential
            .get_token(&["https://management.azure.com/.default"])
            .await?;
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.DataFactory/factories/{}/pipelines/{}/createRun?api-version=2018-06-01",
            self.subscription_id, self.resource_group, self.factory_name, pipeline_name
        );
        let body = json!({ "parameters": parameters.unwrap_or_default() });
        let res = self
            .client
            .post(&url)
            .bearer_auth(token.token.secret())
            .json(&body)
            .send()
            .await?;
        if res.status().is_success() {
            let run_id = res.json::<serde_json::Value>().await?["runId"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            Ok(run_id)
        } else {
            Err(format!("Failed to trigger pipeline run: {}", res.text().await?).into())
        }
    }

    pub async fn get_pipeline_status(
        &self,
        run_id: &str,
    ) -> Result<AdfPipelineRunStatus, Box<dyn std::error::Error>> {
        let token = self
            .credential
            .get_token(&["https://management.azure.com/.default"])
            .await?;
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.DataFactory/factories/{}/pipelineruns/{}?api-version=2018-06-01",
            self.subscription_id, self.resource_group, self.factory_name, run_id
        );
        let res = self
            .client
            .get(&url)
            .bearer_auth(token.token.secret())
            .send()
            .await?;
        if res.status().is_success() {
            Ok(res.json::<AdfPipelineRunStatus>().await?)
        } else {
            Err(format!("Failed to get pipeline status: {}", res.text().await?).into())
        }
    }
}