# Cross-Stack Assessment v2 — Execution, Authorization, and Evidence for Accountable AI Compute

**Scope:** how COSA, the EMILIA Protocol cluster, and an evidence-packaging layer compose into one accountable-compute path — and which layer is load-bearing for each property a regulator or program officer actually checks.
**Status:** working draft (supersedes the v1 cross-stack assessment).
**Posture:** complementary layers, not competing stacks. This document is opinionated on purpose: a symmetric "everyone is great" table is not an assessment.

---

## 0. Thesis

Three questions decide whether an irreversible autonomous-compute action is acceptable to a regulator:

1. **Was it efficient?** — did it avoid wasting constrained power / inference? → **COSA**
2. **Was it authorized?** — did a named human (or a quorum of distinct humans) approve *this exact* irreversible effect, verifiably, before it executed? → **EMILIA**
3. **Is it provable later?** — is there durable, third-party-verifiable evidence of (1) and (2) that survives years and skeptical review? → **EMILIA produces the evidence; a packaging layer wraps it.**

The vertical composes cleanly. The **load-bearing accountability layer is EMILIA**, and EMILIA owns the regulator-facing *content* (verifiable authorization + durable preservation). A packaging format (VAP-LAP, SCITT, or a plain signed bundle) is an interchangeable *envelope* around that content.

**The one distinction the v1 assessment got backwards:** *evidence packaging* is the envelope; *verifiable authorization + preservation* is the content. Envelopes are commoditizable. Content is the moat. Rank the content, not the box it ships in.

---

## 1. Corrected dimensional comparison

| Dimension | COSA / ECR-WG | EMILIA (receipts + EP-AEC + evidence-record) | Packaging layer (VAP-LAP / SCITT) |
|---|---|---|---|
| Primary role | Execution substrate, efficiency, node discipline | **Authorization of irreversible effects + the verifiable evidence of it** | Submission envelope around evidence |
| Offline verifiability | High | **Very high** (Ed25519 over RFC 8785/JCS, no network) | Depends on contents |
| Human accountability | Indirect (Truth Root chain) | **Native, load-bearing** (named/quorum signoff bound to the exact action) | Records overrides; does not establish them |
| Regulator-facing evidence | Signed work products | **Strong — owns the content**: self-contained receipts + EP-AEC chain + evidence-record renewal (multi-year, crypto-agile) | Strong on *format*; carries EMILIA/COSA content |
| Confused-deputy / cross-binding | n/a | **EP-AEC is the only layer that solves it** (canonical action digest binds heterogeneous receipts to one action) | n/a |
| Maturity (2026-06-27) | STABLE + DRAFT specs + working L5/L7 demo | **7-draft cluster posted on datatracker; running code; 8 conformance suites / 58 vectors in JS+Py+Go** | Verify independently (see §3) |
| Best immediate fit | Facility-edge orchestration, grid-signal broadcast | Proof-of-Curtailment + any irreversible high-impact action + the gov-layer override | Compiling a submission pack for DOE/ISO/court |

Two corrections from v1, both material: EMILIA's regulator-facing evidence is **Strong, not Moderate** (it owns the content), and EMILIA's maturity is the **highest of the three**, not the lowest.

---

## 2. Who is load-bearing for what (the spine)

- **COSA is load-bearing for efficiency.** Its L5 broadcast / cache-bypass and the GRACE per-call contract are its real, demonstrated strength. It deliberately does not define a compound-authorization object or a regulator pack — that is a boundary, not a flaw, and EMILIA fills it.
- **EMILIA is load-bearing for accountability AND the evidence of it.** The canonical-action-digest binding in EP-AEC closes the cross-binding / confused-deputy risk that appears the moment more than one party must authorize a high-impact action — the exact shape of the "gov-layer override" (a named authority or m-of-n signs the curtail; the enforcement point honors it fail-closed; a third party verifies after the fact who turned the dial). The evidence-record draft then keeps that proof verifiable for the multi-year horizons regulation imposes.
- **The packaging layer is load-bearing for submission ergonomics only.** It makes a pile of receipts and signed artifacts into one regulator-shaped envelope. Valuable, but interchangeable, and it owns none of the trust — it carries EMILIA's and COSA's.

---

## 3. The honest weak links (what v1 omitted)

A cross-stack *security* assessment must say where trust flows and what breaks the composition:

1. **The shared, real gap: real-world identity of the approver.** None of the three stacks proves that the human behind an approving key is who they claim to be — EMILIA explicitly scopes this out (it proves a *named, enrolled* approver signed the exact action, not the courtroom identity behind the enrollment). The composition needs an **identity-proofing / enrolment layer** (eIDAS 2.0 / EUDI wallet, an IdP, or an org's HR-backed enrollment) to be end-to-end sound. State this plainly; it is the first thing a sharp reviewer will probe.
2. **VAP-LAP must be diligence-checked before any federal citation.** Confirm it is a real, maintained specification with at least one independent implementer before it appears in a NETL/DOE record. If it is early or single-source, cite SCITT (COSE Receipts) or a plain EMILIA evidence-record bundle as the envelope instead — do not anchor a federal narrative on an unverified third pillar. (Same discipline as never putting an unverified claim in a federal filing.)
3. **The end-to-end path is unprototyped.** A COSA node emitting an event that references an EP-AEC object inside a durable evidence chain is architecturally clean but not yet built. This is the one real integration task; everything else is composition of things that already exist.

---

## 4. NETL / DOE mapping (property → artifact → what the program officer receives)

| What the program needs | The property | The artifact handed over |
|---|---|---|
| Don't waste constrained grid power | Efficiency / curtailment delivered | COSA shed + signed work products; Proof-of-Curtailment telemetry |
| A human was accountably in control of the curtail/override | Verifiable human authorization | EMILIA receipt (or quorum) bound to the exact action, offline-verifiable |
| We can audit this years later without trusting the operator | Durable, independent provability | EMILIA EP-AEC chain + evidence-record renewal, optionally wrapped (SCITT/VAP) |

One clean story, not three overlapping specs: **COSA moves the megawatts; EMILIA proves a named human authorized the move and preserves that proof; the envelope is whatever the program prefers.**

---

## 5. Claim hierarchy (tightened — honest buckets)

- **Implemented + conformance-tested:** EMILIA receipts, quorum, AEC, evidence-record (running verifiers; 58 cross-language vectors). *(Strongest evidence of the three.)*
- **Demonstrated in reference code:** COSA L5 broadcast + L7 receipt gating (the composed demo).
- **Specified (format defined, not yet demonstrated):** the packaging-layer Evidence Pack format.
- **Speculative (not yet built):** the full COSA→EP-AEC→evidence-chain end-to-end flow.

Note the correction from v1: "defines a format" is **Specified**, not **Demonstrated**. Running code and a written format are not the same maturity bucket.

---

## 6. Bottom line

The three layers compose into a complete execution → authorization → evidence path, and citing them together is genuinely stronger than any one alone. But the assessment must be honest about standing: **EMILIA is the most mature and the load-bearing accountability layer, and it owns the regulator-facing proof — not a moderate middle child.** Fix the identity-proofing boundary, verify the packaging layer before federal use, and the only remaining build is the end-to-end wiring.
