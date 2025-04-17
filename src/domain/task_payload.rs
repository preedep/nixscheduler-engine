use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "task_type", content = "payload")]
pub enum TaskPayload {
    #[serde(rename = "adf_pipeline")]
    AdfPipeline(AdfConfig),
    #[serde(rename = "aws_stepfn")]
    AwsStepFunction(AwsStepFnConfig),
    #[serde(rename = "shell_command")]
    ShellCommand(ShellCommandConfig),
    #[serde(rename = "print")]
    Print(PrintConfig),
}

impl TaskPayload {
    pub fn task_type_name(&self) -> &'static str {
        match self {
            TaskPayload::AdfPipeline(_) => "adf_pipeline",
            TaskPayload::AwsStepFunction(_) => "aws_stepfn",
            TaskPayload::ShellCommand(_) => "shell_command",
            TaskPayload::Print(_) => "print",
        }
    }

    pub fn as_print(&self) -> Option<&PrintConfig> {
        match self {
            TaskPayload::Print(config) => Some(config),
            _ => None,
        }
    }
    pub fn as_adf(&self) -> Option<&AdfConfig> {
        match self {
            TaskPayload::AdfPipeline(config) => Some(config),
            _ => None,
        }
    }

    pub fn as_shell_command(&self) -> Option<&ShellCommandConfig> {
        match self {
            TaskPayload::ShellCommand(config) => Some(config),
            _ => None,
        }
    }

    pub fn as_stepfn(&self) -> Option<&AwsStepFnConfig> {
        match self {
            TaskPayload::AwsStepFunction(config) => Some(config),
            _ => None,
        }
    }
}

impl Display for TaskPayload {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.task_type_name().fmt(f)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdfConfig {
    pub subscription_id: String,
    pub resource_group: String,
    pub factory_name: String,
    pub pipeline: String,
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AwsStepFnConfig {
    pub arn: String,
    pub input: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogicAppConfig {
    pub endpoint: String,
    pub auth_type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShellCommandConfig {
    pub command: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PrintConfig {
    pub message: String,
}
