# IICP & CLC Monotonic Delegation Trace

**Document:** [`TRACE-20261017-01`](../../specs/TRACE_20261017_MONOTONIC_DELEGATION_AND_SENTINEL_REFUSAL.md)  
**Target Group:** IETF DMSC / IICP Architecture Touchpoint  
**License:** Apache-2.0  

This cleanroom directory contains a standalone, zero-dependency reference runner for the **Monotonic Delegation & Live Operational Sentinel Refusal** pattern.

It formally conforms to `draft-wei-capability-language-core` (Conformance Class **CLC-D**, Profile **CLC-1.19**) across:
- **§13 Monotonic Containment** (`Contains(G0, WO1) -> True`)
- **§9 Static Authorization** (`Authorize(WO1, Op) -> allow_unresolved`)
- **§8.5 Resolution Evaluation** (`Resolve(decision, telemetry_state) -> deny: redundant_power_path:violated`)

All hops feature **real Ed25519 digital signatures** calculated over RFC 8785 canonical JSON bytes.

It proves the critical boundary articulated during DMSC discussions:
> *An operation satisfies its delegated authorization constraints (static CLC containment), but must still be refused fail-closed by the local execution sentinel due to live operational conditions.*

---

## Conformance Vector (`vectors/ccx-iicp-001.json`)

This runner includes the standalone test vector [`vectors/ccx-iicp-001.json`](vectors/ccx-iicp-001.json) formatted for direct ingestion into the upstream IETF DMSC conformance suite (`varwof/capability` / `containment-crosswalk-vectors.json`).

Attribution: `Justin D. Kintzele (J Diesel NY / ECR-WG)`.

---

## Quickstart

```bash
# Formatted execution summary
python run_trace.py

# Raw JSON output for automated ingestion
python run_trace.py --json
```

---

## The Five Hops Executed

1. **Hop 0 (Root Grant $G_0$):** Regional demand-response authority issues a macro curtailment grant (up to 10 kW across multiple rack assets over 120 minutes) signed with Ed25519.
2. **Hop 1 (Delegation $WO_1$):** Orchestrator monotonically narrows authority to a single asset (`vertiv-gxt5-node01`), single receptacle group (Group 2), 2.7 kW max, and a 15-minute window, signed with Ed25519.
3. **Hop 2 (CLC-D Static Containment & Authorization):** Evaluates that $WO_1 \subseteq G_0$ via CLC-D Layer 1/2/3 rules; evaluates `Authorize(WO1, Op)` returning `allow_unresolved` on physical telemetry and time obligations.
4. **Hop 3 (Live Telemetry Evaluation):** Out-of-band sensor capture detects loss of utility redundancy (`ATS Source B = 0V / FAULT_SINGLE_FEED`) and thermal excursion (`34.8°C`).
5. **Hop 4 & 5 (Sentinel CLC Resolve, Gating & Receipt):** Sentinel resolves obligations via CLC §8.5 (`violated ≻ satisfied ≻ unknown`). `redundant_power_path` is violated, collapsing verdict deterministically to `deny`. **0 bytes written, 0 packets sent**. Emits an immutable, signed refusal receipt (`EP-RECEIPT-v1`).

