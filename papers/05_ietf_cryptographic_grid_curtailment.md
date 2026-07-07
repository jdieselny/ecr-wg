# Cryptographic Grid Curtailment for Agentic AI Workloads

## Abstract

This document defines a cryptographic profile for authorizing and proving bounded, reversible curtailment of electrical load by autonomous or agentic AI systems. It composes the Cognitive Open Systems Architecture (COSA) execution substrate with EMILIA Protocol authorization receipts and evidence packaging layers to produce offline-verifiable Proof-of-Curtailment bundles. The profile uses Ed25519 keys enrolled under a Truth Root for human (or quorum) accountability and supports edge Consumption Proofs for settlement-grade verification without requiring real-time connectivity to a central authority.

The mechanism binds a market-authorized party's curtailment order to facility-edge telemetry via canonical action digests, enabling regulators (e.g., DOE, ISOs, utilities) to audit compliance years later using only public artifacts and clean-room verifiers.

## 1. Introduction

Large AI datacenters represent a new class of flexible load on the electric grid. Demand-response programs already pay such loads to curtail during scarcity events, but current practice relies on self-reported logs and trusted operators. This creates risks of over-claiming, confused-deputy attacks (where an unauthorized party triggers curtailment), and unverifiable settlement.

The Efficiency-Centered Reasoning Working Group (ECR-WG) and the EMILIA Protocol collaboration address this by layering:

- COSA for workload scheduling, caching, and power-constraint enforcement at the facility edge.
- EMILIA receipts for cryptographically signed, human-authorized curtailment orders.
- Evidence packaging (e.g., SCITT or VAP-LAP) for durable, third-party-verifiable audit trails.

This profile, "grid.curtailment", standardizes the composition for IETF consumption. It introduces no new cryptography; it profiles existing mechanisms (Ed25519 signatures over RFC 8785 JCS-canonicalized objects, Merkle proofs, and consumption evidence) to the specific use case of grid-responsive AI compute.

The core innovation is the binding of low-grade waste heat recovery and compute curtailment events to named human authorization via offline-verifiable bundles. This enables "thermodynamic arbitrage" at scale: residential or edge nodes can reclaim waste heat as utility while datacenters provide verifiable flexibility.

## 2. Architecture

### 2.1. Layered Composition

The architecture composes three load-bearing layers (see cross-stack-assessment-v2.md for detailed mapping):

1. **COSA Execution Substrate (L1-L7)**: Provides the actuation seam. Workload throttling, aggressive caching (COGSTOR Re-Absorption), hardware power constraints (e.g., NVML), and L5 broadcast of grid signals. GRACE Contract enforces per-call discipline (GOAL, ROUTING, ANCHOR, CONSTRAINTS, EVIDENCE).

2. **EMILIA Authorization Layer**: Supplies cryptographically signed receipts for irreversible actions. A market-authorized party (ISO, utility, or aggregator) issues a `grid.curtailment` order as an EP-RECEIPT-v1. The receipt carries:
   - action_type: "grid.curtailment"
   - target_set, effect_class (e.g., "power_reduction"), magnitude (MW or %)
   - window (not_before, not_after)
   - baseline_method_hash (pins the ISO-prescribed baseline)
   - protected_lanes (life-safety or contractual workloads exempt from shed)
   - telemetry_sources

   Human (or quorum) signoff is bound via the receipt's authorization scope (PIP-013 Human-Oversight Profile). EP-AEC (Authorization Evidence Chain) composes heterogeneous receipts (e.g., grid order + facility acknowledgment) into a single canonical action digest, solving confused-deputy risks.

3. **Evidence Packaging Layer**: Wraps the above for regulatory submission. VAP-LAP or SCITT provides hash-chained events, completeness invariants, and optional external anchors (e.g., RFC 3161 timestamps). COSA emits signed work products and telemetry; EMILIA emits receipts and chains; the envelope makes the bundle submission-ready for DOE/ISO audits.

This composition is "one clean story": COSA moves the megawatts, EMILIA proves a named human authorized the move and preserves the proof, the envelope is interchangeable.

### 2.2. Edge Actuation Flow

1. **Authorize**: Grid authority issues EP receipt for bounded curtailment (e.g., "shed 50 MW from us-east-1 for 2 hours").
2. **Verify & Gate**: Facility controller verifies receipt offline (Ed25519 over JCS bytes) against pinned keys. Fail-closed: no valid receipt, no shed.
3. **Shed**: COSA scheduler enforces via L3 priority markers (derived from receipt hash), L5 cache preference, and L7 PEP gates. Non-protected workloads evicted or deferred; protected lanes preserved.
4. **Measure**: Attested meter (or COGSTOR-derived telemetry) signs samples including baseline_method_hash.
5. **Prove**: Emit Proof-of-Curtailment Bundle: original order receipt + facility ack + attested telemetry + computed delivered kW·h. All elements bound by hashes and signatures.
6. **Settle**: Auditor verifies bundle offline using clean-room verifier. Payment against proof, not self-report.

### 2.3. Consumption Proofs at the Edge

A Consumption Proof is a signed attestation that a specific compute workload was deferred or shed in response to the curtailment order. It includes:
- Hash of the affected COGOBJ or query batch.
- Timestamp and baseline_method_hash binding.
- GRACE EVIDENCE (e.g., cache hit metrics, power telemetry delta).
- Link to the authorizing EP receipt via canonical digest.

These proofs are emitted at the edge (L5/L7) without phoning home, enabling settlement-grade audit while preserving privacy (selective disclosure via the packaging layer).

## 3. Proof-of-Curtailment

