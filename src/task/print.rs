use crate::task::handler::TaskHandler;
use async_trait::async_trait;

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
