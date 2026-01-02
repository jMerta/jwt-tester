use crate::error::{AppError, AppResult};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use jsonwebtoken::{
    decode, decode_header, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData,
    Validation,
};
use serde_json::Value;

#[derive(Debug)]
pub struct DecodedToken {
    pub header_json: Value,
    pub payload_json: Value,
}

#[derive(Clone)]
pub struct VerifyOptions {
    pub alg: Algorithm,
    pub leeway_secs: u64,
    pub ignore_exp: bool,
    pub iss: Option<String>,
    pub sub: Option<String>,
    pub aud: Vec<String>,
    pub require: Vec<String>,
}

pub fn decode_unverified(token: &str) -> AppResult<DecodedToken> {
    let parts: Vec<&str> = token.trim().split('.').collect();
    if parts.len() != 3 {
        return Err(AppError::invalid_token(
            "token must have 3 dot-separated segments",
        ));
    }
    let header_bytes = URL_SAFE_NO_PAD
        .decode(parts[0])
        .map_err(|e| AppError::invalid_token(format!("invalid base64url header segment: {e}")))?;
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|e| AppError::invalid_token(format!("invalid base64url payload segment: {e}")))?;

    let header_json: Value = serde_json::from_slice(&header_bytes)
        .map_err(|e| AppError::invalid_token(format!("header is not valid JSON: {e}")))?;
    let payload_json: Value = serde_json::from_slice(&payload_bytes)
        .map_err(|e| AppError::invalid_token(format!("payload is not valid JSON: {e}")))?;

    Ok(DecodedToken {
        header_json,
        payload_json,
    })
}

pub fn decode_header_only(token: &str) -> AppResult<Header> {
    decode_header(token).map_err(AppError::from)
}

pub fn verify_token(
    token: &str,
    key: &DecodingKey,
    opts: VerifyOptions,
) -> AppResult<TokenData<Value>> {
    let mut validation = Validation::new(opts.alg);
    validation.required_spec_claims.clear();
    validation.leeway = opts.leeway_secs;
    validation.validate_nbf = true;

    if opts.ignore_exp {
        validation.validate_exp = false;
    }

    if opts.aud.is_empty() {
        validation.validate_aud = false;
    } else {
        validation.set_audience(&opts.aud);
    }

    if let Some(iss) = opts.iss {
        validation.set_issuer(&[iss]);
    }

    if let Some(sub) = opts.sub {
        validation.sub = Some(sub);
    }

    let data = decode::<Value>(token.trim(), key, &validation).map_err(AppError::from)?;

    if !opts.require.is_empty() {
        let claims_obj = data
            .claims
            .as_object()
            .ok_or_else(|| AppError::invalid_claims("claims must be a JSON object"))?;
        for name in opts.require {
            if !claims_obj.contains_key(&name) {
                return Err(AppError::invalid_claims(format!(
                    "missing required claim: {name}"
                )));
            }
        }
    }

    Ok(data)
}

pub fn encode_token(header: &Header, claims: &Value, key: &EncodingKey) -> AppResult<String> {
    encode::<Value>(header, claims, key).map_err(AppError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorKind;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn now_ts() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
    }

    #[test]
    fn decode_unverified_rejects_bad_segments() {
        let err = decode_unverified("a.b").unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidToken);

        let err = decode_unverified("$$$.@@@.###").unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidToken);
    }

    #[test]
    fn decode_unverified_rejects_bad_json() {
        let header = URL_SAFE_NO_PAD.encode(b"notjson");
        let payload = URL_SAFE_NO_PAD.encode(b"{}");
        let token = format!("{header}.{payload}.sig");
        let err = decode_unverified(&token).unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidToken);
    }

    #[test]
    fn decode_header_and_verify_claims() {
        let header = Header::new(Algorithm::HS256);
        let claims = json!({
            "sub": "user",
            "exp": now_ts() + 3600
        });
        let token = encode_token(&header, &claims, &EncodingKey::from_secret(b"secret"))
            .expect("encode token");
        let decoded = decode_unverified(&token).expect("decode token");
        assert_eq!(decoded.payload_json["sub"], "user");

        let header = decode_header_only(&token).expect("decode header");
        assert_eq!(header.alg, Algorithm::HS256);
    }

    #[test]
    fn verify_token_requires_claims_and_allows_missing_exp() {
        let header = Header::new(Algorithm::HS256);
        let claims = json!({
            "sub": "user",
            "exp": now_ts() + 3600
        });
        let token = encode_token(&header, &claims, &EncodingKey::from_secret(b"secret"))
            .expect("encode token");

        let opts = VerifyOptions {
            alg: Algorithm::HS256,
            leeway_secs: 0,
            ignore_exp: false,
            iss: None,
            sub: None,
            aud: Vec::new(),
            require: vec!["role".to_string()],
        };
        let err = verify_token(&token, &DecodingKey::from_secret(b"secret"), opts).unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidClaims);

        let claims = json!({ "sub": "user" });
        let token = encode_token(&header, &claims, &EncodingKey::from_secret(b"secret"))
            .expect("encode token");
        let opts = VerifyOptions {
            alg: Algorithm::HS256,
            leeway_secs: 0,
            ignore_exp: false,
            iss: None,
            sub: None,
            aud: Vec::new(),
            require: Vec::new(),
        };
        let data =
            verify_token(&token, &DecodingKey::from_secret(b"secret"), opts).expect("verify token");
        assert_eq!(data.claims["sub"], "user");

        let opts = VerifyOptions {
            alg: Algorithm::HS256,
            leeway_secs: 0,
            ignore_exp: false,
            iss: None,
            sub: None,
            aud: Vec::new(),
            require: vec!["exp".to_string()],
        };
        let err = verify_token(&token, &DecodingKey::from_secret(b"secret"), opts).unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidClaims);
    }
}
