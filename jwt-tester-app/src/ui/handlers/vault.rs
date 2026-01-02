use super::super::AppState;
use super::api::{api_err, require_csrf, ApiList, ApiOk};
use super::types::{
    AddKeyReq, AddProjectReq, AddTokenReq, ExportReq, GenerateKeyReq, ImportReq, ProjectFilter,
    SetDefaultKeyReq,
};
use crate::keygen::{
    generate_key_material, parse_ec_curve, KeyGenSpec, DEFAULT_HMAC_BYTES, DEFAULT_RSA_BITS,
};
use crate::vault::{KeyEntryInput, ProjectInput, TokenEntryInput};
use crate::vault_export::ExportBundle;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

pub(crate) async fn list_projects(State(state): State<AppState>) -> impl IntoResponse {
    match state.vault.list_projects() {
        Ok(projects) => Json(ApiList {
            ok: true,
            data: projects,
        })
        .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(api_err(err.to_string())),
        )
            .into_response(),
    }
}

pub(crate) async fn add_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<AddProjectReq>,
) -> impl IntoResponse {
    if require_csrf(&headers, state.csrf.as_str()).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(api_err("CSRF token missing/invalid")),
        )
            .into_response();
    }

    match state.vault.add_project(ProjectInput {
        name: req.name,
        description: req.description,
        tags: req.tags.unwrap_or_default(),
    }) {
        Ok(saved) => Json(ApiList {
            ok: true,
            data: saved,
        })
        .into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(api_err(err.to_string()))).into_response(),
    }
}

pub(crate) async fn set_default_key(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<SetDefaultKeyReq>,
) -> impl IntoResponse {
    if require_csrf(&headers, state.csrf.as_str()).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(api_err("CSRF token missing/invalid")),
        )
            .into_response();
    }

    let project = match state.vault.find_project_by_id(&id) {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (StatusCode::BAD_REQUEST, Json(api_err("project not found"))).into_response();
        }
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(api_err(err.to_string())),
            )
                .into_response();
        }
    };

    if let Some(key_id) = req.key_id.as_deref() {
        match state.vault.list_keys(Some(&project.id)) {
            Ok(keys) if keys.iter().any(|k| k.id == key_id) => {}
            Ok(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(api_err("key not found in project")),
                )
                    .into_response();
            }
            Err(err) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(api_err(err.to_string())),
                )
                    .into_response();
            }
        }
    }

    match state
        .vault
        .set_default_key(&project.id, req.key_id.as_deref())
    {
        Ok(_) => Json(json!({
            "ok": true,
            "data": { "project_id": project.id, "default_key_id": req.key_id }
        }))
        .into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(api_err(err.to_string()))).into_response(),
    }
}

pub(crate) async fn delete_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if require_csrf(&headers, state.csrf.as_str()).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(api_err("CSRF token missing/invalid")),
        )
            .into_response();
    }

    match state.vault.delete_project(&id) {
        Ok(_) => Json(ApiOk { ok: true }).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(api_err(err.to_string()))).into_response(),
    }
}

pub(crate) async fn list_keys(
    State(state): State<AppState>,
    Query(filter): Query<ProjectFilter>,
) -> impl IntoResponse {
    match state.vault.list_keys(filter.project_id.as_deref()) {
        Ok(keys) => Json(ApiList {
            ok: true,
            data: keys,
        })
        .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(api_err(err.to_string())),
        )
            .into_response(),
    }
}

pub(crate) async fn add_key(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<AddKeyReq>,
) -> impl IntoResponse {
    if require_csrf(&headers, state.csrf.as_str()).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(api_err("CSRF token missing/invalid")),
        )
            .into_response();
    }

    let input = KeyEntryInput {
        project_id: req.project_id,
        name: req.name,
        kind: req.kind,
        secret: req.secret,
        kid: req.kid,
        description: req.description,
        tags: req.tags.unwrap_or_default(),
    };

    match state.vault.add_key(input) {
        Ok(saved) => Json(ApiList {
            ok: true,
            data: saved,
        })
        .into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(api_err(err.to_string()))).into_response(),
    }
}

