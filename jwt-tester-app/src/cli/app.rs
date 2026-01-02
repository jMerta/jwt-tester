use super::crypto::{EncodeArgs, VerifyArgs, VerifyCommonArgs};
use super::vault::VaultArgs;
use clap::{Parser, Subcommand, ValueEnum};
use std::net::IpAddr;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "jwt-tester")]
#[command(about = "JWT CLI + local UI (MVP)", long_about = None)]
#[command(version)]
pub struct App {
    /// Output machine-readable JSON
    #[arg(long)]
    pub json: bool,

    /// Disable ANSI color output
    #[arg(long)]
    pub no_color: bool,

    /// Suppress non-essential output
    #[arg(long)]
    pub quiet: bool,

    /// Verbose diagnostics (no secrets)
    #[arg(long, short = 'v')]
    pub verbose: bool,

    /// Persist vault metadata to disk (SQLite). When set, vault is in-memory only.
    #[arg(long)]
    pub no_persist: bool,

    /// Override data directory used for persistence.
    #[arg(long)]
    pub data_dir: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Start a local-only web UI for working with JWTs and managing keys.
    #[cfg(feature = "ui")]
    Ui(UiArgs),

    /// Manage the local vault (projects, keys, tokens).
    Vault(VaultArgs),

    /// Decode a JWT without verifying it (UNVERIFIED).
    Decode(DecodeArgs),

    /// Verify a JWT signature using a key from the vault or direct input.
    Verify(VerifyArgs),

    /// Encode a JWT using a key from the vault or direct input.
    Encode(EncodeArgs),

    /// Inspect a JWT with human-friendly summaries.
    Inspect(InspectArgs),

    /// Split JWT segments (decoded header/payload + signature bytes).
    Split(SplitArgs),

    /// Generate shell completion scripts.
    Completion(CompletionArgs),
}

#[cfg(feature = "ui")]
#[derive(Parser, Debug, Clone)]
pub struct UiArgs {
    /// Host to bind the UI server to (default: 127.0.0.1)
    #[arg(long, default_value = "127.0.0.1")]
    pub host: IpAddr,

    /// Port to bind to (0 = ephemeral)
    #[arg(long, default_value_t = 0)]
    pub port: u16,

    /// Dangerous: allow binding to non-localhost addresses.
    #[arg(long)]
    pub allow_remote: bool,

    /// Force rebuild of UI assets before starting the server.
    #[arg(long)]
    pub build: bool,

    /// Run the Vite dev server (hot reload) alongside the API.
    #[arg(long)]
    pub dev: bool,

    /// Path to the npm executable (override PATH).
    #[arg(long)]
    pub npm: Option<PathBuf>,
}

#[derive(Parser, Debug)]
pub struct DecodeArgs {
    /// Render exp/nbf/iat as RFC3339 timestamps (utc|local|+HH:MM)
    #[arg(long, num_args = 0..=1, default_missing_value = "utc")]
    pub date: Option<String>,

    #[command(flatten)]
    pub verify: VerifyCommonArgs,

    /// Write JSON output to file (implies JSON output)
    #[arg(long)]
    pub out: Option<PathBuf>,

    /// The JWT to decode, or '-' to read from stdin.
    pub token: String,
}

#[derive(Parser, Debug)]
pub struct InspectArgs {
    /// Render exp/nbf/iat as RFC3339 timestamps (utc|local|+HH:MM)
    #[arg(long, num_args = 0..=1, default_missing_value = "utc")]
    pub date: Option<String>,

    /// Show base64url segments
    #[arg(long)]
    pub show_segments: bool,

    /// The JWT to inspect, or '-' to read from stdin.
    pub token: String,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum SplitFormat {
    #[value(name = "text")]
    Text,
    #[value(name = "json")]
    Json,
}

#[derive(Parser, Debug)]
pub struct SplitArgs {
    /// Output format
    #[arg(long, value_enum, default_value_t = SplitFormat::Text)]
    pub format: SplitFormat,

    /// The JWT to split, or '-' to read from stdin.
    pub token: String,
}

#[derive(Parser, Debug)]
pub struct CompletionArgs {
    /// Shell type
    #[arg(value_enum)]
    pub shell: CompletionShell,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum CompletionShell {
    #[value(name = "bash")]
    Bash,
    #[value(name = "zsh")]
    Zsh,
    #[value(name = "fish")]
    Fish,
    #[value(name = "powershell")]
    Powershell,
    #[value(name = "elvish")]
    Elvish,
    #[value(name = "nushell")]
    Nushell,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_split_args_and_decode_date() {
        let app = App::try_parse_from(["jwt-tester", "split", "--format", "json", "tok"])
            .expect("parse split");
        match app.command {
            Command::Split(args) => assert!(matches!(args.format, SplitFormat::Json)),
            _ => panic!("expected split command"),
        }

        let app = App::try_parse_from(["jwt-tester", "decode", "--date", "local", "tok"])
            .expect("parse decode");
        match app.command {
            Command::Decode(args) => assert_eq!(args.date.as_deref(), Some("local")),
            _ => panic!("expected decode command"),
        }
    }

    #[test]
    fn parse_completion_shell() {
        let app = App::try_parse_from(["jwt-tester", "completion", "bash"]).expect("parse");
        match app.command {
            Command::Completion(args) => assert!(matches!(args.shell, CompletionShell::Bash)),
            _ => panic!("expected completion command"),
        }
    }

    #[cfg(feature = "ui")]
    #[test]
    fn parse_ui_args_with_build_dev_and_npm() {
        let app = App::try_parse_from([
            "jwt-tester",
            "ui",
            "--build",
            "--dev",
            "--npm",
            "C:\\npm.cmd",
        ])
        .expect("parse ui");
        match app.command {
            Command::Ui(args) => {
                assert!(args.build);
                assert!(args.dev);
                assert_eq!(args.npm, Some(PathBuf::from("C:\\npm.cmd")));
            }
            _ => panic!("expected ui command"),
        }
    }
}
