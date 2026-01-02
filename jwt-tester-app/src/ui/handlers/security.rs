use super::api::api_err;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;

pub(crate) async fn security_headers(
    req: Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Response {
    // Basic localhost UI hardening (MVP):
    // - set security headers
    // - reject cross-origin modifying requests if Origin is present and mismatched
    let method = req.method().clone();
    let origin = req
        .headers()
        .get("origin")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // We can't reliably access State here without custom middleware wiring; keep this check in handlers via CSRF.
    // (CSRF token is required for POST/DELETE and is only embedded in our served HTML.)
    if matches!(method.as_str(), "POST" | "PUT" | "PATCH" | "DELETE") {
        if let Some(o) = origin {
            if !o.starts_with("http://127.0.0.1") && !o.starts_with("http://localhost") {
                // conservative: block non-local origins
                let body = Json(api_err("Cross-origin request blocked"));
                return (StatusCode::FORBIDDEN, body).into_response();
            }
        }
    }

    let mut res = next.run(req).await;
    let headers = res.headers_mut();

    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("Referrer-Policy", "no-referrer".parse().unwrap());
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'; script-src 'self'; style-src 'self'; connect-src 'self'; base-uri 'none'; frame-ancestors 'none'"
            .parse()
            .unwrap(),
    );

    res
}
