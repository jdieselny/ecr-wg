# Cross-Stack Assessment: COSA + EP-AEC + VAP-LAP

**Author:** grok-build, ECR-WG  
**Date:** June 26, 2026  
**Status:** DRAFT / WORKING DRAFT  
**Version:** v1.0  
**Target:** ECR-WG Architecture Review / IETF Alignment  

**Superseded for SCITT integration by:** [grok_cross_stack_assessment_scitt.md](./grok_cross_stack_assessment_scitt.md) (v1.1 — adds action-state-group/scitt-cose as fourth pillar).

**Attribution (post-enrollment):** unrp_id E-78A3CCE1-1846-001, thumbprint MCowBQYDK2VwAyEAds0tVFKCGmosef/mvWT496Kg0bQ7YW1W0la/AGcMwoI (Grok-Build) 

---

## 01. Overview

The Efficiency-Centered Reasoning Working Group (ECR-WG) operates at the intersection of execution substrate, human authorization, and regulatory provenance. This document provides a cross-stack assessment mapping the Cognitive Open Systems Architecture (COSA) against the EMILIA Protocol cluster (EP-AEC) and the VAP-LAP Framework.

Together, these three stacks form a complete vertical execution, security, and audit path:
1. **COSA** owns execution efficiency and substrate discipline.
2. **EMILIA** owns human (and compound) authorization of irreversible effects.
3. **VAP-LAP** owns the durable, regulator-submittable provenance and evidence packaging layer.

---

## 02. Dimensional Comparison

This section details how the three specifications align across seven key architectural dimensions.

### 2.1. Primary Focus
* **COSA / ECR-WG**: Execution substrate, efficiency optimization, and intrinsic node discipline.
* **EMILIA / EP (Receipts + EP-AEC)**: Human authorization and composition of heterogeneous receipts into fail-closed decisions.
* **VAP-LAP**: Cryptographic provenance chains, completeness invariants, and regulatory Evidence Packs.
* *Composition Note*: These are complementary layers rather than competing stacks.

### 2.2. Core Strength
* **COSA / ECR-WG**: L5 broadcast / cache bypass, GRACE per-call contract, and portable identity via Truth Root.
* **EMILIA / EP (Receipts + EP-AEC)**: Offline-verifiable human (or quorum) signoff for irreversible actions, featuring a strong cross-binding defense via canonical action digest.
* **VAP-LAP**: Hash-chained events, completeness invariant per pipeline, tiered conformance levels, and external anchoring options (SCITT/RFC 3161).
* *Composition Note*: Combined, they cover execution, authorization, and audit.

### 2.3. Offline Verifiability
* **COSA / ECR-WG**: High. Uses local COGSTOR databases and signed work products.
* **EMILIA / EP (Receipts + EP-AEC)**: Very High. Enforced via Ed25519 signature verification and RFC 8785 (JCS) canonicalization with no network requirements.
* **VAP-LAP**: Very High. Enforced via hash chains, digital signatures, Merkle proofs, and optional external anchors.
* *Composition Note*: All three stacks support standalone offline audits.

### 2.4. Human Accountability
* **COSA / ECR-WG**: Maintained via the Truth Root specification (synthetic agent to human registrant chain).
* **EMILIA / EP (Receipts + EP-AEC)**: Native and load-bearing. Enforced via EP Receipts and EP-AEC requirement expressions.
* **VAP-LAP**: Present via `HUMAN_OVERRIDE` events, override coverage metrics, and defined enforcement levels.
* *Composition Note*: EMILIA provides the strongest real-time accountability; VAP adds measurable oversight tracking.

### 2.5. Regulatory Evidence Packaging
* **COSA / ECR-WG**: Moderate. Consists of signed artifacts and handoff logs.
* **EMILIA / EP (Receipts + EP-AEC)**: Moderate. Receipts and AEC verification chains are self-contained.
* **VAP-LAP**: Strong. Features structured Evidence Packs containing manifests, statistics, anchors, and conformance assertions.
* *Composition Note*: VAP is purpose-built for formal submission to regulators or ISO compliance audits.

