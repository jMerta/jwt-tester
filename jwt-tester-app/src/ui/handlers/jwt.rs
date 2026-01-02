use super::super::AppState;
use super::api::{api_err, api_err_with_code, require_csrf, ApiList};
use super::types::{EncodeReq, InspectReq, VerifyReq};
use crate::claims;
use crate::cli::{EncodeArgs, JwtAlg, VerifyCommonArgs};
use crate::date_utils::{extract_dates, parse_date_mode};
use crate::error::{AppError, AppResult, ErrorKind};
use crate::jwt_ops::{self, VerifyOptions};
use crate::key_resolver::{
    resolve_encoding_key_with_vault, resolve_verification_key_with_vault, KeySource,
};
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use jsonwebtoken::Algorithm;
use serde_json::json;

pub(crate) async fn encode_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<EncodeReq>,
) -> impl IntoResponse {
    if require_csrf(&headers, state.csrf.as_str()).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(api_err("CSRF token missing/invalid")),
        )
            .into_response();
    }

    let EncodeReq {
        project,
        key_id,
        key_name,
        alg,
        claims,
        kid,
        typ,
        no_typ,
        iss,
        sub,
        aud,
        jti,
        iat,
        no_iat,
        nbf,
        exp,
    } = req;

    let alg = match parse_jwt_alg(&alg) {
        Ok(val) => val,
        Err(err) => {
            return (StatusCode::BAD_REQUEST, Json(api_err_with_code(&err))).into_response();
        }
    };

    let no_typ_flag = no_typ.unwrap_or(false);
    let no_iat_flag = no_iat.unwrap_or(false);
    let aud_list = aud.unwrap_or_default();

    let args = EncodeArgs {
        secret: None,
        key: None,
        key_format: None,
        project: Some(project),
        key_id,
        key_name,
        alg,
        claims: None,
        header: None,
        kid: kid.clone(),
        typ: typ.clone(),
        no_typ: no_typ_flag,
        iss: iss.clone(),
        sub: sub.clone(),
        aud: aud_list.clone(),
        jti: jti.clone(),
        iat: iat.clone(),
        no_iat: no_iat_flag,
        nbf: nbf.clone(),
        exp: exp.clone(),
        claim: Vec::new(),
        claim_file: Vec::new(),
        keep_payload_order: false,
        out: None,
    };

    let (key, key_source) = match resolve_encoding_key_with_vault(&state.vault, &args) {
        Ok(result) => result,
        Err(err) => {
            return (StatusCode::BAD_REQUEST, Json(api_err_with_code(&err))).into_response();
        }
    };

    let claims_raw = claims.unwrap_or_default();
    let base_claims = if claims_raw.trim().is_empty() {
        json!({})
    } else {
        match serde_json::from_str(&claims_raw) {
            Ok(val) => val,
            Err(err) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(api_err(format!("invalid claims JSON: {err}"))),
                )
                    .into_response();
            }
        }
    };

    let standard = claims::StandardClaims {
        iss,
        sub,
        aud: aud_list,
        jti,
        iat,
        nbf,
        exp,
        no_iat: no_iat_flag,
    };

    let claims = match claims::build_claims(base_claims, Vec::new(), standard, Vec::new(), false) {
        Ok(val) => val,
        Err(err) => {
            return (StatusCode::BAD_REQUEST, Json(api_err_with_code(&err))).into_response();
        }
    };

    let mut header = jsonwebtoken::Header::new(Algorithm::from(alg));
    header.kid = kid;
    if no_typ_flag {
        header.typ = None;
    } else if let Some(typ) = typ {
        header.typ = Some(typ);
    } else {
        header.typ = Some("JWT".to_string());
    }

    match jwt_ops::encode_token(&header, &claims, &key) {
        Ok(token) => Json(ApiList {
            ok: true,
            data: json!({ "token": token, "key_source": key_source }),
        })
        .into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(api_err_with_code(&err))).into_response(),
    }
}

