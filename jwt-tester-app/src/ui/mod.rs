mod handlers;

use crate::error::{AppError, AppResult};
use crate::output::{emit_ok, CommandOutput, OutputConfig};
use crate::vault::Vault;
use axum::routing::{delete, get, post};
use axum::Router;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::RngCore;
use std::ffi::OsString;
use std::future::Future;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::Stdio;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::process::{Child, Command};
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct UiConfig {
    pub host: IpAddr,
    pub port: u16,
    pub allow_remote: bool,
    pub no_persist: bool,
    pub data_dir: Option<PathBuf>,
    pub force_build: bool,
    pub dev_mode: bool,
    pub npm_path: Option<PathBuf>,
}

#[derive(Clone)]
pub(super) struct AppState {
    csrf: Arc<String>,
    vault: Vault,
}

const UI_ASSETS_ENV: &str = "JWT_TESTER_UI_ASSETS_DIR";
const UI_NPM_ENV: &str = "JWT_TESTER_NPM";
const UI_DEV_HOST: &str = "127.0.0.1";
const UI_DEV_PORT: u16 = 5173;

pub async fn run_ui(config: UiConfig, output: OutputConfig) -> AppResult<()> {
    validate_bind_target(config.host, config.allow_remote)?;
    if config.force_build {
        ensure_ui_assets(true, config.npm_path.as_deref()).await?;
    } else if !config.dev_mode {
        ensure_ui_assets(false, config.npm_path.as_deref()).await?;
    }

    let mut csrf_raw = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut csrf_raw);
    let csrf = URL_SAFE_NO_PAD.encode(csrf_raw);

    let vault = Vault::open(crate::vault::VaultConfig {
        no_persist: config.no_persist,
        data_dir: config.data_dir,
    })
    .map_err(|e| AppError::internal(format!("failed to open vault: {e}")))?;

    let listener = TcpListener::bind(SocketAddr::new(config.host, config.port))
        .await
        .map_err(|e| AppError::internal(format!("failed to bind UI: {e}")))?;
    let local_addr = listener
        .local_addr()
        .map_err(|e| AppError::internal(format!("failed to get UI address: {e}")))?;
    let base_url = format!("http://{}:{}/", local_addr.ip(), local_addr.port());
    let api_base = format!("http://{}:{}", local_addr.ip(), local_addr.port());

    let mut dev_server = if config.dev_mode {
        Some(spawn_ui_dev_server(&api_base, config.npm_path.as_deref()).await?)
    } else {
        None
    };

    let dev_url = config
        .dev_mode
        .then(|| format!("http://{}:{}/", UI_DEV_HOST, UI_DEV_PORT));

    info!("UI started at {base_url}");
    if let Some(url) = &dev_url {
        info!("UI dev server running at {url}");
    }
    let text = if output.quiet {
        String::new()
    } else if let Some(url) = &dev_url {
        format!("{url}\nAPI: {base_url}")
    } else {
        base_url.clone()
    };
    let payload = if let Some(url) = &dev_url {
        serde_json::json!({ "url": base_url, "dev_url": url })
    } else {
        serde_json::json!({ "url": base_url })
    };
    emit_ok(output, CommandOutput::new(payload, text));

    let state = AppState {
        csrf: Arc::new(csrf),
        vault,
    };

    let app = Router::new()
        .route("/", get(handlers::index))
        .route("/assets/*path", get(handlers::asset))
        .route("/api/health", get(handlers::health))
        .route("/api/csrf", get(handlers::csrf))
        .route("/api/jwt/encode", post(handlers::encode_token))
        .route("/api/jwt/verify", post(handlers::verify_token))
        .route("/api/jwt/inspect", post(handlers::inspect_token))
        .route(
            "/api/vault/projects",
            get(handlers::list_projects).post(handlers::add_project),
        )
        .route(
            "/api/vault/projects/:id/default-key",
            post(handlers::set_default_key),
        )
        .route("/api/vault/projects/:id", delete(handlers::delete_project))
        .route("/api/vault/export", post(handlers::export_vault))
        .route("/api/vault/import", post(handlers::import_vault))
        .route(
            "/api/vault/keys",
            get(handlers::list_keys).post(handlers::add_key),
        )
        .route("/api/vault/keys/generate", post(handlers::generate_key))
        .route("/api/vault/keys/:id", delete(handlers::delete_key))
        .route(
            "/api/vault/tokens",
            get(handlers::list_tokens).post(handlers::add_token),
        )
        .route(
            "/api/vault/tokens/:id/material",
            post(handlers::reveal_token),
        )
        .route("/api/vault/tokens/:id", delete(handlers::delete_token))
        .with_state(state)
        .layer(axum::middleware::from_fn(handlers::security_headers));

    let shutdown = async move {
        if let Err(err) = tokio::signal::ctrl_c().await {
            warn!("failed to install ctrl+c handler: {err}");
        } else {
            info!("UI shutdown requested (ctrl+c)");
        }
        if let Some(child) = dev_server.as_mut() {
            if let Err(err) = child.kill().await {
                warn!("failed to stop UI dev server: {err}");
            }
        }
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
        .map_err(|e| AppError::internal(format!("ui server failed: {e}")))?;
    Ok(())
}

