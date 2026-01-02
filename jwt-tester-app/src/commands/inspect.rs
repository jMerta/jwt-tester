use crate::cli::InspectArgs;
use crate::date_utils::{extract_dates, parse_date_mode};
use crate::error::AppResult;
use crate::io_utils::read_input;
use crate::jwt_ops;
use crate::output::{emit_err, emit_ok, CommandOutput, OutputConfig};
use serde_json::json;

pub fn run(args: InspectArgs, cfg: OutputConfig) -> i32 {
    let result = (|| -> AppResult<CommandOutput> {
        let token = read_input(&args.token)?;
        let decoded = jwt_ops::decode_unverified(&token)?;
        let header = jwt_ops::decode_header_only(&token)?;
        let date_mode = parse_date_mode(args.date)?;
        let dates = extract_dates(&decoded.payload_json, date_mode)?;

        let segments: Vec<&str> = token.trim().split('.').collect();
        let sizes = json!({
            "token_len": token.trim().len(),
            "header_len": segments.first().map(|s| s.len()).unwrap_or(0),
            "payload_len": segments.get(1).map(|s| s.len()).unwrap_or(0),
            "signature_len": segments.get(2).map(|s| s.len()).unwrap_or(0),
        });

        let data = json!({
            "header": decoded.header_json,
            "payload": decoded.payload_json,
            "summary": {
                "alg": format!("{:?}", header.alg),
                "kid": header.kid,
                "typ": header.typ,
                "sizes": sizes,
            },
            "dates": dates.json,
            "segments": if args.show_segments { Some(segments.clone()) } else { None },
        });

        let mut text = String::new();
        text.push_str("UNVERIFIED\n");
        text.push_str(&format!("alg: {:?}\n", header.alg));
        if let Some(kid) = header.kid {
            text.push_str(&format!("kid: {}\n", kid));
        }
        if let Some(typ) = header.typ {
            text.push_str(&format!("typ: {}\n", typ));
        }
        text.push_str(&format!("token length: {}\n", token.trim().len()));
        if args.show_segments {
            text.push_str("segments:\n");
            for (idx, seg) in segments.iter().enumerate() {
                text.push_str(&format!("  [{}] {}\n", idx, seg));
            }
        }
        if !dates.lines.is_empty() {
            text.push_str("dates:\n");
            text.push_str(&dates.lines.join("\n"));
            text.push('\n');
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

#[cfg(test)]
mod tests {
    use super::run;
    use crate::cli::InspectArgs;
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
    fn inspect_run_returns_success() {
        let token = make_token();
        let args = InspectArgs {
            date: Some("utc".to_string()),
            show_segments: true,
            token,
        };
        let code = run(args, cfg());
        assert_eq!(code, 0);
    }
}
