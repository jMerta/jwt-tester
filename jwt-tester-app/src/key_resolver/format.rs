use crate::cli::KeyFormat;
use crate::error::{AppError, AppResult};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey};

pub(super) fn detect_key_format(bytes: &[u8]) -> KeyFormat {
    if bytes.starts_with(b"-----BEGIN") {
        KeyFormat::Pem
    } else {
        KeyFormat::Der
    }
}

pub(super) fn decoding_key_from_bytes(
    alg: Algorithm,
    bytes: &[u8],
    format: KeyFormat,
) -> AppResult<DecodingKey> {
    match (alg, format) {
        (Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512, _) => {
            Ok(DecodingKey::from_secret(bytes))
        }
        (
            Algorithm::RS256
            | Algorithm::RS384
            | Algorithm::RS512
            | Algorithm::PS256
            | Algorithm::PS384
            | Algorithm::PS512,
            KeyFormat::Pem,
        ) => decode_rsa_pem(bytes),
        (
            Algorithm::RS256
            | Algorithm::RS384
            | Algorithm::RS512
            | Algorithm::PS256
            | Algorithm::PS384
            | Algorithm::PS512,
            KeyFormat::Der,
        ) => Ok(DecodingKey::from_rsa_der(bytes)),
        (Algorithm::ES256 | Algorithm::ES384, KeyFormat::Pem) => decode_ec_pem(bytes),
        (Algorithm::ES256 | Algorithm::ES384, KeyFormat::Der) => {
            Ok(DecodingKey::from_ec_der(bytes))
        }
        (Algorithm::EdDSA, KeyFormat::Pem) => decode_ed_pem(bytes),
        (Algorithm::EdDSA, KeyFormat::Der) => Ok(DecodingKey::from_ed_der(bytes)),
    }
}

fn decode_rsa_pem(bytes: &[u8]) -> AppResult<DecodingKey> {
    match DecodingKey::from_rsa_pem(bytes) {
        Ok(key) => Ok(key),
        Err(err) => {
            #[cfg(feature = "keygen")]
            {
                if let Ok(Some(public_pem)) = crate::keygen::rsa_public_pem_from_private(bytes) {
                    if let Ok(key) = DecodingKey::from_rsa_pem(public_pem.as_bytes()) {
                        return Ok(key);
                    }
                }
            }
            Err(AppError::from(err))
        }
    }
}

fn decode_ec_pem(bytes: &[u8]) -> AppResult<DecodingKey> {
    match DecodingKey::from_ec_pem(bytes) {
        Ok(key) => Ok(key),
        Err(err) => {
            #[cfg(feature = "keygen")]
            {
                if let Ok(Some(public_pem)) = crate::keygen::ec_public_pem_from_private(bytes) {
                    if let Ok(key) = DecodingKey::from_ec_pem(public_pem.as_bytes()) {
                        return Ok(key);
                    }
                }
            }
            Err(AppError::from(err))
        }
    }
}

fn decode_ed_pem(bytes: &[u8]) -> AppResult<DecodingKey> {
    match DecodingKey::from_ed_pem(bytes) {
        Ok(key) => Ok(key),
        Err(err) => {
            #[cfg(feature = "keygen")]
            {
                if let Ok(Some(public_pem)) = crate::keygen::ed_public_pem_from_private(bytes) {
                    if let Ok(key) = DecodingKey::from_ed_pem(public_pem.as_bytes()) {
                        return Ok(key);
                    }
                }
            }
            Err(AppError::from(err))
        }
    }
}

