mod claims;
mod cli;
mod commands;
mod date_utils;
mod error;
mod io_utils;
mod jwks;
mod jwt_ops;
mod key_resolver;
#[cfg(feature = "keygen")]
mod keygen;
mod output;
#[cfg(feature = "ui")]
mod ui;
mod vault;
mod vault_export;

#[cfg(all(feature = "ui", feature = "cli-only"))]
compile_error!("Features \"ui\" and \"cli-only\" are mutually exclusive. Build with default features for jwt-tester or with --no-default-features --features cli-only for jwt-tester-cli.");

use clap::Parser;
use cli::{App, Command};
use output::{emit_err, OutputConfig, OutputMode};

fn build_output_config(app: &App) -> OutputConfig {
    OutputConfig {
        mode: if app.json {
            OutputMode::Json
        } else {
            OutputMode::Text
        },
        quiet: app.quiet,
        no_color: app.no_color,
        verbose: app.verbose,
    }
}

#[cfg(feature = "ui")]
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let app = App::parse();
    let output_cfg = build_output_config(&app);

    let exit_code = match app.command {
        Command::Ui(args) => {
            let run = ui::run_ui(
                ui::UiConfig {
                    host: args.host,
                    port: args.port,
                    allow_remote: args.allow_remote,
                    no_persist: app.no_persist,
                    data_dir: app.data_dir,
                    force_build: args.build,
                    dev_mode: args.dev,
                    npm_path: args.npm,
                },
                output_cfg,
            )
            .await;
            match run {
                Ok(()) => 0,
                Err(err) => {
                    emit_err(output_cfg, err.clone());
                    err.exit_code()
                }
            }
        }
        Command::Vault(args) => {
            commands::vault::run(app.no_persist, app.data_dir, args, output_cfg)
        }
        Command::Decode(args) => {
            commands::decode::run(app.no_persist, app.data_dir, args, output_cfg)
        }
        Command::Verify(args) => {
            commands::verify::run(app.no_persist, app.data_dir, args, output_cfg)
        }
        Command::Encode(args) => {
            commands::encode::run(app.no_persist, app.data_dir, args, output_cfg)
        }
        Command::Inspect(args) => commands::inspect::run(args, output_cfg),
        Command::Split(args) => commands::split::run(args, output_cfg),
        Command::Completion(args) => commands::completion::run(args),
    };

    std::process::exit(exit_code);
}

#[cfg(not(feature = "ui"))]
fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let app = App::parse();
    let output_cfg = build_output_config(&app);

    let exit_code = match app.command {
        Command::Vault(args) => {
            commands::vault::run(app.no_persist, app.data_dir, args, output_cfg)
        }
        Command::Decode(args) => {
            commands::decode::run(app.no_persist, app.data_dir, args, output_cfg)
        }
        Command::Verify(args) => {
            commands::verify::run(app.no_persist, app.data_dir, args, output_cfg)
        }
        Command::Encode(args) => {
            commands::encode::run(app.no_persist, app.data_dir, args, output_cfg)
        }
        Command::Inspect(args) => commands::inspect::run(args, output_cfg),
        Command::Split(args) => commands::split::run(args, output_cfg),
        Command::Completion(args) => commands::completion::run(args),
    };

    std::process::exit(exit_code);
}
