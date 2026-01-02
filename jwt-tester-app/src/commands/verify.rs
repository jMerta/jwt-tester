use crate::cli::{JwtAlg, VerifyArgs, VerifyCommonArgs};
use crate::error::{AppError, AppResult, ErrorKind};
use crate::io_utils::read_input;
use crate::jwt_ops::{self, VerifyOptions};
use crate::key_resolver::{resolve_verification_key, KeySource};
use crate::output::{emit_err, emit_ok, CommandOutput, OutputConfig};
use serde_json::json;
use std::path::PathBuf;

pub fn run(
    no_persist: bool,
    data_dir: Option<PathBuf>,
    args: VerifyArgs,
    cfg: OutputConfig,
) -> i32 {
    let result = (|| -> AppResult<CommandOutput> {
        let token = read_input(&args.token)?;
        let outcome = verify_token_with_args(no_persist, data_dir, &args.verify, &token)?;
        Ok(CommandOutput::new(outcome.data, outcome.text))
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

pub struct VerifyOutcome {
    pub data: serde_json::Value,
    pub text: String,
}

pub fn verify_token_with_args(
    no_persist: bool,
    data_dir: Option<PathBuf>,
    args: &VerifyCommonArgs,
    token: &str,
) -> AppResult<VerifyOutcome> {
    let resolved = resolve_alg(args.alg, token)?;
    let key_source = resolve_verification_key(no_persist, data_dir, args, token, resolved.alg)?;
    let verify_opts = VerifyOptions {
        alg: resolved.alg,
        leeway_secs: args.leeway_secs,
        ignore_exp: args.ignore_exp,
        iss: args.iss.clone(),
        sub: args.sub.clone(),
        aud: args.aud.clone(),
        require: args.require.clone(),
    };

    let data = match key_source {
        KeySource::Single(key, label) => {
            let token_data = jwt_ops::verify_token(token, &key, verify_opts)?;
            let mut info = json!({
                "valid": true,
                "claims": token_data.claims,
            });
            if args.explain {
                info["explain"] = build_verify_explain(args, &label, resolved);
            }
            info
        }
        KeySource::Multiple(keys, label) => {
            let mut last_sig_err: Option<AppError> = None;
            for key in keys {
                match jwt_ops::verify_token(token, &key, verify_opts.clone()) {
                    Ok(token_data) => {
                        let mut info = json!({
                            "valid": true,
                            "claims": token_data.claims,
                        });
                        if args.explain {
                            info["explain"] = build_verify_explain(args, &label, resolved);
                        }
                        return Ok(VerifyOutcome {
                            data: info,
                            text: "OK".to_string(),
                        });
                    }
                    Err(err) => {
                        if matches!(err.kind, ErrorKind::InvalidSignature) {
                            last_sig_err = Some(err);
                            continue;
                        }
                        return Err(err);
                    }
                }
            }

            if let Some(err) = last_sig_err {
                return Err(err);
            }

            return Err(AppError::invalid_signature(
                "signature invalid for all candidate keys",
            ));
        }
    };

    Ok(VerifyOutcome {
        data,
        text: "OK".to_string(),
    })
}

#[derive(Clone, Copy)]
struct ResolvedAlg {
    alg: jsonwebtoken::Algorithm,
    inferred: bool,
}

fn resolve_alg(alg: Option<JwtAlg>, token: &str) -> AppResult<ResolvedAlg> {
    if let Some(val) = alg {
        return Ok(ResolvedAlg {
            alg: jsonwebtoken::Algorithm::from(val),
            inferred: false,
        });
    }
    let header = jwt_ops::decode_header_only(token)?;
    Ok(ResolvedAlg {
        alg: header.alg,
        inferred: true,
    })
}

fn build_verify_explain(
    args: &VerifyCommonArgs,
    key_source: &str,
    resolved: ResolvedAlg,
) -> serde_json::Value {
    json!({
        "alg": format!("{:?}", resolved.alg),
        "alg_inferred": resolved.inferred,
        "key_source": key_source,
        "iss": args.iss,
        "sub": args.sub,
        "aud": args.aud,
        "leeway_secs": args.leeway_secs,
        "ignore_exp": args.ignore_exp,
        "require": args.require,
    })
}

#[cfg(test)]
mod tests {
    use super::{build_verify_explain, resolve_alg};
    use crate::cli::{JwtAlg, VerifyCommonArgs};
    use crate::jwt_ops;
    use jsonwebtoken::{Algorithm, EncodingKey, Header};
    use serde_json::json;

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

    fn make_token() -> String {
        let header = Header::new(Algorithm::HS256);
        jwt_ops::encode_token(
            &header,
            &json!({ "sub": "tester" }),
            &EncodingKey::from_secret(b"secret"),
        )
        .expect("encode token")
    }

    #[test]
    fn resolve_alg_infers_from_header() {
        let token = make_token();
        let resolved = resolve_alg(None, &token).expect("resolve");
        assert_eq!(resolved.alg, Algorithm::HS256);
        assert!(resolved.inferred);
    }

    #[test]
    fn resolve_alg_uses_explicit_value() {
        let token = make_token();
        let resolved = resolve_alg(Some(JwtAlg::HS512), &token).expect("resolve");
        assert_eq!(resolved.alg, Algorithm::HS512);
        assert!(!resolved.inferred);
    }

    #[test]
    fn build_verify_explain_contains_expected_fields() {
        let mut args = base_args();
        args.iss = Some("issuer".to_string());
        args.aud = vec!["aud1".to_string()];
        let resolved = resolve_alg(Some(JwtAlg::HS256), &make_token()).expect("resolve");
        let explain = build_verify_explain(&args, "secret", resolved);
        assert_eq!(explain["key_source"], "secret");
        assert_eq!(explain["alg_inferred"], false);
        assert_eq!(explain["iss"], "issuer");
        assert_eq!(explain["aud"][0], "aud1");
    }

    #[test]
    fn verify_run_success() {
        let token = make_token();
        let args = crate::cli::VerifyArgs {
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
                alg: None,
            },
            token,
        };
        let cfg = crate::output::OutputConfig {
            mode: crate::output::OutputMode::Json,
            quiet: true,
            no_color: true,
            verbose: false,
        };
        let code = crate::commands::verify::run(true, None, args, cfg);
        assert_eq!(code, 0);
    }
}
