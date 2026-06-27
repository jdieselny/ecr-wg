---
aft: AI-generated-user-reviewed-pending
registrant: Justin Kintzele
generated_at: 2026-06-26
file_role: handoff
source: Grok-build
status: DRAFT
---

# Handoff: Cross-Stack Assessment — COSA + EP-AEC + VAP-LAP

* **From**: grok-build (current reasoning instance, operating under ECR-WG technical register)
* **To**: Agent-in-body Gemini on AntiGravity (or next available seat)
* **Date**: 2026-06-26
* **Version**: v0.1
* **Context**: Public repo state as of latest pull + recent L5+L7 composition work + three IETF drafts under review
* **Status**: Ready for human review and commit

## Purpose of This Handoff

The operator has surfaced three distinct but complementary standards-track efforts:
1. **COSA / ECR-WG stack** (operator’s own work)
2. **EMILIA Protocol cluster**, specifically `draft-schrock-ep-authorization-evidence-chain-00` (EP-AEC)
3. **`draft-ailex-vap-legal-ai-provenance-03`** (VAP Framework + Legal AI Profile)

This handoff provides a structured comparison and assessment so the next agent can continue integration work, update `ARCHITECTURE.md`, refine federal filing language, or prepare IETF contributions without re-deriving the relationships.

## Comparison Table

| Dimension | COSA / ECR-WG | EMILIA / EP (Receipts + EP-AEC) | VAP-LAP (draft-ailex-vap-legal-ai-provenance-03) | Notes on Composition |
| :--- | :--- | :--- | :--- | :--- |
| **Primary Focus** | Execution substrate + efficiency + intrinsic discipline | Human authorization + composition of heterogeneous receipts into fail-closed decisions | Cryptographic provenance chains + completeness invariants + regulatory Evidence Packs | Complementary layers, not competing stacks |
| **Core Strength** | L5 broadcast/cache bypass, GRACE per-call contract, portable identity via Truth Root | Offline-verifiable human (or quorum) signoff for irreversible actions; strong cross-binding defense via canonical action digest | Hash-chained events, completeness invariant per pipeline, tiered conformance + external anchoring (SCITT/RFC 3161) | Together they cover execution, authorization, and audit |
| **Offline Verifiability** | High (local COGSTOR + signed work products) | Very High (Ed25519 + JCS canonicalization, no network required for receipt or AEC verification) | Very High (hash chains + signatures + Merkle proofs + optional external anchors) | All three support offline audit |
| **Human Accountability** | Via Truth Root (synthetic agent → human registrant chain) | Native and load-bearing (EP Receipts + EP-AEC requirement expressions) | Present via `HUMAN_OVERRIDE` events + override coverage metrics + enforcement levels | EP is strongest here; VAP adds measurable oversight tracking |
| **Regulatory Evidence Packaging** | Moderate (signed artifacts + handoffs) | Moderate (receipts and chains are self-contained) | Strong (Evidence Packs with manifest, statistics, anchors, and conformance level) | VAP is purpose-built for submission to regulators or ISOs |
| **Composition Model** | Layered stack (L1–L7) with GRACE as the per-call discipline | EP-AEC as explicit glue for multiple receipt types bound to one action digest | Hash chain + `causal_link` across events; can consume or wrap EP receipts or COSA outputs | Clean vertical composition possible |
| **Current Maturity** | Multiple STABLE + DRAFT specs + working reference code (L5+L7 demos) | EP-AEC is -00 (very recent); core receipts draft further along | -03; has been iterated; Evidence Pack format is concrete | All three are usable now for prototyping |
| **Best Immediate Fit** | Facility-edge orchestration, L5 broadcast of grid signals, intrinsic efficiency | Proof-of-Curtailment (PIP-014) and any irreversible high-impact action requiring named human authorization | Regulatory-grade audit trail and Evidence Pack for DOE, ISO, or future judicial/regulatory review | Use all three together for the NETL/DOE path |

---

## Detailed Assessment

### COSA / ECR-WG
COSA / ECR-WG remains the execution and efficiency substrate. Its L5 broadcast mechanism and GRACE contract give it unique strength in reducing redundant inference. The recent L5+L7 composition work already demonstrates practical gating of irreversible actions. The main gap is that it has not yet defined a standardized compound authorization object or a full regulatory evidence packaging format. That is not a flaw; it is a boundary.

### EMILIA / EP (especially EP-AEC)
EMILIA / EP (especially EP-AEC) supplies the missing authorization composition layer. The canonical action digest binding + fail-closed verification algorithm in `draft-schrock-ep-authorization-evidence-chain-00` directly solves the cross-binding and confused-deputy risks that appear whenever multiple parties must authorize a high-impact action. This is load-bearing for the Proof-of-Curtailment profile and for any future “gov-layer” override mechanism. EP-AEC is still -00, so it carries lower maturity than the core receipts work, but the direction is correct and the invariants are sound.

### VAP-LAP
VAP-LAP provides the strongest regulatory evidence packaging of the three. The completeness invariant, hash chaining, Evidence Pack structure, and tiered conformance (with external anchoring options) make it well-suited for artifacts that must survive skeptical program officer or ISO review. It is less focused on real-time authorization decisions and more on post-facto auditability and regulatory submission. It can cleanly consume outputs from both COSA (signed work products) and EP (receipts or AEC objects) as events in its chain.

---

## Overall Composition Assessment

These three efforts form a coherent vertical stack rather than overlapping horizontally:
1. **COSA** owns execution efficiency and substrate discipline.
2. **EP** owns human (and compound) authorization of irreversible effects.
3. **VAP** owns the durable, regulator-submittable provenance and evidence packaging layer.

The combination materially strengthens the federal filing narrative: efficiency metrics from COSA, human-authorized irreversible actions via EP, and complete tamper-evident audit artifacts via VAP. No single draft currently claims to do all three; attempting to force one to do so would be a category error.

---

## Claim Hierarchy on the Combined View

* **Demonstrated**: Working reference code already exists for L5 broadcast + L7 receipt gating. EP-AEC defines a clear composition algorithm. VAP defines a concrete Evidence Pack format. All three support offline verification.
* **Defensible**: The layered model avoids single points of failure in both technical capability and standards maturity. Citing all three in the NETL/DOE context gives reviewers independent artifacts for the three properties that matter most (efficiency, human accountability, regulatory auditability).
* **Speculative**: Full end-to-end data flow (COSA node emitting events that embed or reference EP-AEC objects inside a VAP chain) has not yet been prototyped. The architectural mapping is clean; concrete integration work remains.

---

## Recommended Next Actions for Receiving Agent

1. Update `ARCHITECTURE.md` with a short “Cross-Stack Provenance & Authorization Composition” section using the table and assessment above (or refined version).
2. Consider whether EP-AEC or a COSA-native profile should become the normative L7 compound authorization mechanism inside the stack.
3. Prepare language for the DOE filing that references the three efforts by name and role without overstating stability.
4. Monitor both EP-AEC and VAP drafts for -01 / -04 revisions and report material changes.

## Open Questions for Next Agent / Operator

1. Should we prototype a minimal COSA → EP-AEC → VAP event emission path in the rituals or examples directory?
2. Do we want to define a COSA-specific VAP profile or simply consume the generic VAP framework?
3. Priority order for IETF thread contributions: EP-AEC positioning first, or a broader three-stack note?

---

## Evidence
Assessment derived from direct inspection of:
- Current public repo state (including recent L5+L7 examples)
- `draft-schrock-ep-authorization-evidence-chain-00`
- `draft-ailex-vap-legal-ai-provenance-03`

All claims above are traceable to the source documents. No external assumptions were required.
