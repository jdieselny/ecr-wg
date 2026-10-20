# IICP & CLC Monotonic Delegation Trace

**Document:** [`TRACE-20261017-01`](../../specs/TRACE_20261017_MONOTONIC_DELEGATION_AND_SENTINEL_REFUSAL.md)  
**Target Group:** IETF DMSC / IICP Architecture Touchpoint  
**License:** Apache-2.0  

This cleanroom directory contains a standalone, zero-dependency reference runner for the **Monotonic Delegation & Live Operational Sentinel Refusal** pattern.

It formally conforms to `draft-wei-capability-language-core-00` (Conformance Class **CLC-D**, Specification Revision **CLC-1.15**) across:
- **§13 Monotonic Containment** (`Contains(G0, WO1) -> True`)
- **§9 Static Authorization** (`Authorize(WO1, Op) -> allow_unresolved`)
- **§8.5 Resolution Evaluation** (`Resolve(decision, resolutions, now) -> deny: time:violated`)

All hops feature **real Ed25519 digital signatures** calculated over RFC 8785 canonical JSON bytes.

It proves the critical boundary articulated during DMSC discussions:
> *An operation satisfies its delegated authorization constraints (static CLC containment), but must still be refused fail-closed by the local execution sentinel due to live operational conditions (core clock expiration outside authorized execution window).*

---

## Crosswalk Vectors (`vectors/`) & Profile Declaration

This directory provides the standalone test vectors formatted for upstream ingestion into the IETF DMSC crosswalk corpus (`varwof/capability`):

- **[`vectors/ccx-iicp-001.json`](vectors/ccx-iicp-001.json)** (`contain-crosswalk`): Formally asserts CLC-D Layer 1/2/3 monotonic containment (`Contains(parent, child) -> true`).
- **[`vectors/cax-iicp-001.json`](vectors/cax-iicp-001.json)** (`authorize-crosswalk`): Formally asserts CLC §9 authorization (`Authorize(child, op) -> allow_unresolved` carrying residual network and time obligations).
- **[`vectors/crx-iicp-001.json`](vectors/crx-iicp-001.json)** (`resolve-crosswalk`): Formally asserts CLC §8.5 resolution (`Resolve(decision, resolutions, now) -> deny` with refusal reason `time:violated`).
- **[`profile-declarations.json`](profile-declarations.json)**: Declares the `iicp->clc-v1` crosswalk profile (`chain_check: hop-by-hop`, `union_axis: constraint-union`, `constraints_mode: conjunctive`, and carrier-enforced `out_of_scope` boundary dimensions).

All vectors are canonicalized via RFC 8785 JSON Canonicalization Scheme (JCS).

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

1. **Hop 0 (Root Grant $G_0$):** Regional authority issues a macro power-shed grant (`example/power-v1:curtail:*`, up to 10.0 kW, 7200s, circuit groups [1, 2, 3], network CIDR `10.0.0.0/16`, time window `08:00–18:00` UTC) signed with Ed25519.
2. **Hop 1 (Delegation $WO_1$):** Orchestrator monotonically narrows authority to asset `ups-unit-01` (`example/power-v1:curtail:ups-unit-01`), single circuit group (Group 2), 2.5 kW max, 900s, network CIDR `10.0.1.0/24`, and time window `14:00–15:00` UTC, signed with Ed25519.
3. **Hop 2 (CLC-D Static Containment & Authorization):** Evaluates that $WO_1 \subseteq G_0$ via CLC-D Layer 1/2/3 rules; evaluates `Authorize(WO1, Op)` returning `allow_unresolved` carrying the core-recognized residual network and time obligations per §9.
4. **Hop 3 (Live Telemetry & Calling Context):** Caller reports network obligation satisfied; core clock evaluates invocation timestamp against current execution time.
5. **Hop 4 & 5 (Sentinel CLC Resolve, Gating & Receipt):** Sentinel resolves obligations via CLC §8.5: core clock at 15:30:00 UTC evaluates outside the [14:00, 15:00) UTC window, yielding status `time:violated`. Under §8.5 Clause 1, verdict deterministically collapses to `deny`. **0 bytes written, 0 packets sent**. Emits an immutable, signed refusal receipt (`EP-RECEIPT-v1`).

