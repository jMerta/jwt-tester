use super::super::AppState;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use std::path::Path as FsPath;

fn content_type_for(path: &FsPath) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        _ => "application/octet-stream",
    }
}

fn missing_assets_html(path: &FsPath, err: &std::io::Error) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>jwt-tester UI missing assets</title>
    <style>
      body {{ font-family: "Trebuchet MS", "Segoe UI", sans-serif; margin: 40px; color: #1f2937; }}
      code {{ background: #f3f4f6; padding: 2px 6px; border-radius: 6px; }}
      .card {{ border: 1px solid #e5e7eb; padding: 16px 20px; border-radius: 12px; max-width: 720px; }}
    </style>
  </head>
  <body>
    <div class="card">
      <h1>UI assets not found</h1>
      <p>The React UI build output could not be loaded.</p>
      <p>The UI expects prebuilt assets in <code>ui/dist</code>. Seeing this page
         means the assets directory is missing or overridden.</p>
      <p>Expected file: <code>{}</code></p>
      <p>Build it by running:</p>
      <pre><code>cd jwt-tester-app/ui
npm install
npm run build</code></pre>
      <p>You can also force a rebuild via <code>jwt-tester ui --build</code>, or
         run <code>jwt-tester ui --dev</code> and open
         <code>http://127.0.0.1:5173</code> for hot reload.</p>
      <p>You can also point <code>JWT_TESTER_UI_ASSETS_DIR</code> at a prebuilt
         <code>dist</code> directory.</p>
      <p>Error: <code>{}</code></p>
    </div>
  </body>
</html>"#,
        path.display(),
        err
    )
}

pub(crate) async fn index(State(state): State<AppState>) -> impl IntoResponse {
    let index_path = super::super::assets_root().join("index.html");
    match tokio::fs::read_to_string(&index_path).await {
        Ok(html) => {
            let html = html.replace("{csrf}", state.csrf.as_str());
            Html(html).into_response()
        }
        Err(err) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Html(missing_assets_html(&index_path, &err)),
        )
            .into_response(),
    }
}

pub(crate) async fn asset(Path(path): Path<String>) -> impl IntoResponse {
    if path.contains("..") || path.contains('\\') {
        return (StatusCode::BAD_REQUEST, "invalid asset path").into_response();
    }
    let full_path = super::super::assets_root().join("assets").join(&path);
    match tokio::fs::read(&full_path).await {
        Ok(bytes) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", content_type_for(&full_path))
            .body(Body::from(bytes))
            .unwrap_or_else(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to build response",
                )
                    .into_response()
            }),
        Err(_) => (StatusCode::NOT_FOUND, "asset not found").into_response(),
    }
}
