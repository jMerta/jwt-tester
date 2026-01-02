use clap::{Args, Parser, ValueEnum};
use jsonwebtoken::Algorithm;
use std::path::PathBuf;

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum JwtAlg {
    #[value(name = "hs256", alias = "HS256")]
    HS256,
    #[value(name = "hs384", alias = "HS384")]
    HS384,
    #[value(name = "hs512", alias = "HS512")]
    HS512,
    #[value(name = "rs256", alias = "RS256")]
    RS256,
    #[value(name = "rs384", alias = "RS384")]
    RS384,
    #[value(name = "rs512", alias = "RS512")]
    RS512,
    #[value(name = "ps256", alias = "PS256")]
    PS256,
    #[value(name = "ps384", alias = "PS384")]
    PS384,
    #[value(name = "ps512", alias = "PS512")]
    PS512,
    #[value(name = "es256", alias = "ES256")]
    ES256,
    #[value(name = "es384", alias = "ES384")]
    ES384,
    #[value(name = "eddsa", alias = "EdDSA")]
    EdDSA,
}

impl From<JwtAlg> for Algorithm {
    fn from(value: JwtAlg) -> Self {
        match value {
            JwtAlg::HS256 => Algorithm::HS256,
            JwtAlg::HS384 => Algorithm::HS384,
            JwtAlg::HS512 => Algorithm::HS512,
            JwtAlg::RS256 => Algorithm::RS256,
            JwtAlg::RS384 => Algorithm::RS384,
            JwtAlg::RS512 => Algorithm::RS512,
            JwtAlg::PS256 => Algorithm::PS256,
            JwtAlg::PS384 => Algorithm::PS384,
            JwtAlg::PS512 => Algorithm::PS512,
            JwtAlg::ES256 => Algorithm::ES256,
            JwtAlg::ES384 => Algorithm::ES384,
            JwtAlg::EdDSA => Algorithm::EdDSA,
        }
    }
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyFormat {
    #[value(name = "pem")]
    Pem,
    #[value(name = "der")]
    Der,
}

#[derive(Parser, Debug)]
pub struct VerifyArgs {
    #[command(flatten)]
    pub verify: VerifyCommonArgs,

    /// Token to verify, or '-' to read from stdin
    pub token: String,
}

#[derive(Args, Debug, Clone)]
pub struct VerifyCommonArgs {
    /// HMAC secret (raw, @file, -, env:NAME, b64:BASE64, or prompt[:LABEL])
    #[arg(long)]
    pub secret: Option<String>,

    /// Public key (PEM/DER) for RS*/PS*/ES*/EdDSA (supports @file, -, env:NAME, b64:BASE64, prompt[:LABEL])
    #[arg(long)]
    pub key: Option<String>,

    /// JWKS (JSON)
    #[arg(long)]
    pub jwks: Option<String>,

    /// Key format override (pem|der)
    #[arg(long, value_enum)]
    pub key_format: Option<KeyFormat>,

    /// kid selection (for JWKS)
    #[arg(long)]
    pub kid: Option<String>,

    /// Allow JWKS with a single key and no kid
    #[arg(long)]
    pub allow_single_jwk: bool,

    /// Vault project name
    #[arg(long)]
    pub project: Option<String>,

    /// Optional key id to use (otherwise requires the project to have exactly one key)
    #[arg(long)]
    pub key_id: Option<String>,

    /// Optional key name to use (within the project)
    #[arg(long)]
    pub key_name: Option<String>,

    /// Try all keys in the project if the selected/default key fails with InvalidSignature.
    #[arg(long)]
    pub try_all_keys: bool,

    /// Ignore token expiration (exp) during verification
    #[arg(long)]
    pub ignore_exp: bool,

    /// Leeway in seconds for exp/nbf checks
    #[arg(long, default_value_t = 30)]
    pub leeway_secs: u64,

    /// Issuer validation (iss)
    #[arg(long)]
    pub iss: Option<String>,

    /// Subject validation (sub)
    #[arg(long)]
    pub sub: Option<String>,

    /// Audience validation (aud); repeatable
    #[arg(long)]
    pub aud: Vec<String>,

    /// Require claim presence; repeatable
    #[arg(long)]
    pub require: Vec<String>,

    /// Print validation details
    #[arg(long)]
    pub explain: bool,

    /// Algorithm to verify with (omit to infer from token header)
    #[arg(long, value_enum)]
    pub alg: Option<JwtAlg>,
}

#[derive(Parser, Debug)]
pub struct EncodeArgs {
    /// HMAC secret (raw, @file, -, env:NAME, b64:BASE64, or prompt[:LABEL])
    #[arg(long)]
    pub secret: Option<String>,

    /// Private key (PEM/DER) for RS256/ES256/EdDSA (supports @file, -, env:NAME, b64:BASE64, prompt[:LABEL])
    #[arg(long)]
    pub key: Option<String>,

    /// Key format override (pem|der)
    #[arg(long, value_enum)]
    pub key_format: Option<KeyFormat>,

    /// Vault project name
    #[arg(long)]
    pub project: Option<String>,

    /// Optional key id to use (otherwise requires the project to have exactly one key)
    #[arg(long)]
    pub key_id: Option<String>,

    /// Optional key name to use (within the project)
    #[arg(long)]
    pub key_name: Option<String>,

    /// Algorithm to sign with
    #[arg(long, value_enum)]
    pub alg: JwtAlg,

    /// Claims JSON, '-' for stdin, or '@file.json'. Defaults to '{}'.
    #[arg(value_parser)]
    pub claims: Option<String>,

    /// Header JSON, '-' for stdin, or '@file.json'
    #[arg(long)]
    pub header: Option<String>,

    /// Optional kid to place in the header
    #[arg(long)]
    pub kid: Option<String>,

    /// Optional typ to place in the header (default: JWT)
    #[arg(long)]
    pub typ: Option<String>,

    /// Do not set typ in the header
    #[arg(long)]
    pub no_typ: bool,

    /// Standard claims
    #[arg(long)]
    pub iss: Option<String>,
    #[arg(long)]
    pub sub: Option<String>,
    #[arg(long)]
    pub aud: Vec<String>,
    #[arg(long)]
    pub jti: Option<String>,

    /// Issued-at timestamp (seconds or duration); omit value to use now
    #[arg(long, num_args = 0..=1, default_missing_value = "now")]
    pub iat: Option<String>,

    /// Do not set iat
    #[arg(long)]
    pub no_iat: bool,

    /// Not-before timestamp (seconds or duration)
    #[arg(long)]
    pub nbf: Option<String>,

    /// Expiration timestamp (seconds or duration)
    #[arg(long, num_args = 0..=1, default_missing_value = "+30m")]
    pub exp: Option<String>,

    /// Custom claim (k=v); repeatable
    #[arg(long)]
    pub claim: Vec<String>,

    /// JSON claim file to merge; repeatable
    #[arg(long)]
    pub claim_file: Vec<String>,

    /// Preserve payload key order as provided
    #[arg(long)]
    pub keep_payload_order: bool,

    /// Write token to file
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jwt_alg_converts_to_jsonwebtoken_algorithm() {
        assert_eq!(Algorithm::from(JwtAlg::HS256), Algorithm::HS256);
        assert_eq!(Algorithm::from(JwtAlg::RS256), Algorithm::RS256);
        assert_eq!(Algorithm::from(JwtAlg::EdDSA), Algorithm::EdDSA);
    }
}
