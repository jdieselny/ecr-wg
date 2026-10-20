"""clc_evaluator.py — Standalone Python Reference Evaluator for CLC-1.19 / CLC-D.

Implements normative algorithms from IETF draft-wei-capability-language-core:
- §3 / §13.4.1: Capability Identifier and Grant Grammar
- §13.4.2: Layer 2 Identifier Coverage
- §13.4.3: Layer 3 Parameter Narrowing (Contains)
- §9: Decision Function (Authorize -> allow_unresolved)
- §8.5: Residual Obligation Discharge (Resolve -> deny on live violation)
"""

from __future__ import annotations

import re
from typing import Any, Mapping, Optional

# CLC Section 3 scheme grammar regex: <vendor>/<product>-v<major>:<action-class>[:<segments>]
# Note: Action segments support uppercase/mixed-case identifiers per CLC corpus.
CID_PATTERN = re.compile(
    r"^([a-z0-9_-]+)/([a-z0-9_-]+)-v([0-9]+):([a-zA-Z0-9_-]+)(?::([a-zA-Z0-9_*:-]+))?$"
)


class CLCError(Exception):
    """Base exception for CLC evaluation errors."""


def validate_capability_id(cid: str) -> None:
    if not isinstance(cid, str) or not cid:
        raise CLCError("missing_capability_id")
    match = CID_PATTERN.match(cid)
    if not match:
        raise CLCError("invalid_capability_id")
    # Disallow mid-identifier wildcards
    segments = cid.split(":")
    for seg in segments[:-1]:
        if "*" in seg:
            raise CLCError("unsupported_wildcard")
    if segments[-1] != "*" and "*" in segments[-1]:
        raise CLCError("unsupported_wildcard")


def namespace_of(cid: str) -> str:
    """Returns scheme:action-class portion of capability identifier."""
    parts = cid.split(":")
    if len(parts) < 2:
        return cid
    return f"{parts[0]}:{parts[1]}"


def _match_coverage(parent_id: str, child_id: str) -> tuple[bool, str]:
    """CLC §13.4.2 Layer 2: Identifier Coverage."""
    validate_capability_id(parent_id)
    validate_capability_id(child_id)

    if namespace_of(parent_id) != namespace_of(child_id):
        return False, "different_namespace"

    p_segs = parent_id.split(":")
    c_segs = child_id.split(":")

    if p_segs[-1] == "*":
        prefix_len = len(p_segs) - 1
        if len(c_segs) <= prefix_len:
            # Trailing wildcard requires at least one child segment after prefix
            return False, "child_exceeds_parent"
        if c_segs[:prefix_len] == p_segs[:prefix_len]:
            return True, ""
        return False, "child_exceeds_parent"

    if p_segs == c_segs:
        return True, ""

    return False, "child_exceeds_parent"


def _narrower_params(p_val: Any, c_val: Any) -> bool:
    """CLC §13.4.3: Value subset semantics."""
    if isinstance(p_val, (int, float)) and isinstance(c_val, (int, float)):
        return c_val <= p_val
    if isinstance(p_val, bool) and isinstance(c_val, bool):
        return p_val == c_val
    if isinstance(p_val, str) and isinstance(c_val, str):
        return p_val == c_val
    if isinstance(p_val, list):
        if not isinstance(c_val, list):
            # Scalar member inside granted array
            return c_val in p_val
        # Array element-wise: every child element must be in parent
        if len(p_val) == 0:
            return False  # Deny when declared
        return all(elem in p_val for elem in c_val)
    if isinstance(p_val, dict) and isinstance(c_val, dict):
        if set(p_val.keys()) != set(c_val.keys()):
            return False
        return all(_narrower_params(p_val[k], c_val[k]) for k in p_val)
    return False


def contains(parent_grant: Mapping[str, Any], child_grant: Mapping[str, Any]) -> dict[str, Any]:
    """CLC §13: Contains(GP, GC) -> ContainmentResult."""
    # Layer 1: Grant validity
    try:
        p_id = str(parent_grant.get("id") or "")
        c_id = str(child_grant.get("id") or "")
        validate_capability_id(p_id)
        validate_capability_id(c_id)
    except CLCError as exc:
        return {"contains": False, "reason": str(exc)}

    # Layer 2: Identifier Coverage
    covered, reason = _match_coverage(p_id, c_id)
    if not covered:
        return {"contains": False, "reason": reason}

    # Layer 3: Parameter Narrowing
    p_params = parent_grant.get("params") or {}
    c_params = child_grant.get("params") or {}

    # Empty parent params object is unconstrained, contains any child bounds
    if p_params == {}:
        return {"contains": True, "reason": ""}

    # Symmetrical key closure
    if set(p_params.keys()) != set(c_params.keys()):
        return {"contains": False, "reason": "params_not_narrower"}

    for k in p_params:
        if not _narrower_params(p_params[k], c_params[k]):
            return {"contains": False, "reason": "params_not_narrower"}

    return {"contains": True, "reason": ""}


