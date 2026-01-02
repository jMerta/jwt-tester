use super::super::AppState;
use crate::error::AppError;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct ApiOk {
    pub(super) ok: bool,
}

#[derive(Serialize)]
pub(super) struct ApiList<T> {
    pub(super) ok: bool,
    pub(super) data: T,
}

#[derive(Serialize)]
pub(super) struct ApiErr {
    pub(super) ok: bool,
    pub(super) error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) code: Option<String>,
}

pub(crate) async fn health() -> impl IntoResponse {
    Json(ApiOk { ok: true })
}

#[derive(Serialize)]
pub(super) struct ApiCsrf {
    pub(super) ok: bool,
    pub(super) csrf: String,
}

pub(crate) async fn csrf(State(state): State<AppState>) -> impl IntoResponse {
    Json(ApiCsrf {
        ok: true,
        csrf: state.csrf.as_str().to_string(),
    })
}

pub(super) fn api_err(error: impl Into<String>) -> ApiErr {
    ApiErr {
        ok: false,
        error: error.into(),
        code: None,
    }
}

pub(super) fn api_err_with_code(err: &AppError) -> ApiErr {
    ApiErr {
        ok: false,
        error: err.to_string(),
        code: Some(err.code().to_string()),
    }
}

pub(super) fn require_csrf(headers: &HeaderMap, expected: &str) -> Result<(), StatusCode> {
    match headers.get("x-csrf-token").and_then(|v| v.to_str().ok()) {
        Some(v) if v == expected => Ok(()),
        _ => Err(StatusCode::FORBIDDEN),
    }
}
