use crate::error::{AppError, AppResult};
use jsonwebtoken::jwk::{Jwk, JwkSet};
use jsonwebtoken::DecodingKey;

pub fn select_jwk(
    jwks_json: &str,
    token_kid: Option<String>,
    explicit_kid: Option<String>,
    allow_single: bool,
) -> AppResult<Jwk> {
    let set: JwkSet = serde_json::from_str(jwks_json)
        .map_err(|e| AppError::invalid_key(format!("invalid JWKS JSON: {e}")))?;
    if set.keys.is_empty() {
        return Err(AppError::invalid_key("JWKS contains no keys"));
    }

    let kid = explicit_kid.or(token_kid);
    if let Some(kid) = kid {
        return set
            .find(&kid)
            .cloned()
            .ok_or_else(|| AppError::invalid_key(format!("no JWKS key found for kid {kid}")));
    }

    if allow_single && set.keys.len() == 1 {
        return Ok(set.keys[0].clone());
    }

    Err(AppError::invalid_key(
        "JWKS has multiple keys; provide --kid or use --allow-single-jwk",
    ))
}

pub fn decoding_key_from_jwk(jwk: &Jwk) -> AppResult<DecodingKey> {
    DecodingKey::from_jwk(jwk).map_err(AppError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_jwk_by_kid() {
        let jwks = r#"{"keys":[{"kty":"oct","kid":"a","k":"aGVsbG8"},{"kty":"oct","kid":"b","k":"d29ybGQ"}]}"#;
        let jwk = select_jwk(jwks, None, Some("b".to_string()), false).unwrap();
        assert_eq!(jwk.common.key_id.as_deref(), Some("b"));
    }

    #[test]
    fn select_jwk_requires_kid_when_multiple() {
        let jwks = r#"{"keys":[{"kty":"oct","kid":"a","k":"aGVsbG8"},{"kty":"oct","kid":"b","k":"d29ybGQ"}]}"#;
        let err = select_jwk(jwks, None, None, false).unwrap_err();
        assert_eq!(err.kind, crate::error::ErrorKind::InvalidKey);
    }

    #[test]
    fn select_jwk_allows_single_without_kid() {
        let jwks = r#"{"keys":[{"kty":"oct","k":"aGVsbG8"}]}"#;
        let jwk = select_jwk(jwks, None, None, true).unwrap();
        assert!(jwk.common.key_id.is_none());
    }
}