fn assets_root() -> PathBuf {
    resolve_assets_root().0
}

fn resolve_assets_root() -> (PathBuf, bool) {
    match std::env::var(UI_ASSETS_ENV) {
        Ok(value) => (PathBuf::from(value), true),
        Err(_) => (
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("ui")
                .join("dist"),
            false,
        ),
    }
}

fn ui_source_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui")
}

async fn ensure_ui_assets(force_build: bool, npm_override: Option<&Path>) -> AppResult<()> {
    let (assets_root, assets_override) = resolve_assets_root();
    let ui_dir = ui_source_dir();
    let npm_override = npm_override.map(PathBuf::from);
    ensure_ui_assets_with(
        &assets_root,
        assets_override,
        force_build,
        &ui_dir,
        move |path| Box::pin(build_ui_assets(path, npm_override)),
    )
    .await
}

async fn ensure_ui_assets_with<F>(
    assets_root: &Path,
    assets_override: bool,
    force_build: bool,
    ui_dir: &Path,
    build_assets: F,
) -> AppResult<()>
where
    F: for<'a> FnOnce(&'a Path) -> Pin<Box<dyn Future<Output = AppResult<()>> + Send + 'a>>,
{
    let index_path = assets_root.join("index.html");
    if !force_build {
        if index_exists(&index_path).await? {
            return Ok(());
        }
        return Err(AppError::internal(format!(
            "UI assets missing at {}. Run `jwt-tester ui --build` or set {UI_ASSETS_ENV} to prebuilt assets.",
            index_path.display()
        )));
    }
    if assets_override {
        return Err(AppError::internal(format!(
            "Cannot rebuild UI assets while {UI_ASSETS_ENV} is set. Unset it to build from source.",
        )));
    }
    info!(
        "UI assets rebuild requested; running npm install/build in {}",
        ui_dir.display()
    );
    build_assets(ui_dir).await?;
    if index_exists(&index_path).await? {
        return Ok(());
    }
    Err(AppError::internal(format!(
        "UI assets still missing after build at {}. Try `npm run build` in {}.",
        index_path.display(),
        ui_dir.display()
    )))
}

