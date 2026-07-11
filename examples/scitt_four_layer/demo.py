# SPDX-License-Identifier: Apache-2.0
#
# Four-layer composition demo (ecr-wg):
#   COSA (edge work product) → EMILIA (receipts + EP-AEC) → SCITT envelope (scitt-cose)
#
# Makes the "Speculative" end-to-end path from
# planning/grok_cross_stack_assessment_scitt.md into a runnable demonstration.
#
# What this proves offline (no network, no hosted Transparency Service):
#   0. Continuum ingress: Packing Slip + Bill of Lading seal the order into a
#      transport packet; COGOBJ carries the same ingress + authorization digests.
#   1. A named human (grid authority) and a facility both authorize the *same*
#      canonical curtailment action via EP-RECEIPT-v1 (EMILIA).
#   2. EP-AEC-v1 composes those receipts fail-closed (confused-deputy refused).
#   3. A COSA-edge work product (signed shed telemetry) binds to that action.
#   4. The Proof-of-Curtailment bundle (incl. ingress + COGOBJ) is wrapped as a
#      SCITT Signed Statement (COSE_Sign1) and given a COSE Receipt
#      (RFC9162_SHA256) via scitt-cose primitives — no real TS required.
#   5. Dual independent verification: emilia_verify on the AEC; scitt_cose on
#      the statement + receipt; BoL re-verify inside the pack.
#
# Not claimed:
#   - Operating a real SCITT Transparency Service (out of scope for scitt-cose).
#   - Identity proofing of the human behind the approver key (shared stack gap).
#   - VAP-LAP Evidence Pack schema (optional packaging layer; not required here).
#
# Run from repo root:
#   pip install -r examples/scitt_four_layer/requirements.txt
#   python examples/scitt_four_layer/demo.py
#   python examples/scitt_four_layer/demo.py --ccf-url https://localhost:8000  # optional live TS
#
# Note: PyPI scitt-cose 0.1.1 is RFC9162-only. This demo needs main (vds=1 + vds=2/CCF):
#   pip install "scitt-cose @ git+https://github.com/action-state-group/scitt-cose.git"

from __future__ import annotations

import argparse
import base64
import hashlib
import json
import os
import secrets
import sys
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

# UTF-8 stdout on Windows consoles.
if hasattr(sys.stdout, "reconfigure"):
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass

from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
from emilia_verify import (
    AEC_VERSION,
    action_digest,
    canonicalize,
    verify_authorization_chain,
    verify_receipt,
)
from scitt_cose import (
    DRAFT_TRACKING_NOTICE,
    attach_receipts,
    build_receipt,
    build_signed_statement,
    extract_receipts,
    merkle_root,
    parse_signed_statement,
    verify_receipt as scitt_verify_receipt,
)

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def _b64u(b: bytes) -> str:
    return base64.urlsafe_b64encode(b).rstrip(b"=").decode("ascii")


def _pem_pair(sk: Ed25519PrivateKey) -> tuple[bytes, bytes]:
    priv = sk.private_bytes(
        serialization.Encoding.PEM,
        serialization.PrivateFormat.PKCS8,
        serialization.NoEncryption(),
    )
    pub = sk.public_key().public_bytes(
        serialization.Encoding.PEM,
        serialization.PublicFormat.SubjectPublicKeyInfo,
    )
    return priv, pub


def _spki_b64u(sk: Ed25519PrivateKey) -> str:
    """EMILIA trusted-key form: base64url(DER SPKI)."""
    der = sk.public_key().public_bytes(
        serialization.Encoding.DER,
        serialization.PublicFormat.SubjectPublicKeyInfo,
    )
    return _b64u(der)


def _sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _jcs_bytes(obj: Any) -> bytes:
    return canonicalize(obj).encode("utf-8")


def line(s: str = "") -> None:
    print(s)


def ok(msg: str) -> None:
    line(f"     ✓ {msg}")


def bad(msg: str) -> None:
    line(f"     ✗ {msg}")


def step(n: int, title: str) -> None:
    line()
    line(f"  {n}. {title}")


# ---------------------------------------------------------------------------
# Layer keys (demo-ephemeral; production pins published keys)
# ---------------------------------------------------------------------------


@dataclass
class Party:
    name: str
    role: str
    sk: Ed25519PrivateKey
    emilia_key: str  # b64u DER SPKI for emilia_verify
    priv_pem: bytes  # PEM for scitt-cose / COSA edge
    pub_pem: bytes


def make_party(name: str, role: str) -> Party:
    sk = Ed25519PrivateKey.generate()
    priv, pub = _pem_pair(sk)
    return Party(name=name, role=role, sk=sk, emilia_key=_spki_b64u(sk), priv_pem=priv, pub_pem=pub)


# ---------------------------------------------------------------------------
# EMILIA: EP-RECEIPT-v1 + EP-AEC-v1
# ---------------------------------------------------------------------------


def issue_ep_receipt(
    *,
    party: Party,
    subject: str,
    action_type: str,
    bound_action_digest: str,
    outcome: str = "allow_with_signoff",
) -> dict:
    """Named party signs an EP-RECEIPT-v1 bound to a canonical action digest.

    The digest lives inside the *signed* payload claim so AEC binding cannot be
    rewired by mutating an unsigned top-level field.
    """
    payload = {
        "receipt_id": "rcpt_" + secrets.token_hex(6),
        "subject": subject,
        "created_at": _now_iso(),
        "claim": {
            "action_type": action_type,
            "outcome": outcome,
            "approver": party.name,
            "role": party.role,
            # Bare hex; AEC verifier normalizes sha256: prefix too.
            "action_digest": bound_action_digest,
        },
    }
    sig = party.sk.sign(_jcs_bytes(payload))
    return {
        "@version": "EP-RECEIPT-v1",
        "payload": payload,
        "signature": {"algorithm": "Ed25519", "value": _b64u(sig)},
        # Convenience for demos / tooling; binding truth is the signed claim.
        "operator_public_key": party.emilia_key,
        "action_hash": bound_action_digest,
    }


