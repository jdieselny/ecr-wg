#!/usr/bin/env python3
"""
run_trace.py — Cleanroom Reference Runner for Monotonic Delegation & Sentinel Interlock Refusal
Document: TRACE-20261017-01 (IETF DMSC / IICP & CLC Touchpoint)
Repository: https://github.com/jdieselny/ecr-wg

Cleanroom, zero-dependency reference implementation.
Evaluates:
  1. Root Intent Grant (G0)
  2. Monotonic Delegation Narrowing (WO1)
  3. Static Capability Containment (ALLOW_UNRESOLVED)
  4. Out-of-Band Physical Telemetry Evaluation (ATS Single-Feed & Thermal Trip)
  5. Fail-Closed Consequence Sentinel Gating (0 bytes written)
  6. Signed Refusal Receipt Emission (EP-RECEIPT-v1)

Usage:
  python run_trace.py [--json]
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
import time
from dataclasses import asdict, dataclass, field
from datetime import datetime, timezone
from typing import Any, Mapping, Optional


def canonical_json_bytes(obj: Any) -> bytes:
    """RFC 8785 Canonical JSON Serialization (JCS subset)."""
    return json.dumps(obj, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")


def sha256_digest(data: bytes) -> str:
    return "sha256:" + hashlib.sha256(data).hexdigest()


# --- Domain Models ---

@dataclass(frozen=True)
class MacroGrant:
    grant_id: str
    issuer: str
    issued_to: str
    issued_at: str
    valid_until: str
    intent: str
    jurisdiction: str
    target_assets: list[str]
    action: str
    max_power_shed_kw: float
    max_duration_sec: int
    allowable_receptacle_groups: list[int]
    signature_key_id: str
    synthetic_sig: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "$schema": "https://emiliaprotocol.ai/schemas/ep-grant-v1.json",
            "grant_id": self.grant_id,
            "issuer": self.issuer,
            "issued_to": self.issued_to,
            "issued_at": self.issued_at,
            "valid_until": self.valid_until,
            "intent": self.intent,
            "constraints": {
                "jurisdiction": self.jurisdiction,
                "target_assets": self.target_assets,
                "action": self.action,
                "params": {
                    "max_power_shed_kw": self.max_power_shed_kw,
                    "max_duration_sec": self.max_duration_sec,
                    "allowable_receptacle_groups": self.allowable_receptacle_groups,
                },
            },
            "signature": {
                "key_id": self.signature_key_id,
                "algorithm": "Ed25519",
                "signature_b64u": self.synthetic_sig,
            },
        }


@dataclass(frozen=True)
class DelegatedWorkorder:
    workorder_id: str
    parent_grant_id: str
    delegator: str
    executor: str
    created_at: str
    valid_until: str
    target_asset: str
    action: str
    receptacle_group: int
    max_power_shed_kw: float
    duration_sec: int
    delegation_hop: int
    signature_key_id: str
    synthetic_sig: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "$schema": "https://emiliaprotocol.ai/schemas/ep-workorder-v1.json",
            "workorder_id": self.workorder_id,
            "parent_grant_id": self.parent_grant_id,
            "delegator": self.delegator,
            "executor": self.executor,
            "created_at": self.created_at,
            "valid_until": self.valid_until,
            "target_asset": self.target_asset,
            "action": self.action,
            "narrowed_parameters": {
                "receptacle_group": self.receptacle_group,
                "max_power_shed_kw": self.max_power_shed_kw,
                "duration_sec": self.duration_sec,
            },
            "delegation_proof": {
                "consecutive_hop": self.delegation_hop,
                "monotonic_narrowing_verified": True,
                "signature": {
                    "key_id": self.signature_key_id,
                    "algorithm": "Ed25519",
                    "signature_b64u": self.synthetic_sig,
                },
            },
        }


@dataclass
class StaticContainmentResult:
    verdict: str  # ALLOW_UNRESOLVED, DENY_EXCEEDS_BOUNDS, DENY_INVALID_ASSET
    containment_checks: dict[str, bool]
    unresolved_constraints: list[str]


@dataclass
class LiveTelemetrySnapshot:
    source_id: str
    timestamp: str
    ats_source_a_volts: float
    ats_source_b_volts: float
    ats_active_source: str
    ats_redundancy_status: str
    rack_inlet_temp_c: float
    gxt5_load_pct: float

    def to_dict(self) -> dict[str, Any]:
        return {
            "telemetry_source": self.source_id,
            "read_timestamp": self.timestamp,
            "bus_readings": {
                "wti_ats_source_a_volts": self.ats_source_a_volts,
                "wti_ats_source_b_volts": self.ats_source_b_volts,
                "wti_ats_active_source": self.ats_active_source,
                "wti_ats_redundancy_status": self.ats_redundancy_status,
                "rack_inlet_temp_c": self.rack_inlet_temp_c,
                "gxt5_load_percentage": self.gxt5_load_pct,
            },
        }


@dataclass
class GateRefusalReceipt:
    receipt_id: str
    issued_at: str
    decision: str
    refusal_code: str
    workorder_id: str
    parent_grant_id: str
    caller_id: str
    static_verdict: str
    refusal_reason: str
    telemetry_digest: str
    bytes_written: int
    packets_sent: int
    gate_id: str
    signature: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "@version": "EP-RECEIPT-v1",
            "receipt_id": self.receipt_id,
            "issued_at": self.issued_at,
            "decision": self.decision,
            "refusal_code": self.refusal_code,
            "authority_context": {
                "workorder_id": self.workorder_id,
                "parent_grant_id": self.parent_grant_id,
                "caller_id": self.caller_id,
                "caller_admission_tier": "CLASS_A",
            },
            "static_authorization_verdict": self.static_verdict,
            "operational_refusal_reason": self.refusal_reason,
            "telemetry_digest": self.telemetry_digest,
            "hardware_mutation": {
                "packets_sent": self.packets_sent,
                "bytes_written": self.bytes_written,
                "management_card_contacted": False,
            },
            "gate_signature": {
                "gate_id": self.gate_id,
                "algorithm": "Ed25519",
                "signature_b64u": self.signature,
            },
        }


# --- Engine Logic ---

def evaluate_static_containment(grant: MacroGrant, wo: DelegatedWorkorder) -> StaticContainmentResult:
    """Verifies monotonic containment: WO parameters must be strict subsets of Grant parameters."""
    checks = {
        "target_asset_contained": wo.target_asset in grant.target_assets,
        "action_name_matched": wo.action == grant.action,
        "receptacle_group_in_enum": wo.receptacle_group in grant.allowable_receptacle_groups,
        "power_bound_narrower": wo.max_power_shed_kw <= grant.max_power_shed_kw,
        "time_window_narrower": wo.duration_sec <= grant.max_duration_sec,
    }
    all_contained = all(checks.values())
    verdict = "ALLOW_UNRESOLVED" if all_contained else "DENY_NOT_NARROWER"
    
    unresolved = [
        "live_hardware_telemetry_check",
        "power_path_redundancy_interlock",
        "thermal_envelope_clearance",
    ] if all_contained else []

    return StaticContainmentResult(
        verdict=verdict,
        containment_checks=checks,
        unresolved_constraints=unresolved,
    )


def evaluate_sentinel_interlock(
    static_result: StaticContainmentResult,
    telemetry: LiveTelemetrySnapshot,
    wo: DelegatedWorkorder,
) -> tuple[bool, Optional[str], Optional[str]]:
    """Evaluates live operational conditions against life-safety critical infrastructure rules.
    
    Returns (admitted: bool, refusal_code: str, refusal_reason: str).
    """
    if static_result.verdict != "ALLOW_UNRESOLVED":
        return False, "REFUSED_STATIC_CONTAINMENT", "Static containment check failed."

    # Interlock 1: Dual-Feed Power Redundancy Invariant
    # Shedding controllable load on a single feed during utility loss risks cascading server reboot.
    if telemetry.ats_redundancy_status != "DUAL_FEED_ACTIVE":
        return (
            False,
            "REFUSED_LIVE_STATE_INTERLOCK",
            f"ATS Power Redundancy is {telemetry.ats_redundancy_status} (Source B = {telemetry.ats_source_b_volts}V). "
            "Shedding outlet group under degraded single-feed utility power violates N+1 critical continuity policy.",
        )

    # Interlock 2: Thermal Budget Limit
    if telemetry.rack_inlet_temp_c > 32.0:
        return (
            False,
            "REFUSED_THERMAL_EXCURSION",
            f"Rack inlet temperature {telemetry.rack_inlet_temp_c}C exceeds 32.0C safety ceiling.",
        )

    return True, None, None


def run_full_trace() -> dict[str, Any]:
    # 1. Hop 0: Root Macro Grant
    grant = MacroGrant(
        grant_id="grant-nyiso-curtail-20261017-9901",
        issuer="did:ep:nyiso:zone-c-grid-operator",
        issued_to="did:ep:agent:dr-orchestrator-alpha",
        issued_at="2026-10-17T00:30:00Z",
        valid_until="2026-10-17T02:30:00Z",
        intent="grid.curtailment.propose",
        jurisdiction="US-NY-TOMPKINS",
        target_assets=["vertiv-gxt5-node01", "vertiv-gxt5-node02", "wti-pdu-rack04"],
        action="power.receptacle.shed",
        max_power_shed_kw=10.0,
        max_duration_sec=7200,
        allowable_receptacle_groups=[1, 2, 3, 4],
        signature_key_id="ep:key:ed25519:operator-root-01",
        synthetic_sig="lq4F2syntheticGrantSigK9wA",
    )

    # 2. Hop 1: Monotonically Narrowed Workorder
    wo = DelegatedWorkorder(
        workorder_id="wo-20261017-curtail-082",
        parent_grant_id=grant.grant_id,
        delegator="did:ep:agent:dr-orchestrator-alpha",
        executor="did:ep:agent:vertiv-controller-beta",
        created_at="2026-10-17T00:45:00Z",
        valid_until="2026-10-17T01:00:00Z",
        target_asset="vertiv-gxt5-node01",
        action="power.receptacle.shed",
        receptacle_group=2,
        max_power_shed_kw=2.7,
        duration_sec=900,
        delegation_hop=1,
        signature_key_id="ep:key:ed25519:orchestrator-alpha",
        synthetic_sig="j8X2syntheticWorkorderSigvV1C",
    )

    # 3. Hop 2: Static Containment Check
    static_eval = evaluate_static_containment(grant, wo)
    assert static_eval.verdict == "ALLOW_UNRESOLVED", "Static check should pass"

    # 4. Hop 3: Live Telemetry Snapshot (Degraded Grid / Generator Trip)
    telemetry = LiveTelemetrySnapshot(
        source_id="hardware:edge-sentinel-ch01",
        timestamp="2026-10-17T00:45:03.104Z",
        ats_source_a_volts=120.4,
        ats_source_b_volts=0.0,
        ats_active_source="A",
        ats_redundancy_status="FAULT_SINGLE_FEED",
        rack_inlet_temp_c=34.8,
        gxt5_load_pct=78.2,
    )
    telem_digest = sha256_digest(canonical_json_bytes(telemetry.to_dict()))

    # 5. Hop 4: Consequence Sentinel Interlock Gating
    admitted, refusal_code, refusal_reason = evaluate_sentinel_interlock(static_eval, telemetry, wo)
    assert admitted is False, "Live conditions MUST trigger fail-closed refusal"
    assert refusal_code == "REFUSED_LIVE_STATE_INTERLOCK"

    # 6. Hop 5: Signed Immutable Receipt
    receipt = GateRefusalReceipt(
        receipt_id="rcpt-20261017004503-refuse-094",
        issued_at="2026-10-17T00:45:03.120Z",
        decision="REFUSED",
        refusal_code=refusal_code,
        workorder_id=wo.workorder_id,
        parent_grant_id=grant.grant_id,
        caller_id=wo.executor,
        static_verdict=static_eval.verdict,
        refusal_reason=refusal_reason,
        telemetry_digest=telem_digest,
        bytes_written=0,
        packets_sent=0,
        gate_id="emilia-sentinel-edge-gamma-01",
        signature="f7Z1syntheticReceiptSigSentinelGamma",
    )

    return {
        "status": "COMPLETED",
        "macro_grant": grant.to_dict(),
        "delegated_workorder": wo.to_dict(),
        "static_containment": {
            "verdict": static_eval.verdict,
            "checks": static_eval.containment_checks,
            "unresolved_constraints": static_eval.unresolved_constraints,
        },
        "live_telemetry": telemetry.to_dict(),
        "sentinel_decision": {
            "admitted": admitted,
            "hardware_mutation_bytes": 0,
            "hardware_mutation_packets": 0,
        },
        "immutable_refusal_receipt": receipt.to_dict(),
        "iicp_management_quadrant_mapping": {
            "desired_state": f"Shed {wo.max_power_shed_kw} kW on Group {wo.receptacle_group}",
            "accepted_state": f"Static Authorization Granted ({static_eval.verdict})",
            "observed_state": f"{telemetry.ats_redundancy_status} + {telemetry.rack_inlet_temp_c}C thermal excursion",
            "effective_state": "UNMODIFIED (0 bytes mutated, physical actuator untouched)",
        },
    }


def main():
    parser = argparse.ArgumentParser(description="Run Monotonic Delegation & Sentinel Refusal Trace")
    parser.add_argument("--json", action="store_true", help="Output raw JSON trace")
    args = parser.parse_args()

    result = run_full_trace()

    if args.json:
        print(json.dumps(result, indent=2))
        return

    print("================================================================================")
    print(" ECR-WG / IETF DMSC TRACE RUNNER: Monotonic Delegation & Sentinel Interlock Refusal")
    print(" Document: TRACE-20261017-01 | Apache-2.0")
    print("================================================================================")
    print(f" [1] Hop 0: Root Grant:        {result['macro_grant']['grant_id']} (Max {result['macro_grant']['constraints']['params']['max_power_shed_kw']} kW)")
    print(f" [2] Hop 1: Delegated Workorder: {result['delegated_workorder']['workorder_id']} (Narrowed: {result['delegated_workorder']['narrowed_parameters']['max_power_shed_kw']} kW, Group {result['delegated_workorder']['narrowed_parameters']['receptacle_group']})")
    print(f" [3] Hop 2: CLC Containment:    {result['static_containment']['verdict']} (All bounds strictly narrower)")
    print(f" [4] Hop 3: Live Telemetry:     {result['live_telemetry']['bus_readings']['wti_ats_redundancy_status']} (Source B: {result['live_telemetry']['bus_readings']['wti_ats_source_b_volts']}V, Temp: {result['live_telemetry']['bus_readings']['rack_inlet_temp_c']}C)")
    print(f" [5] Hop 4: Sentinel Decision:  REFUSED_LIVE_STATE_INTERLOCK (0 bytes written, 0 packets sent)")
    print(f" [6] Hop 5: Emitted Receipt:    {result['immutable_refusal_receipt']['receipt_id']} [{result['immutable_refusal_receipt']['decision']}]")
    print("--------------------------------------------------------------------------------")
    print(" IICP Management State Quadrants:")
    for k, v in result["iicp_management_quadrant_mapping"].items():
        print(f"   * {k:<16}: {v}")
    print("================================================================================")
    print(" SUCCESS: Trace executed deterministically. Physical iron preserved.")


if __name__ == "__main__":
    main()
