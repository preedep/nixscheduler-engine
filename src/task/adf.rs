use crate::azure::AdfClient;
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
        .map_err(|err| err.to_string())?;

        let res = adf_client
            .trigger_pipeline_run(&adf_config.pipeline, None)
            .await;
        match res {
            Ok(id) => {
                debug!(
                    "trigger pipeline run for {} with parameters {}",
                    id, payload
                );
            }
            Err(e) => {
                return Err(e.to_string());
            }
        }

        Ok(())
    }
}
