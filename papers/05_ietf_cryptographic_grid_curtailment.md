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

The architecture composes four complementary layers (see
`planning/cross-stack-assessment-v2.md` and
`planning/grok_cross_stack_assessment_scitt.md` for the dimensional mapping):

1. **COSA Execution Substrate (L1-L7)**: Provides the actuation seam. Workload throttling, aggressive caching (COGSTOR Re-Absorption), hardware power constraints (e.g., NVML), and L5 broadcast of grid signals. GRACE Contract enforces per-call discipline (GOAL, ROUTING, ANCHOR, CONSTRAINTS, EVIDENCE).

2. **EMILIA Authorization Layer**: Supplies cryptographically signed receipts for irreversible actions. A market-authorized party (ISO, utility, or aggregator) issues a `grid.curtailment` order as an EP-RECEIPT-v1. The receipt carries:
   - action_type: "grid.curtailment"
   - target_set, effect_class (e.g., "power_reduction"), magnitude (MW or %)
   - window (not_before, not_after)
   - baseline_method_hash (pins the ISO-prescribed baseline)
   - protected_lanes (life-safety or contractual workloads exempt from shed)
   - telemetry_sources

   Human (or quorum) signoff is bound via the receipt's authorization scope (PIP-013 Human-Oversight Profile). EP-AEC (Authorization Evidence Chain) composes heterogeneous receipts (e.g., grid order + facility acknowledgment) into a single canonical action digest, solving confused-deputy risks.

3. **Evidence Packaging Layer (optional domain pack)**: Shapes the above for regulatory submission ergonomics. A plain Proof-of-Curtailment JSON bundle is sufficient; VAP-LAP may supply completeness invariants and tiered conformance when diligence-checked. COSA emits signed work products and telemetry; EMILIA emits receipts and chains; the pack is the content that envelopes wrap.

