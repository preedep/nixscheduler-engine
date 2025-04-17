use crate::domain::task_payload::TaskPayload;
use crate::task::handler::TaskHandler;
use async_trait::async_trait;
use log::debug;
use std::thread::sleep;

pub struct PrintTask;

#[async_trait]
impl TaskHandler for PrintTask {
    fn task_type(&self) -> &'static str {
        "print"
    }

    async fn handle(&self, payload: &TaskPayload) -> Result<(), String> {
        debug!("Printing task");
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        Ok(())
    }
}
