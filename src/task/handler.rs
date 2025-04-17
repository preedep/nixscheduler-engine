use async_trait::async_trait;
use crate::domain::task_payload::TaskPayload;

#[async_trait]
pub trait TaskHandler: Send + Sync {
    fn task_type(&self) -> &'static str;
    async fn handle(&self, payload: &TaskPayload) -> Result<(), String>;
}