pub(crate) async fn verify_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<VerifyReq>,
) -> impl IntoResponse {
    if require_csrf(&headers, state.csrf.as_str()).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(api_err("CSRF token missing/invalid")),
        )
            .into_response();
    }

    let VerifyReq {
        project,
        key_id,
        key_name,
        alg,
        token,
        try_all_keys,
        ignore_exp,
        leeway_secs,
        iss,
        sub,
        aud,
        require,
        explain,
    } = req;

    let alg = match parse_jwt_alg_opt(alg.as_deref()) {
        Ok(val) => val,
        Err(err) => {
            return (StatusCode::BAD_REQUEST, Json(api_err_with_code(&err))).into_response();
        }
    };
    let resolved_alg = match resolve_verify_alg(alg, &token) {
        Ok(val) => val,
        Err(err) => {
            return (StatusCode::BAD_REQUEST, Json(api_err_with_code(&err))).into_response();
        }
    };

    let aud_list = aud.unwrap_or_default();
    let require_list = require.unwrap_or_default();

    let args = VerifyCommonArgs {
        secret: None,
        key: None,
        jwks: None,
        key_format: None,
        kid: None,
        allow_single_jwk: false,
        project: Some(project),
        key_id,
        key_name,
        try_all_keys: try_all_keys.unwrap_or(false),
        ignore_exp: ignore_exp.unwrap_or(false),
        leeway_secs: leeway_secs.unwrap_or(30),
        iss: iss.clone(),
        sub: sub.clone(),
        aud: aud_list.clone(),
        require: require_list.clone(),
        explain: explain.unwrap_or(false),
        alg,
    };

    let key_source =
        match resolve_verification_key_with_vault(&state.vault, &args, &token, resolved_alg.alg) {
            Ok(source) => source,
            Err(err) => {
                return (StatusCode::BAD_REQUEST, Json(api_err_with_code(&err))).into_response();
            }
        };

    let verify_opts = VerifyOptions {
        alg: resolved_alg.alg,
        leeway_secs: args.leeway_secs,
        ignore_exp: args.ignore_exp,
        iss,
        sub,
        aud: aud_list,
        require: require_list,
    };

    let source_label = key_source_label(&key_source);
    let build_success = |claims| {
        let mut info = json!({ "valid": true, "claims": claims });
        if args.explain {
            info["explain"] = json!({
                "alg": format!("{:?}", resolved_alg.alg),
                "alg_inferred": resolved_alg.inferred,
                "key_source": source_label.clone(),
                "iss": args.iss,
                "sub": args.sub,
                "aud": args.aud,
                "leeway_secs": args.leeway_secs,
                "ignore_exp": args.ignore_exp,
                "require": args.require,
            });
        }
        Json(ApiList {
            ok: true,
            data: info,
        })
        .into_response()
    };

    match key_source {
        KeySource::Single(key, _label) => match jwt_ops::verify_token(&token, &key, verify_opts) {
            Ok(token_data) => build_success(token_data.claims),
            Err(err) => (StatusCode::BAD_REQUEST, Json(api_err_with_code(&err))).into_response(),
        },
        KeySource::Multiple(keys, _label) => {
            let mut last_sig_err: Option<AppError> = None;
            for key in keys {
                match jwt_ops::verify_token(&token, &key, verify_opts.clone()) {
                    Ok(token_data) => return build_success(token_data.claims),
                    Err(err) => {
                        if matches!(err.kind, ErrorKind::InvalidSignature) {
                            last_sig_err = Some(err);
                            continue;
                        }
                        return (StatusCode::BAD_REQUEST, Json(api_err_with_code(&err)))
                            .into_response();
                    }
                }
            }
            let err = last_sig_err.unwrap_or_else(|| {
                AppError::invalid_signature("signature invalid for all candidate keys")
            });
            (StatusCode::BAD_REQUEST, Json(api_err_with_code(&err))).into_response()
        }
    }
}

pub(crate) async fn inspect_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<InspectReq>,
) -> impl IntoResponse {
    if require_csrf(&headers, state.csrf.as_str()).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(api_err("CSRF token missing/invalid")),
        )
            .into_response();
    }

    let date_mode = match parse_date_mode(req.date) {
        Ok(mode) => mode,
        Err(err) => {
            return (StatusCode::BAD_REQUEST, Json(api_err_with_code(&err))).into_response();
        }
    };

    let decoded = match jwt_ops::decode_unverified(&req.token) {
        Ok(val) => val,
        Err(err) => {
            return (StatusCode::BAD_REQUEST, Json(api_err_with_code(&err))).into_response();
        }
    };

    let header = match jwt_ops::decode_header_only(&req.token) {
        Ok(val) => val,
        Err(err) => {
            return (StatusCode::BAD_REQUEST, Json(api_err_with_code(&err))).into_response();
        }
    };

    let dates = match extract_dates(&decoded.payload_json, date_mode) {
        Ok(val) => val,
        Err(err) => {
            return (StatusCode::BAD_REQUEST, Json(api_err_with_code(&err))).into_response();
        }
    };

    let segments: Vec<&str> = req.token.trim().split('.').collect();
    let sizes = json!({
        "token_len": req.token.trim().len(),
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
        "segments": if req.show_segments.unwrap_or(false) { Some(segments) } else { None },
    });

    Json(ApiList { ok: true, data }).into_response()
}

fn parse_jwt_alg(raw: &str) -> AppResult<JwtAlg> {
    match raw.trim().to_lowercase().as_str() {
        "hs256" => Ok(JwtAlg::HS256),
        "hs384" => Ok(JwtAlg::HS384),
        "hs512" => Ok(JwtAlg::HS512),
        "rs256" => Ok(JwtAlg::RS256),
        "rs384" => Ok(JwtAlg::RS384),
        "rs512" => Ok(JwtAlg::RS512),
        "ps256" => Ok(JwtAlg::PS256),
        "ps384" => Ok(JwtAlg::PS384),
        "ps512" => Ok(JwtAlg::PS512),
        "es256" => Ok(JwtAlg::ES256),
        "es384" => Ok(JwtAlg::ES384),
        "eddsa" => Ok(JwtAlg::EdDSA),
        _ => Err(AppError::invalid_key("unsupported algorithm")),
    }
}

fn parse_jwt_alg_opt(raw: Option<&str>) -> AppResult<Option<JwtAlg>> {
    let val = match raw {
        Some(value) if !value.trim().is_empty() => value.trim(),
        _ => return Ok(None),
    };
    if val.eq_ignore_ascii_case("auto") {
        return Ok(None);
    }
    parse_jwt_alg(val).map(Some)
}

#[derive(Clone, Copy)]
struct ResolvedAlg {
    alg: Algorithm,
    inferred: bool,
}

fn resolve_verify_alg(alg: Option<JwtAlg>, token: &str) -> AppResult<ResolvedAlg> {
    if let Some(val) = alg {
        return Ok(ResolvedAlg {
            alg: Algorithm::from(val),
            inferred: false,
        });
    }
    let header = jwt_ops::decode_header_only(token)?;
    Ok(ResolvedAlg {
        alg: header.alg,
        inferred: true,
    })
}

fn key_source_label(source: &KeySource) -> String {
    match source {
        KeySource::Single(_, label) => label.clone(),
        KeySource::Multiple(_, label) => label.clone(),
    }
}