pub(crate) async fn generate_key(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<GenerateKeyReq>,
) -> impl IntoResponse {
    if require_csrf(&headers, state.csrf.as_str()).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(api_err("CSRF token missing/invalid")),
        )
            .into_response();
    }

    let kind = req.kind.trim().to_ascii_lowercase();
    let spec = match kind.as_str() {
        "hmac" => KeyGenSpec::Hmac {
            bytes: req.hmac_bytes.unwrap_or(DEFAULT_HMAC_BYTES),
        },
        "rsa" => KeyGenSpec::Rsa {
            bits: req.rsa_bits.unwrap_or(DEFAULT_RSA_BITS),
        },
        "ec" => match parse_ec_curve(req.ec_curve.as_deref()) {
            Ok(curve) => KeyGenSpec::Ec { curve },
            Err(err) => {
                return (StatusCode::BAD_REQUEST, Json(api_err(err.to_string()))).into_response();
            }
        },
        "eddsa" => KeyGenSpec::EdDsa,
        other => {
            return (
                StatusCode::BAD_REQUEST,
                Json(api_err(format!(
                    "unsupported key kind '{other}' for generation"
                ))),
            )
                .into_response();
        }
    };

    let (secret, format) = match generate_key_material(spec) {
        Ok(secret) => (secret, if kind == "hmac" { "base64url" } else { "pem" }),
        Err(err) => {
            return (StatusCode::BAD_REQUEST, Json(api_err(err.to_string()))).into_response()
        }
    };

    let input = KeyEntryInput {
        project_id: req.project_id,
        name: req.name,
        kind,
        secret: secret.clone(),
        kid: req.kid,
        description: req.description,
        tags: req.tags.unwrap_or_default(),
    };

    match state.vault.add_key(input) {
        Ok(saved) => Json(ApiList {
            ok: true,
            data: json!({
                "key": saved,
                "material": secret,
                "format": format
            }),
        })
        .into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(api_err(err.to_string()))).into_response(),
    }
}

pub(crate) async fn delete_key(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if require_csrf(&headers, state.csrf.as_str()).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(api_err("CSRF token missing/invalid")),
        )
            .into_response();
    }

    match state.vault.delete_key(&id) {
        Ok(_) => Json(ApiOk { ok: true }).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(api_err(err.to_string()))).into_response(),
    }
}

pub(crate) async fn list_tokens(
    State(state): State<AppState>,
    Query(filter): Query<ProjectFilter>,
) -> impl IntoResponse {
    match state.vault.list_tokens(filter.project_id.as_deref()) {
        Ok(tokens) => Json(ApiList {
            ok: true,
            data: tokens,
        })
        .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(api_err(err.to_string())),
        )
            .into_response(),
    }
}

pub(crate) async fn reveal_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if require_csrf(&headers, state.csrf.as_str()).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(api_err("CSRF token missing/invalid")),
        )
            .into_response();
    }

    match state.vault.get_token_material(&id) {
        Ok(token) => Json(ApiList {
            ok: true,
            data: json!({ "token": token }),
        })
        .into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(api_err(err.to_string()))).into_response(),
    }
}

pub(crate) async fn add_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<AddTokenReq>,
) -> impl IntoResponse {
    if require_csrf(&headers, state.csrf.as_str()).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(api_err("CSRF token missing/invalid")),
        )
            .into_response();
    }

    let input = TokenEntryInput {
        project_id: req.project_id,
        name: req.name,
        token: req.token,
    };

    match state.vault.add_token(input) {
        Ok(saved) => Json(ApiList {
            ok: true,
            data: saved,
        })
        .into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(api_err(err.to_string()))).into_response(),
    }
}

pub(crate) async fn delete_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if require_csrf(&headers, state.csrf.as_str()).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(api_err("CSRF token missing/invalid")),
        )
            .into_response();
    }

    match state.vault.delete_token(&id) {
        Ok(_) => Json(ApiOk { ok: true }).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(api_err(err.to_string()))).into_response(),
    }
}

pub(crate) async fn export_vault(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ExportReq>,
) -> impl IntoResponse {
    if require_csrf(&headers, state.csrf.as_str()).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(api_err("CSRF token missing/invalid")),
        )
            .into_response();
    }

    match state.vault.export_bundle(&req.passphrase) {
        Ok(bundle) => {
            let bundle_json = match serde_json::to_string_pretty(&bundle) {
                Ok(text) => text,
                Err(err) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(api_err(format!("serialize bundle: {err}"))),
                    )
                        .into_response()
                }
            };
            Json(ApiList {
                ok: true,
                data: json!({ "bundle": bundle_json }),
            })
            .into_response()
        }
        Err(err) => (StatusCode::BAD_REQUEST, Json(api_err(err.to_string()))).into_response(),
    }
}

pub(crate) async fn import_vault(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ImportReq>,
) -> impl IntoResponse {
    if require_csrf(&headers, state.csrf.as_str()).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(api_err("CSRF token missing/invalid")),
        )
            .into_response();
    }

    let bundle: ExportBundle = match serde_json::from_str(&req.bundle) {
        Ok(bundle) => bundle,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(api_err(format!("invalid bundle JSON: {err}"))),
            )
                .into_response()
        }
    };

    match state
        .vault
        .import_bundle(&bundle, &req.passphrase, req.replace.unwrap_or(false))
    {
        Ok(()) => Json(ApiOk { ok: true }).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(api_err(err.to_string()))).into_response(),
    }
}
