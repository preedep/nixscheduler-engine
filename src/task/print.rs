use async_trait::async_trait;
use crate::task::handler::TaskHandler;

pub struct PrintTask;

#[async_trait]
impl TaskHandler for PrintTask {
    fn task_type(&self) -> &'static str {
        "print"
    }

    async fn handle(&self, payload: &str) -> Result<(), String> {
        println!("[print task] {}", payload);
        Ok(())
    }
}