4. **SCITT / COSE Receipts Substrate**: Makes the pack (or a digest of it) registrable and third-party-checkable under IETF SCITT Signed Statements and COSE Receipts. The ECR-WG reference client is the payload-agnostic [scitt-cose](https://github.com/action-state-group/scitt-cose) library (Action State Group; Apache-2.0): COSE_Sign1 build/verify, RFC 9162 inclusion proofs (`vds=1`), and CCF `ccf.v1` receipts (`vds=2`). **scitt-cose is not a Transparency Service** — it verifies and provides primitives; operating a log (or consuming Microsoft scitt-ccf-ledger / another TS) is a separate operational concern.

This composition is "one clean story": COSA moves the megawatts, EMILIA proves a named human authorized the move and preserves the proof, the pack is the regulator-facing content, and SCITT/scitt-cose is the interchangeable wire envelope plus inclusion proof.

### 2.1.1. Implemented Path (reference demo)

As of 2026-07-11 the vertical is **demonstrated offline** in-repo, not only specified:

| Step | Artifact | Verifier |
|---|---|---|
| Continuum ingress | Packing Slip (hash-sealed) + Bill of Lading (Ed25519) | recompute slip hash; verify BoL signature |
| COGOBJ packet | same ingress + `authorization.action_digest` on the cognitive object | structural + hash equality |
| Canonical `grid.curtailment` action | `action_digest = SHA-256(JCS(action))` | `emilia_verify.action_digest` |
| Grid + facility EP-RECEIPT-v1 | receipts bound to that digest in the *signed* claim | `emilia_verify.verify_receipt` |
| EP-AEC-v1 | requirement `grid_order AND facility_ack`; confused-deputy refuse | `emilia_verify.verify_authorization_chain` |
| COSA edge work product | shed telemetry signed and action-bound | Ed25519 over JCS |
| SCITT Signed Statement | COSE_Sign1 over JCS(Proof-of-Curtailment bundle) | `scitt_cose.parse_signed_statement` |
| Dual independent logs | two RFC9162 COSE Receipts over the same leaf; cross-key reject | `scitt_cose.verify_receipt` |
| CCF interop (crypto) | frozen real scitt-ccf-ledger v7.0.6 receipt (`vds=2`) | same `verify_receipt` path |
| Negatives | tampered statement, wrong leaf | fail-closed |

Runnable entry point: `examples/scitt_four_layer/demo.py` (see that directory’s README). Optional live registration against a SCRAPI/CCF endpoint is available via `--ccf-url`; production CCF 7.x may additionally require a `did:x509` issuer profile, which is deliberately out of scope for the default offline demo.

**Claim hierarchy note:** dual-log registration and CCF *verification* are demonstrated. Hosting a production Transparency Service, multi-year evidence-record renewal inside the same bundle, and identity-proofing of approvers remain operational/policy work — not missing architecture.

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
- **Ingress envelope (Continuum):** Packing Slip + Hash and Bill of Lading — how the order entered the overlay (see `specs/primitives/packing-slip.md`, `bill-of-lading.md`). Prototype versions `ECR-PACKING-SLIP-v0.1` / `ECR-BILL-OF-LADING-v0.1`.
- **COGOBJ packet:** the same ingress hashes plus `authorization.action_digest`, so the cognitive object and the audit pack answer the same questions (see `thesis/COGOBJ_SCHEMA.md` v2.1).
- The authorizing receipt(s) and EP-AEC chain.
- Facility posture acknowledgment (signed, binding baseline_method_hash).
- Attested telemetry (meter or COGSTOR-derived, Ed25519-signed).
- Computed delivered curtailment (baseline - actual, against pinned method).

**Separation of concerns:** Packing Slip / BoL do **not** live inside the EMILIA action object. Mutating ingress must not rewrite `action_digest`. Authorization answers *who approved the irreversible effect*; ingress answers *how the cargo was sealed and routed*.

Verification requires:
- Packing Slip hash recomputes; BoL `sender_signature` verifies; `packing_slip_hash` matches.
- COGOBJ `ingress.packing_slip_hash` and `authorization.action_digest` match the pack.
- Order verifies against authority key.
- Ack verifies against facility key.
- Telemetry verifies against meter key.
- baseline_method_hash matches across elements.
- Recomputed delivered kW·h equals claimed value.

All offline, using the @emilia-protocol/verify-independent clean-room suite (authorization) plus the demo's BoL/COGOBJ checks (ingress).

## 4. Identity and Authorization

### 4.1. Ed25519 Enrollment (Truth Root)

Agents and facilities enroll via the Truth Root (specs/truth-root.md). Each enrollment produces an Ed25519 keypair. The thumbprint is the SPKI DER Base64 representation of the public key.

Example (Grok-Build enrollment):
- unrp_id: E-78A3CCE1-1846-001
- thumbprint: MCowBQYDK2VwAyEAxf9pDw+okMCMBDh01Seo3MlqfvRyUVb187XBHCOuljI=

Human accountability is bound at enrollment time by the registrant (e.g., Justin Kintzele). Revocation is by registrant decision, recorded in the append-only registry.

### 4.2. Authorization Binding

A curtailment order is authorized by a named human (or EP-QUORUM m-of-n) via device-bound Class-A signoff (WebAuthn or equivalent). The signoff is bound to the exact action via the canonical receipt digest.

This solves the confused-deputy problem: an unauthorized party cannot forge a valid receipt that will pass offline verification. The L3 marker (derived from the digest) ensures routing and scheduling respect only authorized orders.

Quorum is required for hard cuts (large MW or full-site).

## 5. Offline Verifiability

All artifacts are designed for offline verification:

- Receipts and bundles use Ed25519 signatures over JCS-canonicalized objects (RFC 8785).
- No network calls required for core verification.
- The clean-room verifier suite (`@emilia-protocol/verify-independent` and the independent Rust verifier in `rust/ep-cleanroom-verifier`) implements the full stack (receipts, quorum, AEC, consumption proofs, telemetry). The Rust cleanroom currently reports **163/163** on the public pack and produces settlement-grade external statements.

Auditors years later can:
1. Fetch public enrollment registry and pinned keys.
2. Verify receipt signatures and chains.
3. Recompute delivered curtailment against the pinned baseline method.
4. Validate consumption proofs against edge telemetry.

No trust in the operator's logs is required beyond the cryptographic bindings.

## 6. Relationship to Other Work

- Profiles EMILIA Protocol receipts and EP-AEC (I-D.schrock-ep-authorization-receipts, etc.).
- Composes with COSA GRACE Contract and COGSTOR.
- SCITT envelope and COSE Receipts: tracks `draft-ietf-scitt-architecture` and `draft-ietf-cose-merkle-tree-proofs` (RFC Editor Queue as of assessment date); substrate RFCs 9052/9053/9162/9597 via scitt-cose. CCF profile receipts (`CCF_LEDGER_SHA256` / vds=2) interoperate at the verifier layer.
- Domain packaging remains optional (plain bundle today; VAP-LAP if diligence-checked).
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
- cross-stack-assessment-v2.md, grok_cross_stack_assessment_scitt.md
- examples/scitt_four_layer/ (implemented composition demo)
- scitt-cose (https://github.com/action-state-group/scitt-cose)
- scitt-ccf-ledger (https://github.com/microsoft/scitt-ccf-ledger)
- @emilia-protocol/verify-independent / emilia-verify conformance suite

## Authors' Addresses

Justin Kintzele (J Diesel NY, LLC)
Grok-Build (agent-04, xAI Grok Build TUI)

---

<!-- AGENT-SIGNATURE
agent_id: E-78A3CCE1-1846-001
thumbprint: MCowBQYDK2VwAyEAxf9pDw+okMCMBDh01Seo3MlqfvRyUVb187XBHCOuljI=
role: Grok-Build (Grok 4.3 Build TUI)
-->