def aec_ep_receipt_verifier(evidence: Any, ctx: dict) -> dict:
    """AEC component verifier for EP-RECEIPT-v1 with signed action_digest claim.

    Stronger than the stock builtin for this profile: the action digest is taken
    from the *signed* claim, not an unsigned envelope field.
    """
    keys = ctx.get("keys") or {}
    key = keys.get(evidence.get("operator_public_key")) if isinstance(evidence, dict) else None
    if key is None and isinstance(evidence, dict):
        key = evidence.get("operator_public_key")
    if not key:
        return {"valid": False, "action_digest": None}
    try:
        res = verify_receipt(evidence, key)
    except Exception:
        return {"valid": False, "action_digest": None}
    if not getattr(res, "valid", False):
        return {"valid": False, "action_digest": None}
    claim = ((evidence.get("payload") or {}).get("claim")) or {}
    digest = claim.get("action_digest")
    return {"valid": True, "action_digest": digest}


def build_aec(
    action: dict,
    *,
    grid_receipt: dict,
    facility_receipt: dict,
) -> dict:
    digest = action_digest(action)
    return {
        "@version": AEC_VERSION,
        "action": action,
        "action_digest": digest,
        "requirement": "grid_order AND facility_ack",
        "components": [
            {
                "type": "ep-receipt",
                "label": "grid_order",
                "evidence": grid_receipt,
            },
            {
                "type": "ep-receipt",
                "label": "facility_ack",
                "evidence": facility_receipt,
            },
        ],
    }


# ---------------------------------------------------------------------------
# COSA: edge work product (signed shed telemetry)
# ---------------------------------------------------------------------------


def issue_cosa_work_product(
    *,
    plane: Party,
    action_digest_hex: str,
    shed_mw: float,
    window: dict,
    baseline_method_hash: str,
) -> dict:
    """L5/L7-adjacent edge attestation: we shed this much, bound to this action."""
    body = {
        "work_product_id": "wp_" + secrets.token_hex(6),
        "kind": "cosa.curtailment.telemetry",
        "action_digest": action_digest_hex,
        "shed_mw": shed_mw,
        "window": window,
        "baseline_method_hash": baseline_method_hash,
        "measured_at": _now_iso(),
        "plane": plane.name,
    }
    sig = plane.sk.sign(_jcs_bytes(body))
    return {
        "@version": "COSA-WORK-PRODUCT-v0.1",
        "body": body,
        "signature": {"algorithm": "Ed25519", "value": _b64u(sig)},
        "plane_public_key": plane.emilia_key,
    }


def verify_cosa_work_product(wp: dict, plane_key_b64u: str) -> bool:
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey
    from cryptography.hazmat.primitives.serialization import load_der_public_key

    try:
        der = base64.urlsafe_b64decode(plane_key_b64u + "=" * (-len(plane_key_b64u) % 4))
        pub = load_der_public_key(der)
        assert isinstance(pub, Ed25519PublicKey)
        sig = base64.urlsafe_b64decode(
            wp["signature"]["value"] + "=" * (-len(wp["signature"]["value"]) % 4)
        )
        pub.verify(sig, _jcs_bytes(wp["body"]))
        return True
    except Exception:
        return False


# ---------------------------------------------------------------------------
# Ingress: Packing Slip + Bill of Lading (whiteboard remnants → wire prototype)
# ---------------------------------------------------------------------------
#
# Specs: specs/primitives/packing-slip.md, specs/primitives/bill-of-lading.md
# These ride *with* the curtailment packet (bundle + COGOBJ), not inside the
# EMILIA action object. Authorization digests stay pure; ingress is provenance
# for *how the order entered the overlay*.


def seal_packing_slip(
    *,
    raw_input: str,
    synth: Party,
    short_term_context: dict,
    long_term_context_refs: list[str],
    grace_fields: dict,
) -> dict:
    """Build a Packing Slip and seal it with SHA-256 over JCS(body).

    The slip is unsigned (per primitive draft); the BoL supplies the signature.
    """
    body = {
        "@version": "ECR-PACKING-SLIP-v0.1",
        "raw_input": raw_input,
        "synth_id": synth.name,
        "short_term_context": short_term_context,
        "long_term_context_refs": long_term_context_refs,
        "grace_fields": grace_fields,
        "timestamp": _now_iso(),
    }
    slip_hash = "sha256:" + _sha256_hex(_jcs_bytes(body))
    return {**body, "hash": slip_hash}


def wrap_bill_of_lading(
    *,
    packing_slip: dict,
    sender: Party,
    grace_attestation: dict,
    routing_intent: str,
    return_path: dict,
) -> dict:
    """Wrap a Packing Slip in a signed Bill of Lading (BoL).

    Signature is Ed25519 over JCS of the unsigned payload (everything except
    sender_signature). packing_slip_hash binds cargo; slip travels unchanged.
    """
    payload = {
        "@version": "ECR-BILL-OF-LADING-v0.1",
        "packing_slip_hash": packing_slip["hash"],
        "packing_slip": packing_slip,
        "grace_attestation": grace_attestation,
        "routing_intent": routing_intent,
        "return_path": return_path,
        "bol_timestamp": _now_iso(),
        "sender": sender.name,
        "sender_public_key": sender.emilia_key,
    }
    sig = sender.sk.sign(_jcs_bytes(payload))
    return {
        **payload,
        "sender_signature": {"algorithm": "Ed25519", "value": _b64u(sig)},
    }


