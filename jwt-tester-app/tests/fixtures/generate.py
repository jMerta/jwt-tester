from __future__ import annotations

import base64
import json
from pathlib import Path

from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric import ec, ed25519, rsa

ROOT = Path(__file__).resolve().parent


def b64url(data: bytes) -> str:
    return base64.urlsafe_b64encode(data).rstrip(b"=").decode("ascii")


def write_text(path: Path, text: str) -> None:
    path.write_text(text, encoding="utf-8")


def write_bytes(path: Path, data: bytes) -> None:
    path.write_bytes(data)


# HMAC secrets
hmac_secret = b"test-secret-please-rotate"
hmac_alt = b"alt-secret-for-negative-tests"
write_text(ROOT / "hmac.key", hmac_secret.decode("utf-8"))
write_text(ROOT / "hmac_alt.key", hmac_alt.decode("utf-8"))

# RSA 2048 (PKCS8 private, SPKI public)
rsa_key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
rsa_private_pem = rsa_key.private_bytes(
    serialization.Encoding.PEM,
    serialization.PrivateFormat.PKCS8,
    serialization.NoEncryption(),
)
rsa_public_pem = rsa_key.public_key().public_bytes(
    serialization.Encoding.PEM,
    serialization.PublicFormat.SubjectPublicKeyInfo,
)
rsa_private_der = rsa_key.private_bytes(
    serialization.Encoding.DER,
    serialization.PrivateFormat.TraditionalOpenSSL,
    serialization.NoEncryption(),
)
rsa_public_der = rsa_key.public_key().public_bytes(
    serialization.Encoding.DER,
    serialization.PublicFormat.PKCS1,
)
write_bytes(ROOT / "rsa_private.pem", rsa_private_pem)
write_bytes(ROOT / "rsa_public.pem", rsa_public_pem)
write_bytes(ROOT / "rsa_private.der", rsa_private_der)
write_bytes(ROOT / "rsa_public.der", rsa_public_der)

# EC P-256
_ec_key = ec.generate_private_key(ec.SECP256R1())
ec_private_pem = _ec_key.private_bytes(
    serialization.Encoding.PEM,
    serialization.PrivateFormat.PKCS8,
    serialization.NoEncryption(),
)
ec_public_pem = _ec_key.public_key().public_bytes(
    serialization.Encoding.PEM,
    serialization.PublicFormat.SubjectPublicKeyInfo,
)
ec_private_der = _ec_key.private_bytes(
    serialization.Encoding.DER,
    serialization.PrivateFormat.PKCS8,
    serialization.NoEncryption(),
)
ec_public_der = _ec_key.public_key().public_bytes(
    serialization.Encoding.X962,
    serialization.PublicFormat.UncompressedPoint,
)
write_bytes(ROOT / "ec256_private.pem", ec_private_pem)
write_bytes(ROOT / "ec256_public.pem", ec_public_pem)
write_bytes(ROOT / "ec256_private.der", ec_private_der)
write_bytes(ROOT / "ec256_public.der", ec_public_der)

# EC P-384
_ec384_key = ec.generate_private_key(ec.SECP384R1())
ec384_private_pem = _ec384_key.private_bytes(
    serialization.Encoding.PEM,
    serialization.PrivateFormat.PKCS8,
    serialization.NoEncryption(),
)
ec384_public_pem = _ec384_key.public_key().public_bytes(
    serialization.Encoding.PEM,
    serialization.PublicFormat.SubjectPublicKeyInfo,
)
ec384_private_der = _ec384_key.private_bytes(
    serialization.Encoding.DER,
    serialization.PrivateFormat.PKCS8,
    serialization.NoEncryption(),
)
ec384_public_der = _ec384_key.public_key().public_bytes(
    serialization.Encoding.X962,
    serialization.PublicFormat.UncompressedPoint,
)
write_bytes(ROOT / "ec384_private.pem", ec384_private_pem)
write_bytes(ROOT / "ec384_public.pem", ec384_public_pem)
write_bytes(ROOT / "ec384_private.der", ec384_private_der)
write_bytes(ROOT / "ec384_public.der", ec384_public_der)

# Ed25519
ed_key = ed25519.Ed25519PrivateKey.generate()
ed_private_pem = ed_key.private_bytes(
    serialization.Encoding.PEM,
    serialization.PrivateFormat.PKCS8,
    serialization.NoEncryption(),
)
ed_public_pem = ed_key.public_key().public_bytes(
    serialization.Encoding.PEM,
    serialization.PublicFormat.SubjectPublicKeyInfo,
)
ed_private_der = ed_key.private_bytes(
    serialization.Encoding.DER,
    serialization.PrivateFormat.PKCS8,
    serialization.NoEncryption(),
)
ed_public_der = ed_key.public_key().public_bytes(
    serialization.Encoding.Raw,
    serialization.PublicFormat.Raw,
)
write_bytes(ROOT / "ed25519_private.pem", ed_private_pem)
write_bytes(ROOT / "ed25519_public.pem", ed_public_pem)
write_bytes(ROOT / "ed25519_private.der", ed_private_der)
write_bytes(ROOT / "ed25519_public.der", ed_public_der)

# JWKS (RSA + HMAC)
rsa_numbers = rsa_key.public_key().public_numbers()
modulus = rsa_numbers.n.to_bytes((rsa_numbers.n.bit_length() + 7) // 8, "big")
exponent = rsa_numbers.e.to_bytes((rsa_numbers.e.bit_length() + 7) // 8, "big")

jwks = {
    "keys": [
        {
            "kty": "RSA",
            "use": "sig",
            "alg": "RS256",
            "kid": "rsa1",
            "n": b64url(modulus),
            "e": b64url(exponent),
        },
        {
            "kty": "oct",
            "alg": "HS256",
            "kid": "hmac1",
            "k": b64url(hmac_secret),
        },
    ]
}
write_text(ROOT / "jwks.json", json.dumps(jwks, indent=2))

jwks_single = {
    "keys": [
        {
            "kty": "RSA",
            "use": "sig",
            "alg": "RS256",
            "n": b64url(modulus),
            "e": b64url(exponent),
        }
    ]
}
write_text(ROOT / "jwks_single.json", json.dumps(jwks_single, indent=2))

print("Generated fixtures in", ROOT)
