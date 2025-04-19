use crate::azure::{AdfClient, AdfPipelineRunStatus, AdfPipelineStatus};
use crate::domain::task_payload::TaskPayload;
use crate::task::handler::TaskHandler;
use async_trait::async_trait;
use log::debug;

pub struct AdfTask;

#[async_trait]
impl TaskHandler for AdfTask {
    fn task_type(&self) -> &'static str {
        "adf_pipeline"
    }

    async fn handle(&self, payload: &TaskPayload) -> Result<(), String> {
        let adf_config = payload.as_adf().ok_or("Invalid payload for ADF task")?;
        debug!("ADF task handler");

        let adf_client = AdfClient::new(
            adf_config.subscription_id.clone(),
            adf_config.resource_group.clone(),
            adf_config.factory_name.clone(),
        )
            .map_err(|e| e.to_string())?;

        let id = adf_client
            .trigger_pipeline_run(&adf_config.pipeline, None)
            .await
            .map_err(|e| e.to_string())?;

        debug!("Triggered pipeline run: {} with parameters {}", id, payload);

        loop {
            let status = adf_client
                .get_pipeline_status(&id)
                .await
                .map_err(|e| e.to_string())?;

            match status.status {
                AdfPipelineStatus::Succeeded => {
                    debug!("Pipeline run succeeded");
                    break;
                }
                AdfPipelineStatus::Failed => {
                    debug!("Pipeline run failed");
                    return Err(status.message.unwrap_or_else(|| "Pipeline run failed".to_string()));
                }
                _ => debug!("Pipeline run status: {:?}", status),
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }

        Ok(())
    }
}