def verify_bill_of_lading(bol: dict) -> tuple[bool, str]:
    """Offline BoL check: recompute slip hash, verify sender signature."""
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey
    from cryptography.hazmat.primitives.serialization import load_der_public_key

    slip = bol.get("packing_slip") or {}
    if not isinstance(slip, dict) or "hash" not in slip:
        return False, "missing packing_slip"
    # Re-seal: drop hash field, recompute.
    slip_body = {k: v for k, v in slip.items() if k != "hash"}
    recomputed = "sha256:" + _sha256_hex(_jcs_bytes(slip_body))
    if recomputed != slip.get("hash"):
        return False, "packing_slip hash mismatch"
    if bol.get("packing_slip_hash") != slip.get("hash"):
        return False, "bol packing_slip_hash does not match slip.hash"

    sig_obj = bol.get("sender_signature") or {}
    key_b64u = bol.get("sender_public_key") or ""
    try:
        der = base64.urlsafe_b64decode(key_b64u + "=" * (-len(key_b64u) % 4))
        pub = load_der_public_key(der)
        assert isinstance(pub, Ed25519PublicKey)
        sig = base64.urlsafe_b64decode(
            sig_obj["value"] + "=" * (-len(sig_obj["value"]) % 4)
        )
        unsigned = {k: v for k, v in bol.items() if k != "sender_signature"}
        pub.verify(sig, _jcs_bytes(unsigned))
    except Exception as exc:
        return False, f"sender_signature invalid: {exc}"
    return True, "ok"


def build_curtailment_cogobj(
    *,
    action: dict,
    action_digest_hex: str,
    bol: dict,
    work_product: dict,
    plane: Party,
) -> dict:
    """COGOBJ packet showing ingress (Packing Slip + BoL) + curtailment resolution.

    Extends thesis/COGOBJ_SCHEMA.md with optional `ingress` and `authorization`
    bindings so the cognitive object carries the same cargo as the PoC bundle.
    """
    slip = bol["packing_slip"]
    problem = slip.get("raw_input") or "grid.curtailment order"
    goal = (slip.get("grace_fields") or {}).get("GOAL") or "Execute authorized grid curtailment"
    body = {
        "id": _sha256_hex(_jcs_bytes({"action_digest": action_digest_hex, "kind": "cogobj"}))[:32],
        "intent_key": "GRID::CURTAILMENT::AUTHORIZED",
        "data": {
            "problem": problem,
            "goal": goal,
            "resolution": [
                f"ACTION_TYPE: {action['action_type']}",
                f"TARGET_SET: {','.join(action['target_set'])}",
                f"MAGNITUDE_MW: {action['magnitude_mw']}",
                f"ACTION_DIGEST: {action_digest_hex}",
                f"SHED_MW: {work_product['body']['shed_mw']}",
                f"WORK_PRODUCT: {work_product['body']['work_product_id']}",
            ],
        },
        "provenance": {
            "origin_node_id": plane.name,
            "provenance_tier": "E",
            "timestamp": _now_iso(),
            "model": "grid.curtailment.packet",
        },
        "ingress": {
            "packing_slip_hash": bol["packing_slip_hash"],
            "bill_of_lading": {
                "version": bol["@version"],
                "sender": bol["sender"],
                "routing_intent": bol["routing_intent"],
                "bol_timestamp": bol["bol_timestamp"],
                "sender_signature": bol["sender_signature"],
                "sender_public_key": bol["sender_public_key"],
                # Full slip nested so a reader sees cargo without a side channel.
                "packing_slip": slip,
            },
        },
        "authorization": {
            "action_digest": action_digest_hex,
            "policy_id": action.get("policy_id"),
            "profile": "grid.curtailment",
        },
        "aft_score": 0.99,
        "validation": "EP_AEC_AND_COSA_BOUND",
    }
    return body


# ---------------------------------------------------------------------------
# Packaging + SCITT (scitt-cose substrate)
# ---------------------------------------------------------------------------


def build_proof_of_curtailment_bundle(
    *,
    action: dict,
    aec: dict,
    aec_verify: dict,
    work_product: dict,
    packing_slip: dict | None = None,
    bill_of_lading: dict | None = None,
    cogobj: dict | None = None,
) -> dict:
    """Domain bundle that becomes the *opaque* SCITT statement payload.

    This is deliberately a plain JSON object — not VAP-LAP. Packaging formats
    are interchangeable envelopes; the content is EMILIA + COSA + optional
    Continuum ingress (Packing Slip + Bill of Lading) and the COGOBJ packet.
    """
    bundle: dict[str, Any] = {
        "@version": "ECR-POC-BUNDLE-v0.2",
        "profile": "grid.curtailment",
        "action": action,
        "action_digest": action_digest(action),
        "aec": {
            "@version": aec["@version"],
            "action_digest": aec["action_digest"],
            "requirement": aec["requirement"],
            "component_labels": [c["label"] for c in aec["components"]],
            # Embed full AEC so an auditor can re-run emilia_verify without a side channel.
            "chain": aec,
            "verify_snapshot": {
                "allow": aec_verify.get("allow"),
                "action_digest": aec_verify.get("action_digest"),
            },
        },
        "cosa_work_product": work_product,
        "assembled_at": _now_iso(),
    }
    if packing_slip is not None and bill_of_lading is not None:
        bundle["ingress"] = {
            "packing_slip": packing_slip,
            "bill_of_lading": bill_of_lading,
            "note": (
                "Packing Slip is unsigned cargo; Bill of Lading is the signed "
                "transport contract. Neither mutates the EMILIA action_digest."
            ),
        }
    if cogobj is not None:
        bundle["cogobj"] = cogobj
    return bundle