async fn index_exists(path: &Path) -> AppResult<bool> {
    match tokio::fs::metadata(path).await {
        Ok(_) => Ok(true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(AppError::internal(format!(
            "failed to access UI assets at {}: {err}",
            path.display()
        ))),
    }
}

async fn build_ui_assets(ui_dir: &Path, npm_override: Option<PathBuf>) -> AppResult<()> {
    if !ui_dir.exists() {
        return Err(AppError::internal(format!(
            "UI source directory missing at {}. Set {UI_ASSETS_ENV} to prebuilt assets or reinstall the UI sources.",
            ui_dir.display()
        )));
    }
    run_npm(ui_dir, &["install"], npm_override.as_deref()).await?;
    run_npm(ui_dir, &["run", "build"], npm_override.as_deref()).await
}

async fn run_npm(ui_dir: &Path, args: &[&str], npm_override: Option<&Path>) -> AppResult<()> {
    let step = args.join(" ");
    let invocation = resolve_npm_invocation(npm_override)?;
    info!(
        "Running npm {step} in {} via {}",
        ui_dir.display(),
        invocation.display
    );
    let status = build_npm_command(&invocation)
        .args(args)
        .current_dir(ui_dir)
        .status()
        .await
        .map_err(|err| {
            let hint = if err.kind() == std::io::ErrorKind::NotFound {
                format!(
                    "npm was not found (tried {}). Ensure Node.js/npm is on PATH, set {UI_NPM_ENV}/--npm to the npm path, or set {UI_ASSETS_ENV} to prebuilt assets.",
                    invocation.display
                )
            } else {
                "npm failed to start.".to_string()
            };
            AppError::internal(format!("failed to run npm {step}: {err}. {hint}"))
        })?;
    if status.success() {
        Ok(())
    } else {
        let code = status
            .code()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        Err(AppError::internal(format!(
            "npm {step} failed (exit code {code})."
        )))
    }
}

#[derive(Debug)]
struct NpmInvocation {
    program: OsString,
    prefix: Vec<OsString>,
    display: String,
}

fn resolve_npm_invocation(npm_override: Option<&Path>) -> AppResult<NpmInvocation> {
    if let Some(path) = npm_override {
        return build_npm_invocation_from_path(path.to_path_buf());
    }

    if let Ok(value) = std::env::var(UI_NPM_ENV) {
        let path = PathBuf::from(value);
        if !path.exists() {
            return Err(AppError::internal(format!(
                "{UI_NPM_ENV} points to missing npm path: {}",
                path.display()
            )));
        }
        return build_npm_invocation_from_path(path);
    }

    if let Some(path) = find_in_path("npm") {
        return build_npm_invocation_from_path(path);
    }

    Err(AppError::internal(format!(
        "npm was not found. Install Node.js/npm, ensure it is on PATH, set {UI_NPM_ENV}/--npm to the npm path, or set {UI_ASSETS_ENV} to prebuilt assets."
    )))
}

fn build_npm_command(invocation: &NpmInvocation) -> Command {
    let mut command = Command::new(&invocation.program);
    if !invocation.prefix.is_empty() {
        command.args(&invocation.prefix);
    }
    command
}

async fn spawn_ui_dev_server(api_base: &str, npm_override: Option<&Path>) -> AppResult<Child> {
    let ui_dir = ui_source_dir();
    let invocation = resolve_npm_invocation(npm_override)?;
    let mut command = build_npm_command(&invocation);
    command
        .arg("run")
        .arg("dev")
        .arg("--")
        .arg("--host")
        .arg(UI_DEV_HOST)
        .arg("--port")
        .arg(UI_DEV_PORT.to_string())
        .current_dir(ui_dir)
        .env("JWT_TESTER_API_URL", api_base)
        .env("BROWSER", "none")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    command.spawn().map_err(|err| {
        AppError::internal(format!(
            "failed to start UI dev server: {err}. Ensure npm is installed or set {UI_NPM_ENV}/--npm."
        ))
    })
}

fn build_npm_invocation_from_path(path: PathBuf) -> AppResult<NpmInvocation> {
    if !path.is_file() {
        return Err(AppError::internal(format!(
            "npm path does not exist or is not a file: {}",
            path.display()
        )));
    }
    #[cfg(windows)]
    {
        let ext = path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase());
        let Some(ext) = ext else {
            return Err(AppError::internal(format!(
                "npm path {} has no extension. On Windows point {UI_NPM_ENV}/--npm to npm.cmd or npm.exe.",
                path.display()
            )));
        };
        if ext == "cmd" || ext == "bat" {
            let cmd = cmd_program();
            let display = format!("{} /C {}", cmd.to_string_lossy(), path.display());
            return Ok(NpmInvocation {
                program: cmd,
                prefix: vec![OsString::from("/C"), path.as_os_str().to_os_string()],
                display,
            });
        }
        if ext != "exe" && ext != "com" {
            return Err(AppError::internal(format!(
                "npm path {} has unsupported extension. On Windows point {UI_NPM_ENV}/--npm to npm.cmd or npm.exe.",
                path.display()
            )));
        }
    }
    let display = path.display().to_string();
    Ok(NpmInvocation {
        program: path.into_os_string(),
        prefix: Vec::new(),
        display,
    })
}

fn find_in_path(program: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    let extensions = path_extensions();
    #[cfg(windows)]
    {
        for dir in std::env::split_paths(&paths) {
            if program.contains(std::path::MAIN_SEPARATOR) {
                let candidate = dir.join(program);
                if candidate.is_file() {
                    return Some(candidate);
                }
            } else {
                for ext in &extensions {
                    let candidate = dir.join(format!("{program}{ext}"));
                    if candidate.is_file() {
                        return Some(candidate);
                    }
                }
            }
        }
        return None;
    }
    #[cfg(not(windows))]
    {
        for dir in std::env::split_paths(&paths) {
            if program.contains(std::path::MAIN_SEPARATOR) {
                let candidate = dir.join(program);
                if candidate.is_file() {
                    return Some(candidate);
                }
            } else {
                let base = dir.join(program);
                if base.is_file() {
                    return Some(base);
                }
                for ext in &extensions {
                    let candidate = dir.join(format!("{program}{ext}"));
                    if candidate.is_file() {
                        return Some(candidate);
                    }
                }
            }
        }
        None
    }
}

