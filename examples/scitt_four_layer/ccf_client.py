# SPDX-License-Identifier: Apache-2.0
"""Minimal SCRAPI client for optional live CCF / stub SCITT registration.

Adapted from the integration client in action-state-group/scitt-cose
(tests/test_ccf_interop.py), simplified for the ecr-wg four-layer demo.

Environment / CLI:
  SCITT_CCF_URL          base URL (e.g. https://localhost:8000)
  SCITT_CCF_TLS_VERIFY=0 skip TLS verify for local VIRTUAL sandboxes

Honesty:
  * Real scitt-ccf-ledger 7.x often requires did:x509 issuers and issues
    vds=2 (ccf.v1) receipts. Plain URL issuers may 400.
  * scitt.ccf.dev has been NXDOMAIN; prefer a local docker sandbox.
  * A local stub that mints RFC9162_SHA256 (vds=1) receipts also works and
    exercises the HTTP path without CCF.
"""

from __future__ import annotations

import time
from typing import Optional


class CcfClient:
    def __init__(
        self,
        base_url: str,
        *,
        verify_tls: bool = True,
        poll_interval: float = 1.0,
        timeout: float = 60.0,
    ) -> None:
        import requests  # optional dependency for live path only

        self._s = requests.Session()
        self._s.verify = verify_tls
        self._base = base_url.rstrip("/")
        self._poll = poll_interval
        self._timeout = timeout

    def is_reachable(self) -> bool:
        for path in ("/.well-known/did.json", "/entries", "/node/network"):
            try:
                r = self._s.get(
                    f"{self._base}{path}",
                    timeout=5.0,
                    allow_redirects=False,
                )
                if r.status_code < 500:
                    return True
            except Exception:
                continue
        return False

    def submit(self, signed_statement: bytes) -> bytes:
        """Submit COSE_Sign1; return COSE Receipt bytes when committed."""
        r = self._s.post(
            f"{self._base}/entries",
            data=signed_statement,
            headers={"Content-Type": "application/cose"},
            timeout=self._timeout,
            allow_redirects=False,
        )

        if r.status_code == 303:
            location = r.headers.get("location") or r.headers.get("Location") or ""
            if not location:
                raise RuntimeError("CCF/SCRAPI: 303 without Location header")
            entry_url = location if location.startswith("http") else f"{self._base}{location}"
            return self._poll_entry_v09(entry_url)

        if r.status_code in (200, 201):
            return r.content

        if r.status_code == 202:
            body = self._decode_body(r)
            op_id = body.get("OperationId") or body.get("operationId")
            if op_id:
                entry_id = self._poll_operation_legacy(op_id)
                return self._fetch_receipt_legacy(entry_id)
            entry_id = body.get("EntryId") or body.get("entryId")
            if entry_id:
                return self._fetch_receipt_legacy(entry_id)
            raise RuntimeError(f"legacy 202: unexpected body keys {list(body)}")

        raise RuntimeError(f"submit failed {r.status_code}: {r.text[:300]}")

    def fetch_log_public_key_pem(self, *, vds: Optional[int] = None) -> bytes:
        """Best-effort TS public key for verify_receipt.

        vds=2 → prefer CCF service cert from /node/network.
        otherwise → DID assertionMethod, then service cert as fallback.
        """
        if vds == 2:
            try:
                return self._service_cert_pub_pem()
            except Exception:
                pass
        try:
            return self._did_assertion_pub_pem()
        except Exception:
            return self._service_cert_pub_pem()

    # --- internals ---------------------------------------------------------

    def _decode_body(self, r) -> dict:
        if not r.content:
            return {}
        try:
            import cbor2

            body = cbor2.loads(r.content)
            if isinstance(body, dict):
                return body
        except Exception:
            pass
        try:
            return r.json()
        except Exception as exc:
            raise RuntimeError(f"unreadable body: {r.content[:80]!r}") from exc

    def _poll_entry_v09(self, entry_url: str) -> bytes:
        deadline = time.monotonic() + self._timeout
        while time.monotonic() < deadline:
            r = self._s.get(entry_url, timeout=self._timeout, allow_redirects=False)
            if r.status_code == 200:
                return r.content
            if r.status_code in (302, 307, 429, 503):
                time.sleep(self._poll)
                continue
            r.raise_for_status()
        raise TimeoutError(f"entry {entry_url} not committed in {self._timeout}s")

    def _poll_operation_legacy(self, op_id: str) -> str:
        deadline = time.monotonic() + self._timeout
        while time.monotonic() < deadline:
            r = self._s.get(f"{self._base}/app/operations/{op_id}", timeout=self._timeout)
            if r.status_code not in (200, 202):
                r.raise_for_status()
            body = self._decode_body(r)
            status = body.get("Status") or body.get("status")
            if status == "succeeded":
                return body.get("EntryId") or body.get("entryId") or op_id
            if status == "failed":
                raise RuntimeError(f"operation failed: {body}")
            time.sleep(self._poll)
        raise TimeoutError(f"operation {op_id} timed out")

    def _fetch_receipt_legacy(self, entry_id: str) -> bytes:
        for path in (f"/entries/{entry_id}", f"/app/entries/{entry_id}/receipt"):
            r = self._s.get(f"{self._base}{path}", timeout=self._timeout)
            if r.status_code == 200 and r.content:
                return r.content
        r.raise_for_status()
        return r.content

    def _service_cert_pub_pem(self) -> bytes:
        from cryptography import x509
        from cryptography.hazmat.primitives.serialization import Encoding, PublicFormat

        r = self._s.get(f"{self._base}/node/network", timeout=self._timeout)
        r.raise_for_status()
        cert_pem = r.json()["service_certificate"]
        cert = x509.load_pem_x509_certificate(cert_pem.encode())
        return cert.public_key().public_bytes(Encoding.PEM, PublicFormat.SubjectPublicKeyInfo)

    def _did_assertion_pub_pem(self) -> bytes:
        import base64

        from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey
        from cryptography.hazmat.primitives.serialization import Encoding, PublicFormat

        r = self._s.get(f"{self._base}/.well-known/did.json", timeout=self._timeout)
        r.raise_for_status()
        did_doc = r.json()
        for key_ref in did_doc.get("assertionMethod", []):
            jwk = key_ref.get("publicKeyJwk", {}) if isinstance(key_ref, dict) else {}
            if jwk.get("kty") == "OKP" and jwk.get("crv") == "Ed25519":
                pk = Ed25519PublicKey.from_public_bytes(
                    base64.urlsafe_b64decode(jwk["x"] + "==")
                )
                return pk.public_bytes(Encoding.PEM, PublicFormat.SubjectPublicKeyInfo)
        raise RuntimeError("no Ed25519 assertionMethod key in DID document")


def receipt_vds(receipt: bytes) -> Optional[int]:
    """Read protected-header vds (label 395) from a COSE_Sign1 receipt, if present."""
    try:
        import cbor2

        tag = cbor2.loads(receipt)
        phdr = cbor2.loads(tag.value[0])
        vds = phdr.get(395)
        return int(vds) if vds is not None else None
    except Exception:
        return None