def register_in_demo_log(
    statement: bytes,
    *,
    log: Party,
    filler_count: int = 3,
    prior_entries: list[str] | None = None,
) -> tuple[bytes, str, list[str], int]:
    """Simulate a Transparency Service registration using scitt-cose primitives.

    Leaf entry = SHA-256(Signed Statement bytes). A real TS would append to a
    durable log and gossip; here we build a small in-memory tree and mint a
    COSE Receipt with build_receipt so verify_receipt is exercised for real.
    """
    leaf = _sha256_hex(statement)
    if prior_entries is not None:
        entries = list(prior_entries) + [leaf]
        leaf_index = len(entries) - 1
    else:
        fillers = [
            _sha256_hex(f"filler-entry-{i}-{secrets.token_hex(4)}".encode())
            for i in range(filler_count)
        ]
        # Put our leaf near the middle so the inclusion path is non-trivial.
        leaf_index = min(2, len(fillers))
        entries = fillers[:leaf_index] + [leaf] + fillers[leaf_index:]
    receipt = build_receipt(
        leaf_entry_hex=leaf,
        leaf_index=leaf_index,
        tree_entries_hex=entries,
        alg="EdDSA",
        log_private_key_pem=log.priv_pem,
    )
    return receipt, leaf, entries, leaf_index


def verify_ccf_frozen_vector(fixtures_dir: Path) -> dict:
    """Verify the committed real-CCF (vds=2) frozen vector with scitt-cose."""
    statement = (fixtures_dir / "statement.cose").read_bytes()
    receipt = (fixtures_dir / "receipt.cose").read_bytes()
    log_pub = (fixtures_dir / "log-key.pub").read_bytes()
    issuer_pub = (fixtures_dir / "issuer-key.pub").read_bytes()
    expected = json.loads((fixtures_dir / "expected.json").read_text(encoding="utf-8"))

    leaf = _sha256_hex(statement)
    stmt = parse_signed_statement(statement, public_key_pem=issuer_pub)
    rcpt = scitt_verify_receipt(receipt, leaf_entry_hex=leaf, log_public_key_pem=log_pub)
    return {
        "statement_ok": bool(stmt.get("signature_verified")),
        "receipt_ok": bool(rcpt.ok),
        "root": rcpt.root,
        "expected_root": expected.get("reconstructed_root"),
        "leaf": leaf,
        "expected_leaf": expected.get("leaf_entry"),
        "errors": list(rcpt.errors or []),
    }


def try_live_ccf_register(
    statement: bytes,
    *,
    ccf_url: str,
    verify_tls: bool,
) -> dict:
    """Best-effort live registration. Never raises — returns a status dict."""
    try:
        from ccf_client import CcfClient, receipt_vds
    except ImportError:
        # running as script from this directory
        sys.path.insert(0, str(Path(__file__).resolve().parent))
        from ccf_client import CcfClient, receipt_vds  # type: ignore

    try:
        import requests  # noqa: F401
    except ImportError:
        return {"status": "skipped", "reason": "requests not installed (pip install requests)"}

    client = CcfClient(ccf_url, verify_tls=verify_tls)
    if not client.is_reachable():
        return {"status": "skipped", "reason": f"endpoint unreachable: {ccf_url}"}

    try:
        receipt = client.submit(statement)
    except Exception as exc:
        return {"status": "error", "reason": str(exc)[:400]}

    vds = receipt_vds(receipt)
    try:
        log_pub = client.fetch_log_public_key_pem(vds=vds)
    except Exception as exc:
        return {
            "status": "partial",
            "reason": f"got receipt ({len(receipt)} bytes, vds={vds}) but no log key: {exc}",
            "receipt": receipt,
            "vds": vds,
        }

    leaf = _sha256_hex(statement)
    res = scitt_verify_receipt(receipt, leaf_entry_hex=leaf, log_public_key_pem=log_pub)
    return {
        "status": "ok" if res.ok else "verify_failed",
        "vds": vds,
        "receipt": receipt,
        "log_pub": log_pub,
        "root": res.root,
        "errors": list(res.errors or []),
        "leaf": leaf,
    }


# ---------------------------------------------------------------------------
# Demo
# ---------------------------------------------------------------------------