fn path_extensions() -> Vec<String> {
    #[cfg(windows)]
    {
        let allowed = ["exe", "cmd", "bat", "com"];
        if let Some(value) = std::env::var_os("PATHEXT") {
            let value = value.to_string_lossy();
            let items = value
                .split(';')
                .filter(|item| !item.is_empty())
                .map(|item| {
                    let trimmed = item.trim_start_matches('.').to_ascii_lowercase();
                    trimmed
                })
                .filter(|item| allowed.contains(&item.as_str()))
                .map(|item| format!(".{item}"))
                .collect::<Vec<_>>();
            if !items.is_empty() {
                return items;
            }
        }
        return vec![
            ".exe".to_string(),
            ".cmd".to_string(),
            ".bat".to_string(),
            ".com".to_string(),
        ];
    }
    #[cfg(not(windows))]
    {
        Vec::new()
    }
}

#[cfg(windows)]
fn cmd_program() -> OsString {
    std::env::var_os("ComSpec").unwrap_or_else(|| OsString::from("cmd"))
}

fn validate_bind_target(host: IpAddr, allow_remote: bool) -> AppResult<()> {
    let is_local = match host {
        IpAddr::V4(v4) => v4.is_loopback(),
        IpAddr::V6(v6) => v6.is_loopback(),
    };
    if !is_local && !allow_remote {
        return Err(AppError::invalid_key(format!(
            "Refusing to bind UI to non-localhost address {host}. Use --allow-remote to override (dangerous)."
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ensure_ui_assets_with, resolve_npm_invocation, validate_bind_target, UI_NPM_ENV};
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tempfile::tempdir;
    #[cfg(windows)]
    use {std::env, std::sync::Mutex};

    #[cfg(windows)]
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn validate_bind_target_allows_loopback() {
        assert!(validate_bind_target(IpAddr::V4(Ipv4Addr::LOCALHOST), false).is_ok());
        assert!(validate_bind_target(IpAddr::V6(Ipv6Addr::LOCALHOST), false).is_ok());
    }

    #[test]
    fn validate_bind_target_rejects_remote_without_override() {
        let err = validate_bind_target(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)), false)
            .expect_err("expected error");
        assert!(err.to_string().contains("Refusing to bind UI"));
    }

    #[test]
    fn validate_bind_target_allows_remote_with_override() {
        assert!(validate_bind_target(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)), true).is_ok());
    }

    #[tokio::test]
    async fn ensure_ui_assets_skips_build_when_present() {
        let dir = tempdir().expect("tempdir");
        let assets_root = dir.path().join("dist");
        std::fs::create_dir_all(&assets_root).expect("create assets dir");
        std::fs::write(assets_root.join("index.html"), "<html/>").expect("write index");
        let ui_dir = dir.path().join("ui");
        std::fs::create_dir_all(&ui_dir).expect("create ui dir");

        let calls = Arc::new(AtomicUsize::new(0));
        let calls_clone = Arc::clone(&calls);
        let result = ensure_ui_assets_with(&assets_root, false, false, &ui_dir, move |_| {
            let calls = Arc::clone(&calls_clone);
            Box::pin(async move {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn ensure_ui_assets_errors_when_override_missing() {
        let dir = tempdir().expect("tempdir");
        let assets_root = dir.path().join("dist");
        std::fs::create_dir_all(&assets_root).expect("create assets dir");
        let ui_dir = dir.path().join("ui");
        std::fs::create_dir_all(&ui_dir).expect("create ui dir");

        let calls = Arc::new(AtomicUsize::new(0));
        let calls_clone = Arc::clone(&calls);
        let result = ensure_ui_assets_with(&assets_root, true, false, &ui_dir, move |_| {
            let calls = Arc::clone(&calls_clone);
            Box::pin(async move {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
        })
        .await;

        assert!(result.is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn ensure_ui_assets_errors_when_missing_without_build() {
        let dir = tempdir().expect("tempdir");
        let assets_root = dir.path().join("dist");
        std::fs::create_dir_all(&assets_root).expect("create assets dir");
        let ui_dir = dir.path().join("ui");
        std::fs::create_dir_all(&ui_dir).expect("create ui dir");

        let calls = Arc::new(AtomicUsize::new(0));
        let calls_clone = Arc::clone(&calls);
        let assets_clone = assets_root.clone();
        let result = ensure_ui_assets_with(&assets_root, false, false, &ui_dir, move |_| {
            let calls = Arc::clone(&calls_clone);
            let assets_clone = assets_clone.clone();
            Box::pin(async move {
                calls.fetch_add(1, Ordering::SeqCst);
                std::fs::write(assets_clone.join("index.html"), "<html/>").expect("write index");
                Ok(())
            })
        })
        .await;

        assert!(result.is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[cfg(windows)]
    #[test]
    fn resolve_npm_env_override_cmd_path() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let dir = tempdir().expect("tempdir");
        let npm_cmd = dir.path().join("npm.cmd");
        std::fs::write(&npm_cmd, "@echo off").expect("write npm cmd");

        let prev = env::var(UI_NPM_ENV).ok();
        env::set_var(UI_NPM_ENV, &npm_cmd);

        let invocation = resolve_npm_invocation(None).expect("invocation");
        let program = invocation.program.to_string_lossy().to_lowercase();
        assert!(program == "cmd" || program.ends_with("cmd.exe"));
        assert_eq!(
            invocation
                .prefix
                .first()
                .map(|value| value.to_string_lossy()),
            Some("/C".into())
        );
        let npm_cmd_lower = npm_cmd.to_string_lossy().to_lowercase();
        assert!(invocation
            .prefix
            .iter()
            .any(|value| value.to_string_lossy().to_lowercase() == npm_cmd_lower));

        match prev {
            Some(value) => env::set_var(UI_NPM_ENV, value),
            None => env::remove_var(UI_NPM_ENV),
        }
    }

    #[cfg(windows)]
    #[test]
    fn resolve_npm_env_override_missing_path_errors() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let prev = env::var(UI_NPM_ENV).ok();
        env::set_var(UI_NPM_ENV, r"C:\missing\npm.cmd");

        let err = resolve_npm_invocation(None).expect_err("expected error");
        assert!(err.to_string().contains(UI_NPM_ENV));

        match prev {
            Some(value) => env::set_var(UI_NPM_ENV, value),
            None => env::remove_var(UI_NPM_ENV),
        }
    }

    #[cfg(windows)]
    #[test]
    fn resolve_npm_override_cmd_path() {
        let dir = tempdir().expect("tempdir");
        let npm_cmd = dir.path().join("npm.cmd");
        std::fs::write(&npm_cmd, "@echo off").expect("write npm cmd");

        let invocation = resolve_npm_invocation(Some(&npm_cmd)).expect("invocation");
        let program = invocation.program.to_string_lossy().to_lowercase();
        assert!(program == "cmd" || program.ends_with("cmd.exe"));
        assert_eq!(
            invocation
                .prefix
                .first()
                .map(|value| value.to_string_lossy()),
            Some("/C".into())
        );
        let npm_cmd_lower = npm_cmd.to_string_lossy().to_lowercase();
        assert!(invocation
            .prefix
            .iter()
            .any(|value| value.to_string_lossy().to_lowercase() == npm_cmd_lower));
    }

    #[cfg(not(windows))]
    #[test]
    fn resolve_npm_override_path() {
        let dir = tempdir().expect("tempdir");
        let npm = dir.path().join("npm");
        std::fs::write(&npm, "#!/bin/sh\necho npm\n").expect("write npm");

        let invocation = resolve_npm_invocation(Some(&npm)).expect("invocation");
        assert_eq!(invocation.program, npm.clone().into_os_string());
        assert!(invocation.prefix.is_empty());
    }

    #[tokio::test]
    async fn ensure_ui_assets_force_build_runs_even_when_present() {
        let dir = tempdir().expect("tempdir");
        let assets_root = dir.path().join("dist");
        std::fs::create_dir_all(&assets_root).expect("create assets dir");
        std::fs::write(assets_root.join("index.html"), "<html/>").expect("write index");
        let ui_dir = dir.path().join("ui");
        std::fs::create_dir_all(&ui_dir).expect("create ui dir");

        let calls = Arc::new(AtomicUsize::new(0));
        let calls_clone = Arc::clone(&calls);
        let result = ensure_ui_assets_with(&assets_root, false, true, &ui_dir, move |_| {
            let calls = Arc::clone(&calls_clone);
            Box::pin(async move {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn ensure_ui_assets_force_build_rejects_override() {
        let dir = tempdir().expect("tempdir");
        let assets_root = dir.path().join("dist");
        std::fs::create_dir_all(&assets_root).expect("create assets dir");
        let ui_dir = dir.path().join("ui");
        std::fs::create_dir_all(&ui_dir).expect("create ui dir");

        let calls = Arc::new(AtomicUsize::new(0));
        let calls_clone = Arc::clone(&calls);
        let result = ensure_ui_assets_with(&assets_root, true, true, &ui_dir, move |_| {
            let calls = Arc::clone(&calls_clone);
            Box::pin(async move {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
        })
        .await;

        assert!(result.is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
}
