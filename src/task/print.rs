use crate::task::handler::TaskHandler;
use async_trait::async_trait;
use log::debug;

pub struct PrintTask;

#[async_trait]
impl TaskHandler for PrintTask {
    fn task_type(&self) -> &'static str {
        "print"
    }

    async fn handle(&self, payload: &str) -> Result<(), String> {
        debug!("[print task] {}", payload);
        Ok(())
    }
}
