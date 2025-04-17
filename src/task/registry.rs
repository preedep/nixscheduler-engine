use crate::task::handler::TaskHandler;
use crate::task::print::PrintTask;
use log::debug;
use std::collections::HashMap;
use std::sync::Arc;

pub struct TaskRegistry {
    handlers: HashMap<String, Arc<dyn TaskHandler>>,
}

impl TaskRegistry {
    pub fn new() -> Self {
        let mut reg = Self {
            handlers: HashMap::new(),
        };
        reg.register_builtin();
        reg
    }
    pub fn print_all_handlers(&self) {
        for (key, handler) in &self.handlers {
            debug!("Task Type: {}", key);
        }
    }
    pub fn register<H: TaskHandler + 'static>(&mut self, handler: H) {
        self.handlers
            .insert(handler.task_type().to_string(), Arc::new(handler));
    }

    pub fn get(&self, task_type: &str) -> Option<Arc<dyn TaskHandler>> {
        self.handlers.get(task_type).cloned()
    }

    fn register_builtin(&mut self) {
        self.register(PrintTask);
    }
}
