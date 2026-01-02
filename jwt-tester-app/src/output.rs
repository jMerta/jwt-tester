use crate::error::AppError;
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy)]
pub enum OutputMode {
    Json,
    Text,
}

#[derive(Debug, Clone, Copy)]
pub struct OutputConfig {
    pub mode: OutputMode,
    pub quiet: bool,
    pub no_color: bool,
    pub verbose: bool,
}

#[derive(Debug)]
pub struct CommandOutput {
    pub data: Value,
    pub text: String,
}

impl CommandOutput {
    pub fn new(data: Value, text: impl Into<String>) -> Self {
        Self {
            data,
            text: text.into(),
        }
    }
}

pub fn emit_ok(cfg: OutputConfig, output: CommandOutput) {
    match cfg.mode {
        OutputMode::Json => {
            let body = json!({
                "ok": true,
                "data": output.data,
            });
            println!("{}", body);
        }
        OutputMode::Text => {
            if !output.text.is_empty() {
                println!("{}", output.text);
            } else if !cfg.quiet {
                println!("OK");
            }
        }
    }
}

pub fn emit_err(cfg: OutputConfig, err: AppError) {
    match cfg.mode {
        OutputMode::Json => {
            println!("{}", err.as_json());
        }
        OutputMode::Text => {
            let prefix = if cfg.verbose {
                format!("[{}] ", err.code())
            } else {
                String::new()
            };
            if cfg.no_color {
                eprintln!("{}{}", prefix, err);
            } else {
                eprintln!("\u{1b}[31m{}{}\u{1b}[0m", prefix, err);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;

    #[test]
    fn emit_ok_json_and_text_do_not_panic() {
        let cfg = OutputConfig {
            mode: OutputMode::Json,
            quiet: false,
            no_color: true,
            verbose: false,
        };
        emit_ok(cfg, CommandOutput::new(json!({ "ok": true }), "OK"));

        let cfg = OutputConfig {
            mode: OutputMode::Text,
            quiet: true,
            no_color: true,
            verbose: false,
        };
        emit_ok(cfg, CommandOutput::new(json!({}), ""));
    }

    #[test]
    fn emit_err_json_and_text_do_not_panic() {
        let err = AppError::invalid_token("bad token");
        let cfg = OutputConfig {
            mode: OutputMode::Json,
            quiet: false,
            no_color: true,
            verbose: false,
        };
        emit_err(cfg, err.clone());

        let cfg = OutputConfig {
            mode: OutputMode::Text,
            quiet: false,
            no_color: true,
            verbose: true,
        };
        emit_err(cfg, err);
    }
}
