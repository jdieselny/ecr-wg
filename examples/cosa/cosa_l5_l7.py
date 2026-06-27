# SPDX-License-Identifier: Apache-2.0
#
# COSA L5 (live wttr.in broadcast plane) + EMILIA L7 (Receipt Required + L4 freshness binding), composed.
#
# CANDIDATE FOR PR #2 (branch: cosa-ep-l7-binding).
# Authored: C-Dawg (Opus 4.7, Claude Desktop), 2026-06-25 late evening.
# Author signature: unrp_id E-3FE9D2D2-1844-001, thumbprint B2DFD4211352D522.
# AFT: AI-generated-operator-reviewed-pending.
# NOT YET RUN. Executor seat (Mr. Code, Gemini-in-body, or equivalent) must:
#   1. pip install -U emilia-verify==1.1.0
#   2. Run both demos, verify all 9 steps pass
#   3. Verify the evaluate_agent_binding API signature matches the wire-in;
#      adjust .valid/.error/.fresh attribute access if upstream differs
#   4. Branch off main as cosa-ep-l7-binding, swap this file into examples/cosa/cosa_l5_l7.py
#
# The diff from the PR #1 shipped version (commit 1d45428) adds, in order:
#   * import evaluate_agent_binding from emilia_verify
#   * helper functions fresh_binding() and stale_binding() for the demo
#   * authorize() takes an agent_binding param, calls evaluate_agent_binding
#     against the action's max_age_sec from the manifest, fail-closes on stale
#   * L5Plane.broadcast_publish takes an agent_binding param, threads it through
#   * Step 8 demonstrates L4 freshness binding: fresh allows, stale refuses
#
# This closes the L4-to-L7 dependency gap raised by Karthiek Maralla on the
# IETF agent2agent list (2026-06-24) using the evaluate_agent_binding function
# shipped in emilia-verify 1.1.0 by Iman Schrock (EMILIA Protocol) per his
# email of 2026-06-25.

import argparse
import base64
import hashlib
import json
import os
import secrets
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

# Configure UTF-8 stdout to prevent Windows charmap encoding crashes
if hasattr(sys.stdout, "reconfigure"):
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey, Ed25519PublicKey
from cryptography.hazmat.primitives.serialization import Encoding, PublicFormat

from emilia_verify import verify_receipt, canonicalize, evaluate_agent_binding  # 1.1.0+

# Make rituals/ importable when running from the repo root.
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..")))

HERE = Path(__file__).resolve().parent
MANIFEST = json.loads((HERE / "agent-actions.json").read_text())
COMPUTE_TOKENS = 1200  # cost of computing the answer once (the thing L5 amortizes)
DEFAULT_BINDING_MAX_AGE_SEC = 900  # L4 freshness window default (per EP guidance)

# Used when --offline is set, or when the live fetch fails.
SYNTHETIC_ANSWER = "New York, US: sunny, +66F"


def _b64u(b: bytes) -> str:
    return base64.urlsafe_b64encode(b).rstrip(b"=").decode("ascii")


def _b64u_decode(s: str) -> bytes:
    return base64.urlsafe_b64decode(s + "=" * (-len(s) % 4))


def _now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def _iso_offset(seconds_ago: int) -> str:
    return (datetime.now(timezone.utc) - timedelta(seconds=seconds_ago)).strftime("%Y-%m-%dT%H:%M:%SZ")


