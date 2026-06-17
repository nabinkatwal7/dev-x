use crate::{
    error::AppError,
    models::{CommandExecutionResult, CommandExecutionStatus, ExecuteCommandPayload},
};

pub struct CommandExecutor;

impl CommandExecutor {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(
        &self,
        payload: ExecuteCommandPayload,
    ) -> Result<CommandExecutionResult, AppError> {
        match payload.command_id.as_str() {
            "data.format-json" => format_json(&payload.input, false),
            "data.minify-json" => format_json(&payload.input, true),
            _ => Err(AppError::Internal(format!(
                "unknown command id: {}",
                payload.command_id
            ))),
        }
    }
}

fn format_json(input: &str, minify: bool) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: if minify {
                "data.minify-json".into()
            } else {
                "data.format-json".into()
            },
            title: if minify {
                "Minify JSON".into()
            } else {
                "Format JSON".into()
            },
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Paste a JSON document to transform it.".into(),
        });
    }

    let value: serde_json::Value = serde_json::from_str(trimmed)
        .map_err(|error| AppError::Internal(format!("invalid JSON: {error}")))?;

    let output = if minify {
        serde_json::to_string(&value)?
    } else {
        serde_json::to_string_pretty(&value)?
    };

    Ok(CommandExecutionResult {
        command_id: if minify {
            "data.minify-json".into()
        } else {
            "data.format-json".into()
        },
        title: if minify {
            "Minified JSON".into()
        } else {
            "Formatted JSON".into()
        },
        output,
        status: CommandExecutionStatus::Success,
        summary: if minify {
            "JSON payload minified successfully.".into()
        } else {
            "JSON payload formatted successfully.".into()
        },
    })
}
