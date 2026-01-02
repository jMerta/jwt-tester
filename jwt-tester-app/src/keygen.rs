use crate::error::{AppError, AppResult};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use pkcs8::{DecodePrivateKey, LineEnding};
use rand::RngCore;
use rsa::pkcs1::DecodeRsaPrivateKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EcCurve {
    P256,
    P384,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyGenSpec {
    Hmac { bytes: usize },
    Rsa { bits: usize },
    Ec { curve: EcCurve },
    EdDsa,
}

pub const DEFAULT_HMAC_BYTES: usize = 32;
pub const DEFAULT_RSA_BITS: usize = 2048;
pub const DEFAULT_EC_CURVE: EcCurve = EcCurve::P256;

const HMAC_MIN_BYTES: usize = 16;
const HMAC_MAX_BYTES: usize = 128;
const RSA_ALLOWED_BITS: [usize; 3] = [2048, 3072, 4096];

pub fn generate_key_material(spec: KeyGenSpec) -> AppResult<String> {
    match spec {
        KeyGenSpec::Hmac { bytes } => generate_hmac_secret(bytes),
        KeyGenSpec::Rsa { bits } => generate_rsa_key(bits),
        KeyGenSpec::Ec { curve } => generate_ec_key(curve),
        KeyGenSpec::EdDsa => generate_eddsa_key(),
    }
}

pub fn parse_ec_curve(value: Option<&str>) -> AppResult<EcCurve> {
    match value.map(|v| v.trim().to_ascii_lowercase()) {
        None => Ok(DEFAULT_EC_CURVE),
        Some(v) if v == "p-256" || v == "p256" => Ok(EcCurve::P256),
        Some(v) if v == "p-384" || v == "p384" => Ok(EcCurve::P384),
        Some(other) => Err(AppError::invalid_key(format!(
            "unsupported EC curve '{other}' (use P-256 or P-384)"
        ))),
    }
}

pub fn rsa_public_pem_from_private(private_pem: &[u8]) -> AppResult<Option<String>> {
    let pem_str = match std::str::from_utf8(private_pem) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };
    let private = rsa::RsaPrivateKey::from_pkcs8_pem(pem_str)
        .or_else(|_| rsa::RsaPrivateKey::from_pkcs1_pem(pem_str))
        .ok();
    let Some(private) = private else {
        return Ok(None);
    };
    let public = rsa::RsaPublicKey::from(&private);
    let pem = rsa::pkcs8::EncodePublicKey::to_public_key_pem(&public, LineEnding::LF)
        .map_err(|e| AppError::internal(format!("rsa public pem encode failed: {e}")))?;
    Ok(Some(pem.to_string()))
}

pub fn ec_public_pem_from_private(private_pem: &[u8]) -> AppResult<Option<String>> {
    let pem_str = match std::str::from_utf8(private_pem) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };
    if let Ok(secret) = p256::SecretKey::from_pkcs8_pem(pem_str)
        .or_else(|_| p256::SecretKey::from_sec1_pem(pem_str))
    {
        let public = secret.public_key();
        let pem = p256::pkcs8::EncodePublicKey::to_public_key_pem(&public, LineEnding::LF)
            .map_err(|e| AppError::internal(format!("p256 public pem encode failed: {e}")))?;
        return Ok(Some(pem.to_string()));
    }
    if let Ok(secret) = p384::SecretKey::from_pkcs8_pem(pem_str)
        .or_else(|_| p384::SecretKey::from_sec1_pem(pem_str))
    {
        let public = secret.public_key();
        let pem = p384::pkcs8::EncodePublicKey::to_public_key_pem(&public, LineEnding::LF)
            .map_err(|e| AppError::internal(format!("p384 public pem encode failed: {e}")))?;
        return Ok(Some(pem.to_string()));
    }
    Ok(None)
}

pub fn ed_public_pem_from_private(private_pem: &[u8]) -> AppResult<Option<String>> {
    let pem_str = match std::str::from_utf8(private_pem) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };
    let key = ed25519_dalek::SigningKey::from_pkcs8_pem(pem_str).ok();
    let Some(key) = key else {
        return Ok(None);
    };
    let public = key.verifying_key();
    let pem = ed25519_dalek::pkcs8::EncodePublicKey::to_public_key_pem(&public, LineEnding::LF)
        .map_err(|e| AppError::internal(format!("ed25519 public pem encode failed: {e}")))?;
    Ok(Some(pem.to_string()))
}

