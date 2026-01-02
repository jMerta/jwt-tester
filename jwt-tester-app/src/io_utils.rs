use crate::error::{AppError, AppResult};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde_json::Value;
use std::io::IsTerminal;
use std::io::{Read, Write};

fn prompt_label(spec: &str) -> Option<&str> {
    if spec == "prompt" {
        Some("")
    } else {
        spec.strip_prefix("prompt:")
    }
}

fn read_prompt_value(prompt: &str) -> std::io::Result<String> {
    eprint!("{prompt}");
    std::io::stderr().flush()?;
    rpassword::read_password()
}

pub fn read_input(spec: &str) -> AppResult<String> {
    if let Some(label) = prompt_label(spec) {
        if !std::io::stdin().is_terminal() {
            return Err(AppError::invalid_token(
                "prompt input requires a TTY; use '-', '@file', or env:NAME".to_string(),
            ));
        }
        let prompt = if label.trim().is_empty() {
            "Enter value: "
        } else {
            label
        };
        let value = read_prompt_value(prompt)
            .map_err(|e| AppError::invalid_token(format!("failed to read prompt: {e}")))?;
        return Ok(value.trim().to_string());
    }
    if spec == "-" {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| AppError::invalid_token(format!("failed to read stdin: {e}")))?;
        return Ok(buf.trim().to_string());
    }
    if let Some(rest) = spec.strip_prefix('@') {
        let data = std::fs::read_to_string(rest)
            .map_err(|e| AppError::invalid_token(format!("failed to read file {rest}: {e}")))?;
        return Ok(data.trim().to_string());
    }
    if let Some(env) = spec.strip_prefix("env:") {
        return std::env::var(env)
            .map_err(|_| AppError::invalid_key(format!("env var {env} not set")));
    }
    Ok(spec.to_string())
}

pub fn read_input_bytes(spec: &str) -> AppResult<Vec<u8>> {
    if let Some(label) = prompt_label(spec) {
        if !std::io::stdin().is_terminal() {
            return Err(AppError::invalid_key(
                "prompt input requires a TTY; use '-', '@file', or env:NAME".to_string(),
            ));
        }
        let prompt = if label.trim().is_empty() {
            "Enter value: "
        } else {
            label
        };
        let value = read_prompt_value(prompt)
            .map_err(|e| AppError::invalid_key(format!("failed to read prompt: {e}")))?;
        return Ok(value.trim().as_bytes().to_vec());
    }
    if spec == "-" {
        let mut buf = Vec::new();
        std::io::stdin()
            .read_to_end(&mut buf)
            .map_err(|e| AppError::invalid_token(format!("failed to read stdin: {e}")))?;
        return Ok(buf);
    }
    if let Some(rest) = spec.strip_prefix('@') {
        let data = std::fs::read(rest)
            .map_err(|e| AppError::invalid_key(format!("failed to read file {rest}: {e}")))?;
        return Ok(data);
    }
    if let Some(rest) = spec.strip_prefix("b64:") {
        let decoded = STANDARD
            .decode(rest)
            .map_err(|e| AppError::invalid_key(format!("invalid base64 secret: {e}")))?;
        return Ok(decoded);
    }
    if let Some(env) = spec.strip_prefix("env:") {
        let val = std::env::var(env)
            .map_err(|_| AppError::invalid_key(format!("env var {env} not set")))?;
        return Ok(val.into_bytes());
    }
    Ok(spec.as_bytes().to_vec())
}

pub fn read_json_value(spec: &str) -> AppResult<Value> {
    let raw = read_input(spec)?;
    serde_json::from_str(&raw).map_err(|e| AppError::invalid_token(format!("invalid JSON: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use uuid::Uuid;

    #[test]
    fn read_input_reads_file_and_env() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("input.txt");
        std::fs::write(&path, " hello \n").expect("write file");
        let file_val = read_input(&format!("@{}", path.display())).expect("read file");
        assert_eq!(file_val, "hello");

        let var = format!("JWT_TESTER_ENV_{}", Uuid::new_v4());
        std::env::set_var(&var, "env-value");
        let env_val = read_input(&format!("env:{var}")).expect("read env");
        assert_eq!(env_val, "env-value");
        std::env::remove_var(&var);
    }

    #[test]
    fn read_input_env_missing_errors() {
        let var = format!("JWT_TESTER_ENV_{}", Uuid::new_v4());
        std::env::remove_var(&var);
        let err = read_input(&format!("env:{var}")).expect_err("expected error");
        assert!(err.to_string().contains("env var"));
    }

    #[test]
    fn read_input_prompt_requires_tty() {
        if std::io::stdin().is_terminal() {
            return;
        }
        let err = read_input("prompt").expect_err("expected prompt error");
        assert!(err.to_string().contains("TTY"));
    }

    #[test]
    fn read_input_bytes_env_and_b64_errors() {
        let var = format!("JWT_TESTER_ENV_{}", Uuid::new_v4());
        std::env::set_var(&var, "bytes");
        let bytes = read_input_bytes(&format!("env:{var}")).expect("read env bytes");
        assert_eq!(bytes, b"bytes");
        std::env::remove_var(&var);

        let err = read_input_bytes("b64:!!!").expect_err("expected b64 error");
        assert!(err.to_string().contains("invalid base64"));
    }

    #[test]
    fn read_input_bytes_prompt_requires_tty() {
        if std::io::stdin().is_terminal() {
            return;
        }
        let err = read_input_bytes("prompt").expect_err("expected prompt error");
        assert!(err.to_string().contains("TTY"));
    }

    #[test]
    fn read_json_value_invalid_errors() {
        let err = read_json_value("{not-json}").expect_err("expected json error");
        assert!(err.to_string().contains("invalid JSON"));
    }
}
