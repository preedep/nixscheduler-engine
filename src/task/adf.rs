use std::error::Error;
use async_trait::async_trait;
use log::debug;
use sqlx::encode::IsNull::No;
use crate::azure::AdfClient;
use crate::task::handler::TaskHandler;
use crate::task::print::PrintTask;

pub struct AdfTask;

#[async_trait]
impl TaskHandler for AdfTask{
    fn task_type(&self) -> &'static str {
        "adf"
    }

    async fn handle(&self, payload: &str) -> Result<(), String> {
        
        let subscription_id = "";
        let resource_group = "";
        let adf_name = "";
        let pipeline_name = "";
        
        let adf_client = 
            AdfClient::new(subscription_id.to_string(),
                           resource_group.to_string(),
                           adf_name.to_string()).map_err(|err| err.to_string())?;
        
        let res = adf_client.trigger_pipeline_run(pipeline_name,None).await;
        match res {
            Ok(id) => {
                debug!("trigger pipeline run for {} with parameters {}", id, payload);
            }
            Err(e) => {
                return Err(e.to_string());
            }
        }
        
        Ok(())
    }
}