fn generate_hmac_secret(bytes: usize) -> AppResult<String> {
    if !(HMAC_MIN_BYTES..=HMAC_MAX_BYTES).contains(&bytes) {
        return Err(AppError::invalid_key(format!(
            "HMAC secret length must be between {HMAC_MIN_BYTES} and {HMAC_MAX_BYTES} bytes"
        )));
    }
    let mut buf = vec![0u8; bytes];
    rand::rngs::OsRng.fill_bytes(&mut buf);
    Ok(URL_SAFE_NO_PAD.encode(buf))
}

fn generate_rsa_key(bits: usize) -> AppResult<String> {
    if !RSA_ALLOWED_BITS.contains(&bits) {
        return Err(AppError::invalid_key(
            "RSA key size must be 2048, 3072, or 4096 bits".to_string(),
        ));
    }
    let mut rng = rand::rngs::OsRng;
    let key = rsa::RsaPrivateKey::new(&mut rng, bits)
        .map_err(|e| AppError::internal(format!("rsa keygen failed: {e}")))?;
    let pem = rsa::pkcs8::EncodePrivateKey::to_pkcs8_pem(&key, LineEnding::LF)
        .map_err(|e| AppError::internal(format!("rsa pem encode failed: {e}")))?;
    Ok(pem.to_string())
}

fn generate_ec_key(curve: EcCurve) -> AppResult<String> {
    let mut rng = rand::rngs::OsRng;
    match curve {
        EcCurve::P256 => {
            let key = p256::SecretKey::random(&mut rng);
            let pem = p256::pkcs8::EncodePrivateKey::to_pkcs8_pem(&key, LineEnding::LF)
                .map_err(|e| AppError::internal(format!("p256 pem encode failed: {e}")))?;
            Ok(pem.to_string())
        }
        EcCurve::P384 => {
            let key = p384::SecretKey::random(&mut rng);
            let pem = p384::pkcs8::EncodePrivateKey::to_pkcs8_pem(&key, LineEnding::LF)
                .map_err(|e| AppError::internal(format!("p384 pem encode failed: {e}")))?;
            Ok(pem.to_string())
        }
    }
}

fn generate_eddsa_key() -> AppResult<String> {
    let mut seed = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut seed);
    let key = ed25519_dalek::SigningKey::from_bytes(&seed);
    let pem = ed25519_dalek::pkcs8::EncodePrivateKey::to_pkcs8_pem(&key, LineEnding::LF)
        .map_err(|e| AppError::internal(format!("ed25519 pem encode failed: {e}")))?;
    Ok(pem.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{DecodingKey, EncodingKey};

    #[test]
    fn generate_hmac_secret_is_base64url() {
        let secret = generate_key_material(KeyGenSpec::Hmac { bytes: 32 }).expect("secret");
        let decoded = URL_SAFE_NO_PAD
            .decode(secret.as_bytes())
            .expect("decode base64");
        assert_eq!(decoded.len(), 32);
    }

    #[test]
    fn generate_rsa_key_is_usable() {
        let pem = generate_key_material(KeyGenSpec::Rsa { bits: 2048 }).expect("pem");
        assert!(EncodingKey::from_rsa_pem(pem.as_bytes()).is_ok());
        let public = rsa_public_pem_from_private(pem.as_bytes())
            .expect("derive public")
            .expect("public pem");
        assert!(DecodingKey::from_rsa_pem(public.as_bytes()).is_ok());
    }

    #[test]
    fn generate_ec_p256_key_is_usable() {
        let pem = generate_key_material(KeyGenSpec::Ec {
            curve: EcCurve::P256,
        })
        .expect("pem");
        assert!(EncodingKey::from_ec_pem(pem.as_bytes()).is_ok());
        let public = ec_public_pem_from_private(pem.as_bytes())
            .expect("derive public")
            .expect("public pem");
        assert!(DecodingKey::from_ec_pem(public.as_bytes()).is_ok());
    }

    #[test]
    fn generate_ec_p384_key_is_usable() {
        let pem = generate_key_material(KeyGenSpec::Ec {
            curve: EcCurve::P384,
        })
        .expect("pem");
        assert!(EncodingKey::from_ec_pem(pem.as_bytes()).is_ok());
        let public = ec_public_pem_from_private(pem.as_bytes())
            .expect("derive public")
            .expect("public pem");
        assert!(DecodingKey::from_ec_pem(public.as_bytes()).is_ok());
    }

    #[test]
    fn generate_eddsa_key_is_usable() {
        let pem = generate_key_material(KeyGenSpec::EdDsa).expect("pem");
        assert!(EncodingKey::from_ed_pem(pem.as_bytes()).is_ok());
        let public = ed_public_pem_from_private(pem.as_bytes())
            .expect("derive public")
            .expect("public pem");
        assert!(DecodingKey::from_ed_pem(public.as_bytes()).is_ok());
    }
}