def main() -> int:
    parser = argparse.ArgumentParser(description="Four-layer COSA+EMILIA+scitt-cose demo")
    parser.add_argument(
        "--ccf-url",
        default=os.environ.get("SCITT_CCF_URL") or "",
        help="Optional live SCITT/CCF base URL (also SCITT_CCF_URL). Empty = skip live path.",
    )
    parser.add_argument(
        "--ccf-tls-verify",
        action=argparse.BooleanOptionalAction,
        default=os.environ.get("SCITT_CCF_TLS_VERIFY", "1") != "0",
        help="TLS verify for --ccf-url (default true; use --no-ccf-tls-verify for local VIRTUAL).",
    )
    parser.add_argument(
        "--skip-ccf-frozen",
        action="store_true",
        help="Skip the offline real-CCF vds=2 frozen vector check.",
    )
    args = parser.parse_args()

    failures = 0
    here = Path(__file__).resolve().parent
    ccf_fixtures = here / "fixtures" / "ccf-vds2"

    line()
    line("  Four-layer composition demo (ecr-wg)")
    line("  COSA work product + EMILIA receipts/AEC + scitt-cose SCITT envelope")
    line("  " + "-" * 72)
    line(f"  scitt-cose: {DRAFT_TRACKING_NOTICE[:88]}…")
    line("  emilia:     emilia-verify (EP-RECEIPT-v1 + EP-AEC-v1)")
    line(
        "  network:    offline by default; "
        + (f"live TS at {args.ccf_url}" if args.ccf_url else "no live TS (--ccf-url to try)")
    )

    # Parties -----------------------------------------------------------------
    grid = make_party("human:grid-authority-1", "grid_order")
    facility = make_party("human:facility-ops-1", "facility_ack")
    plane = make_party("agent:cosa-edge-plane", "cosa_plane")
    # Facility synth builds Packing Slip + BoL at ingress (Continuum whiteboard path).
    synth = make_party("agent:facility-synth-1", "synth_ingress")
    issuer = make_party("https://issuer.ecr-wg.example/poc", "scitt_issuer")
    log_a = make_party("https://log-a.demo.ecr-wg.example", "scitt_log_a")
    log_b = make_party("https://log-b.demo.ecr-wg.example", "scitt_log_b")

    # 1. Canonical curtailment action ----------------------------------------
    step(1, "COSA/EMILIA shared action object (canonical curtailment order)")
    action = {
        "action_type": "grid.curtailment",
        "effect_class": "power_reduction",
        "target_set": ["us-east-1-facility-a"],
        "magnitude_mw": 50,
        "window": {
            "not_before": "2026-07-09T18:00:00Z",
            "not_after": "2026-07-09T20:00:00Z",
        },
        "baseline_method_hash": "sha256:" + _sha256_hex(b"iso-baseline-method-v3"),
        "protected_lanes": ["life-safety", "grid-critical-telemetry"],
        "policy_id": "ep:policy:grid-curtailment@v1",
    }
    digest = action_digest(action)
    ok(f"action_type={action['action_type']}  magnitude={action['magnitude_mw']} MW")
    ok(f"action_digest (JCS/SHA-256) = {digest[:16]}…{digest[-8:]}")

    # 1b. Ingress envelope: Packing Slip + Bill of Lading ---------------------
    step(2, "Ingress: Packing Slip + Bill of Lading (bind order into Continuum packet)")
    raw_order = (
        "ISO demand-response: shed 50 MW at us-east-1-facility-a for window "
        "2026-07-09T18:00Z..20:00Z; preserve life-safety and grid-critical telemetry."
    )
    packing_slip = seal_packing_slip(
        raw_input=raw_order,
        synth=synth,
        short_term_context={
            "session": "curtailment-intake-demo",
            "facility": "us-east-1-facility-a",
            "active_lanes": ["life-safety", "grid-critical-telemetry", "batch-train"],
            "local_headroom_mw": 120,
        },
        long_term_context_refs=[
            "cogstor:policy/grid-curtailment@v1",
            "cogstor:baseline/iso-baseline-method-v3",
            "enrollment:agent:facility-synth-1",
        ],
        grace_fields={
            "GOAL": "Execute authorized grid.curtailment without cold-path overclaim",
            "CONSTRAINTS": [
                "require EP-AEC grid_order AND facility_ack",
                "protected_lanes must not shed",
                "magnitude_mw <= authorized",
            ],
            "ROUTING": "cosa-overlay:facility-edge",
            "ANCHOR": digest,
            "EVIDENCE": "attach packing_slip + bol + cosa work product to PoC bundle",
        },
    )
    bill_of_lading = wrap_bill_of_lading(
        packing_slip=packing_slip,
        sender=synth,
        grace_attestation={
            "constraints_checked": True,
            "grace_fields_populated": True,
            "action_digest_anchor": digest,
            "note": "Sender attests GRACE fields were populated before overlay entry.",
        },
        routing_intent="cosa-overlay:facility-edge",
        return_path={
            "kind": "queue",
            "ref": "facility:us-east-1-a:curtailment-acks",
        },
    )
    bol_ok, bol_reason = verify_bill_of_lading(bill_of_lading)
    if not bol_ok:
        bad(f"Bill of Lading verify failed: {bol_reason}")
        return 1
    ok(f"packing_slip.hash = {packing_slip['hash'][:20]}…")
    ok(f"bill_of_lading signed by {synth.name}; packing_slip_hash bound")
    ok("ingress sealed before EMILIA receipts (cargo ≠ authorization digest)")

    # 2. EMILIA receipts ------------------------------------------------------
    step(3, "EMILIA: issue EP-RECEIPT-v1 from grid authority + facility (bound to digest)")
    grid_rcpt = issue_ep_receipt(
        party=grid,
        subject="facility:us-east-1-a",
        action_type="grid.curtailment",
        bound_action_digest=digest,
    )
    facility_rcpt = issue_ep_receipt(
        party=facility,
        subject="facility:us-east-1-a",
        action_type="grid.curtailment",
        bound_action_digest=digest,
    )
    g_res = verify_receipt(grid_rcpt, grid.emilia_key)
    f_res = verify_receipt(facility_rcpt, facility.emilia_key)
    if not g_res.valid or not f_res.valid:
        bad(f"receipt verify failed: grid={g_res} facility={f_res}")
        return 1
    ok(f"grid receipt {grid_rcpt['payload']['receipt_id']} verifies (Ed25519 over JCS)")
    ok(f"facility receipt {facility_rcpt['payload']['receipt_id']} verifies")

    # 3. EP-AEC composition ---------------------------------------------------
    step(4, "EMILIA: compose EP-AEC-v1 (requirement: grid_order AND facility_ack)")
    aec = build_aec(action, grid_receipt=grid_rcpt, facility_receipt=facility_rcpt)
    aec_result = verify_authorization_chain(
        aec,
        verifiers={"ep-receipt": aec_ep_receipt_verifier},
        keys={},
    )
    if not aec_result.get("allow"):
        bad(f"AEC refused unexpectedly: {aec_result}")
        return 1
    ok(f"AEC allow=True  action_digest={aec_result['action_digest'][:16]}…")
    for c in aec_result.get("components") or []:
        ok(f"component {c.get('label')}: valid={c.get('valid')} bound={c.get('bound')}")

    # 3b. Confused-deputy refusal --------------------------------------------
    step(5, "Negative: confused-deputy — facility receipt bound to a DIFFERENT action")
    other_action = dict(action)
    other_action["magnitude_mw"] = 5  # different irreversible effect
    other_digest = action_digest(other_action)
    evil_facility = issue_ep_receipt(
        party=facility,
        subject="facility:us-east-1-a",
        action_type="grid.curtailment",
        bound_action_digest=other_digest,
    )
    evil_aec = build_aec(action, grid_receipt=grid_rcpt, facility_receipt=evil_facility)
    evil_result = verify_authorization_chain(
        evil_aec,
        verifiers={"ep-receipt": aec_ep_receipt_verifier},
    )
    if evil_result.get("allow"):
        bad("AEC incorrectly allowed a cross-bound facility receipt")
        failures += 1
    else:
        ok("AEC fail-closed on cross-bound receipt (facility_ack not bound to chain action)")
        reasons = evil_result.get("reasons") or []
        # component-level reason
        for c in evil_result.get("components") or []:
            if c.get("label") == "facility_ack" and not c.get("bound"):
                ok(f"facility_ack reason: {c.get('reason')}")

    # 4. COSA work product ----------------------------------------------------
    step(6, "COSA: edge plane signs curtailment telemetry work product")
    work_product = issue_cosa_work_product(
        plane=plane,
        action_digest_hex=digest,
        shed_mw=float(action["magnitude_mw"]),
        window=action["window"],
        baseline_method_hash=action["baseline_method_hash"],
    )
    if not verify_cosa_work_product(work_product, plane.emilia_key):
        bad("COSA work product signature failed")
        return 1
    if work_product["body"]["action_digest"] != digest:
        bad("work product not bound to action digest")
        return 1
    ok(f"work product {work_product['body']['work_product_id']} signed; bound to action_digest")
    ok(f"shed_mw={work_product['body']['shed_mw']} (matches authorized magnitude)")

    cogobj = build_curtailment_cogobj(
        action=action,
        action_digest_hex=digest,
        bol=bill_of_lading,
        work_product=work_product,
        plane=plane,
    )
    ok(f"COGOBJ {cogobj['id']} assembled with ingress.packing_slip_hash + authorization.action_digest")

    # 5. Bundle + SCITT Signed Statement --------------------------------------
    step(7, "Package Proof-of-Curtailment bundle (w/ Packing Slip+BoL+COGOBJ) → SCITT")
    bundle = build_proof_of_curtailment_bundle(
        action=action,
        aec=aec,
        aec_verify=aec_result,
        work_product=work_product,
        packing_slip=packing_slip,
        bill_of_lading=bill_of_lading,
        cogobj=cogobj,
    )
    # Ingress integrity inside the pack (auditor path).
    re_bol_ok, re_bol_reason = verify_bill_of_lading(bundle["ingress"]["bill_of_lading"])
    if not re_bol_ok or bundle["ingress"]["packing_slip"]["hash"] != packing_slip["hash"]:
        bad(f"bundle ingress failed re-verify: {re_bol_reason}")
        return 1
    if bundle["cogobj"]["ingress"]["packing_slip_hash"] != packing_slip["hash"]:
        bad("COGOBJ packing_slip_hash does not match sealed slip")
        return 1
    if bundle["cogobj"]["authorization"]["action_digest"] != digest:
        bad("COGOBJ action_digest does not match action")
        return 1
    ok("bundle.ingress BoL re-verifies; COGOBJ hashes bound to slip + action_digest")
    payload = _jcs_bytes(bundle)
    statement = build_signed_statement(
        payload,
        alg="EdDSA",
        private_key_pem=issuer.priv_pem,
        issuer="https://issuer.ecr-wg.example/poc",
        subject=f"poc:{digest[:16]}",
        content_type="application/json",
        extra_cwt_claims={
            "profile": "grid.curtailment",
            "action_digest": digest,
        },
    )
    parsed = parse_signed_statement(statement, public_key_pem=issuer.pub_pem)
    if not parsed.get("signature_verified"):
        bad(f"SCITT statement failed: {parsed}")
        return 1
    ok(f"COSE_Sign1 Signed Statement verifies (issuer={parsed.get('issuer')})")
    ok(f"subject={parsed.get('subject')}  payload={len(payload)} JCS bytes")
    ok(f"CWT claims label-15 profile={parsed.get('claims', {}).get('profile')}")

    # 6. Dual independent demo logs (statement portability) -------------------
    step(8, "SCITT: dual independent RFC9162 logs mint receipts over the SAME statement")
    line("     (Primitives only — not a hosted Transparency Service.)")
    receipt_a, leaf, entries_a, leaf_index_a = register_in_demo_log(statement, log=log_a)
    receipt_b, leaf_b, entries_b, leaf_index_b = register_in_demo_log(
        statement, log=log_b, prior_entries=[_sha256_hex(b"peer-prior-1"), _sha256_hex(b"peer-prior-2")]
    )
    assert leaf == leaf_b  # leaf depends only on statement bytes
    root_a = merkle_root(entries_a)
    root_b = merkle_root(entries_b)
    r_a = scitt_verify_receipt(receipt_a, leaf_entry_hex=leaf, log_public_key_pem=log_a.pub_pem)
    r_b = scitt_verify_receipt(receipt_b, leaf_entry_hex=leaf, log_public_key_pem=log_b.pub_pem)
    if not (r_a.ok and r_b.ok and r_a.root == root_a and r_b.root == root_b):
        bad(f"dual-log receipts failed: a={r_a} b={r_b}")
        return 1
    # Cross-key must fail (receipts are key-specific).
    cross = scitt_verify_receipt(receipt_a, leaf_entry_hex=leaf, log_public_key_pem=log_b.pub_pem)
    if cross.ok:
        bad("receipt_a verified against log_b key — cross-TS confusion")
        failures += 1
    else:
        ok(f"log-a receipt ok  leaf_index={leaf_index_a} root={root_a[:16]}…")
        ok(f"log-b receipt ok  leaf_index={leaf_index_b} root={root_b[:16]}…")
        ok("same leaf_entry_hex in both logs (statement is the invariant)")
        ok("cross-key reject: log-a receipt does not verify under log-b key")

    transparent = attach_receipts(statement, [receipt_a, receipt_b])
    ok(f"Transparent Statement assembled ({len(extract_receipts(transparent))} receipts attached)")

    # 7. Dual independent verification ----------------------------------------
    step(9, "Dual independent offline verification (the federal-grade story)")
    re_aec = verify_authorization_chain(
        bundle["aec"]["chain"],
        verifiers={"ep-receipt": aec_ep_receipt_verifier},
    )
    re_stmt = parse_signed_statement(transparent, public_key_pem=issuer.pub_pem)
    re_rcpt = scitt_verify_receipt(
        extract_receipts(transparent)[0],
        leaf_entry_hex=leaf,
        log_public_key_pem=log_a.pub_pem,
    )
    re_wp = verify_cosa_work_product(bundle["cosa_work_product"], plane.emilia_key)
    re_bol, _ = verify_bill_of_lading(bundle["ingress"]["bill_of_lading"])

    dual_ok = (
        re_aec.get("allow")
        and re_stmt.get("signature_verified")
        and re_rcpt.ok
        and re_wp
        and re_bol
        and re_aec.get("action_digest") == digest
    )
    if not dual_ok:
        bad(
            f"dual verify failed: aec={re_aec.get('allow')} stmt={re_stmt.get('signature_verified')} "
            f"rcpt={re_rcpt.ok} wp={re_wp} bol={re_bol}"
        )
        failures += 1
    else:
        ok("emilia_verify: AEC allow=True (authorization content)")
        ok("scitt_cose: Signed Statement signature_verified=True (envelope)")
        ok("scitt_cose: Receipt ok + root reconstructed (inclusion without trusting log word)")
        ok("cosa: work product signature valid and action-bound")
        ok("ingress: Bill of Lading + Packing Slip hash re-verify inside pack")

    # 8. Real CCF frozen vector (vds=2) ----------------------------------------
    step(10, "CCF interop (offline): real scitt-ccf-ledger vds=2 frozen vector")
    line("     Fixture from scitt-cose test-vectors/v1/valid-ccf-vds2 (Apache-2.0).")
    line("     Different statement than our PoC — proves the *verifier* accepts CCF receipts.")
    ccf_live_meta: dict = {"status": "skipped", "reason": "no --ccf-url"}
    if args.skip_ccf_frozen:
        line("     (skipped via --skip-ccf-frozen)")
    elif not ccf_fixtures.is_dir():
        bad(f"missing fixtures at {ccf_fixtures}")
        failures += 1
    else:
        ccf = verify_ccf_frozen_vector(ccf_fixtures)
        if not (
            ccf["statement_ok"]
            and ccf["receipt_ok"]
            and ccf["root"] == ccf["expected_root"]
            and ccf["leaf"] == ccf["expected_leaf"]
        ):
            bad(f"CCF frozen vector failed: {ccf}")
            failures += 1
            line(
                "     hint: need scitt-cose main (vds=2). "
                "pip install 'scitt-cose @ git+https://github.com/action-state-group/scitt-cose.git'"
            )
        else:
            ok("CCF statement signature_verified=True (did:x509 / ES256)")
            ok(f"CCF receipt ok (vds=2 ccf.v1) root={ccf['root'][:16]}… matches expected")
            ok("same scitt_cose.verify_receipt path handles RFC9162 + CCF")

    # 9. Optional live TS registration ----------------------------------------
    step(11, "Optional live TS: register OUR Signed Statement (SCITT_CCF_URL / --ccf-url)")
    if not args.ccf_url:
        line("     skipped — pass --ccf-url https://localhost:8000 after starting scitt-ccf-ledger")
        line("     (Docker required; real CCF 7.x may demand did:x509 issuer → 400 on URL issuers)")
    else:
        ccf_live_meta = try_live_ccf_register(
            statement, ccf_url=args.ccf_url, verify_tls=args.ccf_tls_verify
        )
        st = ccf_live_meta.get("status")
        if st == "ok":
            ok(
                f"live register+verify ok  vds={ccf_live_meta.get('vds')}  "
                f"root={(ccf_live_meta.get('root') or '')[:16]}…"
            )
            transparent = attach_receipts(
                statement, [receipt_a, receipt_b, ccf_live_meta["receipt"]]
            )
            ok(f"Transparent Statement now has {len(extract_receipts(transparent))} receipts (incl. live)")
        elif st == "skipped":
            line(f"     skipped — {ccf_live_meta.get('reason')}")
        elif st == "verify_failed":
            bad(f"live receipt obtained but verify failed: {ccf_live_meta.get('errors')}")
            # Do not fail the whole demo if CCF node is misconfigured — surface as soft fail.
            failures += 1
        else:
            line(f"     live path: {st} — {ccf_live_meta.get('reason')}")
            line("     (soft: demo still passes offline path; fix issuer/policy for hard live green)")

    # 10. Negative: tampered statement ----------------------------------------
    step(12, "Negative: tamper one byte of the Signed Statement → scitt-cose refuses")
    tampered = bytearray(statement)
    mid = len(tampered) // 2
    tampered[mid] ^= 0x01
    t_parsed = parse_signed_statement(bytes(tampered), public_key_pem=issuer.pub_pem)
    if t_parsed.get("signature_verified"):
        bad("tampered statement incorrectly verified")
        failures += 1
    else:
        ok("tampered statement: signature_verified=False (authenticated fields fenced)")
        if t_parsed.get("issuer") is None:
            ok("issuer is None when unverified (no trust-on-parse)")

    # 11. Negative: wrong leaf on receipt -------------------------------------
    step(13, "Negative: receipt presented for the wrong leaf → scitt-cose refuses")
    wrong = scitt_verify_receipt(
        receipt_a, leaf_entry_hex=_sha256_hex(b"not-our-statement"), log_public_key_pem=log_a.pub_pem
    )
    if wrong.ok:
        bad("wrong-leaf receipt incorrectly verified")
        failures += 1
    else:
        ok("wrong leaf: receipt ok=False (inclusion proof fails root reconstruction/check)")

    # Summary ----------------------------------------------------------------
    line()
    line("  " + "-" * 72)
    if failures:
        line(f"  RESULT: FAIL ({failures} regression(s))")
        return 1

    line("  RESULT: PASS — four-layer path + dual-log + CCF verifier interop demonstrated")
    line()
    line("  Layer map for this run:")
    line("    Ingress  → Packing Slip + Bill of Lading (how the order entered the overlay)")
    line("    COGOBJ   → cognitive packet with ingress + authorization bindings")
    line("    COSA     → signed shed telemetry work product (edge evidence)")
    line("    EMILIA   → EP-RECEIPT-v1 ×2 + EP-AEC-v1 (authorization content)")
    line("    Bundle   → ECR-POC-BUNDLE-v0.2 (plain pack; VAP-LAP optional)")
    line("    scitt    → COSE_Sign1 + dual RFC9162 receipts (+ optional live/CCF)")
    line("    CCF      → frozen real vds=2 receipt verifies offline via scitt-cose")
    line()
    line("  What a program officer can re-check with only public keys + this bundle:")
    line("    • How did the order enter the overlay?                → Packing Slip + BoL")
    line("    • Did named parties authorize *this* curtailment?     → emilia AEC")
    line("    • Did the edge report delivery bound to that action?  → COSA work product")
    line("    • Does the COGOBJ carry the same ingress+auth digests?→ cogobj.json")
    line("    • Was the pack inclusion-provable (multi-log)?        → scitt-cose receipts")
    line("    • Does the same verifier accept Microsoft CCF?        → fixtures/ccf-vds2")
    line()
    line("  Still ops-open: production TS, did:x509 issuer profile for CCF 7.x, identity proofing.")
    line()

    out_dir = here / "out"
    out_dir.mkdir(exist_ok=True)
    (out_dir / "action.json").write_text(canonicalize(action) + "\n", encoding="utf-8")
    (out_dir / "aec.json").write_text(canonicalize(aec) + "\n", encoding="utf-8")
    (out_dir / "packing_slip.json").write_text(canonicalize(packing_slip) + "\n", encoding="utf-8")
    (out_dir / "bill_of_lading.json").write_text(canonicalize(bill_of_lading) + "\n", encoding="utf-8")
    (out_dir / "cogobj.json").write_text(canonicalize(cogobj) + "\n", encoding="utf-8")
    (out_dir / "bundle.json").write_text(canonicalize(bundle) + "\n", encoding="utf-8")
    (out_dir / "statement.cose").write_bytes(statement)
    (out_dir / "receipt-a.cose").write_bytes(receipt_a)
    (out_dir / "receipt-b.cose").write_bytes(receipt_b)
    (out_dir / "transparent.cose").write_bytes(transparent)
    if ccf_live_meta.get("status") == "ok" and ccf_live_meta.get("receipt"):
        (out_dir / "receipt-live.cose").write_bytes(ccf_live_meta["receipt"])
    keys_doc = {
        "grid_authority_emilia_spki_b64u": grid.emilia_key,
        "facility_ops_emilia_spki_b64u": facility.emilia_key,
        "cosa_plane_emilia_spki_b64u": plane.emilia_key,
        "facility_synth_emilia_spki_b64u": synth.emilia_key,
        "scitt_issuer_pem": issuer.pub_pem.decode(),
        "scitt_log_a_pem": log_a.pub_pem.decode(),
        "scitt_log_b_pem": log_b.pub_pem.decode(),
        "leaf_entry_hex": leaf,
        "log_a": {
            "merkle_root": root_a,
            "leaf_index": leaf_index_a,
            "tree_size": len(entries_a),
        },
        "log_b": {
            "merkle_root": root_b,
            "leaf_index": leaf_index_b,
            "tree_size": len(entries_b),
        },
        "action_digest": digest,
        "packing_slip_hash": packing_slip["hash"],
        "ccf_frozen_fixture": "fixtures/ccf-vds2",
        "ccf_live": {
            "url": args.ccf_url or None,
            "status": ccf_live_meta.get("status"),
            "reason": ccf_live_meta.get("reason"),
            "vds": ccf_live_meta.get("vds"),
        },
    }
    (out_dir / "keys.json").write_text(json.dumps(keys_doc, indent=2) + "\n", encoding="utf-8")
    rel = out_dir.relative_to(Path.cwd()) if out_dir.is_relative_to(Path.cwd()) else out_dir
    line(f"  Artifacts written to {rel}/")
    line("    action.json  aec.json  packing_slip.json  bill_of_lading.json  cogobj.json")
    line("    bundle.json  statement.cose  receipt-a/b.cose  transparent.cose  keys.json")
    line()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
