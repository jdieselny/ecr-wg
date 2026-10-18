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
CID_PATTERN = re.compile(
    r"^([a-z0-9_-]+)/([a-z0-9_-]+)-v([0-9]+):([a-z0-9_-]+)(?::([a-z0-9_*:-]+))?$"
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

    # Evaluate constraints
    constraints = list(grant.get("constraints") or [])
    unresolved: list[str] = []

    for c in constraints:
        # Native core evaluator for time windows (syntactic check)
        if c.startswith("varwof/constraint-v1:time"):
            unresolved.append(c)
        elif ":" in c:
            # Scheme-specific constraint (e.g. emilia/telemetry-v1:*)
            unresolved.append(c)
        else:
            return {"verdict": "deny", "reason": f"unknown_constraint:{c}", "unresolved": []}

    if unresolved:
        # Normalized and deterministically sorted list of residual obligations
        sorted_unresolved = sorted(list(set(unresolved)))
        return {
            "verdict": "allow_unresolved",
            "reason": "",
            "unresolved": sorted_unresolved,
        }

    return {"verdict": "allow", "reason": "", "unresolved": []}


def resolve(
    decision: Mapping[str, Any],
    resolutions: list[Mapping[str, str]],
    now: Optional[str] = None,
) -> dict[str, Any]:
    """CLC §8.5: Resolve(decision, resolutions, now?) -> Decision.
    
    Collapses residual obligations under: violated ≻ satisfied ≻ unknown.
    """
    verdict = decision.get("verdict")
    if verdict in ("allow", "deny"):
        return dict(decision)

    res_map: dict[str, str] = {}
    for r in resolutions:
        c = r.get("constraint", "")
        st = r.get("status", "unknown")
        # violated ≻ satisfied ≻ unknown
        curr = res_map.get(c, "unknown")
        if st == "violated" or curr == "violated":
            res_map[c] = "violated"
        elif st == "satisfied" and curr != "violated":
            res_map[c] = "satisfied"
        else:
            if c not in res_map:
                res_map[c] = st

    unresolved = list(decision.get("unresolved") or [])
    remaining: list[str] = []

    for c in unresolved:
        st = res_map.get(c, "unknown")
        if st == "violated":
            return {
                "verdict": "deny",
                "reason": f"{c.split(':')[-1]}:violated",
                "unresolved": [],
            }
        elif st == "unknown":
            remaining.append(c)

    if not remaining:
        return {"verdict": "allow", "reason": "", "unresolved": []}

    return {
        "verdict": "allow_unresolved",
        "reason": "",
        "unresolved": sorted(remaining),
    }
