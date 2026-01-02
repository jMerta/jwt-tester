use super::format::{decoding_key_from_bytes, detect_key_format, encoding_key_from_bytes};
use super::project::{expected_kind, resolve_project_key_single, resolve_project_keys};
use crate::cli::{EncodeArgs, VerifyCommonArgs};
use crate::error::{AppError, AppResult};
use crate::io_utils::{read_input, read_input_bytes};
use crate::jwks;
use crate::jwt_ops;
use crate::vault::{Vault, VaultConfig};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey};
use std::path::PathBuf;

#[derive(Clone)]
pub enum KeySource {
    Single(DecodingKey, String),
    Multiple(Vec<DecodingKey>, String),
}

pub fn resolve_verification_key(
    no_persist: bool,
    data_dir: Option<PathBuf>,
    args: &VerifyCommonArgs,
    token: &str,
    alg: Algorithm,
) -> AppResult<KeySource> {
    let vault = Vault::open(VaultConfig {
        no_persist,
        data_dir,
    })
    .map_err(|e| AppError::invalid_key(e.to_string()))?;
    resolve_verification_key_with_vault(&vault, args, token, alg)
}

pub fn resolve_verification_key_with_vault(
    vault: &Vault,
    args: &VerifyCommonArgs,
    token: &str,
    alg: Algorithm,
) -> AppResult<KeySource> {
    let direct = args.secret.is_some() || args.key.is_some() || args.jwks.is_some();
    if direct {
        if args.try_all_keys {
            return Err(AppError::invalid_key(
                "--try-all-keys is only valid with --project",
            ));
        }
        if let Some(jwks_spec) = &args.jwks {
            let jwks_raw = read_input(jwks_spec)?;
            let header = jwt_ops::decode_header_only(token)?;
            let jwk = jwks::select_jwk(
                &jwks_raw,
                header.kid,
                args.kid.clone(),
                args.allow_single_jwk,
            )?;
            let key = jwks::decoding_key_from_jwk(&jwk)?;
            return Ok(KeySource::Single(key, "jwks".to_string()));
        }

        if args.secret.is_some() && args.key.is_some() {
            return Err(AppError::invalid_key(
                "provide only one of --secret or --key",
            ));
        }

        if let Some(secret) = &args.secret {
            if !matches!(alg, Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512) {
                return Err(AppError::invalid_key(
                    "--secret is only valid with HS256/384/512",
                ));
            }
            let secret = read_input_bytes(secret)?;
            let key = DecodingKey::from_secret(&secret);
            return Ok(KeySource::Single(key, "secret".to_string()));
        }

        if let Some(key_spec) = &args.key {
            if matches!(alg, Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512) {
                return Err(AppError::invalid_key(
                    "--key is only valid with RSA/PS/EC/EdDSA algorithms",
                ));
            }
            let bytes = read_input_bytes(key_spec)?;
            let format = args.key_format.unwrap_or_else(|| detect_key_format(&bytes));
            let key = decoding_key_from_bytes(alg, &bytes, format)?;
            return Ok(KeySource::Single(key, "key".to_string()));
        }
    }

    let project = args
        .project
        .clone()
        .ok_or_else(|| AppError::invalid_key("provide --project or a direct key input"))?;
    let header = jwt_ops::decode_header_only(token)?;
    let token_kid = header.kid.clone();
    let (project_entry, candidates) = resolve_project_keys(
        vault,
        &project,
        &args.key_id,
        &args.key_name,
        token_kid,
        args.try_all_keys,
    )?;

    let expected_kind = expected_kind(alg);
    let mut matching_keys = Vec::new();
    for key in candidates {
        if key.kind.to_lowercase() != expected_kind {
            continue;
        }
        let material = vault
            .get_key_material(&key.id)
            .map_err(|e| AppError::invalid_key(e.to_string()))?;
        let bytes = material.into_bytes();
        let format = detect_key_format(&bytes);
        let key = decoding_key_from_bytes(alg, &bytes, format)?;
        matching_keys.push(key);
    }

    if matching_keys.is_empty() {
        return Err(AppError::invalid_key(format!(
            "no keys of kind '{}' found in project {}",
            expected_kind, project_entry.name
        )));
    }

    if matching_keys.len() == 1 {
        Ok(KeySource::Single(
            matching_keys.remove(0),
            "vault".to_string(),
        ))
    } else {
        Ok(KeySource::Multiple(matching_keys, "vault".to_string()))
    }
}