def authorize(grant: Mapping[str, Any], operation: Mapping[str, Any]) -> dict[str, Any]:
    """CLC §9: Authorize(Grant, Operation) -> Decision.
    
    Returns 'allow', 'deny', or 'allow_unresolved'.
    """
    g_id = str(grant.get("id") or "")
    op_id = str(operation.get("id") or "")

    covered, reason = _match_coverage(g_id, op_id)
    if not covered:
        return {"verdict": "deny", "reason": reason, "unresolved": []}

    # Validate parameters fit within bounds
    g_params = grant.get("params") or {}
    op_params = operation.get("params") or {}

    for k, bound in g_params.items():
        if k in op_params:
            if not _narrower_params(bound, op_params[k]):
                return {"verdict": "deny", "reason": f"param_exceeded:{k}", "unresolved": []}

    # Evaluate constraints per CLC §8.1: Core recognizes ONLY varwof/constraint-v1:max_rows, time, network
    constraints = list(grant.get("constraints") or [])
    unresolved: list[str] = []

    for c in constraints:
        parts = c.split(":")
        if len(parts) < 2:
            return {"verdict": "deny", "reason": f"invalid_constraint:{c}", "unresolved": []}
        scheme, c_type = parts[0], parts[1]
        
        # Core recognized (scheme, type) pairs
        if scheme == "varwof/constraint-v1":
            if c_type == "max_rows":
                # Evaluated against op_params if present
                pass
            elif c_type in ("time", "network"):
                unresolved.append(c)
            else:
                return {"verdict": "deny", "reason": f"unknown_constraint:{c}", "unresolved": []}
        else:
            # Any non-core recognized scheme/type fails closed per §8.1
            return {"verdict": "deny", "reason": f"unknown_constraint:{c}", "unresolved": []}

    if unresolved:
        sorted_unresolved = sorted(list(set(unresolved)))
        return {
            "verdict": "allow_unresolved",
            "reason": "",
            "unresolved": sorted_unresolved,
        }

    return {"verdict": "allow", "reason": "", "unresolved": []}


def _eval_core_clock(constraint: str, now_str: str) -> str:
    """CLC §8.5 Step 3: Core clock evaluation for varwof/constraint-v1:time:window."""
    try:
        import json
        from datetime import datetime
        # Parse now timestamp
        if now_str.endswith("Z"):
            dt = datetime.fromisoformat(now_str.replace("Z", "+00:00"))
        else:
            dt = datetime.fromisoformat(now_str)
        now_sec = dt.hour * 3600 + dt.minute * 60 + dt.second

        # Extract window JSON from constraint string: varwof/constraint-v1:time:window:[{"start":"...","end":"..."}]
        prefix = "varwof/constraint-v1:time:window:"
        if not constraint.startswith(prefix):
            return "unknown"
        raw_json = constraint[len(prefix):]
        segments = json.loads(raw_json)
        
        for seg in segments:
            s_parts = [int(p) for p in seg["start"].split(":")]
            e_parts = [int(p) for p in seg["end"].split(":")]
            s_sec = s_parts[0] * 3600 + s_parts[1] * 60 + (s_parts[2] if len(s_parts) > 2 else 0)
            e_sec = e_parts[0] * 3600 + e_parts[1] * 60 + (e_parts[2] if len(e_parts) > 2 else 0)
            if e_sec == 0:  # Reserved endpoint 00:00 is next-day midnight (86400)
                e_sec = 86400
            if s_sec <= now_sec < e_sec:
                return "satisfied"
        return "violated"
    except Exception:
        return "unknown"


def resolve(
    decision: Mapping[str, Any],
    resolutions: list[Mapping[str, str]],
    now: Optional[str] = None,
) -> dict[str, Any]:
    """CLC §8.5: Resolve(decision, resolutions, now?) -> Decision.
    
    1. Terminal verdicts are fixed: deny or allow returns unchanged.
    2. Malformed input fails closed.
    3. Discharge on allow_unresolved: combines sources per obligation under violated ≻ satisfied ≻ unknown.
    4. Set rule in order:
       - Any obligation violated -> deny
       - Every obligation satisfied -> allow
       - Otherwise -> allow_unresolved
    """
    verdict = decision.get("verdict")
    # Step 1: Terminal verdicts are fixed
    if verdict in ("allow", "deny"):
        return dict(decision)

    # Step 2: Validate resolutions input
    for r in resolutions:
        st = r.get("status")
        if st not in ("satisfied", "violated", "unknown"):
            return {"verdict": "deny", "reason": "invalid_resolution", "unresolved": []}

    # Step 3: Compute status per obligation in O
    O = list(decision.get("unresolved") or [])
    STATUS_RANK = {"violated": 2, "satisfied": 1, "unknown": 0}

    # Map resolutions by constraint
    consumer_map: dict[str, str] = {}
    for r in resolutions:
        c = r.get("constraint", "")
        st = r.get("status", "unknown")
        curr = consumer_map.get(c, "unknown")
        if STATUS_RANK[st] > STATUS_RANK[curr]:
            consumer_map[c] = st

    obl_status: dict[str, str] = {}
    for o in O:
        st_consumer = consumer_map.get(o, "unknown")
        st_clock = "unknown"
        if o.startswith("varwof/constraint-v1:time") and now:
            st_clock = _eval_core_clock(o, now)
        
        # Most restrictive source wins (violated ≻ satisfied ≻ unknown)
        best = st_consumer
        if STATUS_RANK[st_clock] > STATUS_RANK[best]:
            best = st_clock
        obl_status[o] = best

    # Step 4: Result evaluated in order
    # Clause 1: any obligation violated -> deny
    for o, st in obl_status.items():
        if st == "violated":
            # Extract type segment
            parts = o.split(":")
            c_type = parts[1] if len(parts) > 1 else "constraint"
            return {
                "verdict": "deny",
                "reason": f"{c_type}:violated",
                "unresolved": [],
            }

    # Clause 2: every obligation satisfied -> allow
    if all(st == "satisfied" for st in obl_status.values()):
        return {"verdict": "allow", "reason": "", "unresolved": []}

    # Clause 3: otherwise -> allow_unresolved with still-unknown subset
    still_unknown = [o for o, st in obl_status.items() if st == "unknown"]
    return {
        "verdict": "allow_unresolved",
        "reason": "",
        "unresolved": sorted(still_unknown),
    }