### 3.1. Receipt Shape

A `grid.curtailment` receipt (EP-RECEIPT-v1 profiled per PIP-013):

```json
{
  "@version": "EP-RECEIPT-v1",
  "action": {
    "action_type": "grid.curtailment",
    "effect_class": "power_reduction",
    "target_set": ["us-east-1"],
    "magnitude": 50,
    "window": {
      "not_before": "2026-07-07T18:00:00Z",
      "not_after": "2026-07-07T20:00:00Z"
    },
    "baseline_method_hash": "sha256:..."
  },
  "human_oversight": {
    "control_mode": "on_the_loop"
  },
  "approver": "ep:approver:grid-authority-1",
  "issued_at": "...",
  "expires_at": "...",
  "nonce": "...",
  "policy_id": "ep:policy:grid-curtailment@v1"
}
```

L3 priority_marker = sha256(JCS(canonical_receipt))

### 3.2. Bundle Construction

The Proof-of-Curtailment Bundle is an EP evidence-record or SCITT statement containing:
- The authorizing receipt(s).
- Facility posture acknowledgment (signed, binding baseline_method_hash).
- Attested telemetry (meter or COGSTOR-derived, Ed25519-signed).
- Computed delivered curtailment (baseline - actual, against pinned method).

Verification requires:
- Order verifies against authority key.
- Ack verifies against facility key.
- Telemetry verifies against meter key.
- baseline_method_hash matches across elements.
- Recomputed delivered kW·h equals claimed value.

All offline, using the @emilia-protocol/verify-independent clean-room suite.

## 4. Identity and Authorization

### 4.1. Ed25519 Enrollment (Truth Root)

Agents and facilities enroll via the Truth Root (specs/truth-root.md). Each enrollment produces an Ed25519 keypair. The thumbprint is the SPKI DER Base64 representation of the public key.

Example (Grok-Build enrollment):
- unrp_id: E-78A3CCE1-1846-001
- thumbprint: MCowBQYDK2VwAyEAds0tVFKCGmosef/mvWT496Kg0bQ7YW1W0la/AGcMwoI=

Human accountability is bound at enrollment time by the registrant (e.g., Justin Kintzele). Revocation is by registrant decision, recorded in the append-only registry.

### 4.2. Authorization Binding

A curtailment order is authorized by a named human (or EP-QUORUM m-of-n) via device-bound Class-A signoff (WebAuthn or equivalent). The signoff is bound to the exact action via the canonical receipt digest.

This solves the confused-deputy problem: an unauthorized party cannot forge a valid receipt that will pass offline verification. The L3 marker (derived from the digest) ensures routing and scheduling respect only authorized orders.

Quorum is required for hard cuts (large MW or full-site).

## 5. Offline Verifiability

All artifacts are designed for offline verification:

- Receipts and bundles use Ed25519 signatures over JCS-canonicalized objects (RFC 8785).
- No network calls required for core verification.
- The clean-room verifier suite (`@emilia-protocol/verify-independent`) implements the full stack (receipts, quorum, AEC, consumption proofs, telemetry) using only native Node.js crypto. It passes 161/161 conformance vectors and produces settlement-grade statements.

Auditors years later can:
1. Fetch public enrollment registry and pinned keys.
2. Verify receipt signatures and chains.
3. Recompute delivered curtailment against the pinned baseline method.
4. Validate consumption proofs against edge telemetry.

No trust in the operator's logs is required beyond the cryptographic bindings.

## 6. Relationship to Other Work

- Profiles EMILIA Protocol receipts and EP-AEC (I-D.schrock-ep-authorization-receipts, etc.).
- Composes with COSA GRACE Contract and COGSTOR.
- Evidence packaging aligns with SCITT (RFC 9334 et al.) and VAP-LAP.
- Addresses gaps identified in IETF agentic AI taxonomy and DAWN use cases (see papers/04_...).

## 7. Security Considerations

- **Key Compromise**: Enrollment keys must be protected at the edge (e.g., HSM or secure enclave). Revocation is supported.
- **Replay**: Nonces and windows (expires_at) prevent replay.
- **Baseline Gaming**: The pinned baseline_method_hash makes method swaps or input manipulation detectable.
- **Confused Deputy**: Canonical action digest + EP-AEC binding ensures only authorized orders trigger scheduling changes.
- **Privacy**: Selective disclosure and content minimization are supported at the packaging layer.
- **Low-Grade Heat / Edge Recovery**: The profile supports "thermodynamic arbitrage" use cases (e.g., residential runners reclaiming waste heat) but requires separate privacy and safety analysis for edge deployments.

## 8. IANA Considerations

This document requests registration of the "grid.curtailment" action_type in the EMILIA Protocol action-type registry, with reference to this profile and the EP authorization receipt specification.

## 9. References

- [I-D.schrock-ep-authorization-receipts]
- [I-D.schrock-ep-authorization-evidence-chain]
- ECR-WG specs: grace-contract.md, truth-root.md, cogstor.md, air-protocol.md
- cross-stack-assessment-v2.md
- @emilia-protocol/verify-independent conformance suite

## Authors' Addresses

Justin Kintzele (J Diesel NY, LLC)
Grok-Build (agent-04, xAI Grok 4.3 Build TUI)

---

<!-- AGENT-SIGNATURE
agent_id: E-78A3CCE1-1846-001
thumbprint: MCowBQYDK2VwAyEAds0tVFKCGmosef/mvWT496Kg0bQ7YW1W0la/AGcMwoI=
role: Grok-Build (Grok 4.3 Build TUI)
-->
