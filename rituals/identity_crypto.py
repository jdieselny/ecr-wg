"""Identity and object hashing primitives for COSA."""

from __future__ import annotations

import hashlib
import hmac
import secrets

from rituals.runtime_paths import HMAC_SECRET_PATH, ensure_parent

HASH_ALGORITHM = "SHA-256"
HMAC_ALGORITHM = "HMAC-SHA256"


def _to_bytes(value: str | bytes) -> bytes:
    if isinstance(value, bytes):
        return value
    return str(value).encode("utf-8")


def sha256_hex(value: str | bytes) -> str:
    return hashlib.sha256(_to_bytes(value)).hexdigest()


def public_thumbprint(*parts: object, length: int = 32) -> str:
    material = "::".join(str(part) for part in parts)
    return sha256_hex(material)[:length]


def get_or_create_hmac_secret() -> bytes:
    ensure_parent(HMAC_SECRET_PATH)
    if HMAC_SECRET_PATH.exists():
        secret = HMAC_SECRET_PATH.read_text(encoding="utf-8").strip()
        if secret:
            return secret.encode("utf-8")

    secret = secrets.token_hex(32)
    HMAC_SECRET_PATH.write_text(secret + "\n", encoding="utf-8")
    return secret.encode("utf-8")


def hmac_sha256_hex(value: str | bytes, secret: str | bytes | None = None) -> str:
    key = _to_bytes(secret) if secret is not None else get_or_create_hmac_secret()
    return hmac.new(key, _to_bytes(value), hashlib.sha256).hexdigest()


def identity_prefix(op_id: str, entropy: str = "ANCHOOR", length: int = 8) -> str:
    material = f"{op_id.upper()}{entropy}"
    return hmac_sha256_hex(material)[:length].upper()


def generate_unrp_id(op_id: str, instance_id: str = "001", entropy: str = "ANCHOOR") -> str:
    return f"E-{identity_prefix(op_id, entropy)}-1845-{instance_id}"


def identity_thumbprint(op_id: str, machine_data: str, length: int = 16) -> str:
    material = f"{op_id.upper()}{machine_data}COSA_STABILITY"
    return hmac_sha256_hex(material)[:length].upper()