pub fn resolve_encoding_key(
    no_persist: bool,
    data_dir: Option<PathBuf>,
    args: &EncodeArgs,
) -> AppResult<(EncodingKey, String)> {
    let vault = Vault::open(VaultConfig {
        no_persist,
        data_dir,
    })
    .map_err(|e| AppError::invalid_key(e.to_string()))?;
    resolve_encoding_key_with_vault(&vault, args)
}

pub fn resolve_encoding_key_with_vault(
    vault: &Vault,
    args: &EncodeArgs,
) -> AppResult<(EncodingKey, String)> {
    let direct = args.secret.is_some() || args.key.is_some();
    if direct {
        if args.secret.is_some() && args.key.is_some() {
            return Err(AppError::invalid_key(
                "provide only one of --secret or --key",
            ));
        }

        if let Some(secret) = &args.secret {
            let alg = Algorithm::from(args.alg);
            if !matches!(alg, Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512) {
                return Err(AppError::invalid_key(
                    "--secret is only valid with HS256/384/512",
                ));
            }
            let secret = read_input_bytes(secret)?;
            let key = EncodingKey::from_secret(&secret);
            return Ok((key, "secret".to_string()));
        }

        if let Some(key_spec) = &args.key {
            let alg = Algorithm::from(args.alg);
            if matches!(alg, Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512) {
                return Err(AppError::invalid_key(
                    "--key is only valid with RSA/PS/EC/EdDSA algorithms",
                ));
            }
            let bytes = read_input_bytes(key_spec)?;
            let format = args.key_format.unwrap_or_else(|| detect_key_format(&bytes));
            let key = encoding_key_from_bytes(alg, &bytes, format)?;
            return Ok((key, "key".to_string()));
        }
    }

    let project = args
        .project
        .clone()
        .ok_or_else(|| AppError::invalid_key("provide --project or a direct key input"))?;
    let (_project_entry, key) =
        resolve_project_key_single(vault, &project, &args.key_id, &args.key_name)?;
    let expected_kind = expected_kind(Algorithm::from(args.alg));
    if key.kind.to_lowercase() != expected_kind {
        return Err(AppError::invalid_key(format!(
            "key kind '{}' does not match algorithm {:?}",
            key.kind,
            Algorithm::from(args.alg)
        )));
    }

    let material = vault
        .get_key_material(&key.id)
        .map_err(|e| AppError::invalid_key(e.to_string()))?;
    let bytes = material.into_bytes();
    let format = detect_key_format(&bytes);
    let key = encoding_key_from_bytes(Algorithm::from(args.alg), &bytes, format)?;
    Ok((key, "vault".to_string()))
}

#[cfg(test)]
mod tests {
    use super::{resolve_verification_key_with_vault, KeySource};
    use crate::cli::{JwtAlg, VerifyCommonArgs};
    use crate::jwt_ops::{self, VerifyOptions};
    use crate::vault::{KeyEntryInput, ProjectInput, Vault, VaultConfig};
    use jsonwebtoken::{Algorithm, EncodingKey, Header};
    use serde_json::json;

    fn build_vault() -> (Vault, String) {
        let vault = Vault::open(VaultConfig {
            no_persist: true,
            data_dir: None,
        })
        .expect("open vault");
        let project = vault
            .add_project(ProjectInput {
                name: "proj".to_string(),
                description: None,
                tags: Vec::new(),
            })
            .expect("add project");
        (vault, project.id)
    }