pub(super) fn encoding_key_from_bytes(
    alg: Algorithm,
    bytes: &[u8],
    format: KeyFormat,
) -> AppResult<EncodingKey> {
    match (alg, format) {
        (Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512, _) => {
            Ok(EncodingKey::from_secret(bytes))
        }
        (
            Algorithm::RS256
            | Algorithm::RS384
            | Algorithm::RS512
            | Algorithm::PS256
            | Algorithm::PS384
            | Algorithm::PS512,
            KeyFormat::Pem,
        ) => EncodingKey::from_rsa_pem(bytes).map_err(AppError::from),
        (
            Algorithm::RS256
            | Algorithm::RS384
            | Algorithm::RS512
            | Algorithm::PS256
            | Algorithm::PS384
            | Algorithm::PS512,
            KeyFormat::Der,
        ) => Ok(EncodingKey::from_rsa_der(bytes)),
        (Algorithm::ES256 | Algorithm::ES384, KeyFormat::Pem) => {
            EncodingKey::from_ec_pem(bytes).map_err(AppError::from)
        }
        (Algorithm::ES256 | Algorithm::ES384, KeyFormat::Der) => {
            Ok(EncodingKey::from_ec_der(bytes))
        }
        (Algorithm::EdDSA, KeyFormat::Pem) => {
            EncodingKey::from_ed_pem(bytes).map_err(AppError::from)
        }
        (Algorithm::EdDSA, KeyFormat::Der) => Ok(EncodingKey::from_ed_der(bytes)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "keygen")]
    use crate::keygen::{generate_key_material, EcCurve, KeyGenSpec};
    use std::path::PathBuf;

    fn fixture_bytes(name: &str) -> Vec<u8> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name);
        std::fs::read(path).expect("read fixture")
    }

    #[test]
    fn detect_key_format_pem_and_der() {
        assert_eq!(detect_key_format(b"-----BEGIN TEST"), KeyFormat::Pem);
        assert_eq!(detect_key_format(b"\x01\x02\x03"), KeyFormat::Der);
    }

    #[test]
    fn decoding_and_encoding_keys_across_formats() {
        let hmac = b"secret";
        assert!(decoding_key_from_bytes(Algorithm::HS256, hmac, KeyFormat::Pem).is_ok());
        assert!(encoding_key_from_bytes(Algorithm::HS256, hmac, KeyFormat::Der).is_ok());

        let rsa_pub_pem = fixture_bytes("rsa_public.pem");
        let rsa_pub_der = fixture_bytes("rsa_public.der");
        let rsa_priv_pem = fixture_bytes("rsa_private.pem");
        let rsa_priv_der = fixture_bytes("rsa_private.der");
        assert!(decoding_key_from_bytes(Algorithm::RS256, &rsa_pub_pem, KeyFormat::Pem).is_ok());
        assert!(decoding_key_from_bytes(Algorithm::RS256, &rsa_pub_der, KeyFormat::Der).is_ok());
        assert!(encoding_key_from_bytes(Algorithm::RS256, &rsa_priv_pem, KeyFormat::Pem).is_ok());
        assert!(encoding_key_from_bytes(Algorithm::RS256, &rsa_priv_der, KeyFormat::Der).is_ok());

        let ec_pub_pem = fixture_bytes("ec256_public.pem");
        let ec_pub_der = fixture_bytes("ec256_public.der");
        let ec_priv_pem = fixture_bytes("ec256_private.pem");
        let ec_priv_der = fixture_bytes("ec256_private.der");
        assert!(decoding_key_from_bytes(Algorithm::ES256, &ec_pub_pem, KeyFormat::Pem).is_ok());
        assert!(decoding_key_from_bytes(Algorithm::ES256, &ec_pub_der, KeyFormat::Der).is_ok());
        assert!(encoding_key_from_bytes(Algorithm::ES256, &ec_priv_pem, KeyFormat::Pem).is_ok());
        assert!(encoding_key_from_bytes(Algorithm::ES256, &ec_priv_der, KeyFormat::Der).is_ok());

        let ed_pub_pem = fixture_bytes("ed25519_public.pem");
        let ed_pub_der = fixture_bytes("ed25519_public.der");
        let ed_priv_pem = fixture_bytes("ed25519_private.pem");
        let ed_priv_der = fixture_bytes("ed25519_private.der");
        assert!(decoding_key_from_bytes(Algorithm::EdDSA, &ed_pub_pem, KeyFormat::Pem).is_ok());
        assert!(decoding_key_from_bytes(Algorithm::EdDSA, &ed_pub_der, KeyFormat::Der).is_ok());
        assert!(encoding_key_from_bytes(Algorithm::EdDSA, &ed_priv_pem, KeyFormat::Pem).is_ok());
        assert!(encoding_key_from_bytes(Algorithm::EdDSA, &ed_priv_der, KeyFormat::Der).is_ok());
    }

    #[cfg(feature = "keygen")]
    #[test]
    fn decoding_private_pem_falls_back_to_public() {
        let rsa_priv = generate_key_material(KeyGenSpec::Rsa { bits: 2048 }).expect("rsa key");
        assert!(
            decoding_key_from_bytes(Algorithm::RS256, rsa_priv.as_bytes(), KeyFormat::Pem).is_ok()
        );

        let ec_priv = generate_key_material(KeyGenSpec::Ec {
            curve: EcCurve::P256,
        })
        .expect("ec key");
        assert!(
            decoding_key_from_bytes(Algorithm::ES256, ec_priv.as_bytes(), KeyFormat::Pem).is_ok()
        );

        let ed_priv = generate_key_material(KeyGenSpec::EdDsa).expect("ed key");
        assert!(
            decoding_key_from_bytes(Algorithm::EdDSA, ed_priv.as_bytes(), KeyFormat::Pem).is_ok()
        );
    }
}
