use crate::cli::{DecodeArgs, VerifyCommonArgs};
use crate::commands::verify::verify_token_with_args;
use crate::date_utils::{extract_dates, parse_date_mode};
use crate::error::{AppError, AppResult};
use crate::io_utils::read_input;
use crate::jwt_ops;
use crate::output::{emit_err, emit_ok, CommandOutput, OutputConfig};
use serde_json::json;
use std::path::PathBuf;

pub fn run(
    no_persist: bool,
    data_dir: Option<PathBuf>,
    args: DecodeArgs,
    cfg: OutputConfig,
) -> i32 {
    let result = (|| -> AppResult<CommandOutput> {
        let token = read_input(&args.token)?;
        let decoded = jwt_ops::decode_unverified(&token)?;
        let date_mode = parse_date_mode(args.date)?;
        let dates = extract_dates(&decoded.payload_json, date_mode)?;
        let mut data = json!({
            "header": decoded.header_json,
            "payload": decoded.payload_json,
            "dates": dates.json,
        });

        let mut text = String::new();
        let verify_requested = has_verify_request(&args.verify);
        if verify_requested {
            let verify_outcome =
                verify_token_with_args(no_persist, data_dir.clone(), &args.verify, &token)?;
            data["verified"] = json!(true);
            data["verification"] = verify_outcome.data.clone();
            text.push_str("VERIFIED\n");
        } else {
            text.push_str("UNVERIFIED\n");
        }
        text.push_str("Header:\n");
        text.push_str(&serde_json::to_string_pretty(&data["header"]).unwrap_or_default());
        text.push_str("\nPayload:\n");
        text.push_str(&serde_json::to_string_pretty(&data["payload"]).unwrap_or_default());
        if !dates.lines.is_empty() {
            text.push_str("\nDates:\n");
            text.push_str(&dates.lines.join("\n"));
        }

        if let Some(path) = &args.out {
            let body = json!({ "ok": true, "data": data });
            let json_text = serde_json::to_string_pretty(&body)
                .map_err(|e| AppError::internal(format!("serialize output: {e}")))?;
            std::fs::write(path, json_text.as_bytes())
                .map_err(|e| AppError::internal(format!("failed to write {path:?}: {e}")))?;
        }

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

fn has_verify_request(args: &VerifyCommonArgs) -> bool {
    args.secret.is_some()
        || args.key.is_some()
        || args.jwks.is_some()
        || args.project.is_some()
        || args.alg.is_some()
        || args.try_all_keys
        || args.ignore_exp
        || args.leeway_secs != 30
        || args.iss.is_some()
        || args.sub.is_some()
        || !args.aud.is_empty()
        || !args.require.is_empty()
        || args.explain
}

#[cfg(test)]
mod tests {
    use super::has_verify_request;
    use crate::cli::{JwtAlg, VerifyCommonArgs};
    use crate::commands::decode::run;
    use crate::jwt_ops;
    use crate::output::{OutputConfig, OutputMode};
    use jsonwebtoken::{EncodingKey, Header};
    use serde_json::json;
    use tempfile::tempdir;

    fn base_args() -> VerifyCommonArgs {
        VerifyCommonArgs {
            secret: None,
            key: None,
            jwks: None,
            key_format: None,
            kid: None,
            allow_single_jwk: false,
            project: None,
            key_id: None,
            key_name: None,
            try_all_keys: false,
            ignore_exp: false,
            leeway_secs: 30,
            iss: None,
            sub: None,
            aud: Vec::new(),
            require: Vec::new(),
            explain: false,
            alg: None,
        }
    }

    #[test]
    fn has_verify_request_false_when_defaults() {
        let args = base_args();
        assert!(!has_verify_request(&args));
    }

    #[test]
    fn has_verify_request_true_when_any_flag_set() {
        let mut args = base_args();
        args.secret = Some("secret".to_string());
        assert!(has_verify_request(&args));

        let mut args = base_args();
        args.key = Some("key".to_string());
        assert!(has_verify_request(&args));

        let mut args = base_args();
        args.jwks = Some("jwks".to_string());
        assert!(has_verify_request(&args));

        let mut args = base_args();
        args.project = Some("proj".to_string());
        assert!(has_verify_request(&args));

        let mut args = base_args();
        args.alg = Some(JwtAlg::HS256);
        assert!(has_verify_request(&args));

        let mut args = base_args();
        args.try_all_keys = true;
        assert!(has_verify_request(&args));

        let mut args = base_args();
        args.ignore_exp = true;
        assert!(has_verify_request(&args));

        let mut args = base_args();
        args.leeway_secs = 45;
        assert!(has_verify_request(&args));

        let mut args = base_args();
        args.iss = Some("iss".to_string());
        assert!(has_verify_request(&args));

        let mut args = base_args();
        args.sub = Some("sub".to_string());
        assert!(has_verify_request(&args));

        let mut args = base_args();
        args.aud = vec!["aud".to_string()];
        assert!(has_verify_request(&args));

        let mut args = base_args();
        args.require = vec!["claim".to_string()];
        assert!(has_verify_request(&args));

        let mut args = base_args();
        args.explain = true;
        assert!(has_verify_request(&args));
    }

    #[test]
    fn decode_run_with_verify_and_out() {
        let header = Header::new(jsonwebtoken::Algorithm::HS256);
        let token = jwt_ops::encode_token(
            &header,
            &json!({ "sub": "tester" }),
            &EncodingKey::from_secret(b"secret"),
        )
        .expect("encode token");

        let dir = tempdir().expect("tempdir");
        let out_path = dir.path().join("decoded.json");

        let args = crate::cli::DecodeArgs {
            date: Some("utc".to_string()),
            verify: VerifyCommonArgs {
                secret: Some("secret".to_string()),
                key: None,
                jwks: None,
                key_format: None,
                kid: None,
                allow_single_jwk: false,
                project: None,
                key_id: None,
                key_name: None,
                try_all_keys: false,
                ignore_exp: true,
                leeway_secs: 30,
                iss: None,
                sub: None,
                aud: Vec::new(),
                require: Vec::new(),
                explain: true,
                alg: Some(JwtAlg::HS256),
            },
            out: Some(out_path.clone()),
            token,
        };

        let cfg = OutputConfig {
            mode: OutputMode::Json,
            quiet: true,
            no_color: true,
            verbose: false,
        };
        let code = run(true, None, args, cfg);
        assert_eq!(code, 0);
        let written = std::fs::read_to_string(&out_path).expect("read output");
        assert!(written.contains("\"ok\""));
    }
}
