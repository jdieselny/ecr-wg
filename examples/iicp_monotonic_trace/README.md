# IICP & CLC Monotonic Delegation Trace

**Document:** [`TRACE-20261017-01`](../../specs/TRACE_20261017_MONOTONIC_DELEGATION_AND_SENTINEL_REFUSAL.md)  
**Target Group:** IETF DMSC / IICP Architecture Touchpoint  
**License:** Apache-2.0  

This cleanroom directory contains a standalone, zero-dependency reference runner for the **Monotonic Delegation & Live Operational Sentinel Refusal** pattern.

It proves the critical boundary articulated during DMSC discussions:
> *An operation satisfies its delegated authorization constraints (static CLC containment), but must still be refused fail-closed by the local execution sentinel due to live operational conditions.*

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

1. **Hop 0 (Root Grant $G_0$):** Regional demand-response authority issues a macro curtailment grant (up to 10 kW across multiple rack assets over 120 minutes).
2. **Hop 1 (Delegation $WO_1$):** Orchestrator monotonically narrows authority to a single asset (`vertiv-gxt5-node01`), single receptacle group (Group 2), 2.7 kW max, and a 15-minute window.
3. **Hop 2 (CLC Static Containment):** Evaluates that $WO_1 \subseteq G_0$, returning `ALLOW_UNRESOLVED` (authorization verified, but physical constraints unresolved).
4. **Hop 3 (Live Telemetry Evaluation):** Out-of-band sensor capture detects loss of utility redundancy (`ATS Source B = 0V / FAULT_SINGLE_FEED`) and thermal excursion (`34.8°C`).
5. **Hop 4 & 5 (Sentinel Gating & Receipt):** Sentinel trips fail-closed. **0 bytes written, 0 packets sent**. Emits an immutable, signed refusal receipt (`EP-RECEIPT-v1`).
