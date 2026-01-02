use crate::cli::{SplitArgs, SplitFormat};
use crate::error::{AppError, AppResult};
use crate::io_utils::read_input;
use crate::output::{emit_err, emit_ok, CommandOutput, OutputConfig};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde_json::json;

pub fn run(args: SplitArgs, cfg: OutputConfig) -> i32 {
    let result = (|| -> AppResult<CommandOutput> {
        let token = read_input(&args.token)?;
        let parts: Vec<&str> = token.trim().split('.').collect();
        if parts.len() != 3 {
            return Err(AppError::invalid_token(
                "token must have 3 dot-separated segments",
            ));
        }
        let header_bytes = URL_SAFE_NO_PAD
            .decode(parts[0])
            .map_err(|e| AppError::invalid_token(format!("invalid base64url header: {e}")))?;
        let payload_bytes = URL_SAFE_NO_PAD
            .decode(parts[1])
            .map_err(|e| AppError::invalid_token(format!("invalid base64url payload: {e}")))?;
        let signature_bytes = URL_SAFE_NO_PAD
            .decode(parts[2])
            .map_err(|e| AppError::invalid_token(format!("invalid base64url signature: {e}")))?;

        let header_json: serde_json::Value = serde_json::from_slice(&header_bytes)
            .map_err(|e| AppError::invalid_token(format!("header is not valid JSON: {e}")))?;
        let payload_json: serde_json::Value = serde_json::from_slice(&payload_bytes)
            .map_err(|e| AppError::invalid_token(format!("payload is not valid JSON: {e}")))?;

        let sig_hex = hex::encode(&signature_bytes);

        let data = json!({
            "header": header_json,
            "payload": payload_json,
            "signature": {
                "hex": sig_hex,
                "length": signature_bytes.len(),
            },
        });

        if matches!(args.format, SplitFormat::Json) {
            return Ok(CommandOutput::new(data, ""));
        }

        let mut text = String::new();
        text.push_str("Header:\n");
        text.push_str(&serde_json::to_string_pretty(&data["header"]).unwrap_or_default());
        text.push_str("\nPayload:\n");
        text.push_str(&serde_json::to_string_pretty(&data["payload"]).unwrap_or_default());
        text.push_str("\nSignature (hex):\n");
        text.push_str(data["signature"]["hex"].as_str().unwrap_or(""));
        Ok(CommandOutput::new(data, text))
    })();

    match result {
        Ok(out) => {
            emit_ok(cfg, out);
            0
        }
        Err(err) => {
            let code = err.exit_code();
            emit_err(cfg, err);
            code
        }
    }
}

#[cfg(test)]
mod tests {
    use super::run;
    use crate::cli::{SplitArgs, SplitFormat};
    use crate::jwt_ops;
    use crate::output::{OutputConfig, OutputMode};
    use jsonwebtoken::{EncodingKey, Header};
    use serde_json::json;

    fn cfg() -> OutputConfig {
        OutputConfig {
            mode: OutputMode::Json,
            quiet: true,
            no_color: true,
            verbose: false,
        }
    }

    fn make_token() -> String {
        let header = Header::new(jsonwebtoken::Algorithm::HS256);
        jwt_ops::encode_token(
            &header,
            &json!({ "sub": "tester" }),
            &EncodingKey::from_secret(b"secret"),
        )
        .expect("encode token")
    }

    #[test]
    fn split_run_json_returns_success() {
        let token = make_token();
        let args = SplitArgs {
            format: SplitFormat::Json,
            token,
        };
        let code = run(args, cfg());
        assert_eq!(code, 0);
    }

    #[test]
    fn split_run_text_returns_success() {
        let token = make_token();
        let args = SplitArgs {
            format: SplitFormat::Text,
            token,
        };
        let code = run(args, cfg());
        assert_eq!(code, 0);
    }
}