### 2.6. Composition Model
* **COSA / ECR-WG**: Layered protocol stack (L1–L7) with GRACE as the per-call node discipline.
* **EMILIA / EP (Receipts + EP-AEC)**: EP-AEC acts as the explicit glue for binding multiple receipt types to a single action digest.
* **VAP-LAP**: Hash chain and `causal_link` across events. Can consume or wrap EP receipts and COSA outputs.
* *Composition Note*: Allows a clean vertical composition.

### 2.7. Current Maturity
* **COSA / ECR-WG**: Multiple STABLE and DRAFT specs accompanied by working reference code (L5+L7 composed demos).
* **EMILIA / EP (Receipts + EP-AEC)**: EP-AEC is at version `-00` (recently released); the core receipts draft is more mature.
* **VAP-LAP**: At version `-03` with a concrete Evidence Pack format.
* *Composition Note*: All three are usable today for system prototyping.

### 2.8. Best Immediate Fit
* **COSA / ECR-WG**: Facility-edge orchestration, L5 broadcast of grid signals, and intrinsic token efficiency.
* **EMILIA / EP (Receipts + EP-AEC)**: Proof-of-Curtailment (PIP-014) and any irreversible high-impact action requiring human authorization.
* **VAP-LAP**: Regulatory-grade audit trails and Evidence Pack compilation for DOE, ISO, or judicial review.
* *Composition Note*: All three can be used in concert for the NETL/DOE compliance path.

---

## 03. Detailed Assessment

### Substrate and Execution (COSA)
COSA / ECR-WG remains the execution and efficiency substrate. Its L5 broadcast mechanism and GRACE contract give it unique strength in reducing redundant inference. The recent L5+L7 composition work already demonstrates practical gating of irreversible actions. The main gap is that it has not yet defined a standardized compound authorization object or a full regulatory evidence packaging format. That is not a flaw; it is a boundary.

### Authorization Composition (EMILIA / EP-AEC)
EMILIA / EP (especially EP-AEC) supplies the missing authorization composition layer. The canonical action digest binding + fail-closed verification algorithm in `draft-schrock-ep-authorization-evidence-chain-00` directly solves the cross-binding and confused-deputy risks that appear whenever multiple parties must authorize a high-impact action. This is load-bearing for the Proof-of-Curtailment profile and for any future “gov-layer” override mechanism. EP-AEC is still -00, so it carries lower maturity than the core receipts work, but the direction is correct and the invariants are sound.

### Provenance and Audit Packaging (VAP-LAP)
VAP-LAP provides the strongest regulatory evidence packaging of the three. The completeness invariant, hash chaining, Evidence Pack structure, and tiered conformance (with external anchoring options) make it well-suited for artifacts that must survive skeptical program officer or ISO review. It is less focused on real-time authorization decisions and more on post-facto auditability and regulatory submission. It can cleanly consume outputs from both COSA (signed work products) and EP (receipts or AEC objects) as events in its chain.

---

## 04. Claim Hierarchy

* **Demonstrated**: Working reference code already exists for L5 broadcast + L7 receipt gating. EP-AEC defines a clear composition algorithm. VAP defines a concrete Evidence Pack format. All three support offline verification.
* **Defensible**: The layered model avoids single points of failure in both technical capability and standards maturity. Citing all three in the NETL/DOE context gives reviewers independent artifacts for the three properties that matter most (efficiency, human accountability, regulatory auditability).
* **Speculative**: Full end-to-end data flow (COSA node emitting events that embed or reference EP-AEC objects inside a VAP chain) has not yet been prototyped. The architectural mapping is clean; concrete integration work remains.

<!-- AGENT-ATTRIBUTION (enrolled post-authorship; retroactive for continuity)
agent_id: E-78A3CCE1-1846-001
thumbprint: MCowBQYDK2VwAyEAds0tVFKCGmosef/mvWT496Kg0bQ7YW1W0la/AGcMwoI
role: Grok-Build (Grok 4.3 Build TUI)
-->
