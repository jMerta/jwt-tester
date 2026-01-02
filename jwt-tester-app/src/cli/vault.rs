use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct VaultArgs {
    #[command(subcommand)]
    pub cmd: VaultCmd,
}

#[derive(Subcommand, Debug)]
pub enum VaultCmd {
    #[command(subcommand)]
    Project(ProjectCmd),
    #[command(subcommand)]
    Key(KeyCmd),
    #[command(subcommand)]
    Token(TokenCmd),
    /// Export the vault to an encrypted bundle
    Export {
        /// Output path for the bundle (omit to print to stdout)
        #[arg(long)]
        out: Option<PathBuf>,
        /// Passphrase (supports prompt[:LABEL], '-', '@file', or 'env:NAME')
        #[arg(long)]
        passphrase: String,
    },
    /// Import an encrypted bundle into the vault
    Import {
        /// Bundle JSON string, '-', '@file', or 'env:NAME'
        #[arg(long)]
        bundle: String,
        /// Passphrase (supports prompt[:LABEL], '-', '@file', or 'env:NAME')
        #[arg(long)]
        passphrase: String,
        /// Replace existing vault contents before import
        #[arg(long)]
        replace: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProjectCmd {
    Add {
        name: String,
        /// Optional description/notes
        #[arg(long)]
        description: Option<String>,
        /// Optional tags; repeatable
        #[arg(long)]
        tag: Vec<String>,
    },
    List {
        /// Include tags/description in text output.
        #[arg(long)]
        details: bool,
    },
    Delete {
        /// Project id (positional). Use --name to delete by project name.
        id: Option<String>,
        /// Project name to delete.
        #[arg(long)]
        name: Option<String>,
    },
    SetDefaultKey {
        /// Project name or id.
        #[arg(long)]
        project: String,
        #[arg(long)]
        key_id: Option<String>,
        #[arg(long)]
        key_name: Option<String>,
        /// Clear the project's default key.
        #[arg(long)]
        clear: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum KeyCmd {
    Add {
        /// Project name or id.
        #[arg(long)]
        project: String,
        #[arg(long)]
        name: Option<String>,
        /// Kind is stored for UX; should match algorithm family (hmac|rsa|ec|eddsa|jwks)
        #[arg(long, default_value = "hmac")]
        kind: String,
        /// Optional key id hint (kid) for selection
        #[arg(long)]
        kid: Option<String>,
        /// Optional description/notes
        #[arg(long)]
        description: Option<String>,
        /// Optional tags; repeatable
        #[arg(long)]
        tag: Vec<String>,
        /// Key material: literal string, prompt[:LABEL], '-', '@file', or 'env:NAME'
        #[arg(long)]
        secret: String,
    },
    /// Generate key material and store it in the vault
    Generate {
        /// Project name or id.
        #[arg(long)]
        project: String,
        #[arg(long)]
        name: Option<String>,
        /// Kind is stored for UX; should match algorithm family (hmac|rsa|ec|eddsa)
        #[arg(long, default_value = "hmac")]
        kind: String,
        /// Optional key id hint (kid) for selection
        #[arg(long)]
        kid: Option<String>,
        /// Optional description/notes
        #[arg(long)]
        description: Option<String>,
        /// Optional tags; repeatable
        #[arg(long)]
        tag: Vec<String>,
        /// HMAC secret length in bytes (default 32)
        #[arg(long, value_name = "BYTES")]
        hmac_bytes: Option<usize>,
        /// RSA key size (2048, 3072, 4096)
        #[arg(long, value_name = "BITS")]
        rsa_bits: Option<usize>,
        /// EC curve (P-256 or P-384)
        #[arg(long, value_name = "CURVE")]
        ec_curve: Option<String>,
        /// Include generated material in output
        #[arg(long)]
        reveal: bool,
        /// Write generated material to a file
        #[arg(long)]
        out: Option<PathBuf>,
    },
    List {
        /// Project name or id.
        #[arg(long)]
        project: String,
        /// Include tags/description in text output.
        #[arg(long)]
        details: bool,
    },
    Delete {
        /// Key id (positional). Use --project + --name to delete by name.
        id: Option<String>,
        /// Project name or id (required with --name).
        #[arg(long)]
        project: Option<String>,
        /// Key name (requires --project).
        #[arg(long)]
        name: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum TokenCmd {
    Add {
        /// Project name or id.
        #[arg(long)]
        project: String,
        #[arg(long)]
        name: String,
        /// Token: literal string, prompt[:LABEL], '-', '@file', or 'env:NAME'
        #[arg(long)]
        token: String,
    },
    List {
        /// Project name or id.
        #[arg(long)]
        project: String,
        /// Include created timestamp in text output.
        #[arg(long)]
        details: bool,
    },
    Delete {
        /// Token id (positional). Use --project + --name to delete by name.
        id: Option<String>,
        /// Project name or id (required with --name).
        #[arg(long)]
        project: Option<String>,
        /// Token name (requires --project).
        #[arg(long)]
        name: Option<String>,
    },
}
