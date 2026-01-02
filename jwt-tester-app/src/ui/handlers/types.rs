use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct AddKeyReq {
    pub project_id: String,
    pub name: String,
    pub kind: String,
    pub secret: String,
    pub kid: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub(crate) struct GenerateKeyReq {
    pub project_id: String,
    pub name: String,
    pub kind: String,
    pub kid: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub hmac_bytes: Option<usize>,
    pub rsa_bits: Option<usize>,
    pub ec_curve: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct AddProjectReq {
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub(crate) struct AddTokenReq {
    pub project_id: String,
    pub name: String,
    pub token: String,
}

#[derive(Deserialize)]
pub(crate) struct SetDefaultKeyReq {
    pub key_id: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct ExportReq {
    pub passphrase: String,
}

#[derive(Deserialize)]
pub(crate) struct ImportReq {
    pub bundle: String,
    pub passphrase: String,
    pub replace: Option<bool>,
}

#[derive(Deserialize)]
pub(crate) struct EncodeReq {
    pub project: String,
    pub key_id: Option<String>,
    pub key_name: Option<String>,
    pub alg: String,
    pub claims: Option<String>,
    pub kid: Option<String>,
    pub typ: Option<String>,
    pub no_typ: Option<bool>,
    pub iss: Option<String>,
    pub sub: Option<String>,
    pub aud: Option<Vec<String>>,
    pub jti: Option<String>,
    pub iat: Option<String>,
    pub no_iat: Option<bool>,
    pub nbf: Option<String>,
    pub exp: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct VerifyReq {
    pub project: String,
    pub key_id: Option<String>,
    pub key_name: Option<String>,
    pub alg: Option<String>,
    pub token: String,
    pub try_all_keys: Option<bool>,
    pub ignore_exp: Option<bool>,
    pub leeway_secs: Option<u64>,
    pub iss: Option<String>,
    pub sub: Option<String>,
    pub aud: Option<Vec<String>>,
    pub require: Option<Vec<String>>,
    pub explain: Option<bool>,
}

#[derive(Deserialize)]
pub(crate) struct InspectReq {
    pub token: String,
    pub date: Option<String>,
    pub show_segments: Option<bool>,
}

#[derive(Deserialize)]
pub(crate) struct ProjectFilter {
    pub project_id: Option<String>,
}