    fn add_hmac_key(vault: &Vault, project_id: &str, name: &str, kid: Option<&str>, secret: &str) {
        vault
            .add_key(KeyEntryInput {
                project_id: project_id.to_string(),
                name: name.to_string(),
                kind: "hmac".to_string(),
                secret: secret.to_string(),
                kid: kid.map(|s| s.to_string()),
                description: None,
                tags: Vec::new(),
            })
            .expect("add key");
    }

    fn make_token(secret: &str, kid: Option<&str>) -> String {
        let mut header = Header::new(Algorithm::HS256);
        header.kid = kid.map(|s| s.to_string());
        jwt_ops::encode_token(
            &header,
            &json!({"sub": "test"}),
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .expect("encode token")
    }

    fn base_args(project: &str, try_all: bool) -> VerifyCommonArgs {
        VerifyCommonArgs {
            secret: None,
            key: None,
            jwks: None,
            key_format: None,
            kid: None,
            allow_single_jwk: false,
            project: Some(project.to_string()),
            key_id: None,
            key_name: None,
            try_all_keys: try_all,
            ignore_exp: false,
            leeway_secs: 30,
            iss: None,
            sub: None,
            aud: Vec::new(),
            require: Vec::new(),
            explain: false,
            alg: Some(JwtAlg::HS256),
        }
    }

    #[test]
    fn resolve_with_kid_selects_single_key() {
        let (vault, project_id) = build_vault();
        add_hmac_key(&vault, &project_id, "k1", Some("kid1"), "secret1");
        add_hmac_key(&vault, &project_id, "k2", Some("kid2"), "secret2");

        let token = make_token("secret1", Some("kid1"));
        let args = base_args("proj", false);
        let source = resolve_verification_key_with_vault(&vault, &args, &token, Algorithm::HS256)
            .expect("resolve key");

        match source {
            KeySource::Single(key, _) => {
                let opts = VerifyOptions {
                    alg: Algorithm::HS256,
                    leeway_secs: 0,
                    ignore_exp: true,
                    iss: None,
                    sub: None,
                    aud: Vec::new(),
                    require: Vec::new(),
                };
                let data = jwt_ops::verify_token(&token, &key, opts).expect("verify token");
                assert_eq!(data.claims["sub"], "test");
            }
            _ => panic!("expected single key"),
        }
    }

    #[test]
    fn resolve_with_kid_try_all_includes_other_keys() {
        let (vault, project_id) = build_vault();
        add_hmac_key(&vault, &project_id, "k1", Some("kid1"), "secret1");
        add_hmac_key(&vault, &project_id, "k2", Some("kid2"), "secret2");

        let token = make_token("secret1", Some("kid1"));
        let args = base_args("proj", true);
        let source = resolve_verification_key_with_vault(&vault, &args, &token, Algorithm::HS256)
            .expect("resolve key");

        match source {
            KeySource::Multiple(keys, _) => {
                assert_eq!(keys.len(), 2);
                let opts = VerifyOptions {
                    alg: Algorithm::HS256,
                    leeway_secs: 0,
                    ignore_exp: true,
                    iss: None,
                    sub: None,
                    aud: Vec::new(),
                    require: Vec::new(),
                };
                let data = jwt_ops::verify_token(&token, &keys[0], opts).expect("verify token");
                assert_eq!(data.claims["sub"], "test");
            }
            _ => panic!("expected multiple keys"),
        }
    }

    #[test]
    fn resolve_with_missing_kid_errors() {
        let (vault, project_id) = build_vault();
        add_hmac_key(&vault, &project_id, "k1", Some("kid1"), "secret1");

        let token = make_token("secret1", Some("missing"));
        let args = base_args("proj", false);
        let err = match resolve_verification_key_with_vault(&vault, &args, &token, Algorithm::HS256)
        {
            Ok(_) => panic!("expected error"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("no key with kid"));
    }
}
