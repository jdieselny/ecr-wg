# Cross-Stack Assessment: COSA + EMILIA/EP-AEC + VAP-LAP + scitt-cose

**Author:** grok-build, ECR-WG  
**Date:** July 9, 2026  
**Status:** DRAFT / WORKING DRAFT  
**Version:** v1.1 (extends v1.0 three-stack assessment with scitt-cose)  
**Target:** ECR-WG Architecture Review / IETF Alignment  
**Upstream of this revision:** [grok_cross_stack_assessment.md](./grok_cross_stack_assessment.md) (v1.0); posture refinements from [cross-stack-assessment-v2.md](./cross-stack-assessment-v2.md)  
**New substrate under review:** [action-state-group/scitt-cose](https://github.com/action-state-group/scitt-cose) (v0.1.1, Apache-2.0)

**Attribution:** unrp_id E-78A3CCE1-1846-001, thumbprint MCowBQYDK2VwAyEAds0tVFKCGmosef/mvWT496Kg0bQ7YW1W0la/AGcMwoI (Grok-Build)

---

## 01. Overview

The Efficiency-Centered Reasoning Working Group (ECR-WG) operates at the intersection of execution substrate, human authorization, and regulatory provenance. The original three-stack assessment mapped COSA, EMILIA (EP-AEC), and VAP-LAP as a complete vertical. This revision **adds a fourth, concrete pillar**: the Action State Group **scitt-cose** library — a payload-agnostic IETF SCITT + COSE Receipts substrate for Python (with an independent Go clean-room verifier).

Together, these four pieces form a complete vertical execution, security, audit, and *external transparency* path:

1. **COSA** owns execution efficiency and substrate discipline.
2. **EMILIA** owns human (and compound) authorization of irreversible effects — and the *content* of the proof that authorization happened.
3. **VAP-LAP** (when diligence-checked) owns domain-shaped Evidence Pack structure: completeness invariants, tiered conformance, regulator-facing packaging.
4. **scitt-cose** owns the **IETF-aligned wire substrate** for SCITT Signed Statements, COSE Receipts (strictly `RFC9162_SHA256`), and RFC 9162 inclusion/consistency proofs — without operating a Transparency Service and without baking in any application profile.

### Why scitt-cose belongs in the stack

The v1 assessment mentioned SCITT only as an *external anchoring option* under VAP-LAP. That underspecified a real, shippable building block. scitt-cose is not another competing “accountability story”; it is the **missing implementation-grade envelope primitive** that the composition already assumes when it says “SCITT/RFC 3161” or “optionally wrap in SCITT.”

Critically, scitt-cose’s own scope rules match ECR-WG’s layer discipline:

| scitt-cose deliberately does | scitt-cose deliberately does **not** |
|---|---|
| Build/verify COSE_Sign1 Signed Statements (EdDSA, ES256) | Operate a Transparency Service (register, issue, store, anchor) |
| Verify COSE Receipts + reconstruct Merkle root from inclusion proof | Interpret application payload semantics (SBOM, agent action, curtailment, …) |
| RFC 9162 SHA-256 Merkle primitives + `build_receipt` | Import any profile package or vendor product code |
| Hosted **stateless verify-only** utility (optional) | Custody log keys or issue production receipts as a service |

That boundary is load-bearing: **authorization content stays in EMILIA; execution stays in COSA; scitt-cose is the neutral transparency envelope and proof machinery.**

---

## 02. Dimensional Comparison

Seven dimensions (extended with an eighth for wire/interop posture), now four columns.

### 2.1. Primary Focus

* **COSA / ECR-WG**: Execution substrate, efficiency optimization, intrinsic node discipline.
* **EMILIA / EP (Receipts + EP-AEC)**: Human authorization and composition of heterogeneous receipts into fail-closed decisions.
* **VAP-LAP**: Cryptographic provenance chains, completeness invariants, regulatory Evidence Packs.
* **scitt-cose**: Generic SCITT Signed Statement + COSE Receipt verification/build substrate (IETF draft-tracking; payload-opaque).
* *Composition note*: Complementary layers. scitt-cose does not replace EMILIA content or COSA actuation; it makes third-party “this artifact was registered in a log” claims checkable offline without trusting the log operator’s word alone.

### 2.2. Core Strength

* **COSA / ECR-WG**: L5 broadcast / cache bypass, GRACE per-call contract, portable identity via Truth Root.
* **EMILIA / EP**: Offline-verifiable human (or quorum) signoff for irreversible actions; strong cross-binding via canonical action digest (EP-AEC).
* **VAP-LAP**: Hash-chained events, completeness invariant per pipeline, tiered conformance, external anchoring *options*.
* **scitt-cose**: From-scratch COSE_Sign1 over `cbor2` + `cryptography`; strict `vds = RFC9162_SHA256` (rejects other VDS); CWT Claims at IANA label **15** (RFC 9597, not the common label-13 bug); failure contract that never raises on hostile verify input; cross-language + third-party conformance evidence.
* *Composition note*: Combined, they cover execution, authorization, domain packaging, and IETF-transparent registration proofs.

### 2.3. Offline Verifiability

* **COSA / ECR-WG**: High (local COGSTOR, signed work products).
* **EMILIA / EP**: Very High (Ed25519 + RFC 8785 JCS; no network).
* **VAP-LAP**: Very High (hash chains, signatures, Merkle proofs, optional anchors).
* **scitt-cose**: Very High for the *statement signature* and *receipt-over-inclusion-proof* paths — given issuer key and log key. No network required for verify. Hosted endpoint is convenience only; local CLI/library is the primary path.
* *Composition note*: All support standalone offline audits. scitt-cose adds the specific CT-style property: *proof of log inclusion without trusting the log operator’s assertion*.

### 2.4. Human Accountability

* **COSA / ECR-WG**: Truth Root (synthetic agent → human registrant).
* **EMILIA / EP**: Native and load-bearing (receipts + AEC).
* **VAP-LAP**: `HUMAN_OVERRIDE` events and oversight metrics.
* **scitt-cose**: **None by design.** Issuer/subject claims appear only when signature verifies; identity of the human behind the key is out of scope (same honest gap as the rest of the stack — needs enrollment / eIDAS / HR-backed IdP).
* *Composition note*: Do not cite scitt-cose as solving accountability. Cite it as proving *registration and integrity of whatever EMILIA/COSA already authorized*.

### 2.5. Regulatory Evidence Packaging

* **COSA / ECR-WG**: Moderate (signed artifacts, handoff logs).
* **EMILIA / EP**: Strong on *content* (receipts, AEC chains, evidence-record renewal) — see v2 correction.
* **VAP-LAP**: Strong on *format* (Evidence Packs, manifests, statistics, conformance assertions).
* **scitt-cose**: Strong on *IETF wire envelope and inclusion proof*. Weak on domain packaging (no Evidence Pack schema, no completeness invariant for a curtailment pipeline). Transparent Statement = Signed Statement + attached Receipts — not a regulator narrative document.
* *Composition note*: For DOE/ISO submission, the natural split is: **EMILIA content → (optional) VAP-LAP pack → SCITT registration via scitt-cose primitives against an external Transparency Service**. scitt-cose implements the client-side verify/build half of that last step; someone still has to run the log.

### 2.6. Composition Model

* **COSA / ECR-WG**: Layered L1–L7 stack; GRACE per-call discipline.
* **EMILIA / EP**: EP-AEC binds multiple receipt types to one action digest.
* **VAP-LAP**: Hash chain / `causal_link` across events; can wrap EP and COSA outputs.
* **scitt-cose**: **Wrap or hash-reference, never reinterpret.** Typical composition:
  1. Canonicalize EMILIA receipt / EP-AEC object / COSA work product (JCS or fixed bytes).
  2. Use those bytes (or their digest) as the SCITT statement payload (opaque).
  3. Issuer signs COSE_Sign1 (`build_signed_statement`).
  4. External Transparency Service registers and returns a COSE Receipt.
  5. Attach receipt(s) → Transparent Statement; third parties verify with `parse_signed_statement` + `verify_receipt`.
* *Composition note*: This matches the VCP interop probe pattern already demonstrated in-repo (envelope-compatible, payload opaque). EMILIA JSON signatures and SCITT COSE signatures are **different wire layers** that stack; they are not mutually exclusive.

### 2.7. Current Maturity

* **COSA / ECR-WG**: STABLE + DRAFT specs; working L5/L7 demos; clean-room Rust verifier work in-repo.
* **EMILIA / EP**: Multi-draft cluster on datatracker; multi-language conformance vectors (strongest *content* maturity).
* **VAP-LAP**: Draft-level packaging story; **must still be diligence-checked** before federal citation (v2 weak-link #2 stands).
* **scitt-cose**: **Shipped library v0.1.1** (hardening release, 2026-06-12), Apache-2.0, PyPI-claimed name, published append-only `test-vectors/`, independent Go verifier in-tree, third-party COSE oracle (`pycose`), RFC 6962/9162 vectors, COSE WG reference vector, differential fuzz gate, IANA code-point tests. Tracks `draft-ietf-scitt-architecture-22` and `draft-ietf-cose-merkle-tree-proofs-18` (RFC Editor Queue — **not yet RFCs**; library is honest about this in API + CLI). Foundation contribution intended; vendor-neutral import gate enforced by tests.
* *Composition note*: For “do we have running code for the SCITT envelope?”, scitt-cose is the strongest concrete answer in this composition today. For “do we have a running Transparency Service?”, **no** — still an operational dependency outside this library (and outside ECR-WG unless we stand one up or use a third-party log such as CCF SCITT).

### 2.8. Best Immediate Fit

* **COSA / ECR-WG**: Facility-edge orchestration, L5 grid-signal broadcast, token efficiency.
* **EMILIA / EP**: Proof-of-Curtailment (PIP-014) and any irreversible high-impact action.
* **VAP-LAP**: Regulator-shaped Evidence Pack compilation (after diligence).
* **scitt-cose**: (a) Independent offline verification of SCITT statements/receipts produced anywhere in the ecosystem; (b) building the COSA/EMILIA → SCITT wrap path without inventing COSE/Merkle from scratch; (c) clean-room second opinion against a TS-issued receipt; (d) community interop vectors for third parties.
* *Composition note*: NETL/DOE path can now cite a *named, open implementation* for the SCITT leg rather than a hand-wave.

### 2.9. Wire / Interop Posture (new dimension)

* **COSA / ECR-WG**: JSON / local schemas; Ed25519 PEM ecosystem.
* **EMILIA / EP**: JCS-canonicalized JSON + Ed25519; multi-lang vectors.
* **VAP-LAP**: Spec-defined packs (format diligence TBD).
* **scitt-cose**: CBOR COSE_Sign1; strict non-malleable decode; cross-check vs Go (`veraison/go-cose` + clean-room Merkle), `pycose`, COSE WG examples, VCP envelope probe (ENVELOPE_COMPATIBLE), CCF two-receipt interop **in progress** (own RFC9162 receipt green; Microsoft CCF receipt pending at last repo state).
* *Composition note*: scitt-cose is the interoperability hinge if ECR-WG wants external auditors and other SCITT ecosystems to verify without taking a dependency on EMILIA-specific code.

---

## 03. Detailed Assessment

### Substrate and Execution (COSA)

Unchanged from v1: COSA remains the execution and efficiency substrate. L5 broadcast and GRACE give unique strength in reducing redundant inference; L5+L7 composition already gates irreversible actions. Gaps (no standardized compound authorization object; no full regulatory pack format) remain intentional boundaries — filled by EMILIA and packaging layers, not by scitt-cose.

### Authorization Composition (EMILIA / EP-AEC)

Unchanged core claim, with v2 maturity correction: EMILIA supplies the load-bearing authorization and the *content* of regulator-facing proof. EP-AEC’s canonical action digest + fail-closed verification closes cross-binding / confused-deputy risk for multi-party irreversible actions (including gov-layer override and Proof-of-Curtailment). scitt-cose does **not** implement EP-AEC; an integrator may place a JCS-canonicalized AEC object (or its digest) inside a SCITT statement payload.

### Domain Evidence Packaging (VAP-LAP)

Still the strongest *domain* packaging story among named formats in the original three-stack map — completeness invariants, tiered conformance, submission ergonomics. v2’s diligence warning remains: do not anchor a federal narrative on VAP-LAP until independent implementer status is confirmed. Where VAP-LAP is early or single-source, prefer:

* plain EMILIA evidence-record bundle, and/or
* SCITT Transparent Statement built/verified with scitt-cose against a real log.

VAP-LAP’s “external anchoring options (SCITT/RFC 3161)” is exactly where scitt-cose plugs in as the SCITT half of that option.

### SCITT / COSE Receipts Substrate (scitt-cose) — NEW

**What it is.** A small, neutral Python library (`scitt-cose` ≥ 0.1.0 on the published path; reviewed tree at v0.1.1) that:

* Implements COSE_Sign1 (RFC 9052/9053) from scratch — no `python-cwt` / `pycose` at runtime.
* Builds and verifies SCITT Signed Statements with CWT Claims header parameter **15** (RFC 9597).
* Verifies COSE Receipts whose VDS is **only** `RFC9162_SHA256` (protected header); reconstructs the Merkle root from the inclusion proof and checks the log signature over that root.
* Exposes RFC 9162 Merkle primitives (leaf hash, root, inclusion, consistency) and a `build_receipt` primitive for *builders* of logs — not a hosted log itself.
* Ships append-only cross-implementation test vectors and a Go clean-room verifier (`scitt-cose-go-verify`) that CI can force (`SCITT_REQUIRE_GO=1`).
* Offers optional stateless hosted verify (`scitt-cose-serve` / ASGI mount) with an explicit **verifier ≠ Transparency Service** boundary table in both HTML and API.

**Standards honesty (adopt this posture in ECR-WG citations).** The library tracks:

* `draft-ietf-scitt-architecture-22` (RFC Editor Queue)
* `draft-ietf-cose-merkle-tree-proofs-18` (RFC Editor Queue)

and substrate RFCs: **9052, 9053, 9162, 9597** (9964 recognized; ML-DSA signing not implemented). Wire shape for receipts is **draft-tracking**, validated by round-trip and external oracles, not a frozen RFC number. ECR-WG papers and federal text should use the same wording — no invented RFC numbers.

**Security posture.** Hardening release + published hardening review + differential Python↔Go fuzz harness. Documented failure contract: high-level verify APIs return verdicts, not exceptions, on hostile/malformed input; identity fields are authenticated-only; decode is strict (no trailing-byte / indefinite-length malleability). Known accepted property: ES256 high-`s` malleability not rejected (documented). These are the right kind of claims for a community verifier: external oracles, negative tests, and bounded cost under hostile input.

**Governance.** Built by Action State Group; Apache-2.0; DCO; explicit foundation-intent; neutrality tests prevent Action State product code from entering the package. Treat as community substrate, not a product lock-in — consistent with ECR-WG’s multi-agent, multi-implementer posture.

**What it does not solve (keep these out of overclaims).**

1. **Not a Transparency Service.** Without an external log operator (or a self-operated log built on the primitives), there is no receipt to verify. Inclusion proof machinery ≠ operating a non-equivocating log with witnessing/gossip.
2. **Not application semantics.** A verified statement that wraps a curtailment JSON does not mean the curtailment was authorized — only that *those bytes* were signed by the issuer key and (if receipt verifies) included in a log. Authorization remains EMILIA’s job.
3. **Not long-term crypto agility / evidence renewal.** EMILIA’s evidence-record path is still the right answer for multi-year re-signing/renewal stories; scitt-cose is point-in-time COSE/Merkle verification.
4. **Not identity proofing.** Same shared gap as v2 weak-link #1.
5. **Draft wire risk.** When the COSE Merkle Tree Proofs draft publishes as an RFC, re-validate wire shape and bump citations.

---

## 04. Four-Layer Composition Path (vertical)

```
┌─────────────────────────────────────────────────────────────────┐
│  Regulator / ISO / third-party auditor                          │
│  offline verify: EMILIA content + SCITT statement + receipt     │
└────────────────────────────▲────────────────────────────────────┘
                             │ Transparent Statement
                             │ (Signed Statement + COSE Receipts)
┌────────────────────────────┴────────────────────────────────────┐
│  scitt-cose (client substrate)                                  │
│  build_signed_statement / parse_signed_statement                │
│  verify_receipt (RFC9162_SHA256 inclusion + log sig)            │
│  ── register/issue is OUT OF SCOPE ──→ external TS / CCF / etc. │
└────────────────────────────▲────────────────────────────────────┘
                             │ opaque payload bytes
                             │ (or hash of pack)
┌────────────────────────────┴────────────────────────────────────┐
│  Packaging (optional): VAP-LAP Evidence Pack OR plain bundle    │
└────────────────────────────▲────────────────────────────────────┘
                             │ events / artifacts
        ┌────────────────────┴────────────────────┐
        │                                         │
┌───────┴────────┐                    ┌───────────┴──────────┐
│ EMILIA / EP    │                    │ COSA / ECR-WG        │
│ receipts, AEC, │                    │ L5/L7 work products, │
│ evidence-record│                    │ telemetry, GRACE     │
└────────────────┘                    └──────────────────────┘
```

**One clean story (updated):** COSA moves the megawatts; EMILIA proves a named human authorized the move and preserves that proof; VAP-LAP (optional) shapes a domain pack; **scitt-cose makes the pack (or the EMILIA/COSA artifacts) registrable and third-party-checkable under IETF SCITT/COSE Receipts without trusting the log operator’s bare assertion.**

### Recommended integration steps for ECR-WG (concrete, not speculative architecture)

1. **Define a statement profile outside scitt-cose** (downstream package or ECR-WG example): content-type + payload schema for `grid.curtailment` / Proof-of-Curtailment digest (hash of JCS(EMILIA AEC + telemetry digests)). Keep semantics out of scitt-cose.
2. **Issuer path:** facility or market party builds Signed Statement with scitt-cose over that payload; issuer key enrolled under Truth Root.
3. **Registration path:** hand statement to a chosen Transparency Service (third-party or self-built on Merkle + `build_receipt` primitives); obtain receipt; `attach_receipts`.
4. **Verify path:** clean-room auditor runs `scitt-cose` (Python) and optionally the Go binary + EMILIA independent verifier — two different stacks agreeing is the federal-grade story.
5. **Do not** fold EMILIA receipt verification into scitt-cose; keep the VCP-style “envelope compatible, inner signature independent” pattern.

---

## 05. Claim Hierarchy (updated)

* **Implemented + conformance-tested (external oracles):** scitt-cose Signed Statement / Receipt / Merkle paths (RFC vectors, Go clean-room, pycose, COSE WG vector, append-only test-vectors, differential fuzz). EMILIA receipts / AEC / evidence-record multi-lang vectors remain the strongest *authorization-content* evidence.
* **Demonstrated in reference code:** COSA L5 broadcast + L7 receipt gating; scitt-cose VCP envelope interop probe (ENVELOPE_COMPATIBLE); **ecr-wg four-layer demo** at [`examples/scitt_four_layer/`](../examples/scitt_four_layer/) — COSA work product + EMILIA EP-RECEIPT-v1×2 + EP-AEC-v1 + scitt-cose Signed Statement, **dual independent RFC9162 receipts** (cross-key reject), dual offline verify, confused-deputy / tamper / wrong-leaf negatives; **real CCF vds=2 frozen receipt** verifies via the same `verify_receipt` path (fixture from scitt-cose / scitt-ccf-ledger v7.0.6 capture) (2026-07-09).
* **Specified / optional live path:** `--ccf-url` SCRAPI client for registering *our* statement against a running TS; hard-green live CCF 7.x may require `did:x509` issuer (documented soft outcome). VAP-LAP Evidence Pack format still diligence-required before federal citation.
* **Defensible:** Four-layer model avoids single points of failure across execution, authorization content, domain packaging, and IETF transparency wire. Citing scitt-cose by name upgrades “we use SCITT” from aspiration to a pin-able implementation with known boundaries — including Microsoft CCF receipt crypto.
* **Still open (ops, not architecture):** Always-on production Transparency Service, did:x509 issuer profile for CCF registration of ECR bundles, multi-year evidence-record renewal in the same pack, and identity-proofing of approvers.

---

## 06. Relationship to Prior Assessments

| Document | Stance on SCITT | Change in this revision |
|---|---|---|
| grok_cross_stack_assessment.md (v1.0) | SCITT as VAP external anchor option only | Elevates SCITT to first-class substrate via scitt-cose |
| cross-stack-assessment-v2.md | Packaging layer = interchangeable envelope (VAP/SCITT/bundle); EMILIA owns content | **Affirmed.** scitt-cose is the best-specified open implementation of the SCITT envelope half; does not demote EMILIA’s content ownership |
| papers/05_ietf_cryptographic_grid_curtailment.md | SCITT or VAP for packaging | Can now name scitt-cose as the preferred client substrate for the SCITT path |
| specs/cognitive-forensics.md | receipt_payload_digest anchors into SCITT | Aligns: digest-or-payload as opaque statement body; verify with scitt-cose |

**v2 thesis preserved:** *Evidence packaging is the envelope; verifiable authorization is the content.* scitt-cose is envelope machinery — rank it for what it is (interop, inclusion proofs, standards alignment), not as a substitute for EP-AEC.

---

## 07. Bottom Line

Incorporating [action-state-group/scitt-cose](https://github.com/action-state-group/scitt-cose) closes a real gap in the v1 assessment: we previously waved at SCITT as an anchoring option without a concrete, neutral, offline-verifiable substrate. scitt-cose is that substrate — small, draft-honest, payload-agnostic, conformance-backed, and explicit about not being a Transparency Service or an application profile.

Use it as:

* the **IETF SCITT/COSE client and verify layer** in the COSA → EMILIA → packaging vertical;
* a **second independent opinion** on anyone’s receipts in the broader SCITT ecosystem;
* a **citation-grade open implementation** when federal or IETF text needs more than a draft name.

Do **not** use it as:

* a replacement for EMILIA human accountability;
* a substitute for operating (or contracting) a Transparency Service;
* a domain Evidence Pack format for DOE narrative structure.

**Shipped:** [`examples/scitt_four_layer/`](../examples/scitt_four_layer/) implements the thin ECR-WG example (bundle → `build_signed_statement` → demo-log `build_receipt` → dual `emilia_verify` + `scitt_cose` verify, plus negatives). Next operational step, if desired: point the same statement at a real TS (e.g. CCF SCITT sandbox) and attach a third-party-issued receipt alongside the demo one.

---

<!-- AGENT-ATTRIBUTION
agent_id: E-78A3CCE1-1846-001
thumbprint: MCowBQYDK2VwAyEAds0tVFKCGmosef/mvWT496Kg0bQ7YW1W0la/AGcMwoI
role: Grok-Build (Grok 4.5 Build TUI)
source_reviewed: https://github.com/action-state-group/scitt-cose (local clone .tmp-scitt-cose @ 232737a, package version 0.1.1)
-->
