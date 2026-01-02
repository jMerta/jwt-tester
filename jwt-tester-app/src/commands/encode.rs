use crate::claims;
use crate::cli::EncodeArgs;
use crate::error::{AppError, AppResult};
use crate::io_utils::read_json_value;
use crate::jwt_ops;
use crate::key_resolver::resolve_encoding_key;
use crate::output::{emit_err, emit_ok, CommandOutput, OutputConfig};
use jsonwebtoken::jwk::Jwk;
use serde_json::json;
use std::path::PathBuf;

pub fn run(
    no_persist: bool,
    data_dir: Option<PathBuf>,
    args: EncodeArgs,
    cfg: OutputConfig,
) -> i32 {
    let result = (|| -> AppResult<CommandOutput> {
        let (token, key_label) = encode_from_args(no_persist, data_dir, &args)?;
        write_token_output(&args.out, &token)?;
        Ok(build_command_output(token, key_label))
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

fn encode_from_args(
    no_persist: bool,
    data_dir: Option<PathBuf>,
    args: &EncodeArgs,
) -> AppResult<(String, String)> {
    let alg = jsonwebtoken::Algorithm::from(args.alg);
    let (key, key_label) = resolve_encoding_key(no_persist, data_dir, args)?;
    let claims = build_claims_from_args(args)?;
    let header = build_header_from_args(args, alg)?;
    let token = jwt_ops::encode_token(&header, &claims, &key)?;
    Ok((token, key_label))
}

fn build_claims_from_args(args: &EncodeArgs) -> AppResult<serde_json::Value> {
    let base_claims = parse_base_claims(args)?;
    let claim_files = load_claim_files(args)?;
    let standard = build_standard_claims(args);
    claims::build_claims(
        base_claims,
        claim_files,
        standard,
        args.claim.clone(),
        args.keep_payload_order,
    )
}

fn parse_base_claims(args: &EncodeArgs) -> AppResult<serde_json::Value> {
    match args.claims.as_deref() {
        Some(raw) => read_json_value(raw),
        None => Ok(serde_json::Value::Object(serde_json::Map::new())),
    }
}

fn load_claim_files(args: &EncodeArgs) -> AppResult<Vec<serde_json::Value>> {
    args.claim_file
        .iter()
        .map(|spec| read_json_value(spec))
        .collect()
}

fn build_standard_claims(args: &EncodeArgs) -> claims::StandardClaims {
    claims::StandardClaims {
        iss: args.iss.clone(),
        sub: args.sub.clone(),
        aud: args.aud.clone(),
        jti: args.jti.clone(),
        iat: args.iat.clone(),
        nbf: args.nbf.clone(),
        exp: args.exp.clone(),
        no_iat: args.no_iat,
    }
}

fn build_header_from_args(
    args: &EncodeArgs,
    alg: jsonwebtoken::Algorithm,
) -> AppResult<jsonwebtoken::Header> {
    let mut header = jsonwebtoken::Header::new(alg);
    if let Some(header_spec) = args.header.as_deref() {
        let h_val = read_json_value(header_spec)?;
        apply_header_overrides(&mut header, h_val, alg)?;
    }
    header.kid = args.kid.clone();
    if args.no_typ {
        header.typ = None;
    } else if let Some(typ) = &args.typ {
        header.typ = Some(typ.clone());
    } else {
        header.typ = Some("JWT".to_string());
    }
    Ok(header)
}

fn write_token_output(out_path: &Option<PathBuf>, token: &str) -> AppResult<()> {
    if let Some(out_path) = out_path {
        std::fs::write(out_path, token.as_bytes())
            .map_err(|e| AppError::internal(format!("failed to write {out_path:?}: {e}")))?;
    }
    Ok(())
}

fn build_command_output(token: String, key_label: String) -> CommandOutput {
    let text = token.clone();
    let data = json!({ "token": token, "key": key_label });
    CommandOutput::new(data, text)
}

fn apply_header_overrides(
    header: &mut jsonwebtoken::Header,
    value: serde_json::Value,
    alg: jsonwebtoken::Algorithm,
) -> AppResult<()> {
    let obj = value
        .as_object()
        .ok_or_else(|| AppError::invalid_claims("header JSON must be an object"))?;

    for (key, val) in obj {
        match key.as_str() {
            "typ" => header.typ = parse_opt_string(val, "typ")?,
            "kid" => header.kid = parse_opt_string(val, "kid")?,
            "cty" => header.cty = parse_opt_string(val, "cty")?,
            "jku" => header.jku = parse_opt_string(val, "jku")?,
            "jwk" => {
                if val.is_null() {
                    header.jwk = None;
                } else {
                    let jwk: Jwk = serde_json::from_value(val.clone())
                        .map_err(|e| AppError::invalid_claims(format!("invalid jwk: {e}")))?;
                    header.jwk = Some(jwk);
                }
            }
            "x5u" => header.x5u = parse_opt_string(val, "x5u")?,
            "x5c" => header.x5c = parse_opt_string_list(val, "x5c")?,
            "x5t" => header.x5t = parse_opt_string(val, "x5t")?,
            "x5t#S256" => header.x5t_s256 = parse_opt_string(val, "x5t#S256")?,
            "alg" => {
                let expected = format!("{:?}", alg);
                let provided = val
                    .as_str()
                    .ok_or_else(|| AppError::invalid_claims("header alg must be a string"))?;
                if !provided.eq_ignore_ascii_case(&expected) {
                    return Err(AppError::invalid_claims(format!(
                        "header alg '{provided}' does not match --alg {expected}"
                    )));
                }
            }
            other => {
                return Err(AppError::invalid_claims(format!(
                    "unsupported header field '{other}'"
                )));
            }
        }
    }
    Ok(())
}

fn parse_opt_string(value: &serde_json::Value, label: &str) -> AppResult<Option<String>> {
    if value.is_null() {
        return Ok(None);
    }
    value
        .as_str()
        .map(|s| Some(s.to_string()))
        .ok_or_else(|| AppError::invalid_claims(format!("{label} must be a string or null")))
}

fn parse_opt_string_list(value: &serde_json::Value, label: &str) -> AppResult<Option<Vec<String>>> {
    if value.is_null() {
        return Ok(None);
    }
    let arr = value.as_array().ok_or_else(|| {
        AppError::invalid_claims(format!("{label} must be an array of strings or null"))
    })?;
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        let s = item.as_str().ok_or_else(|| {
            AppError::invalid_claims(format!("{label} must contain only strings"))
        })?;
        out.push(s.to_string());
    }
    Ok(Some(out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::JwtAlg;
    use crate::output::OutputMode;
    use jsonwebtoken::Algorithm;
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn parse_opt_string_handles_null_and_string() {
        assert_eq!(parse_opt_string(&json!(null), "typ").unwrap(), None);
        assert_eq!(
            parse_opt_string(&json!("JWT"), "typ").unwrap(),
            Some("JWT".to_string())
        );
    }

    #[test]
    fn parse_opt_string_rejects_non_string() {
        let err = parse_opt_string(&json!(123), "typ").expect_err("expected error");
        assert!(err.to_string().contains("typ must be a string"));
    }

    #[test]
    fn parse_opt_string_list_handles_null_and_array() {
        assert_eq!(parse_opt_string_list(&json!(null), "x5c").unwrap(), None);
        assert_eq!(
            parse_opt_string_list(&json!(["a", "b"]), "x5c").unwrap(),
            Some(vec!["a".to_string(), "b".to_string()])
        );
    }

    #[test]
    fn parse_opt_string_list_rejects_invalid_inputs() {
        let err = parse_opt_string_list(&json!("nope"), "x5c").expect_err("expected error");
        assert!(err.to_string().contains("x5c must be an array"));
        let err = parse_opt_string_list(&json!(["ok", 1]), "x5c").expect_err("expected error");
        assert!(err.to_string().contains("x5c must contain only strings"));
    }

    #[test]
    fn apply_header_overrides_rejects_unknown_and_alg_mismatch() {
        let mut header = jsonwebtoken::Header::new(Algorithm::HS256);
        let err = apply_header_overrides(&mut header, json!({ "nope": "x" }), Algorithm::HS256)
            .expect_err("expected error");
        assert!(err.to_string().contains("unsupported header field"));

        let mut header = jsonwebtoken::Header::new(Algorithm::RS256);
        let err = apply_header_overrides(&mut header, json!({ "alg": "HS256" }), Algorithm::RS256)
            .expect_err("expected error");
        assert!(err.to_string().contains("does not match --alg"));
    }

    #[test]
    fn build_header_sets_typ_and_kid() {
        let args = EncodeArgs {
            secret: Some("secret".to_string()),
            key: None,
            key_format: None,
            project: None,
            key_id: None,
            key_name: None,
            alg: JwtAlg::HS256,
            claims: None,
            header: None,
            kid: Some("kid-1".to_string()),
            typ: None,
            no_typ: false,
            iss: None,
            sub: None,
            aud: Vec::new(),
            jti: None,
            iat: None,
            no_iat: false,
            nbf: None,
            exp: None,
            claim: Vec::new(),
            claim_file: Vec::new(),
            keep_payload_order: false,
            out: None,
        };
        let header = build_header_from_args(&args, Algorithm::HS256).expect("header");
        assert_eq!(header.kid.as_deref(), Some("kid-1"));
        assert_eq!(header.typ.as_deref(), Some("JWT"));
    }

    #[test]
    fn build_header_respects_no_typ() {
        let mut args = EncodeArgs {
            secret: Some("secret".to_string()),
            key: None,
            key_format: None,
            project: None,
            key_id: None,
            key_name: None,
            alg: JwtAlg::HS256,
            claims: None,
            header: None,
            kid: None,
            typ: None,
            no_typ: true,
            iss: None,
            sub: None,
            aud: Vec::new(),
            jti: None,
            iat: None,
            no_iat: false,
            nbf: None,
            exp: None,
            claim: Vec::new(),
            claim_file: Vec::new(),
            keep_payload_order: false,
            out: None,
        };
        let header = build_header_from_args(&args, Algorithm::HS256).expect("header");
        assert_eq!(header.typ, None);

        args.no_typ = false;
        args.typ = Some("JOSE".to_string());
        let header = build_header_from_args(&args, Algorithm::HS256).expect("header");
        assert_eq!(header.typ.as_deref(), Some("JOSE"));
    }

    #[test]
    fn parse_base_claims_errors_on_invalid_json() {
        let args = EncodeArgs {
            secret: Some("secret".to_string()),
            key: None,
            key_format: None,
            project: None,
            key_id: None,
            key_name: None,
            alg: JwtAlg::HS256,
            claims: Some("not-json".to_string()),
            header: None,
            kid: None,
            typ: None,
            no_typ: false,
            iss: None,
            sub: None,
            aud: Vec::new(),
            jti: None,
            iat: None,
            no_iat: false,
            nbf: None,
            exp: None,
            claim: Vec::new(),
            claim_file: Vec::new(),
            keep_payload_order: false,
            out: None,
        };
        let err = parse_base_claims(&args).expect_err("expected error");
        assert!(err.to_string().contains("invalid JSON"));
    }

    #[test]
    fn run_encode_writes_output_and_header_override() {
        let dir = tempdir().expect("tempdir");
        let out_path = dir.path().join("token.txt");
        let claim_file = dir.path().join("claims.json");
        std::fs::write(&claim_file, r#"{ "role": "admin" }"#).expect("write claims");

        let args = EncodeArgs {
            secret: Some("secret".to_string()),
            key: None,
            key_format: None,
            project: None,
            key_id: None,
            key_name: None,
            alg: JwtAlg::HS256,
            claims: Some("{\"sub\":\"user\"}".to_string()),
            header: Some("{\"typ\":\"JWT\",\"kid\":\"kid-1\"}".to_string()),
            kid: None,
            typ: None,
            no_typ: false,
            iss: None,
            sub: None,
            aud: Vec::new(),
            jti: None,
            iat: None,
            no_iat: false,
            nbf: None,
            exp: Some("+10m".to_string()),
            claim: Vec::new(),
            claim_file: vec![format!("@{}", claim_file.display())],
            keep_payload_order: false,
            out: Some(out_path.clone()),
        };

        let cfg = OutputConfig {
            mode: OutputMode::Json,
            quiet: true,
            no_color: true,
            verbose: false,
        };
        let code = run(true, None, args, cfg);
        assert_eq!(code, 0);
        let written = std::fs::read_to_string(&out_path).expect("read token");
        assert_eq!(written.trim().split('.').count(), 3);
    }
}