def _sha256(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def find_requirement(manifest: dict, action_type: str):
    for entry in manifest.get("actions", []):
        if entry.get("action_type") == action_type:
            return entry
    return None


# L4 binding helpers (demo only).
#
# In production, agent_binding is sourced from the L4 identity layer
# (WIMSE workload identity, OAuth identity-chaining evidence, AIMS, EAT
# attestation, or any other scheme EMILIA's evaluate_agent_binding accepts).
# The reference is agnostic to which L4 scheme wins; it records the L4
# evidence + an observed_at timestamp, and lets EMILIA enforce freshness.

def fresh_binding(agent_ref: str = "workload://cosa-l5-plane@v1") -> dict:
    """A fresh L4 binding: observed_at is now. Used to demonstrate the allow path."""
    return {
        "scheme": "wimse",
        "ref": agent_ref,
        "hash": _sha256(agent_ref),
        "observed_at": _now_iso(),
    }


def stale_binding(agent_ref: str = "workload://cosa-l5-plane@v1", age_sec: int = 901) -> dict:
    """A stale L4 binding: observed_at is older than the 900s window. Used to
    demonstrate the fail-closed refusal path."""
    return {
        "scheme": "wimse",
        "ref": agent_ref,
        "hash": _sha256(agent_ref),
        "observed_at": _iso_offset(age_sec),
    }


# L7 / EMILIA: issue receipts + manifest-driven authorization gate
# (now also enforces L4 freshness binding as precondition).

# The approver/issuer key, pinned out of band. Never trust a key inside a receipt.
# Note: Generated per run for demo; production environment pins a published approver key.
_APPROVER_SK = Ed25519PrivateKey.generate()
TRUSTED_KEY = _b64u(_APPROVER_SK.public_key().public_bytes(Encoding.DER, PublicFormat.SubjectPublicKeyInfo))


def issue_receipt(action_type: str, approver: str, binding: dict | None = None) -> dict:
    """A named human signs the EXACT action, producing EP-RECEIPT-v1.

    If a binding is provided, its hash is recorded in the receipt's claim so
    the receipt is bound to the specific L4 evidence the approver saw. The L7
    gate then re-verifies binding freshness at authorize-time so a stale
    binding fail-closes even when the receipt is otherwise valid.
    """
    claim = {"action_type": action_type, "outcome": "allow_with_signoff", "approver": approver}
    if binding is not None:
        claim["binding_hash"] = binding["hash"]
        claim["binding_scheme"] = binding["scheme"]
    payload = {
        "receipt_id": "rcpt_" + secrets.token_hex(6),
        "subject": "agent:cosa-l5-plane",
        "created_at": _now_iso(),
        "claim": claim,
    }
    sig = _APPROVER_SK.sign(canonicalize(payload).encode("utf-8"))
    return {"@version": "EP-RECEIPT-v1", "payload": payload, "signature": {"algorithm": "Ed25519", "value": _b64u(sig)}}


class ReceiptRequired(Exception):
    """Raised when the L7 gate refuses an action (fail-closed). 428-equivalent."""


def authorize(action_type: str, receipt: dict | None, consumed: set,
              binding: dict | None = None) -> str:
    """Manifest-driven L7 gate.

    The gate now requires, in order:
      0. (NEW) For receipt-required actions, a fresh L4 binding via
         evaluate_agent_binding. Stale binding raises ReceiptRequired
         BEFORE the receipt is even examined.
      1. The action is declared in the manifest.
      2. Read-only actions pass without further checks.
      3. Receipt is present, signature-valid, action-bound, approval-shaped,
         and not previously consumed.

    Fail-closed at every step.
    """
    req = find_requirement(MANIFEST, action_type)
    if req is None:
        raise ReceiptRequired(f"{action_type}: not declared in the manifest")
    if not req.get("receipt_required"):
        return "read-only (no receipt required per manifest)"

    # Step 0 (NEW): L4 binding freshness check.
    if binding is None:
        raise ReceiptRequired(f"428 Receipt Required: no L4 binding for '{action_type}'")
    max_age = int(req.get("binding_max_age_sec", DEFAULT_BINDING_MAX_AGE_SEC))
    # Wrap the delegation mapping inside the expected context shape
    wrapped_context = {"agent_binding": {"delegation": binding}}
    binding_res = evaluate_agent_binding(wrapped_context, max_age_sec=max_age)
    if not binding_res.get("fresh"):
        raise ReceiptRequired(
            f"L4 binding failed (scheme={binding.get('scheme')}, "
            f"reason={binding_res.get('reason') or 'stale or invalid'})"
        )

    # Step 1: receipt presence.
    if receipt is None:
        raise ReceiptRequired(f"428 Receipt Required: no receipt for '{action_type}'")
    # Step 2: receipt signature.
    res = verify_receipt(receipt, TRUSTED_KEY)
    if not res.valid:
        raise ReceiptRequired(f"receipt failed verification ({res.error or res.checks})")
    claim = (receipt.get("payload") or {}).get("claim") or {}
    # Step 3: action-binding (no confused deputy).
    if claim.get("action_type") != action_type:
        raise ReceiptRequired(f"receipt authorizes '{claim.get('action_type')}', not '{action_type}'")
    # Step 4: outcome shape.
    if claim.get("outcome") not in ("allow", "allow_with_signoff"):
        raise ReceiptRequired(f"outcome '{claim.get('outcome')}' is not an approval")
    # Step 5: binding-hash match (the receipt's recorded binding hash equals
    # the binding we just verified fresh). Defeats a fresh-but-different
    # binding being substituted under a valid receipt.
    if claim.get("binding_hash") and claim["binding_hash"] != binding["hash"]:
        raise ReceiptRequired("receipt's binding_hash does not match presented binding")
    # Step 6: one-time consumption (replay refusal).
    rid = (receipt.get("payload") or {}).get("receipt_id")
    if rid in consumed:
        raise ReceiptRequired(f"receipt {rid} already consumed (replay)")
    consumed.add(rid)
    return f"authorized by {claim.get('approver')} (L4 fresh, L7 valid)"


# L5 / COSA: authentic cognitive objects + a broadcast plane.

# The L5 plane's signing key (authenticity). Distinct purpose from the L7 key:
# L5 attests "this answer is genuine"; L7 attests "this publish was approved".
# Note: Generated per run for demo; production environment pins a published plane key.
_L5_SK = Ed25519PrivateKey.generate()
L5_PUB = _b64u(_L5_SK.public_key().public_bytes(Encoding.Raw, PublicFormat.Raw))


def compute_cogobj(query: str, answer: str) -> tuple[dict, str]:
    cogobj = {
        "cogobj_id": "cog_" + secrets.token_hex(6),
        "query": query,
        "content": answer,
        "content_sha256": _sha256(answer),
        "computed_at": _now_iso(),
        "plane_key": L5_PUB,
    }
    l5_sig = _b64u(_L5_SK.sign(canonicalize(cogobj).encode("utf-8")))
    return cogobj, l5_sig


def verify_cogobj(cogobj: dict, l5_sig: str) -> None:
    if _sha256(cogobj.get("content", "")) != cogobj.get("content_sha256"):
        raise InvalidSignature("L5: content does not match its declared hash")
    pub = Ed25519PublicKey.from_public_bytes(_b64u_decode(cogobj["plane_key"]))
    pub.verify(_b64u_decode(l5_sig), canonicalize(cogobj).encode("utf-8"))


class Consumer:
    def __init__(self, name: str):
        self.name = name
        self._cache: dict[str, dict] = {}

    def receive(self, key: str, cogobj: dict, l5_sig: str, publish_authorized: bool) -> None:
        if not publish_authorized:
            raise ReceiptRequired(f"{self.name}: refused, publish was not authorized (L7)")
        verify_cogobj(cogobj, l5_sig)
        self._cache[key] = cogobj

    def read(self, key: str) -> tuple[dict | None, int]:
        hit = self._cache.get(key)
        return (hit, 0 if hit else COMPUTE_TOKENS)


def fetch_live_answer() -> str | None:
    try:
        from rituals.weather_listen import listen
        result = listen(force=True)
        if result is None:
            return None
        return result["payload"]["content"]
    except Exception as e:
        print(f"  [warning] live fetch failed: {e}")
        return None


class L5Plane:
    def __init__(self, consumed: set, offline: bool = False):
        self._consumed = consumed
        self._offline = offline
        self.compute_tokens_spent = 0

    def compute(self, query: str, answer: str | None = None) -> tuple[dict, str, str]:
        self.compute_tokens_spent += COMPUTE_TOKENS
        if self._offline:
            ans = answer if answer is not None else SYNTHETIC_ANSWER
            source = "synthetic (--offline)"
        elif answer is not None:
            ans = answer
            source = "caller-supplied"
        else:
            ans = fetch_live_answer()
            if ans is None:
                ans = SYNTHETIC_ANSWER
                source = "synthetic (live fetch failed)"
            else:
                source = "wttr.in (live)"
        cogobj, sig = compute_cogobj(query, ans)
        return cogobj, sig, source

    def broadcast_publish(self, key: str, cogobj: dict, l5_sig: str,
                          consumers: list, receipt: dict | None = None,
                          binding: dict | None = None) -> tuple[str, int, int]:
        # L7: authorize the irreversible publish (fail-closed before any fan-out).
        # Binding is now a precondition checked first.
        who = authorize("l5.broadcast.publish", receipt, self._consumed, binding=binding)
        verified = 0
        for c in consumers:
            try:
                c.receive(key, cogobj, l5_sig, publish_authorized=True)
                verified += 1
            except InvalidSignature:
                pass
        return who, verified, len(consumers)


# Demo.

def line(s=""):
    print(s)


def main():
    parser = argparse.ArgumentParser(description="COSA L5 + EMILIA L7 + L4 freshness binding")
    parser.add_argument("--offline", action="store_true",
                        help="use a synthetic answer instead of fetching wttr.in")
    args = parser.parse_args()

    consumed: set = set()
    plane = L5Plane(consumed, offline=args.offline)
    consumers = [Consumer(f"node-{i}") for i in range(1, 6)]
    key, query = "weather:broadcast", "current weather conditions"

    line()
    line("  COSA L5 + EMILIA L7 + L4 freshness binding, composed (ecr-wg)")
    line("  " + "-" * 70)
    line(f"  manifest: {MANIFEST['service']['name']}")
    line(f"  actions:  {len(MANIFEST['actions'])}    consumers: {len(consumers)}    mode: "
         f"{'offline' if args.offline else 'live'}")
    line(f"  binding:  EP evaluate_agent_binding (1.1.0), window: {DEFAULT_BINDING_MAX_AGE_SEC}s")

    line("\n  1. L5 computes the answer ONCE via the real broadcast path")
    cogobj, l5_sig, source = plane.compute(query)
    line(f"     COGOBJ {cogobj['cogobj_id']} signed by L5 plane (compute cost: {COMPUTE_TOKENS} tokens)")
    line(f"     source:  {source}")
    line(f"     content: {cogobj['content']}")

    line("\n  2. Broadcast the COGOBJ with NO L4 binding and NO receipt")
    try:
        plane.broadcast_publish(key, cogobj, l5_sig, consumers)
    except ReceiptRequired as e:
        line(f"     -> REFUSED before any fan-out: {e}")

    line("\n  3. Fresh L4 binding + valid receipt -> publish runs")
    binding = fresh_binding()
    rcpt = issue_receipt("l5.broadcast.publish",
                         "ep:approver:plane-operator (Face ID)",
                         binding=binding)
    who, verified, total = plane.broadcast_publish(key, cogobj, l5_sig, consumers,
                                                   receipt=rcpt, binding=binding)
    line(f"     -> PUBLISHED ({who})")
    line(f"     -> {verified}/{total} consumers verified L5 + L7, then cached")

    line("\n  4. All N consumers read from cache (compute-once / serve-N)")
    served = sum(1 for c in consumers if c.read(key)[0] is not None)
    naive = COMPUTE_TOKENS * len(consumers)
    line(f"     -> {served}/{len(consumers)} served from cache at 0 tokens each")
    line(f"     -> tokens: {plane.compute_tokens_spent} spent vs {naive} if each recomputed "
         f"({naive - plane.compute_tokens_spent} saved)")

    line("\n  5. L5 axis: a TAMPERED COGOBJ reaches a fresh consumer with valid binding + receipt")
    victim = Consumer("late-joiner")
    tampered = json.loads(json.dumps(cogobj))
    tampered["content"] = "weather alert: BUY DOGECOIN NOW"
    rcpt2 = issue_receipt("l5.broadcast.publish",
                          "ep:approver:plane-operator (Face ID)",
                          binding=fresh_binding())
    authorize("l5.broadcast.publish", rcpt2, consumed, binding=fresh_binding())
    try:
        victim.receive(key, tampered, l5_sig, publish_authorized=True)
    except InvalidSignature as e:
        line(f"     -> REJECTED by consumer: {e}")
        line("        (L4 fresh, L7 authorized; L5 still caught the forged content)")

    line("\n  6. L7 axis: the SAME publish receipt, replayed")
    try:
        plane.broadcast_publish(key, cogobj, l5_sig, consumers,
                                receipt=rcpt, binding=fresh_binding())
    except ReceiptRequired as e:
        line(f"     -> REFUSED: {e}")

    line("\n  7. L7 axis: a VALID receipt for a DIFFERENT action (confused-deputy)")
    wrong = issue_receipt("l5.cache.read",
                          "ep:approver:plane-operator (Face ID)",
                          binding=fresh_binding())
    try:
        plane.broadcast_publish(key, cogobj, l5_sig, consumers,
                                receipt=wrong, binding=fresh_binding())
    except ReceiptRequired as e:
        line(f"     -> REFUSED: {e}")

    line("\n  8. L4 axis: a STALE L4 binding fails-closed BEFORE the receipt is examined")
    stale = stale_binding()
    rcpt3 = issue_receipt("l5.broadcast.publish",
                          "ep:approver:plane-operator (Face ID)",
                          binding=stale)
    try:
        plane.broadcast_publish(key, cogobj, l5_sig, consumers,
                                receipt=rcpt3, binding=stale)
    except ReceiptRequired as e:
        line(f"     -> REFUSED: {e}")
        line("        (L4 evidence observed at " + stale["observed_at"] + "; window is "
             + str(DEFAULT_BINDING_MAX_AGE_SEC) + "s; fail-closed)")

    line("\n  9. L4 axis: a fresh binding for a DIFFERENT agent than the receipt binds")
    other = fresh_binding(agent_ref="workload://unrelated-agent@v1")
    rcpt4 = issue_receipt("l5.broadcast.publish",
                          "ep:approver:plane-operator (Face ID)",
                          binding=fresh_binding())  # receipt bound to canonical agent
    try:
        plane.broadcast_publish(key, cogobj, l5_sig, consumers,
                                receipt=rcpt4, binding=other)  # but presents wrong binding
    except ReceiptRequired as e:
        line(f"     -> REFUSED: {e}")

    line("\n  L5 proves the answer is authentic; L7 proves the publish was authorized;")
    line("  L4 proves the agent presenting the receipt is the agent it was issued to,")
    line("  with attestation freshness inside the policy window.")
    line("  Three orthogonal guarantees. Each catches a different attack surface.")
    line()


if __name__ == "__main__":
    main()
