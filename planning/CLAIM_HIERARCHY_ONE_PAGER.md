# Verifiable Grid Curtailment: Coalition Claim Hierarchy
**EMILIA · COSA · Actionstate**

This one-pager outlines the honest boundary lines of the verifiable grid curtailment stack. It defines exactly what is running in code (demonstrated), what is relied upon from adjacent work (cited), and what remains outside the current scope (not claimed).

---

## 1. Composition Architecture: One Digest, Five Claims

```mermaid
graph TD
    %% ECR-WG palette: high-contrast strokes and restrained fills for projection and print.
    linkStyle default stroke:#475569,stroke-width:2px;
    classDef cosa fill:#E8F4F8,stroke:#247A96,stroke-width:3px,color:#0F172A;
    classDef emilia fill:#EAF5EE,stroke:#327A4B,stroke-width:3px,color:#0F172A;
    classDef actionstate fill:#F0ECF8,stroke:#6B55A3,stroke-width:3px,color:#0F172A;
    classDef settlement fill:#FBEEE4,stroke:#A85B2A,stroke-width:3px,color:#0F172A;

    EVENT["Grid Curtailment Action<br>(action_digest)"]:::cosa
    
    L1["1. Grid Demand<br>(EMILIA receipt)"]:::emilia
    L2["2. Named Human Auth<br>(EMILIA AEC)"]:::emilia
    L3a["3a. Controller Telemetry<br>(COSA attestation)"]:::cosa
    L3b["3b. Physical Meter Leg<br>(Steven's Open Socket)"]:::actionstate
    L4["4. Tamper-Evident Record<br>(SCITT envelope)"]:::actionstate
    L5["5. Composed Settlement<br>(composed evidence bundle)"]:::settlement

    EVENT --> L1
    EVENT --> L2
    EVENT --> L3a
    EVENT --> L3b
    EVENT --> L4
    
    L1 --> L5
    L2 --> L5
    L3a --> L5
    L3b --> L5
    L4 --> L5
```

---

## 2. Claim Hierarchy Matrix

### Leg 1 & 2: WHO (EMILIA)
* **Demonstrated:** 
  * Ed25519-signed `EP-RECEIPT-v1` representing grid authority demand and facility acknowledgment.
  * Composed `EP-AEC-v1` (Authorization Evidence Chain) gating action execution.
  * Externally authored from-spec Rust verifier (construction independence is the implementer's attestation, auditable in the public source), CI-enforced against the byte-pinned 164-vector campaign (16 suites) plus the 359-case hostility corpus with zero divergences. The same-team JS/Py/Go suite is 193 vectors across 17 suites.
  * Fail-closed defense verifying action digest bindings to prevent confused-deputy attacks.
* **Cited:** 
  * Formal logic guarantees (TLA+ / Tamarin proofs) for authorization receipt state transitions.
  * Multi-language vector equivalence (Go, Python, Node references).
* **Not Claimed:** 
  * Out-of-band KYC or courtroom-grade identity verification of the human physical identity behind the Ed25519 signing key.

### Leg 3a: Vertical & Execution Substrate (COSA / ECR-WG)
* **Demonstrated:** 
  * Megawatt actuation telemetry (pluggable scheduler stubs: NVML, Slurm, k8s).
  * Packing Slip and Bill of Lading ingress primitives binding iso/grid orders to the stack.
  * `COGOBJ` structure linking ingress cargo, intent, and authorization receipts.
  * Single-command on-premise installation script verifying the four-layer offline path.
* **Cited:** 
  * Live workload redirection and hardware scheduler power-capping interfaces.
* **Not Claimed:** 
  * Baseline tariff modeling accuracy. COSA makes baseline computation *tamper-evident against method swapping and telemetry backfill* but does not model utility baseline perfection.

### Leg 3b & 4: WHAT & Record (Actionstate)
* **Demonstrated:** 
  * Controller-reported execution attestation and digest bindings.
  * `scitt-cose` payload-agnostic Signed Statement (COSE_Sign1) envelope.
  * Dual independent RFC 9162 Merkle tree inclusion logs.
  * CCF `vds=2` verifier compatibility executing offline.
* **Cited:** 
  * Class-1 Agent Action Capsule (AAC) spec and conformance vectors.
* **Not Claimed:** 
  * Attested physical utility meter hardware. The meter is a separate, independent third-party attestor at the composition level (Steven’s open socket).

### Leg 5: Settlement Composition
* **Demonstrated:** 
  * Multi-attestation verification linking the four-layer evidence bundle to the shared action digest.
* **Not Claimed:** 
  * Commercial payment rails, automated settlement payouts, or regulatory dispute arbitration.

---

> [!IMPORTANT]
> **Consensus Quote:** *"Three of five legs in code is inventory, not marketing."*
> This repository is not a slider; it is a runnable verification packet that turns a data center's flexibility promise into audit-proof evidence.
<!-- AGENT-SIGNATURE
agent_id: E-C54030DF-1852-001
thumbprint: MCowBQYDK2VwAyEA+kLnvOH8EtfA8bPEpMxxBZk/Fa5BWh7N7x9KRnOwSy8=
model_version_id: openai-codex-gpt5
manifest_digest: 6bc42b927a54b00f5cc476df7d1e658c473a93ab2fe8edd7eff0158e0887bcf0
environment_digest: 69d309198d35a0336bf4ab4b205e332bb8782109bc6079ff65a487c45443c6c8
input_context_digest: 3f21004479a2825552e22355bc5824482b59c43e31724ab63ac1ee682c55740d
output_digest: 4582fae7a49d74565b79fcca2c3cbe6039a2c3ec5bd965127a5644313950974f
prev_output_digest: none
timestamp: 2026-07-16T02:37:37Z
signature_algorithm: Ed25519
signature_b64u: 4ziYdNDfSwdKMmC4aSDgFit1eW1B1H0zoCGATowvjyYTCOqcqpRqktxYGjERQcwa3FODnv0SydH77iAbOGZaCg
-->

